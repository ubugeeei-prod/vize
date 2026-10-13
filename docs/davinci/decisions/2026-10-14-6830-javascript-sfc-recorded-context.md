# Read the JavaScript SFC source context from its recorded bodies

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

`authenticateJavaScriptSfcSource` compared all 88 files named by the
JavaScript SFC source context, including product sources such as
`crates/vize_canon/src/batch/virtual_project/project.rs` and
`crates/vize/tests/support/event_completion_project.rs`, byte for byte with the
bodies recorded in `javascript-sfc-source-context*.json`. The native
lexical-dollar packet also copied those files. Any edit to one of them failed
the `native-lexical-dollar-receipt` tooling test and the native phase evidence
job, and deleting one (for example a committed plan shard) failed the copy.

The rows describe the tree recorded when the context merged, which never
changes. The historical views are now built from those recorded bodies, the
live tree is no longer pinned to them, and the native lexical-dollar packet no
longer copies the files the context names. The packet still copies and
verifies its own helpers, the lexical-dollar Rust sources and the corpus, and
the producer and native Rust test phases still run. The receipt tests pass
with neutral edits to both product files above.
