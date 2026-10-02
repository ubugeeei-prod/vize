# Formatter history: strict Clippy repair

Issue: [#6882](https://github.com/ubugeeei-prod/vize/issues/6882).
This change repairs source lint failures exposed by the bounded engineering
control campaign. It changes no product route, historical expectation,
original audit or native acceptance state.

## Actual failed attempt

Canonical diagnostic source `00f745b4791debc2d4b4e3e6137672a4a3668492`
ran in [36960693701](https://github.com/ubugeeei-prod/vize/actions/runs/36960693701),
attempt 1. The whole workflow and job `110693502427` failed. The actual
`cargo clippy --locked --profile ci -p vize_glyph --all-targets
--message-format=json-render-diagnostics -- -D warnings` process exited 101,
with no signal or process error. Its complete stderr is 20,585 bytes,
SHA256 `4afe93ad5bbca7fa4a6e7a47fffd7b86257660c488259cc772d9d53cfa2b4fea`.
The JSON stdout records unsuccessful build completion; it contains no
compiler-message rows, so the retained stderr supplies the actual diagnostics.

Read-only transport
[36967112656](https://github.com/ubugeeei-prod/vize/actions/runs/36967112656)
and job `110713156060` succeeded at
`ac1a8203667fac6f1027f277d26abffa10d65b63`. Authenticated artifact
`11207154278` has service digest
`3a32e9b1199c84a4eb0d86bd58a046ad2295a15aa577a1e4a1aa4a5f7435fca0`.
The transport retains all 1,048 file inventory hashes and 1,031 complete
non-executable files through 339 frames, with zero missing files or binary
binding errors. Frozen binaries are independently rehashed and receipt-bound;
their literal executable bytes and ZIP bytes are not transported or executed.
The exact 4,114,839-byte log has SHA256
`3cf7e58202bab3dab8b5ca6c4c1f523a003e004a5b68d72ed5d1a1c28b474cef`.
A GitHub log-framing UTF-8 BOM before frame 173's timestamp is removed only in
a separately retained decoder copy. Capsule bytes remain unchanged and match
the complete gzip, JSON and individual file hashes.

The failed attempt and its original source, plan, authority, raw reports,
receipts and artifact remain immutable. Recovered observations qualify only
that source. They do not prove this repaired source, all 24 controls, a
performance budget, protected checks, merge or fix-history closure.

## Bounded source repair

The diagnostics contain 21 errors: seven in the observer, four in the import
sorting integration tests, eight from snapshot macro expansion and two from
an unused test-only directive adapter.

- The observer uses the existing CompactString formatting API for four error
  conversions and one interpolated message. Checked JSON object insertion
  replaces two potentially panicking string indexes. Options and complete
  error/output expectations remain unchanged.
- The import-sorting helper returns `Result` and propagates its three errors.
  Existing assertions and literal expectations remain in `#[test]` functions
  under the unchanged workspace test policy. Error display uses CompactString.
- Eight authored-content tests compare the complete payload of their existing
  literal `.snap` files. None of those files is regenerated. The CSS comment
  case compares the complete snapshot payload after adding exactly the snapshot writer's
  one LF. Its immutable source-built byte reference already records this
  56-byte output, while the serialized snapshot payload is 57 bytes.
- The default-Vue test-only `normalize_attribute` wrapper and re-export have
  no remaining callers anywhere in Glyph source or tests. They are removed;
  the production `normalize_attribute_with_vue_version` body is unchanged.
  Original wrapper source remains in immutable Git history and control proofs.

No lint allowance, policy, gate, budget, original classification, corpus
input/options/expected/error byte, capture receipt or historical source pin
is changed. The repaired observer and test-owner hashes are new source
identities; old observations are never relabeled as executions of this change.

## Original source witness

Replacing the eight snapshot assertion macros changes the test owner's source
hash, while its original captured-source pin must remain unchanged. A 3,907-byte
non-executing `.txt` asset preserves the exact original Git source from
`cc87bb5960ea9e49e82672205df919de58bb4b24`, SHA256
`9affca9435fc96cc470bf07840c71ed17844629c3361749484c893269855fa5d`.
The only admitted repaired owner is SHA256
`f6b9060822826c8a3942ce9a7c40f5876bf82fb7b2bfef81f10ff3c68aec8ec0`.
The API source-artifact loader and S007 current-law validator require this
specific old/current binding, all eight raw function hashes, unchanged
input/options prefixes and fixed-point suffixes, and all eight full snapshot
hashes. S007 also requires its exact two original revision pins and registered
function names. Every other owner retains its existing strict validation.

This witness requires no runtime Git history or fetch, and supplies source
authority only. The observer, build receipt and campaign source identity still
read and execute the actual current source. Unknown or missing owners,
functions, source assets and snapshots, changed pins, bodies and snapshot bytes,
and escaped symlinks are rejected, including in a checkout without `.git`.
Original corpus manifests, expected bytes, 119 source references and 28 prior
campaign control pins remain immutable. The old failed attempt remains failed.

The assertion gate rejected a redundant `ends_with` assertion in the first
private repair. That check is removed: equality of the complete actual output
plus one LF against the full 57-byte snapshot already requires the exact
56-byte output without a terminal LF. The original policy and complete oracle
remain unchanged. The Glyph consumer inventory records the changed test import
and removal of the unused adapter. New source contracts require a hosted run.

## Required delivery

The repair must precede prepared/literal/completion consumers in the native
GitHub Stack, so every logical layer inherits compilable observer source.
Each logical head still requires its own Actions and protected queue checks.
New source-built 300-case API and five-scenario CLI captures, selected Rust
law Cargo JSON/binary receipts, all eight repaired test functions, strict Clippy
and all engineering-control
evidence remain required. Applicable performance measurements and the new
#7258 API/config/CLI history remain unfinished. Native handled/equivalent and
whole-fix closure counts remain zero; #6882 stays open.
