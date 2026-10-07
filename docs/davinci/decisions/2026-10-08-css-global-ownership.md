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
