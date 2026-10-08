# Universal global CSS ownership

Issue: [#7976](https://github.com/ubugeeei-prod/vize/issues/7976).
Prerequisite: [#8266](https://github.com/ubugeeei-prod/vize/pull/8266), actual
parent `807970136d3c0b45d173a525b0bdc8c86b7df291`.

## Decision

Extend the existing retained-template proof to an unnamespaced outer universal
and one leading inner universal followed by a fully validated, nonempty literal
class/ID suffix. A universal adds no restriction. The suffix must still contain
a necessary class or ID whose absence is established by the existing complete
template facts. Continue checking every token, including tokens after an absent
class. Outer class, ID, type and universal selectors never establish absence.

The production change uses `Component::ExplicitUniversalType` and the already
parsed inner `Token::Delim('*')`. It collects no new template facts, reparses no
input and introduces no pipeline stage, serialization or public API. The native
route inherits the prerequisite driver's retained root and CSS finalization.

Bare inner universals, every namespace, attributes, combinators, pseudos,
multiple globals and unsupported tokens retain their conservative findings.
Single-root fallthrough, dynamic bindings, components, non-HTML elements,
external templates and non-HTML template languages remain unknown. A nested
local rule under a foreign global parent still receives its own finding.

## Transform authority and scope

Vue's pinned
[v3.5.38 scoped-style transform](https://raw.githubusercontent.com/vuejs/core/v3.5.38/packages/compiler-sfc/src/style/pluginScoped.ts)
replaces the whole selector with the global argument. Its
[global transform tests](https://raw.githubusercontent.com/vuejs/core/v3.5.38/packages/compiler-sfc/__tests__/compileStyle.spec.ts)
also pin removal of a preceding outer selector. The same branch appears in
[v3.5.42](https://raw.githubusercontent.com/vuejs/core/v3.5.42/packages/compiler-sfc/src/style/pluginScoped.ts).
The absence proof therefore belongs to the global subject. For example,
`*.absent:global(*.local)` remains a finding when the retained template owns
`.local`; `*:global(*.foreign.local)` can be excluded only when a closed template
proves that `.foreign` is absent.

The original reported deep/slotted cases already have their exact exemptions.
Reachable local global subjects still need findings. No original acceptance
case warrants suppressing every global selector. Attributes, combinators and
pseudos remain unfinished and require separately bounded ownership designs.

## Immutable history and authored transition

The 62 original compound vector objects and source bytes remain unchanged.
`issue-7976-universal/semantic-transitions.json` independently authors complete
before/after packets for exactly `outer-universal` and `inner-universal`. Each
before packet must equal its original whole object. Each after packet changes
only `expectedStarts`, `expectedCli` and `expectedPlain` to the complete zero
SFC result. The original standalone CSS finding, source, filename, style range
and every other field remain unchanged. Both Rust and Node loaders enforce
these exact boundaries before running the whole compound corpus.

The original compound corpus SHA-256 is
`8544832d6f63006f4f9f350df25faf931c4d02001ddb277714493a09a0dc37a6`.
The new `source.json` pins bytes and SHA-256 for all 18 historical witnesses,
including the original external/global/compound packets and the prerequisite's
native packets. It also pins the three new authored files. The historical
182-case CSS boundary and 16-case native boundary remain separately identifiable.

## Complete result obligations

Forty-three independently authored new packets cover 11 zero and 32 finding
cases. Their full diagnostic fields, counts, JSON, plain output and CSS offsets
come from authored expectations, rather than captured product output.

| Obligation                                                                                                                                              | Required result                                                            |
| ------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------- |
| Absent leading-universal class/ID subjects; outer universal; both universals; decoded escapes; selector lists containing only exempt subjects           | Complete zero SFC result, with standalone CSS findings retained            |
| Owned class/ID, bare universal, outer-absent/inner-local subject and mixed selector list                                                                | Complete SFC finding                                                       |
| Namespaces, attributes, filters/pseudos, combinators, comments, internal whitespace, repeated/nonleading star, comma inside global and multiple globals | Complete conservative SFC finding                                          |
| Single root, dynamic/component/foreign-namespace roots, external or non-HTML template                                                                   | Complete conservative SFC finding                                          |
| Foreign global parent with a nested local rule                                                                                                          | Only the local child is reported; standalone CSS reports both declarations |
| Scripted and scriptless query-free sources with a type rule enabled                                                                                     | Same complete native result without needing Corsa                          |
| CSS disabled                                                                                                                                            | Complete empty result with zero error/warning counts                       |

The new source API test executes all 43 packets on normal and genuine native
routes, yielding 86 distinct case/route contracts. Enabling
`type/require-typed-props` selects the existing native driver; an intentionally
missing Corsa path exposes any accidental query. Each route repeatedly checks
standalone CSS, whole SFC results and both formatters, then checks CSS disable.
The prerequisite's actual type-rule activation and script/type/CSS order laws
remain unchanged.

The existing source CLI observer retains its original 310 compound executions
and appends 215 executions using the authored native configuration. Each packet
runs three full JSON repetitions, one full plain report and one disabled-rule
report. The existing source build identity and receipt are mandatory. Raw
process status, signal, error, stderr, stdout, complete expectations and
before/after source/config are persisted before assertions. The existing
compound artifact now contains 525 observations; across original CSS observers,
the 858-call boundary plus 215 additions gives 1,073 observations. Build, upload
and observer stages are reused without another native build or new workflow.

Persist the one observation object immediately after process capture, before
post-call source/config reads. Attach the after-state to that same object and
persist again. A failed post-call read therefore retains its raw failure packet
without adding another observation or hiding its failure.
Keep every original observation ID and prefix the new IDs with `universal-`;
assert that all 525 persisted observation IDs are unique even when the immutable
and newly authored corpora use the same descriptive case name.

## Preparation and delivery

Pure corpus custody/transition tests pass with five tests and zero skips.
Rust formatting, JavaScript formatting and source lint pass. The actual consumer
migration generator adds exactly one L0 test/dev row for
`css_global_universal_ownership.rs:6`, and its full 19-file check passes. The
actual Croquis producer reports all 20 artifacts current with no delta.
The tracked source-length gate against the actual parent reports no new or
grown source file exceeding 350 lines. Existing full workspace discovery owns
the new Rust integration target; existing tooling discovery owns the new pure
test and the extended CLI observer.

This isolated `wt` child starts at the actual prerequisite head above. The root
delivery task owns the unique canonical record row 77, the dependent PR, native
Stack registration, fresh exact-head whole-history/API/CLI Actions, protected
queue acceptance, actual merge and installed-release replay. Source preparation
grants no runtime, merge or publication credit. Keep #7976 open while those
qualifications and the broader unsupported ownership constructs remain pending.

## Canonical clause for root integration

[#7976 universal global ownership](./2026-10-08-css-universal-global-ownership.md)
extends the existing retained-template proof only to an unnamespaced outer
universal and one leading inner universal followed by a fully validated nonempty
literal class/ID suffix. Outer selectors never establish absence; bare/namespaced
universals, attributes, combinators, pseudos, multiple globals and unknown roots
remain conservative, with no added parse or stage. All 62 original compound
vectors and 18 historical fixture witnesses remain byte-identical; separately
authored whole before/after packets permit exactly the two old universal controls
to become zero, preserving standalone CSS warnings and every other field.
Forty-three new positive/negative whole-result packets run on normal/native API
routes and add 215 actual source CLI observations through the existing
observer/build receipt/upload, retaining the original 858-call boundary. Actual
consumer/Croquis producers pass; one genuine L0 test/dev consumer row is added,
no Croquis delta. This child begins at actual #8266 head
`807970136d3c0b45d173a525b0bdc8c86b7df291`. Fresh exact-head
whole-history/API/CLI Actions, protected native Stack acceptance, actual merge
and installed-release replay remain required; #7976 stays open.

## First source formatting correction

Exact child `1d8c5ca0` fails
[Check 37735137821](https://github.com/ubugeeei-prod/vize/actions/runs/37735137821/job/113172866282):
the configured JavaScript gate rejects formatting in this late companion record.
Run the actual Markdown formatter on the companion and paired canonical record.
The source grammar, every original and new fixture, full expectations, observer
IDs/counts and all gates remain unchanged. Preserve this real failed source;
fresh successor Actions and protected native Stack qualification are required.
