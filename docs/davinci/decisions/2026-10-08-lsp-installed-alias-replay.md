# Original alias transactions on the public npm provider

An installed provider must satisfy the same finite #8011 contract as the source
CLI. Prepare a separate runner without changing the stock Rust helper, sources,
expectations or independent goldens. Preparation executes no public Vize/native
provider and assigns no publication, installation or source-inclusion identity.

Paired decision: [#8011 comment](https://github.com/ubugeeei-prod/vize/issues/8011#issuecomment-6054726482).

## Required authorities

The runner consumes three independently reviewed receipts and their exact SHA256
values. Missing, stale or altered authority fails before launching a provider.

| Receipt                     | Required evidence                                                                                                                                                                                                                                   |
| --------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Signed source delivery      | Raw GitHub #8262 PR and commit records: actual merged state, main base, merge time, exact merge SHA and valid signature. That SHA must be an ancestor of both public C and H.                                                                       |
| Public npm installation     | Actual Node, CLI wrapper/distribution, native path and successful original loader return; immutable C/H/tag/run; registry provenance, lock integrity, and every installed routing JS/native/Corsa file equal to integrity-verified public archives. |
| Original project dependency | Vue 3.6.0-rc.6 / TypeScript 6.0.3 and the exact 26-package graph from original c2bc Playground/lock/helper bytes; fresh separate npm installation, matching registry integrity and whole graph/payload custody.                                     |

The public installation schema is `vize-public-registry-install-v1`. Its bounded
`vize-public-registry-payload-files-v1` manifest identifies every ordinary public
package member. The runner rechecks each file digest, size, ownership and complete
file set before and after each session, including `@vizejs/native` routing files
and the public optional Corsa platform package. A similarly named installed
package or resolution-only native path is insufficient.

Launch the authenticated absolute Node with explicit `--require` of the reviewed
read-only custody hook and the installed `vize/bin/vize lsp --stdio`. The hook
calls the original Node `.node` loader with unchanged arguments and returns its
unchanged result. Acceptance requires the expected native loader actually to
return with unchanged path/hash and the CLI-selected public bundled Corsa path.
The transport separately requires normal complete shutdown. The nine initial
Node/native/version/Corsa override variables must be empty. The original explicit
workspace `corsaPath` is absent from the installed project configuration; the
public CLI configures its own authenticated bundled runtime. All other original
project and tsconfig options remain unchanged.

The source-delivery receipt shape is:

```text
{ schema: "vize-signed-source-delivery-v1",
  repository: "ubugeeei-prod/vize",
  pullRequest: <whole actual REST PR 8262 record>,
  commit: <whole actual REST merge commit record> }
```

## Frozen whole transactions

Load the authenticated historical c2bc receipt and all 32 complete compressed
packets. Verify each gzip and decompressed raw SHA before parsing. Read only the
independently authored `expected`; captured `actual` never becomes a new oracle.
Authenticate the original E source and eight supplemental source/golden files by
independent SHA. Preserve 26 transaction sessions plus six definition sessions,
all origins, LF/CRLF and whole query order. Bind only each explicitly identified
authored E URI in frozen expectations to its canonical fresh project URI. Reject
unknown expected URI values or WorkspaceEdit keys; never normalize actual data.

Each fresh session retains the whole native TS2322 diagnostic and independent
clean repair. Apply the actual returned whole WorkspaceEdit to original UTF-16
coordinates, write/read the disk result, and retain exact version-2 diagnostics
before independently installing the complete authored golden for version 3.
Application errors and mismatches remain visible. Preserve complete empty
publication arrays, every raw client/server/stderr byte, parsed whole response
and notification, partial phase observation, deadline and terminal outcome.
Failed initialization and later requests also retain transport and native
journals. All 32 sessions must succeed; no expected-edit substitution, partial
result normalization, source binary or workspace Corsa fallback is permitted.

## Reproduce the original project dependency

Use `prepare-vue.ts` with explicit absolute Node/npm paths and their reviewed
SHA256 values, a canonical repository path and a fresh output directory. It
loads the fixed c2bc git files, installs only the exact source-locked 26-package
Vue closure with scripts disabled, and verifies registry integrities and complete
payloads. The generated `vize-stock-alias-vue-install-v1` receipt stays separate
from the public Vize lock. Every session rechecks that fixture graph before and
after running; the only project dependency link points to this owned Vue root.

The current private fixture preparation authenticated Vue 3.6.0-rc.6 and
TypeScript 6.0.3, with all 26 npm integrity values equal to the original c2bc
lock. The whole graph SHA256 is
`b475054f45cd1d8bae786b3b7bb1e2cc92c9ccb508e86a4ed6e94d421b326b96`.
Independent disposable-copy controls rejected modified runtime payload, lock and
receipt, unowned dependency symlink and substituted Vue version. This proves
fixture custody only; no Vize/native provider executed. The stale shared
Playground beta.10 link and unrelated stable/warm-resource Vue version grant no
original32 authority.

## Run after actual source and public delivery

From a checkout containing the original history and runner:

```sh
<authenticated-absolute-node> tests/tooling/lsp-installed-original-alias-replay.ts \
  --authority <absolute-reviewed-public-receipt> \
  --authority-sha256 <actual-public-receipt-sha256> \
  --source-delivery <absolute-reviewed-signed-delivery-receipt> \
  --source-delivery-sha256 <actual-delivery-receipt-sha256> \
  --vue-authority <absolute-reviewed-stock-vue-receipt> \
  --vue-authority-sha256 <actual-vue-receipt-sha256> \
  --output <fresh-canonical-absolute-output-directory>
```

The ordinary hosted tooling entry
`lsp-installed-original-alias-replay.test.ts` admits the complete framing,
application, frozen-case, native-journal, public-payload and Vue-custody
falsification suite. Its success is helper validation. Actual installed32,
actual #8262 inclusion in C/H and complete source/protected execution remain
mandatory for #8011 closure. This finite alias replay grants no imported/complex
alias or stock ContentMapper parity credit.

The first ecfe hosted `check-js` rejected snapshot spread and default object
stringification of a nested error cause. Preserve its complete authenticated
job log and receipt in `supplemental/historical-ecfe-helper-lint`. Retain the
necessary publication wait snapshot with `.slice()` while removing matching
waits, and record nested/circular causes with unbounded Node inspection. The
zero-warning gate, whole packets, sources/goldens and terminal custody are
unchanged. All 31 falsification laws and strict TypeScript checking pass after
the bounded correction; fresh hosted lint is mandatory. Paired decision:
[#8011 comment](https://github.com/ubugeeei-prod/vize/issues/8011#issuecomment-6055019121).
