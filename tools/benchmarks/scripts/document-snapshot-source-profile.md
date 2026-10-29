# Document snapshot source counters

The two instrumented producers are DocumentStore::uris and DocumentStore::vue_texts. Each method records one entry call, a synchronous span around exactly its original traversal/clone/sort work, and post-span counters for returned items and serialized URI UTF8 bytes. vue_texts also records returned text UTF8 bytes.

Returned item/byte counters emit one observation per completed method call; their units sum the payload. An empty output has one observation and zero units. URI units include percent escapes in Url::as_str. Text units count UTF8 bytes, preserving CRLF; they are not UTF16 positions or Unicode scalar counts. vue_texts reports returned lowercase .vue entries rather than all scanned documents.

The post-span summation loops are profiling macro operands. They compile away in normal builds, together with counters and clocks. Instrumented runtime-disabled builds can still execute those operand loops, so their overhead must be measured separately from normal product time. Collector registration and readback allocations are excluded by the existing profiler contract.

Logical payload volume is not physical memcpy, requested allocation traffic, live heap or RSS. The synchronous span separately receives current-thread allocation deltas; other threads/backend processes are outside that span. Counters aggregate instrumented Rust threads and must be read only after producers stop. An unexecuted source site establishes no fact about other snapshot paths.

The bounded carrier first observes an empty store, then five documents through four concurrent readers: three lowercase Vue URIs, a TypeScript URI and an uppercase .Vue URI. It retains an empty text, percent-escaped URI, Japanese/emoji UTF8 and CRLF. Exact observations are 20 URI items/500 serialized bytes and 12 Vue items/336 URI bytes/68 text bytes. A separate edit/rename/close phase expects 94 total URI bytes, 46 Vue URI bytes and one text byte while earlier snapshots remain owned.

The tiny Actions carrier compares normal results with profiling-only exact counts and exercises the existing disabled-operand law. Real LSP profiles and normal timing pairs follow only after that carrier passes. The source hooks preserve store fields, revisions, guard lifetimes, async boundaries and the returned ordering policy.
