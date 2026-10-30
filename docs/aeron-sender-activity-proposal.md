# Draft: sender visits only publications with work

Status: draft for an Aeron upstream discussion. Not posted.

## Problem

Each duty cycle, `aeron_driver_sender_do_send` (`aeron_driver_sender.c`) calls
`aeron_network_publication_send` on every network publication. For a quiet publication that call
still reads the sender position, scans the term for a new frame, checks the heartbeat and setup
timers, runs flow control `on_idle`, updates `has_receivers` and checks retransmits. The cost
grows with the number of publications, and new data waits on average half a cycle before it is
sent.

Measured with rusteron's `many_streams` bench (Aeron 1.54.0 C driver, i5-8500T, one UDP channel
per stream, conductor, sender and receiver each on its own pinned core; desktop box, so read the
structure, not the absolute numbers):

| Quiet network publications | Sender cycle p50 | p99 |
|---|---|---|
| 1 | 0.87 µs | 0.95 µs |
| 64 | 3.9 µs | 4.1 µs |
| 128 | 9.6 µs | 12.2 µs |
| 256 | 21.6 µs | 24.6 µs |

With 128 streams each way at 10,000 msg/s in total (256 publications in one driver), round-trip
p50 rises from 11.7 µs (1 stream) to 49.8 µs. Keeping the 255 extra publications but sending
on one stream gives 47.6 µs, so the driver accounts for nearly all of the rise. In that run the
sender cycle is 23 µs p50, against 5 µs for the receiver. On two hosts with 128 streams the same
costs predict about +13 µs per round trip, and the measured rise is +12 µs.

## Proposal

The sender visits a publication in a cycle only if it is marked active or has a timer due.

1. **Activity bitmap in shared memory.** The driver gives each network publication a slot
   index when it creates it, and writes the index into the log metadata (a new field). The
   bitmap lives in the CnC file (for example, a run of driver-owned counters), 1 bit per slot.
2. **Publisher marks.** After a successful `offer`, `tryClaim` commit or `offerv`, the client
   loads the slot's word and, only if the bit is clear, sets it with an atomic OR. A busy
   publication pays one load per message; nothing else changes on the hot path.
3. **Sender takes.** Each cycle the sender swaps each non-zero word to zero and runs
   `aeron_network_publication_send` for each set bit. Clear-then-send means a mark made during
   the send is seen next cycle, not lost.
4. **Timers for the rest.** A timer wheel in the sender schedules the per-publication work that
   does not depend on new data: heartbeat (100 ms), setup while unconnected, flow control
   `on_idle` and receiver liveness, and the linger/end-of-stream checks. A NAK or status message
   handled by the sender marks its publication active, which covers retransmits and a
   newly opened flow-control window.
5. **Fallback.** A publication that sent data in its last visit stays active for the next
   cycle (it may be window-limited), so a publisher blocked on flow control is not starved.

The sender's per-cycle cost then scales with the active publications, not all of them.

## Points to settle upstream

- Log metadata layout and the CnC bitmap are client/driver protocol: the Java, C and C++ clients
  and both drivers need the change, and a version check for old clients (a client that does not
  mark must still be served, for example by keeping a full walk for its publications).
- Exclusive and concurrent publications both mark; concurrent publishers may race on the same
  word, which the atomic OR handles.
- Slot reuse after a publication closes must not let a late mark from the old publication
  wake the new one wrongly (a spurious visit is harmless; a missed one is not).
- IPC publications are not in the sender loop and are unaffected.
- The receiver has a smaller version of the same pattern: its slow tick visits every image for
  status messages and loss (5 µs per cycle at 256 images here). A subscriber-position mark
  could do the same there.
- The conductor's cycle also grows with stream count (20 µs at 256 here), but it is not on the
  data path.

## Workaround today

Multiplex logical streams over fewer Aeron streams, with a key in the message. 128 logical
streams over 1 Aeron stream each way measured 11.8 µs round-trip p50, the same as 1 stream.
