# Skip absent ASCII symbol queries

Issue: [#3952](https://github.com/ubugeeei-prod/vize/issues/3952).

## Original failure

Release candidate 0.447.0 failed the original Misskey churn check in
[job 114666493185](https://github.com/ubugeeei-prod/vize/actions/runs/38205021668/job/114666493185).
Its first file lifecycle sample reached 133,040 KiB, or 129.921875 MiB,
above the unchanged 128 MiB server limit. The preceding forty edit cycles
remained below that limit. The sample follows the complete create, symbol,
close, delete and document fences; it is not a mid-build allocation sample.

The [retained original archive](./fixtures/3952-misskey-lifecycle-rss/original-447-metrics.zip)
contains the complete raw metrics and summary. Its declared CI build used
`cargo build --profile ci -p vize --features legacy`, source `ef34d1a0`,
and binary SHA-256 `76092afa8137fefa2cefb505a686188770daa62ea38d0bf38c806c38f7f3cc2c`.
The uploaded transport observations omit the actual churn owner 84670,
so they do not establish a full raw-wire or allocation callsite attribution.

## Decision

The first lifecycle invokes workspace-wide symbol search for the first time.
The ordinary JS/TS collector currently parses every file before matching
authored declaration names. Skip parsing only when a nonempty ASCII query
of at most 32 bytes is absent, ignoring ASCII case, from wholly ASCII source
without backslashes. Accepted identifiers in this case are literal source
slices. Unicode, escapes, longer and empty queries retain the original parser.
The length bound limits the extra scan and does not limit search results.

Keep all existing syntax rejection, declaration kinds, order and UTF16 ranges.
Six complete-result controls cover ASCII matching, decoded and Unicode names,
query boundaries, ignored AST nodes, malformed input and prior vector contents.
The finite source qualification binds both changed native bodies and all
sixteen unchanged parser, caller and original workload bodies.

## Acceptance

The eligible file fraction and RSS benefit are unmeasured. Require ordinary
Actions and the same original forty edit cycles, sixty-three RSS samples,
128 MiB server, 384 MiB process tree and three-process limits. Keep source,
recipe, binary and complete workload receipts distinct from the historical
failure. A passing gate alone does not establish allocation causality or
the type checker's 10x target. Add no optional diagnostic campaign or RPC.
