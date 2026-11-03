# Draft comment for aeron-io/aeron#2213

I can reproduce this deterministically on **1.53.3** with AddressSanitizer;
**1.51.0 does not reproduce**. The reproducer is a single C file plus a
Dockerfile (attached / in this folder). It starts an embedded C media driver,
uses loopback UDP, and aborts with:

```
ERROR: AddressSanitizer: heap-use-after-free
READ of size 8
  #0 aeron_int64_to_ptr_hash_map_get
  #1 aeron_data_packet_dispatcher_remove_matching_state
  #2 aeron_receive_channel_endpoint_on_remove_matching_state
  #3 aeron_driver_receiver_on_remove_matching_state
  ...
freed by thread T4 here:
  #2 aeron_receive_channel_endpoint_delete
  #3 aeron_receive_channel_endpoint_entry_delete
  #4 aeron_driver_conductor_on_check_managed_resources
  ...
```

## Root cause

`aeron_driver_conductor_on_create_publication_image()` stores raw
`endpoint`/`destination` pointers in an asynchronous driver command and asks
the NativeResourceAgent to map the image log buffer. No reference is taken on
the endpoint (`image_ref_count` is only incremented after
`aeron_publication_image_create()` succeeds, i.e. after the map).

While that command is `RUNNING`, `aeron_driver_conductor_process_driver_commands()`
does not read new commands from the receiver-proxy queue, so the receiver's
`on_receive_endpoint_removed` notification waits.

If the last subscription for the endpoint closes in that window,
`aeron_receive_channel_endpoint_try_remove_endpoint()` sees all maps empty and
`image_ref_count == 0`, marks the endpoint `CLOSING`, and sends
`remove_endpoint` to the receiver. The receiver removes the transports and
notifies the conductor. When the map completes,
`aeron_driver_conductor_execute_create_publication_image_create_image()` sees
`CLOSING` and queues `on_remove_init_in_progress` for the same endpoint to the
receiver — *after* `remove_endpoint`. The conductor then processes the delayed
removal notification and the end-of-life check frees the endpoint. The receiver
later processes `on_remove_init_in_progress` on freed memory.

This is a 1.52.0 regression from *"Move log buffer creation/removal out of the
conductor thread"*. In 1.51.0 the image creation (including the raw-log map)
runs synchronously on the conductor, so the endpoint cannot be marked
`CLOSING`/freed between the status check and image creation.

The same window explains the other receiver-side crash in the report
(`aeron_data_packet_dispatcher_remove_publication_image`): it is the same
UAF with a different queued receiver command. `try_remove_endpoint()` also
doesn't guard against an endpoint that is already `CLOSING`, so a duplicate
`remove_endpoint` is possible (`remove_poll_transports` crash).

## Suggested fix

Pin the endpoint (and destination) for the lifetime of the async
image-creation command, e.g. a `command_ref_count` (or a carefully managed
`image_ref_count`) incremented before scheduling the NRA map and released on
every exit path. `try_remove_endpoint()` must consider it; after release it
should call `try_remove_endpoint()` again so an endpoint whose subscription
went away during the map can still be closed.

Additionally, either make the final endpoint free receiver-ordered (receiver
owns the delete after it drains commands up to `remove_endpoint`, or use an
ack/sequence barrier), or at minimum make `try_remove_endpoint()` idempotent
and avoid queueing endpoint commands once the endpoint is `CLOSING`.

## Reproducer

```
docker build --platform linux/amd64 -t aeron-2213-repro .
docker run --rm --platform linux/amd64 aeron-2213-repro
```

The repo has two test hooks that only widen the race (they do not change
driver logic):

* `raw_log_map_func` pauses image log-buffer mapping.
* An incoming interceptor blocks the receiver thread once, so the conductor
  frees the endpoint while a receiver command is still queued.

Without the hooks the same window occurs naturally with
`term-length=64m`, `term_buffer_sparse_file=false` (the NRA touches ~192 MiB)
and a mass subscription close, as described in the issue.
