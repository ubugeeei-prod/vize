The page and `tsconfig` pin authored inputs for #10518. The manifest contributes
two complete explicit `number` declaration hovers as source-inspected preservation
controls. Its generic adapter does not install the experimental provider, so it
is not evidence that typed route params work.

`tests/tooling/lsp-experimental-router-params.test.ts` is the runtime oracle. It
installs the exact public Router 5.4.0 project through
`prepareExperimentalRouterProject`, runs the real route generator, verifies the
entire generated declaration, then exercises the actual stdio server with the
default ecosystem diagnostics enabled. It compares complete diagnostics,
completion items, all five lazy completion resolutions, hover, definition, and
reference responses across LF/CRLF Unicode edits and repair, plus independent
project opt-in and watched configuration replacement.

`public-0.562.0-canonical.json.gz` preserves the original public-server reference
packets and timings with ecosystem diagnostics explicitly disabled. This control
shows that the canonical types and navigation were already available. Public
0.562.0 with ecosystem diagnostics enabled fails the dedicated test on five false
query/hash warnings and returns only the filename-derived string `id` completion.
The test never disables ecosystem diagnostics to pass.

The three expected JSON files retain every response field. Only allocated
workspace and private native session/project address prefixes become `$PROJECT`
and `$VIRTUAL`; authored ranges, revisions, generated-file offsets, candidate
order, details and documentation remain exact. The source specialization of
`definePage` adds the independently counted 35 ASCII bytes
`<"src/pages/products/[id=int].vue">` before the completion location. Those three
numeric offsets are the sole adjustment to the preserved public control packets;
the source-built runtime must confirm them. The archive remains unchanged.
