# Phases
..and their LOE (level of effort).

## Unsure where to fit in
- robust testing of potential sync conflicts
  - after each phase TBH.

## Planned
0. A simple, idiomatic-ish in-memory filesystem representation.  
   LOE: relatively low
   - test how conflicts can be handled
   - concurrent map structure?
1. A simple, idiomatic-ish implementation of the Cloud Mirror example.  
   LOE: relatively low
2. Two-way mirroring(?) (I forget if the example had this)  
   LOE: relatively low
3. Using SQLite to track file block modifications  
   LOE: medium
4. Block exchange protocol on the same host (without encryption)  
   LOE: high
5. Block exchange protocol on the same host (with encryption)  
   LOE: low
6. Block exchange protocol on different hosts, talking to syncthing  
   LOE: high

## Future
- extension of BEP that can use the full benefits of QUIC / HTTP3
- designation of a "central storage" host (i.e. always has all data, like a NAS).
- backup / snapshotting features (using above?)
  - including object storage
- replication settings (make sure N hosts have this file)
- iOS integration
  - photo sync
  - push notifications?
    - setting up a central service for others to use? with e2e encryption for the file names
    - and/or just making it easy for others to set up themselves
    

# side notes
- what kind of fuzzing / address sanitization tools are there to make sure our PCWSTR use is okay?

- for a trampoline, offer a macro to generate the callback functions.
  - I _could_ make it work with multiple implementations too, but that seems highly unnecessary.
    - do I even want this to be a generic library?

- seriously, who is supposed to be in charge of the memory in the callback info / params?
  - looks like it's blocking unless there's a handoff, 