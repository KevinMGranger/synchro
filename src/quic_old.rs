#![allow(dead_code)]
use std::io;
use std::net::SocketAddr;
use std::ops::Range;
use std::sync::Arc;
use std::task::Poll;
use std::task::Waker;

use bytes::BytesMut;
use futures::future::BoxFuture;
use futures::pin_mut;
use quiche::{RecvInfo, SendInfo};
use tokio::pin;
use std::future::Future;
use std::pin::Pin;
use std::task::Context;
use std::task::Poll::*;
use thiserror::Error;
use tokio::net::UdpSocket;
use tokio::sync::Mutex;

// OH MY GOSH THE UDP FRAMED STUFF CAN TAKE A BORROW<UDP> asl;dfkja;slkfjds

// arc mutex?
// pub struct QuicConnection(Arc<)
struct QuicConn {
    conn: quiche::Connection,
    sock: UdpSocket,
    // ah crap this does need to be self-referential
    // cuz the read futures and whatnot need to point to the socket

    // maybe I just do this the "wrong" way for now, and poll / await when
    // I don't need to?

    // I also guess I could drop the cancellation-safe futures?

    // TODO: these could be configured using the quinn config, it has some defaults
    // and can be negotiated!
    // we're not using udpframed because of mutability rules,
    // and that splitting makes accessing the underlying socket impossible.
    recv_buf: BytesMut,
    send_buf: BytesMut,
    to_send: Option<(Range<usize>, SendInfo)>,
    mainloop_wakeup: Option<Waker>,
}

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

// the interleaving and queueing necessary to handle
// socket read/writes and stream read/writes is almost
// a perfect recreation of a async executor itself!
// thus, QuicConn's future impl is a sort of "main loop",
// handling wakers registered per-stream, etc.
// TODO: it remains to be seen if this really needs to be a poll itself.
// could the main loop just be an async function?
// only time will tell, I guess. I know how to do this,
// but maybe a cleaner design will be apparent once it's done.
impl Future for QuicConn {
    type Output = Result<()>;

    fn poll(mut self: Pin<&mut Self>, ctx: &mut Context<'_>) -> std::task::Poll<Self::Output> {
        loop {
            self.recv_loop()?;

            todo!()
        }
    }
}



impl QuicConn {
    /// Try to receive and process as much data as possible.
    /// Returns true if any data was received at all, false otherwise.
    /// Always registers to receive more.
    fn poll_recv(&mut self, ctx: &mut Context<'_>) -> Result<bool> {
        let mut received_any = false;
        // TODO: paired with readable and BytesMut,
        // no need to juggle received_amount.
        let mut received_amount = 0;
        let mut src = None;

        let mut recv_fut = self.sock.recv_from(&mut self.recv_buf);

        pin!(recv_fut);

        // TODO: without as_mut, bad error message. Consider filing issue.
        match recv_fut.as_mut().poll(ctx) {
            Ready(Ok((amount, from))) => {
                received_any = true;
                received_amount = amount;
                src = Some(RecvInfo { from });
            }
            Ready(Err(e)) => return Err(e.into()),
            Pending => {
                return Ok(false);
            }
        }
        drop(recv_fut);

        loop {
            self.conn
                .recv(&mut self.recv_buf[..received_amount], src.unwrap())?;
        }
    }
    // TODO: at some point, these will be combined into one big poll because of borrow checking.
    // or maybe not, since they'll be sequential?
    fn recv_loop(&mut self) -> Result<()> {
        // TODO: when would the connection process fewer bytes than were received?
        // the examples don't show handling that.
        // TODO: I do actually have to register interest, so I can't just rely on try_recv_from.
        loop {
            match self.sock.try_recv_from(&mut self.recv_buf) {
                Ok((amount, from)) => {
                    let _ = self
                        .conn
                        // todo: what would `Done` mean in this context?
                        .recv(&mut self.recv_buf[..amount], RecvInfo { from })?;
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(e) => return Err(QuicError::IoError(e)),
            }
        }
        Ok(())
    }

    // TODO: the example in the docs shows sending as much as possible even to different send targets.
    // TODO: handle timer for `at`. Might affect return type / args / struct members.
    /// Sends a single quic packet
    async fn send(&mut self) -> Result<()> {
        if self.to_send.is_none() {
            let (written, send) = self.conn.send(&mut self.send_buf)?;

            self.to_send = Some((0..written, send));
        }

        let (range_to_send, send) = self.to_send.as_mut().unwrap();

        while !Range::is_empty(range_to_send) {
            let to_send = &self.send_buf[range_to_send.clone()];

            let sent = self.sock.send_to(to_send, send.to).await?;

            range_to_send.start += sent;
        }

        self.to_send = None;

        Ok(())
    }

    /// Sends as many quic packets as there are until `Done` is returned.
    async fn send_all(&mut self) -> Result<()> {
        loop {
            match self.send().await {
                Ok(()) => (),
                Err(e) if e.is_done() => break Ok(()),
                e => break e,
            }
        }
    }
}

fn main() {
    println!("Hello, world!");
}
