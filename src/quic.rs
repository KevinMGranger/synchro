#![allow(dead_code)]
use futures::stream::{SplitSink, SplitStream, Stream, StreamExt};
use futures::{Sink, SinkExt};
use quiche::RecvInfo;
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use std::task::Poll::*;
use std::task::{Context, Poll};
use tokio::pin;
use tokio_util::codec::BytesCodec;
use tokio_util::udp::UdpFramed;

use bytes::BytesMut;
use thiserror::Error;
use tokio::net::UdpSocket;

// region: errors
#[derive(Debug, Error)]
pub enum QuicError {
    #[error("quic error {0}")]
    QuicError(#[from] quiche::Error),
    #[error("io error {0}")]
    IoError(#[from] tokio::io::Error),
}

impl QuicError {
    fn is_done(&self) -> bool {
        matches!(self, QuicError::QuicError(quiche::Error::Done))
    }

    fn would_block(&self) -> bool {
        match self {
            QuicError::IoError(e) if e.kind() == io::ErrorKind::WouldBlock => true,
            _ => false,
        }
    }
}

type Result<T> = std::result::Result<T, QuicError>;
// endregion


// quiche's stream map / application data: do we need it? Can we offer (parts) of it to callers?
// we need it for waiters for reads from other threads


// TODO: wondering if we could make a small "async" abstraction over
// the connection itself. or is that even worth it?
// probably not, we just register interest with each
// call to recv and send.
// now, what about implementing a stream abstraction that implements AsyncWrite and AsyncRead?
//

enum SendState {
    Send,
    // TODO: at-waiting with time and buf
    Flush,
}

// TODO: use conn value
const RESERVE_SIZE: usize = 1024;

// when we get around to putting a mutex here, what scenarios do we have?
/*
all scenarios are something that deals with connection <-> udp versus a reader/writer to/from a stream
(oh that reminsds me, we need to periodically recv anyways)
(or is that what the built-in timer is for?)

anyhow, scenarios:

1. reader wins. needs to read data that isn't there. maybe it tries to recv just in case. says it's not ready but registers with stream map.

okay nvm all the scenarios are the same as long as readers/writers are kind about doing send/recv... right?

So will we have a deadlock in a single-threaded case?
No, because it's not going to hold the lock while waiting for data. It'll get woken up and try to retrieve it. Same for readers/writers who have registered intent.
todo: how do we structure readers/writers wakers in the app data? do we just need one for reading and one for writing? It can be like a file handle where having m
multiple copies of the same stream handle is okay, right?

or... how do we do fairness? earlier stream always wins?
we could do our own scheduling with a map of waiting read/writers by age,
but how much are we recreaitinf from the runtime?
Also need to test if queue loss from async runtime means it neve gets released,
or if a task creating and redoing it would just always be last
 */
struct QuicConn {
    conn: quiche::Connection,
    io: UdpFramed<BytesCodec, UdpSocket>,
    send_state: SendState,
    send_buf: BytesMut,
}

// TODO: repalcing the framed with working the socket directly.
impl QuicConn {
    /// poll, returns if any data was received at all
    fn recv(&mut self, cx: &mut Context<'_>) -> Result<bool> {
        match self.io.poll_next_unpin(cx) {
            Ready(Some(Ok((mut bytes, from)))) => {
                self.conn.recv(&mut bytes, RecvInfo { from })?;
            }
            Ready(Some(Err(e))) => return Err(e.into()),
            Ready(None) => unimplemented!("idk"),
            Pending => return Ok(false),
            _ => todo!(),
        }
        while let Ready(res) = self.io.poll_next_unpin(cx) {
            match res {
                None => unimplemented!("idk"),
                Some(Ok((mut bytes, from))) => {
                    self.conn.recv(&mut bytes, RecvInfo { from })?;
                }
                Some(Err(e)) => return Err(e.into()),
            }
        }

        Ok(true)
    }

    /// send as much as we can
    /// TODO: handle at for delay
    fn send(&mut self, cx: &mut Context<'_>) -> Poll<Result<usize>> {
        loop {
            match self.send_state {
                SendState::Send => 'send: loop {
                    let poll = self.io.poll_ready_unpin(cx)?;
                    if poll.is_pending() {
                        self.send_state = SendState::Flush;
                        break 'send;
                    }
                    // TODO: use config'd size from quiche connection
                    self.send_buf.reserve(RESERVE_SIZE);

                    let (size, target) = match self.conn.send(&mut self.send_buf) {
                        Ok(x) => x,
                        Err(e) if e == quiche::Error::Done => {
                            self.send_state = SendState::Flush;
                            break 'send;
                        }
                        Err(e) => return Ready(Err(e.into())),
                    };
                    let to_send = self.send_buf.split_to(size);
                    self.io.start_send_unpin((to_send, target.to))?;
                },
                SendState::Flush => match self.io.poll_flush_unpin(cx) {
                    Ready(Ok(())) => {
                        self.send_state = SendState::Send;
                    }
                    Ready(Err(e)) => return Ready(Err(e.into())),
                    Pending => return Pending,
                },
            }
        }
    }
}
