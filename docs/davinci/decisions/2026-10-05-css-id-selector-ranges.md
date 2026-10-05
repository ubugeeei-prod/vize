# CSS ID selector diagnostic ranges — 2026-10-05

Issue: [#7981](https://github.com/ubugeeei-prod/vize/issues/7981).
Paired decision: [issue comment](https://github.com/ubugeeei-prod/vize/issues/7981#issuecomment-5993981503).
Byte custody: [issue comment](https://github.com/ubugeeei-prod/vize/issues/7981#issuecomment-5994107580).
CI repair: [issue comment](https://github.com/ubugeeei-prod/vize/issues/7981#issuecomment-5994214970).
Help authority: [issue comment](https://github.com/ubugeeei-prod/vize/issues/7981#issuecomment-5994391973).
Prior reports: [#7055](https://github.com/ubugeeei-prod/vize/issues/7055)
and [#7171](https://github.com/ubugeeei-prod/vize/issues/7171).

The accepted ID components currently receive the style block's content start
and decoded ID length. Different selectors collapse onto that same position;
escaped identifiers also lose their authored width.

Keep the existing Lightning CSS parse and its accepted AST rule/selector walk.
Use each style rule's existing location, whose columns are UTF-16 units, to
locate its original UTF-8 bytes. The already pinned cssparser tokenizer queries
only that rule's prelude. Match its decoded ID tokens within each comma entry
to existing ID components, consuming distinct occurrences in matching order.
Report the complete original token range, including escape terminators.
This adds no stylesheet/selector parse or pipeline stage.

Token blocks skip attributes and functional/deep arguments. Consume a skipped
block immediately so the following token's start is measured after its closing
boundary. Comments, strings, colors, URLs and escaped class hashes do not
become ID findings. Existing functional-pseudo finding scope, nested traversal,
severity/help and config policies remain intact. Existing suppression now uses
the true selector line through the unchanged diagnostic filter.

The corpus preserves all 248 bytes of the original #7981 SFC and every byte of
its JSON config. Its whole findings select `#main-banner` at `12:1–12:13` and
`#banner-text` at `17:3–17:15`. Original #7055 flat/nested files and #7171's file
also remain whole and unchanged. Authored controls are explicitly labeled.
Sixteen SFCs cover LF/CRLF, astral text, multiple inline styles, escaped IDs and
escaped CRLF, repeated IDs and comma lists, nested media/supports/layer rules,
comment/string/attribute boundaries, functional/deep scope and disable comments.
The corpus's scoped Git attributes preserve complete `.txt` input bytes across
platform checkouts, including the two authored raw CRLF controls.

The public Rust API test compares every diagnostic field, complete selected
source bytes and complete JSON/plain reports. The existing source-built CLI
tooling suite validates its current build receipt, writes the original config
unchanged, and retains all 33 actual attempts for complete JSON/plain output
plus the original JSON repeat. No missing binary, failed case or partial
observation can qualify. These are legacy CSS product observations; no new
native-stage eligibility or historical corpus rebaseline is inferred.

The first current Check found an unawaited Node test registration. Await that
promise while preserving all 33 complete attempts and assertions. Retain the
same complete API rows in the existing nextest pr/full result envelope as well
as `target/differential/`, because Rust workers upload that envelope. This adds
no workflow or execution stage; production and all vectors stay unchanged.
Fresh successor execution remains required.

The first original CLI wire receipt also caught an incorrect new expectation:
the existing CSS rule help survives `--help-level none` through the unchanged
direct diagnostic adapter and JSON/plain formatters. Official artifact
11343864947 retains the full original command/output at compiled source
`b87a30c35e6d74cf9acc079bf65874d3854bd8f8`; both corrected selector ranges are
exact. Restore the established help in all complete API/JSON/plain vectors.
Every source/config byte, range, count/order and strict whole assertion remains
exact. This failed first-case receipt does not qualify all 16 API cases or
33 CLI attempts; the successor must execute them all.

TODO: frozen independent source review, fresh exact-source ordinary Actions,
all 16 API and 33 CLI observations, protected full Rust/all104, actual signed
reporter-credited merge and the next supported release remain required.
No local native build or large install is used. The separate Zed #8028 source
and receipts are retained, and rejected logo #7850 remains closed and untouched.
