# Authored slot member hover documentation

The existing full Check at source `53966b1c` (production base `143c1d4a`)
failed the unchanged real VS Code host assertion that `names.current` exposes
`**Primary** invoice slot` Markdown. The original `SlotAuthoring.vue` uses
`#[names.current]` and retains the authored JSDoc on its `current` property.

The authentic failed editor job is
[112081075009](https://github.com/ubugeeei-prod/vize/actions/runs/37405188567/job/112081075009).
Its complete 224,232-byte raw log has SHA256
`8dc84e43a2b6dcf04c7232c399dbb5d23c56d9da7fb5ee99bb85aa27d0053e38`.
The original small failure artifact preserved configuration and tracing, but
omitted SlotAuthoring/RichAuthoring files and the returned Hover objects.
Therefore it does not establish whether the answer was empty, had incorrect
Markdown, or failed at another native/provider boundary.

The bounded diagnostic change records the original source buffer, version,
language, URI, unchanged query/expected documentation and every returned SDK
Hover object before the existing assertions. It preserves the raw serializable
result and all public Markdown fields; this is a VS Code provider result,
not a captured Corsa protocol exchange. The existing failure collector retains
this packet plus all five unchanged rich-authoring fixture files. All original
requests, Markdown checks, definitions, completion checks and timing remain.
No production source, provider, fixture, expectation or workflow changes.

One meaningful existing full Check on this frozen passive-capture source is
necessary to inspect the actual result. Native cause, any production repair,
legacy corpus qualification, protected merge and release remain unfinished.
This integration observation is recorded on LSP fix-history issue #6883 and
must not be attributed to the inlay or compiler change from ancestry alone.

The paired issue record is [comment 6008430599](https://github.com/ubugeeei-prod/vize/issues/6883#issuecomment-6008430599).
