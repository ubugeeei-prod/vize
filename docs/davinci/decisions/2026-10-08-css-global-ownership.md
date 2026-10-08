# Conservative global display ownership

Issue: [#7976](https://github.com/ubugeeei-prod/vize/issues/7976).

The original deep/slotted report was delivered by #8091. Its exact original
SFCs, issue bodies, child files, configurations, 54 complete API/report vectors,
218 source CLI observations and before-correction archive remain unchanged.
The existing `global-local` control must still warn: `:global()` can select a
DOM element owned by this template, so global spelling alone does not prove
that a `v-show` suggestion is unreachable.

On genuine main `177933f33e75af1f696fdc468665e7850025d28b`, the same rule also
warns for a standalone global class, ID or tag absent from the template's known
owned elements. Extend only this proved foreign case. Keep the public unit
`NoDisplayNone`, `CssRule` and standalone CSS interfaces unchanged; a private
SFC rule wrapper consumes facts from the template AST already parsed for lint.
Move the existing ordinary CSS call into that root's lifetime, preserving its
position after script diagnostics and its single existing style parse/pass.
There is no additional parse, stage, serialized intermediate or native claim.

Only an entire standalone simple `:global(.class)`, `:global(#id)` or
`:global(tag)` subject can establish this proof. Use the existing parsed CSS
function tokens, including decoded CSS identifiers, and actual parsed template
attributes, including decoded HTML entities. All selector-list alternatives
must be either already proved deep/slotted or proved foreign global subjects.
Preserve advice for compound/relational/attribute/global descendant queries
whose owned subject is not established. A foreign global ancestor does not
propagate deep ownership to nested children; a local nested declaration warns.

Class/ID absence requires two unconditional native template roots, because a
single root can inherit arbitrary caller class/ID attributes. Static owned
class/ID/tag matches always remain eligible for the existing advice. All dynamic
bindings, DOM HTML properties, event handlers, model/custom
directives, components, slots, foreign namespaces and dynamic `is` roots retain
conservative unknown facts. No unknown is reclassified as foreign. Conditional
and repeated owned descendants remain potential local matches.

Standalone CSS has no template input and retains its warnings. External,
unsupported or fatally refused templates and the current type-aware Corsa
branch without a retained root also keep their previous conservative advice;
do not reparse or add a borrowed/public API merely to extend this slice.
Collect template facts only when the enabled display rule has an inline global
spelling in the style input. Existing CSS rules, severities, ordering, source
ranges, parser flags, conditional-rule behavior and disable controls stay exact.

The new independent corpus contains 66 whole controls, separately authored from
these ownership conditions and existing declaration/report contracts. Compare
complete standalone CSS results at nonzero offsets, complete SFC `LintResult`,
full JSON and plain reports over three repeated queries, and whole rule-off
results. Every CSS input must successfully parse a nonempty stylesheet. Nine
controls prove foreign ownership; the remaining controls retain advice, with
the nested case preserving only its local child declaration. These expectations
are source-authored, not captured product output.

Independent source review found that `className` property writes and the camel
form of `class-name` also introduce dynamic classes. Treat both spellings as
unknown class facts regardless of modifier and append four complete warning
controls (`:className`, `:className.prop`, `:class-name.camel`, `.className`).
Preserve all initial 47 case objects with their independently pinned object
hash; this is a source correction before any compiled acceptance.

The same review identifies `classList`/`class-list` writes as dynamic too:
the [primary DOM interface](https://dom.spec.whatwg.org/#interface-element)
forwards assignment to the token list value. Append the corresponding four
full warning controls while preserving all prior 51 objects. No readonly
getter assumption may establish class absence.

Review also identifies bound `onClick` and function `ref` callbacks and literal
DOM callback/property attributes. Refuse ownership proof for every binding
directive instead of maintaining a partial DOM property allow-list; literal
`on*`, `is`, `ref`, `className`, `classList`, `innerHTML` and `outerHTML` inputs
also retain unknown ownership. Seven appended whole warning controls preserve
all 55 previous objects. No arbitrary script-body analysis is added.

The existing default parser does not supply a native-tag classifier, so a
lowercase or kebab component can carry `ElementType::Element`. Only actual
HTML-native tags from the existing `is_html_tag` predicate establish known
native roots/tags; all others stay unknown. Four appended complete component
class/tag controls preserve all prior 62 objects and require no parser change.

The receipt-bound source CLI observer makes 330 calls: three complete JSON
queries, one complete plain report and one rule-off query for each control.
Retain every argv, stdout, stderr, status, source/hash and config before and
after each call, and persist raw failures before assertions. The new catalog
also freezes all eleven historical fixture files by complete byte count/hash;
the existing 54/218 tests and observers run without edits. Together, the old
and new CLI observers retain 548 complete process observations.

Local source formatting, authored fixture checks, assertion lint and generated
consumer inventory checks are preparation only. Required exact-head Actions
must execute both complete corpora and source-built observers before protected
admission. Root alone manages the independent queue candidate and actual merge.
Keep #7976 open until the literal originals, foreign global cases and preserved
local/dynamic controls pass on the actual delivered source; public release
qualification remains a separately tracked root-owned obligation. No budgets,
workflows, historical goldens or unrelated compiler CSS sources are changed.

Initial a90 Actions compiled the source, passed warning-denying Clippy and all
four Rust workers, but tooling rejected the CSS module's growth from 424 to
426 lines and stale generated rule-source anchors. Preserve these real failed
jobs. Colocate the complete byte-exact new private helper under its owning
display-rule module in a move-only import/wiring commit, restoring the CSS
module to 424; regenerate only four owning EN/JA reference/index anchors and
the genuine consumer inventory. Every new/historical fixture and observer
remains byte-exact. No fixture split or cap/gate waiver is needed. Require fresh
successor source and full protected qualification without transferring a90
runtime acceptance.

Actual signed delivery on 2026-10-08

#8221 merged at `504426dbff2eac74084ce6ddbb0d2ad9241a6c64` on
2026-10-08T00:26:37Z. Fresh protected Check 37706344985 passes every required
upload and all four terminal checks, both 120-case owning API corpora, all 548
complete CLI observations and all 104 unchanged ceilings across three identical
measurements. Fresh main `7431d6b` retains all eleven historical fixtures and
eleven owned source/corpus/observer files byte-exact to source `693b9ba8`; all
218 old observation objects match original protected #8091 exactly, including
the literal original no-diagnostic Expected.

Keep #7976 open. This delivers proved foreign standalone simple class/ID/tag
subjects; broader compound/attribute global subjects and the type-aware route
without a retained template root remain conservatively unclassified. In the
complete retained corpus, `global-compound-unknown` still warns at byte 126 for
`:global(.foreign.local)` on known native fragment roots. Local global matches,
dynamic inputs and single-root caller fallthrough deliberately retain advice.
TODO: complete broader global ownership policy with original inputs and whole
controls retained; root separately owns installed 0.435.2 acceptance.

Next bounded compound-global preparation

Use genuine delivered main `7431d6b` and the existing parsed inner global tokens.
Vue global scope replaces the entire outer selector; an absent outer class, ID
or type never establishes foreign ownership. Validate exactly one global and
every outer component as same-compound class/ID/native type. Inner arguments
accept an edge-trimmed literal class/ID sequence with an optional native type
prefix. Validate the entire sequence before accepting a necessary absent inner
class or ID, using unchanged complete template facts and decoded parser names.
Internal whitespace/comments, namespaces, universals, multiple globals,
attributes, pseudos, combinators and unknown roots remain conservative. All
class/ID matching stays case sensitive; no extra parse, stage or API is added.

Freeze all eleven historical files and both original 66-case/source fixtures.
Separately authored whole before/after semantic packets change only
`global-compound-unknown` and `outer-compound-unknown`: complete SFC/API, JSON
and plain warning packets become zero, while original source, style offsets,
standalone CSS expectations and every other field remain exact. Strict complete
object delta laws apply in both API and source reference resolution; the
original full assertions and 120-case/548-call execution counts remain.

Add 62 independently authored complete CSS/SFC/JSON/plain/off controls and a
310-call receipt-bound whole CLI observer, including reverse outer-missing but
inner-local subjects, absent-first followed by unsupported tokens, escaped
names, selector lists, nesting, dynamic DOM properties and fallthrough. Total
qualification is 182 API cases and 858 CLI calls, with all original 218 process
objects still byte-exact to protected #8091. Local formatting, four authored
reference tests, assertion lint and generated consumer inventory pass; compiled
source/native Actions, unchanged protected 104 ceilings and actual signed
delivery remain pending. The broader global and no-retained-root policy and
installed public qualification keep #7976 open.

Initial compound source Check 37709829672 rejects the direct slice at helper
line 144 under denied `clippy::indexing_slicing`, before owning Rust execution.
Use checked `get` on the same derived inclusive edge-trim range, preserving
all token interpretation, complete sources/expectations and 182/858 obligations.
Retain the real failed log; require fresh successor source/native/protected
qualification without a lint waiver or failed-head runtime transfer.
