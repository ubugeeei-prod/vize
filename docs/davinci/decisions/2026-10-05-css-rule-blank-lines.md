# Preserve authored CSS declaration groups (#7826)

Issue: [#7826](https://github.com/ubugeeei-prod/vize/issues/7826).
Paired [source decision](https://github.com/ubugeeei-prod/vize/issues/7826#issuecomment-5987109062).

Actual main `d1a25ec1da2efca98534520ff8ebe84d34708e3d` retains the existing
LightningCSS parse/print, color protection, selective fixed-point stabilization
and authored-token fallback. The original Flat.vue loses a separator in normal
printing; Nested.vue's animation and `.5` values select the layout fallback,
which also drops separators. This is a source explanation of the reported
regression; fresh execution must establish the repaired output.

A private pass after the existing print/selection restores at most one authored
blank line between declaration or child-rule groups inside a rule. It requires
matching complete significant token streams before publishing any replacement,
keeps quoted strings, CSS escapes and complete comments indivisible, ignores structural punctuation
inside parentheses/brackets, and copies the printer's indentation and selected
newline. A token mismatch returns the existing formatted output atomically.
No new parse, pipeline stage, serialization or CSS bypass is added. Existing
nested/top-level comment and SCSS policies remain intact. In-rule comments still
select the original raw-content path before this helper; two whole SFC/CLI
controls freeze comment punctuation and the real grouping around it. Already
matching output returns before the full token walk, including the ordinary
protected CSS fixture with its between-rule separator.

The additive canonical JSON corpus pins 27 complete source/output/option plans,
including the exact three original issue inputs, scoped/module/opacity, collapsed
separators, after-child and media nesting, quoted punctuation/escaped quotes,
unquoted URL comment bytes, escaped Unicode/hex selectors, real comments,
CRLF/Auto, tabs/width and SCSS. Compact CSS must still be formatted, and a blank
line before a closing brace must not create a declaration group. Expectations
are authored references, not described as captured results. The public Rust
APIs compare every complete output across three passes. The existing authentic
formatter observer build executes the same plans in its existing merge-only
full suite; ordinary source Rust checks prove the API laws first. Nine default SFC plans also
use the existing source-built CLI and exact check/dry/write/recheck status,
stdout/stderr and whole file bytes. A bounded always-retained report binds the
corpus, actual API/CLI build receipts and raw process observations, including
partial failures. No extra observer build or CI stage is introduced. Four
standalone style controls retain the existing no-final-newline API contract
for reindentation or comment-aware chunks; this repair does not alter it.

Original differential inputs/captured assets, other current references, defaults
and all 104 instruction ceilings remain unchanged. The protected SFC input includes ordinary CSS and its separate
between-rule blank line: actual unchanged-budget acceptance must prove this
repair has no regression; do not infer acceptance from source checks.

Independent source review clears complete print/token alignment, escape safety
and structural-gap ownership. The direct comment law is a defensive lexer
contract; it adds no claim that existing nested-comment contents are formatted.
Current state: source review and formatting only. Fresh exact-source Actions,
protected complete suites and unchanged caps, then actual signed merge are
required. Remove a known-red candidate and fix its source; do not waive a gate.
The verified reporter trailer is
`Co-authored-by: ubugeeei <71201308+ubugeeei@users.noreply.github.com>`.
This independent wt branch starts from actual main; no native Stack dependency
or native/history/default-replacement credit is claimed.

TODO: post-source/protected terminal receipts and actual signed merge here and
on the issue. Publication/public installed-CLI proof follows the release owner's
first-cut fence and explicit thaw. Broader style/dialect/native history remains
unfinished under the existing roadmap.

Initial source Check37255950703 at1239803014 passes actual Clippy/build and
all four Rust workers; the27-case full-output law executes in111594105881
and the defensive comment law in111594105808. Tooling1–4 reject the stale
complete style current-owner pin; tooling1 also rejects the owned import shard.
Refresh only exact current-source transitions for the added module/helper
visibility, retain every original captured asset/function/golden/default, and
regenerate the formatter shard through the actual existing Node producer behind
the Rust wrapper. The original CSS escape law remains byte-identical; eleven pure
validator controls and all19 generator artifacts pass locally. Fresh complete
source acceptance remains required, and no protected runtime/cap credit transfers.

Paired [custody correction](https://github.com/ubugeeei-prod/vize/issues/7826#issuecomment-5987225634)
retains old denominator controls and adds original-pin/revision/function/current-
source forgery negatives. Executed e7e052d837 has parents d1a25/123980 and
tree25817aba4278e9c5a7149641f61bd10ab8fcec51 equal to literal123980.

Reviewed complete `formatted.trim() == trimmed` equality now returns the same
existing printed/color-restored bytes before allocating the authored token Vecs.
The pre-existing trimmed input contract makes this the identical old token-
comparison/no-restoration result; no fixture special case, mirrored test, new
stage or budget change is introduced. Any layout difference retains the existing
fallback and token-aligned group restoration. Old123 Rust execution and a197
custody remain distinct historical heads. The new production source requires
fresh exact-head Actions/protected all104; no measured speed or cap credit is claimed.
Paired [equality decision](https://github.com/ubugeeei-prod/vize/issues/7826#issuecomment-5987274362).

Exact73d source Check37257179839 is terminalSUCCESS: all four Rust and tooling
workers, full source report, Nuxt3/4, Title and Zizmor pass. Executed40c9937478
has parents ef506012/73d and tree26639d18; every owned producer/test/corpus/
metadata/companion byte is literal73d, with only legitimate main canonical
composition. The27-case law runs in111598786428 and lexer law111598786482.
The actual prospective queue base b8bca915 cannot merge the overlapping canonical
clause. Remove that queue entry, move only the owned clause to an existing
earlier canonical line, retain all other decisions/350 lines and production/
golden bytes, prove the prospective merge-tree clean and require fresh source
Actions before re-entry. ProtectedAPI27/CLI9/full suites/all104 remain required.
Paired [composition receipt](https://github.com/ubugeeei-prod/vize/issues/7826#issuecomment-5987672767).

Exact ba928 source Check37260303796 passes all source/Rust/tooling gates. Its
signed protected candidate621d43bb at Check37262614184 passes the unchanged
100+4 instruction ceilings with three identical runs, but tooling3 job111613103972
rejects exactly `capture/style-block-keeps-box-values-and-implicit-nested-selectors/sfc/0`.
The source-built complete output185 bytes equals the original input
`c6532257a2cfa6bf988ced53ccc1cbc208a04a775b3efd81102ac52c4bd964ef`;
the historical184-byte output `83a16896ba2557fb7d502b60a384e513cde36104ff8d0641c56f2b6df4ab4d39`
removes the authored LF before `.item` at byte142. Remove the known-red queue
entry; keep that strict historical mismatch and all300 original inputs/output
assets/source pins immutable. A reviewed literal authority qualifies only this
whole SFC/current-owner/default-option coordinate. Three complete current
passes must equal the explicit185-byte reference; the historical comparison
must remain different. Its pack reports23 historical matches, one separately
qualified current match and one internal observation, never24 historical matches.
Forged source/API/options/value/omitted-LF/refinement metadata are rejected.
The compact `style/1` #7866 proposal remains separate and unpublished. Old27/9
runtime observations were not reached after this assertion, so fresh exact source,
protected104/API27/CLI9/full Rust and signed merge remain required.
Paired [current-reference decision](https://github.com/ubugeeei-prod/vize/issues/7826#issuecomment-5988258080).
Independent source review rehashes all seven files, proves deleting current
byte142 yields the entire historical184-byte output, and clears the strict
single-coordinate qualification/counter scope. Its ten pure controls and the
owner's thirteen pure controls pass; no local formatter/native execution occurs.
Fresh source adoption and complete protected execution remain required.
