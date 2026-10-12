# Attribute order and object binding precedence

Date: 2026-10-12. Issue: [#6881](https://github.com/ubugeeei-prod/vize/issues/6881).
Paired decision: [6118704774](https://github.com/ubugeeei-prod/vize/issues/6881#issuecomment-6118704774).

The existing public Patina `vue/attribute-order` rule incorrectly cleared its
ordering state at every `v-bind="object"`. The original Real Project Matrix
[38205026873](https://github.com/ubugeeei-prod/vize/actions/runs/38205026873),
attempt 1, source `ef34d1a0409fc711a6c5cefcf0708b0cce78f502`, retains 367
false negatives and 10 false positives for this rule across 17 usable projects
in the 21 captured shards. This observation does not establish a terminal full
22-shard run or prove that this slice repairs every finding.

## Decision

Use the existing default category order from
[`eslint-plugin-vue@10.9.2`](https://github.com/vuejs/eslint-plugin-vue/blob/9aa463a4aa4ad3c37d7facacda2e40ad7880ea92/lib/rules/attributes-order.js).
Dynamic, static and boolean attributes remain one group; unique attributes and
slots remain one group. The Matrix enabled this rule at warning severity with
no ordering, alphabetical or length options.

- Skip an object bind only when an original immediate neighbor is an ordinary
  attribute, `v-bind`, or `v-model`. Keep earlier ordering state, so
  `@click="click" v-bind="attrs" :title="title"` still reports `:title`.
- An isolated object bind uses the next ordinary attribute, non-object bind or
  model's position. If none follows, use the ordinary-attribute group. A cached
  lookahead advances monotonically: linear time, no additional allocation,
  parse, lowering, or pipeline stage.
- Attribute values containing `{{` do not clear ordering. Dynamic directive
  arguments such as `:[id]` do not acquire the static `id` group.
- The pinned parser reads literal `#="..."` as an ordinary attribute. Match
  that classification locally in this rule, including spread adjacency and
  lookahead. Preserve `#default`, named slots, `v-slot`, `key` and `ref` groups.

Keep rule metadata, default severity, preset membership, localized messages,
help, whole original source spans, and the absence of autofixes unchanged.
This changes the existing product rule; it adds no Davinci native provider.
The opt-in native template route continues to refuse this unprovided rule.

## Corpus and limits

The committed corpus retains nine complete pinned public inputs: HeyUI,
TDesign, Vue-JS-Modal, Gridea, four Naive UI examples, and SPlayer. Each retains
its repository revision, source SHA-256 and whole reference diagnostic vector.
Four entire original native file entries and per-file comparisons remain
separate historical observations. The 35 finite controls cover isolated and
adjacent spreads, multiple spreads, lookahead, all default groups, dynamic
arguments, named and bare slots, suppression, unrelated structural findings,
Unicode and CRLF. Tests compare complete service payloads, configured error
severity, disabled-rule behavior and the whole unprovided native refusal.

The original HeyUI finding follows `<!-- eslint-disable -->`. The Matrix
reference omitted the Vue comment processor while Patina honored suppression;
retain both results and resolve that producer disagreement separately. Gridea
line 59 retains an original one-column false-positive/false-negative pair after
an emoji: the reference uses UTF-16 columns and the native report uses Unicode
scalar columns. This slice preserves the original byte spans and does not
normalize the Matrix reports. The nine original bare-slot false positives in
Naive UI and SPlayer are covered by the ordinary-attribute classification.

Source review compared the candidate semantics on authentic pinned parser
nodes with all 44 JS reference vectors; all matched. This is source-only
evidence. Patina compilation and execution, exact-head Actions, the protected
merge queue, successor publication, and a fresh unchanged full Matrix remain
required. Unusable project refusals, zero divergence budgets, every original
input, and the required release project set remain intact. #6881 stays open.

The first exact-source [Check 38212202052](https://github.com/ubugeeei-prod/vize/actions/runs/38212202052)
at `bb4106ba61` failed the unchanged Clippy indexing gate before test execution.
The helper now uses checked access for that one predicate, preserving the
enumerated caller's ordering and all 44 original vectors. The
[paired correction](https://github.com/ubugeeei-prod/vize/issues/6881#issuecomment-6118982283)
retains the failed source result and requires fresh qualification without a
lint, corpus, budget, metadata or native-provider waiver.

The `57c74a7` successor compiled and passed Clippy, then the unchanged named-slot
control failed both normal and configured-error service checks. Active Relief
directives retain prefix-only raw names: `#default` also stores raw `#`. The
bare-slot predicate additionally requires no argument, preserving actual named
slot groups without changing parser ownership. The assertion gate separately
rejected a prefix-based inventory count; compare the exact ordered nine original
IDs instead. The [paired correction](https://github.com/ubugeeei-prod/vize/issues/6881#issuecomment-6119190323)
preserves all 44 source/oracle vectors, complete failed shard logs and the passing
native refusal, requiring fresh whole service execution and every original gate.
