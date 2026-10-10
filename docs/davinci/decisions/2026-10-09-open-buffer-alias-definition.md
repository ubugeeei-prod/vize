# Component definitions for open alias targets

Trackers: [#3952](https://github.com/ubugeeei-prod/vize/issues/3952) and
[#3957](https://github.com/ubugeeei-prod/vize/issues/3957).

Paired decisions: [editor support](https://github.com/ubugeeei-prod/vize/issues/3957#issuecomment-6064331257)
and [LSP correctness](https://github.com/ubugeeei-prod/vize/issues/3952#issuecomment-6064332029).

A component imported through a configured TypeScript path alias must remain
navigable when its dependency exists only as an editor-open buffer. The
component-tag definition returns the same complete authored target URI and
zero-length file-origin range as a relative import. Closing an absent target
removes that definition; saving and deleting the target retain the existing
filesystem behavior.

## Reproduction

The independent before observation used an existing genuine native CLI built
from archived `5ed1feff6b2a2b74905bb14b46f970906679a511`, with structural editor
features enabled and type checking disabled. A single session opened
`components/Child.vue` without writing that file. Relative component definition
returned its complete scalar Location, while the equivalent `@ui/*` import
returned `null`. The process shut down successfully and its binary digest
remained unchanged.

The [standalone corpus](../../../tests/_fixtures/differential/lsp/open-buffer-alias-definition/case.json)
retains the complete original before receipt, raw client/server frames and
stderr under `historical-5ed/`. Its source comparison records ten definition,
document-store, handler, configuration and package-manifest object families
equal to actual main `7d645dbca9d3be2e0c96feae7cd826f0f49b02fa`. Unrelated CSS,
completion, constructor and workspace-lock differences do not make that
archived binary a current-main build. This observation does not claim an
external Corsa startup or public-package execution.

The two immutable raw wire captures are classified as binary by exact names in
their fixture-local `.gitattributes`. Their CRLF `Content-Length` framing is
preserved byte-for-byte; no global attribute, wildcard or packet normalization
is introduced. The first staged whitespace check mistook that framing for text
trailing whitespace. The original Rust-runner stdout remains reviewable text;
only its exact `full-corpus/test.log.txt` name unsets the whitespace attribute,
retaining genuine progress-prefix spaces and the final blank line. Source files
have no whitespace exception. All 18 stored corpus hashes remain unchanged.

The same frozen real-CLI regression consumer subsequently ran both LF and CRLF
against that unchanged archived binary. All 28 complete responses were retained:
24 controls matched, while the open alias and open preferred-target phases
failed in each session. Both servers exited successfully. Only the test consumer
was compiled against the retained binary's existing dependency fingerprints; the
product was neither patched nor rebuilt. Complete packets, stderr, compile
identities and failures remain under `historical-5ed/full-corpus/`.

## Resolution and request cost

Only the private import resolver and the existing component/import definition
consumer change. The state-aware resolver runs the complete original disk probe
for each matched alias target first. A saved later-extension `.vue` candidate
therefore retains authority even if an earlier `.ts` candidate is open only in
the editor. A saved result performs no additional URI construction or document
lookup; an injected callback law checks that the saved later-extension path
invokes the open-file callback zero times.

After all disk candidates for that target miss, the same candidate order checks
the borrowed document store. Each candidate is normalized once and tested by
its URI key. There is no document enumeration or text cloning, new cache,
request-time network operation, parser pass, serialization or provider stage.
Existing alias pattern and target ordering, bounded re-export traversal,
relative imports, implicit Nuxt/project aliases and filesystem-only consumers
retain their behavior. Suffix-wildcard and exact-key precedence work is outside
this change.

## Regression contract and delivery

The corpus authors complete scalar Locations and exact `null` results before
provider execution. Native handler tests and real `vize lsp` stdio tests cover
absent, open-only, closed, saved and deleted dependencies, including the
relative positive control. Separate target-array controls cover a saved
fallback, an open first target and fallback restoration after close. LF and
CRLF inputs use UTF-8 filenames and spaces, brackets and `#` in file URIs.
Stdio tests compare complete JSON-RPC response envelopes, exact diagnostic
barriers and successful graceful shutdown. The corpus has a dedicated consumer
because the generic differential LSP manifest does not represent this lifecycle.

Full source Actions and protected delivery remain required. Existing performance
inputs, complete outputs, recipes and numeric ceilings remain unchanged. This
bounded structural-navigation correction does not complete the broader tooling
trackers or claim publication.

The original-400 source qualifier admits only the six reviewed Rust producer and
test paths as literal entries at its existing set's tail. Four empty spacer rows
are removed to keep the qualifier at 350 physical rows; every existing statement,
token, allowed path, CSS import and spread remains. No wildcard, new helper,
workload, output, build recipe or numeric ceiling changes.

The first source head `8551c921` failed
[Check 37809518308](https://github.com/ubugeeei-prod/vize/actions/runs/37809518308/job/113422983388)
before the new regression tests executed: strict Clippy rejected the handler
test's `std::format!`. Replacing only that macro with the existing approved
`vize_l0::cstr!` retains its authored interpolation, newline conversion, complete
expectations and all product/fixture bytes. The full failed log is retained
(495,068 bytes, SHA-256
`d480f7b37332996212c35ffa7d64426105252471d0d7558c04864c1e182d07ba`);
fresh complete successor Actions remain required, with no failed-build runtime
credit or lint exception.

The existing Draft's next exact source `5d91bcb5` failed
[Check 37810962192](https://github.com/ubugeeei-prod/vize/actions/runs/37810962192):
the generated LSP consumer inventory omitted the new handler's `vize_l0`
test/dev reference, and two URI substring assertions violated the exact-oracle
policy. The bounded successor retains every product and frozen corpus byte,
adds the single generated row, and compares both complete target URIs with
independently authored UTF-8 percent-encoded filename literals. Existing whole
response, lifecycle, diagnostic and shutdown expectations remain unchanged.

This source is prepared from actual signed main
`26031a4fbb30a1e86511919bafc7035b08440ef3`. Local policy/URI authoring checks
do not qualify current native LSP execution, protected delivery or publication;
fresh exact-head Actions and the original unchanged performance gates remain
required. This adds no release prerequisite and closes neither tracker.

The final metadata-main transplant uses actual signed release-integration main
`4c2bebb9a586e3eaab698b41e5b58ed5f8181852` and preserves all 37 incoming
version-metadata files. The prior `cfce1724` source's
[original-400 qualifier](https://github.com/ubugeeei-prod/vize/actions/runs/38015772349)
passed against its own main26031 common ancestor; that historical receipt does
not qualify this transplant. Frozen corpus identities, all 18 hashes, 28 whole
before responses and every original performance condition remain unchanged.
Fresh hosted source/native checks and the unchanged original-400 protocol must
qualify this exact successor against its genuine main4c2bebb common ancestor.
Both trackers remain open; protected delivery and released-consumer verification
remain pending.
