# Retained property-value layout (2026-10-03)

Issue: [#6840](https://github.com/ubugeeei-prod/vize/issues/6840).

## Decision

The native DOM property codec uses the actual retained JavaScript root to
select inline or multiline object spelling. A literal root stays inline,
including its original surrounding trivia and comments. A direct identifier
stays inline only when its complete decoded source equals its actual semantic
identifier name. Other roots and identifier windows use multiline spelling.
Multiple properties and existing class/style normalization remain multiline.

This is output formatting. It grants no binding class, runtime access, purity,
constness, block eligibility or patch demand. Those decisions remain genuine L3
facts. L4 inspects only the root tag and, for a direct identifier, its retained
name; it does not traverse the AST, trim text, parse, serialize, or add a stage.
The existing single binding-layout scan incorporates this test before the
same append writer encodes properties. Expression source/projection/access
checks remain transactional, including NoLinks emission.

The selected Vue 3.5.35 compiler emits a single `msg` value inline, but emits
` msg `, `/*kept*/msg`, `msg/*kept*/`, `msg + 1`, `msg.length` and `(msg)` in a
multiline property object. Literal roots such as ` 42 ` and `'kept'/*雪🌸*/`
remain inline and preserve those bytes. This corrects complete module output,
including the late import preamble, rather than comparing a render body alone.

## Original complete references and genuine consumers

The closed `dom-props-vue-3.5.35.json` pack retains fourteen original sources,
options, full modules, unmodified raw upstream maps and semantic executions.
Source, template, expression, code and raw-map hashes are checked; the pinned
compiler is rerun twice. Seven declared-context development fixtures consume
real retained ASTs and complete resolver tables through the existing bare
entry. Seven literal fixtures use actual Program/Component/Vue File factories,
the owner-bound L3 file analysis, and the separate literal-only `emit_file`.
The File inputs retain whole-file offsets, Unicode block comments and an
unused genuine setup declaration; that declaration authorizes no render read.

Every complete module byte and helper import order matches the pinned
reference. Recorded and NoLinks output/helper order agree. Bare identifier
rewrites retain their actual named accessor links; literal links remain
anonymous and authenticate the original file/AST/source windows. Native maps
keep complete whole-file sourcesContent. Raw upstream maps remain separately
regenerated unchanged: complete upstream map equivalence is not established.

All fourteen generated native modules execute in the actual Vue runtime under
two contexts each. File literal renders receive throwing context, setup,
props, data and options objects. Values, regular expressions, VNode shape,
patch flags and dynamic children match all twenty-eight original outcomes.
The exact preceding codec fails the new complete-module law at the retained
identifier-trivia fixture; the corrected codec passes it.

The bounded proof compiles complete registered L3/L4 source with authenticated
File provider libraries and coherent cached L0/OXC/selected stock L1. It runs
fifty-two L4 unit laws, the existing four File pipeline laws, one additional
seven-input File law, fifteen new Node reference laws, compile-fail privacy
and strict whole-production L4 Clippy. The earlier eight complete File modules
and sixteen actual native renders are rechecked without replacing their frozen
source/proof artifacts. This is not a current whole-workspace Cargo, hosted
Actions, no_std, performance, or native product result.

## Remaining work

Genuine same-file external render access is a separate provider chain:
authenticated L2 native exposure, sealed L3 decisions, then an additive L4
consumer. Setup syntax, FileDependent values or caller policy cannot supply
that authority. The original file entry continues to refuse every reference.
No normal L4 dependency on the upper/native producer is added.

Escaped identifiers and non-reference member/property key spelling still need
genuine resolver spelling facts or an explicitly checked refusal. This shape
codec neither rewrites those names nor claims their byte parity. File If/For,
the full Vue surface/dialects, complete maps and performance remain unfinished.
True provider replay, exact-head Actions, native Stack/queue delivery and
compiler fix-history [#6880](https://github.com/ubugeeei-prod/vize/issues/6880)
remain required before product migration. No production route changes.
