# CSS multi-value declaration layout (#7968)

Issue: [#7968](https://github.com/ubugeeei-prod/vize/issues/7968).

The existing CSS print joins authored continuation values. Restore complete
one-value-per-line layout for transition and box-shadow lists, including compact
inputs, and preserve an authored multiline list for other ordinary properties.
Vendor-prefixed transition/shadow declarations follow the same rule. Single
values, function arguments, quoted commas and custom-property token streams
retain their existing ownership. The existing nested-comment policy stays in
place. Other compact property-list wrapping policies remain outside this slice;
this implements both reported properties and the issue's minimum preservation
requirement for other authored lists.

The source uses the existing whole-token alignment pass after the existing
LightningCSS parse/print and authored-value selection. Pending continuation gap
edits commit only at a declaration boundary in a block owned by the existing
parse; nested selectors and custom brace contents cannot establish ownership.
A complete token mismatch discards every pending edit. One continuation indent
uses configured tabs/width/EOL. No CSS parse, pipeline stage or serialization is
added. Parsed transition/shadow list lengths prevent the equal-layout fast path
from skipping those lists, while unrelated equal CSS retains that fast path.

The additive, independently authored corpus contains 27 full SFC references:
original Tip/Card, compact and already-printed lists, hybrid shadow indentation,
three values/important, vendor prefixes, nested authored shorthands, selector
lists, scoped/module, comments, function/quoted/custom commas, ordinary authored background lists, tabs, width and CRLF/Auto. Rust
checks original/reference CSS parse/print semantics and three complete actual
public-API passes. The existing source observer reuses its build and records raw
options/API observations plus 22 default CLI check/dry/write/recheck controls.
No historical input/output/capture asset is changed.

Validation and delivery are pending: exact-head Actions, actual protected
historical corpus/API/CLI qualification, all unchanged instruction ceilings,
actual merge, and supported installed-CLI release proof. No speed, native
formatter migration or fix-history completion is claimed. Root owns queue
admission and release so independent slices can progress during hosted checks.
The reporter's verified public credit is
`Co-authored-by: ubugeeei <71201308+ubugeeei@users.noreply.github.com>`.

Source review adds an already-printed inline Card control and preserves parse-owned
transition/shadow list detection when color protection makes the parsed property
an unparsed var-token list. Only outer parsed comma tokens opt into the equality
guard; nested function/variable arguments retain their ownership. This strengthens
the original whole-output corpus to 27 API/22 CLI cases before hosted qualification.

## Hosted continuation/reference repair

Initial hosted source `8091637475` compiled the existing source-built CLI and
exposed intentional output overlap with #7915 continuation references and the
#7926 declaration-comma control. Keep every original issue/input/expected/carrier/
manifest/stock byte frozen. An explicit authority pins 12 complete current
references: 11 of 12 continuation cases and one of 17 rule-layout cases.
Each row binds the exact original corpus, input, historical output, options and
complete CLI plan, plus independently authored current output/hash. Only two
previously unchanged controls need new complete initial check/write streams.
Whole diagnostics, DOM/SSR, CSSOM/computed declarations, complete process bytes,
three writes/fixed points, check-no-write and denominator gates remain strict.
Reports preserve both comparisons and count current qualification separately;
historical mismatch never becomes an original match. Five complete current
outputs equal the separate pre-existing stock Prettier records. Pure authority/
input/options/history/CLI forgery controls execute locally; hosted current
runtime qualification remains required. Authority SHA256:
`8e53853840d474c0d9cd6a29eaa0ab061a8e3e46df7f1c45fd82d705e0a6220c`.

The new value state helper and parsed custom-brace refusal law move to a private
`style/values.rs` in a separate refactor commit with identical behavior. The
existing alignment/lexer file remains below the unchanged 350-line ceiling;
canonical decisions retain 350 lines. The complete current style owner hash
advances only for its private module declaration, while every original law body
and captured asset remains fixed. The generated owned census is regenerated.
Independent source review found no blocking producer/authority issue before
this hosted reference repair; fresh exact-head Actions is still required.

## Rust controls use the same current authority

Exact-head Actions `e47ff92607` passed all four tooling shards, including the
12 complete continuation and 17 rule-layout observer/CLI/CSSOM plans. Rust
shards retained their older direct expected comparisons and failed on exactly
the same intentional layouts. Route those two Rust integration controls through
the existing reviewed 12-row authority; retain every original input/output and
all three actual public SFC passes. The Rust adapter hashes the entire authority
and complete original corpus (including every CLI stream), exact input, original
expected bytes and current output, and binds original configured options. It
asserts 11 continuation and one rule-layout refinement and a separate strict
historical mismatch on every affected pass. SHA256 is test-only; production
dependencies and formatter work do not change. Hosted Rust qualification is
required on the updated source; earlier tooling success is not transferred.

## Actual-main Stack source preparation

The existing #8178 source is genuinely replayed onto signed actual main
`8c7727613de6213e0a52cde96eeb295d01c9bef5`. Preserve complete CSS implementation,
corpus, reference-authority and original source-law bytes alongside incoming
hugged interpolation source and controls. Keep all 27 API cases, three public
passes per case and 22 complete CLI plans; earlier source greens grant no
acceptance to this successor. The full Stack retains 54 API/27 CLI cases,
original300 historical cases and strict separate historical/current comparisons.
Fresh exact-head Actions, required native/protected full observer/API/CLI,
unchanged budgets, actual Stack delivery and supported installed release proof
remain pending. The common canonical union records this preparation separately
from projected documentation and historical runtime evidence.

## Recovery on the actual worker-metadata repair

Refresh the existing Stack from frozen source `227912e35dc1d49e5a9c8bc71054d20a6b0298c8`
by a genuine merge of signed actual main
`b1b9895e2132021bad64b6452df502bf4f50f033`. Preserve the complete CSS
implementation, original width and layout controls, authored punctuation, all
twelve current reference entries (eleven continuation and one rule-layout),
and complete historical/current output and CLI streams. Retain incoming pinned
formatter vendor, Cargo integration, and bounded worker-metadata helper and
workflow bytes. No expected vector, original law, budget or runtime cap changes.

The earlier top candidate `19030b67372ce2ff07b26a2ffac01b2fd2910ed9`
failed its report after observing an incomplete shard-upload step, despite four
successful workers. That result is not a merge or a green qualification. The
actual-main repair supplies bounded fresh metadata reads; its effect on this
Stack still requires new exact-head source Actions and a new protected
candidate. Retain all 27 API/22 CLI cases, the full Stack's 54 API/27 CLI cases,
all original 300 historical cases and 29 controls, native execution and
instruction budgets. Actual signed delivery and installed release qualification
remain pending. New recovery links use canonical row 347 while its entire
original prefix and every other main row remain unchanged.
