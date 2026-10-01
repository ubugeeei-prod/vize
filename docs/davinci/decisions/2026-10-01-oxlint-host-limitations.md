# Direct Oxlint Vue host limitations (#7004, #7005)

Assessment on 2026-10-01: the direct Oxlint JS-plugin host still supplies extracted
script programs rather than a whole SFC. A plugin cannot put its diagnostic
outside that program or dispatch a Program visitor for a template-only SFC
which the host never gives it. The upstream [API support
page](https://oxc.rs/docs/guide/usage/linter/js-plugins.html#api-support) still
lists custom Vue formats and parsers as unsupported.

The empty-script RangeError is already fixed by #7155 using a Program-node
fallback. #7130 preserves authored SFC locations through the `oxlint-vize`
wrapper and verifies JSON spans for ordinary, empty-script, and scriptless SFCs.
Those changes are present on current main. They do not establish correct spans
or template-only dispatch in direct `oxlint` or its editor integration.

Keep #7004 and #7005 open with their existing upstream-blocked status. Users can
run `oxlint-vize`, native `vize lint`, or Vize's native Vite+ integration. Before
closing the direct-host issues, reproduce with supported current Oxlint and
verify a real direct-plugin JSON span on the authored template node, an empty
script without a crash, and a template-only diagnostic without injecting or
rewriting authored source. Do not treat an unsupported host as a clean lint run
or count a wrapper as direct-host acceptance.
