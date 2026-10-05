# Explicit unknown template checks

Refs [#7874](https://github.com/ubugeeei-prod/vize/issues/7874), the earlier
[#7234](https://github.com/ubugeeei-prod/vize/issues/7234) and
[#7266](https://github.com/ubugeeei-prod/vize/pull/7266). This is a private,
unexecuted source proposal based on actual main
`8a8521d6897bbe3fd0af0cbfaebd83f4fc933933`; publication and all runtime acceptance
remain pending. The finite first-v0.433 admission hold remains in force.

## Original authority

The live #7874 reporter is `ubugeeei` (public account ID `71201308`), the
maintainer. This is not a third-party report. Preserve all three complete issue
code blocks in `tests/_fixtures/differential/typechecker/unknown-template-options`:

| Original         | Bytes | SHA256                                                             |
| ---------------- | ----: | ------------------------------------------------------------------ |
| `src/Child.vue`  |   120 | `32fa2cf70a75ee2b7d7c5ddecb8df37ef60c8de103217dacf2a2386579cd3ed8` |
| `src/Parent.vue` |   227 | `c2ff284ede74ed6c7007f1890003ab04930c1b4822cb2212701afc0b075b7c1e` |
| `tsconfig.json`  |   299 | `43b2d2a49191b2addeebf1a9bba969874453b50858a6c3a28fb57150fabbbd21` |

The reported command is `vize check --no-config` on 0.432.0. The original Vue
package version is unspecified; the proposed hosted controls use the existing
genuine locked Vue 3.6.0-rc.10 package and native TypeScript 7.0.2. Those are
current controls, not an authenticated execution of the historical installation.
The original vue-tsc prose supplies semantic obligations, not a native message
or generated output baseline. Live open-PR searches found no duplicate at the
source-preparation checkpoint; recheck before any publication.

## Causal source findings and bounded correction

The flattened configuration already reads both options. The omission is in
production TypeScript generation:

- A native-root child's recorded fallthrough marker selects an unconditional
  `Record<string, unknown>`, so an explicitly strict parent accepts the extra
  `unknownProp`. Preserve the older no-tsconfig #4461 contract. Carry an internal
  explicit-strict authority from the existing flattened Vue option and authored
  per-file comments; close only that arbitrary tail. Keep declared prop values,
  Vue public props and the existing real global HTML attribute surface. Propagate
  the same authority to the existing generic export signature.
- Custom directive collection required an expression. The original valueless
  `v-not-a-directive` therefore never reached the existing registry-presence
  emitter. Collect valueless or empty-value names during the existing traversal
  only when strict directive checking is enabled. Reuse that emitter and its
  original name mapping in authored order. Setup bindings, local/global
  registrations and builtin directives keep their existing resolution. No
  dangling hook/value call is fabricated for an absent expression.

Move the existing naming law into its own test module in a move-only commit.
The proposal adds no parse, compiler stage, serialized level record, per-hint
process, production diagnostic filter or weakened comparison. Existing absent
settings and the original #4461 fallthrough vectors remain unchanged.

## Proposed closed controls and custody

The untouched original project is one of seven complete option cases: original
true flags, props false, directives false, all false, all absent, strictTemplates
implicit true, and strictTemplates with explicit false overrides. The two SFC
inputs are byte-exact in every case; only authored control configurations differ.

A separate authored `Oracle.ts.txt` specifies the label prop, Vue public/native
attribute types, missing component and missing directive independently of Vize
output. Its complete direct native CLI stream must contain exactly the authored
three locations/codes and intact messages. That native compiler-specific type
display is then compared in full with every public CLI diagnostic message and
every editor diagnostic; it is never a captured Vize-generated expectation.
The public CLI law compares the entire JSON report, effective program/options,
all file vectors, exit status, complete stderr and conserved input bytes.
The editor law compares the complete `DiagnosticService` vectors for both files
under all seven configurations, including every range and optional field.
It is a direct service test, not a stdio-RPC execution claim.

The existing source-native Actions step retains every original cargo command
and adds a required bounded script for these CLI and editor targets. Runtime
capture retains original/control inputs, complete native/CLI stdout, stderr,
status, actual argv/cwd, source SHA, executable/native hashes and actual pinned
native version output. Original fixture hashes and authored mappings are checked
without recording new expectations from Vize. Source-only laws also cover
explicit-vs-absent generation, comments and generic tail contracts.

## Pending gates and TODOs

Only source formatting, shell syntax, the existing pure assertion lint and
inventory generation have run locally. No Rust build, native probe, LSP session,
performance experiment or hosted runtime has run for this proposal.

Before delivery: independent technical source review, fresh exact-head ordinary
and source-native Actions, complete original/control runtime artifact audit,
protected full Rust/differential suites and all unchanged 104 instruction gates,
actual signed reporter-credited merge, then a supported public release. A new
queue admission remains forbidden during the finite first-release hold.

Keep old corpus vectors, all caps and history denominators intact. Full generic
cross-project option policy, whole type-checker fix history #6879, full LSP
history #6883, native default replacement, cold/query completion and the global
10x objective remain unfinished. Do not mark #7874 fixed from source review or
an old successful run.

Paired private-source decision: [issue comment](https://github.com/ubugeeei-prod/vize/issues/7874#issuecomment-5997806537).

## Independent source review checkpoint

The fixed private commit `5b50b9c73582dad824768c07cc15caffcfacda1b` and its
complete 32-path source packet received independent `SOURCE_ONLY_CLEAR`. The
review authenticated the original three inputs, actual reporter, explicit option
precedence, both prop producers, the existing directive traversal/name mapping,
all old workflow commands, unchanged #4461 fixture and complete canonical record.
The source is now authorized for publication as a Draft with automatic current-head
Actions; this supersedes the initial private-review-pending checkpoint above.
No Rust/native runtime or diagnostic-vector acceptance follows from this review.
Fresh whole original/control artifacts, protected suites and actual delivery
remain required. Ready/queue admission remains subject to the first-release hold.

## First hosted runtime and complete progress contract

Draft #8056 source `1924cb19702dff96209cf580aa4f8f25caba5a83` reached the new
native CLI test in run `37339177996`, then failed its incorrect empty-stderr
expectation. Public `check/runner/execution.rs` emits two complete progress lines
for the two original files and the exact program root. Compare those entire
source-defined bytes, including both newlines; do not suppress or filter output.

Official artifact `11356809677` has SHA256
`2604168d106471e7394b18b17793ec6c990f2201d3d02a3c4d7c94b9685b51fe`;
all 645 members passed CRC and safe unique-path checks in memory. Retained original
input/config bytes, native version/status/full three-row oracle and complete
public JSON independently match the authored original obligation. The failed
test had not reached its whole JSON assertion, remaining six settings or editor
vectors. This separate artifact comparison does not turn the failed gate green.
Only the stderr test contract changes; all production, original inputs, full
diagnostic expectations and budgets remain unchanged. Fresh exact-head native
and ordinary Actions must qualify the complete seven-case CLI and fourteen
direct-service vectors before protected delivery.

## Known fallthrough compatibility and source-derived service contract

The first ordinary source run `37339178587` exposed a real regression in two
unchanged upstream projects: `fallthroughAttributes` and
`fallthroughAttributes_checkRequired` acquire TS2353 on their declared `bar`
forwarding. The strict tail must retain genuine keys from the existing child
fallthrough producer. Map those known keys to optional `unknown` values, exclude
the arbitrary string index and leave required/value checks intact. The
non-generic helper omits already accepted Vue public/global HTML keys and returns
literal `{}` when no extra key remains, preserving the original native type
display. The generic call uses the same index refusal on its existing exact
fallthrough reference. Absent/default authority keeps the older open tail.
Every upstream source/config/oracle and the old #4461 fixture remain unchanged.

That run also rejects two new test-only Clippy shapes. Name the existing case
tuple and propagate a checked diagnostic-index result to the test boundary;
no added suppression, input, predicate or diagnostic selection change.

The stderr successor `f4d1fd01fbef6ce3eb21035799d34ffa447a210d` genuinely passes
all seven CLI cases in native run `37341124150`, then its first editor assertion
exposes two incorrect expectations. Existing generation emits binding/directive
checks before prop calls, so the service preserves component/directive/prop
order. The existing linear mapping maps the 16-byte quoted camel name onto the
first 16 bytes of the 17-byte authored directive: range `9:9..9:25`, zero based.
Record that actual source-derived mapping limitation; do not claim a complete
transformed-token endpoint or change the old mapper. Keep every actual field and
returned order in the full-vector comparison, without sorting or filtering the
response. This successor changes the authored service expectation, not production
ordering or mapping. Its old failed run remains failed; all fourteen service
vectors, seven CLI cases and complete compatibility corpus require fresh proof.

## Bounded correction review and actual-main replay

Private `4996a202051600a3c18de9a7038938cd094763e2` received independent bounded
source-only CLEAR for both known-key producers, unchanged default branches and
the documented mapping limitation. No runtime acceptance transfers from `f4d`.
The five owned commits were genuinely replayed onto actual signed main
`9fe172ced209720642d7051c8a94229042cffa4c`, producing source `f6ec46c3` and tree
`03304604cdd08f4b18c6730291f61ae213dd2f37`, identical to the independent merge
projection. All 28 noncomposition owned blobs and all five full author/date/body/
footer records remain exact; the four composition files retain the incoming
diagnostic modules, complete canonical clauses and genuine census unions.
Both pure inventories pass and the canonical record remains 350 lines. Publish
this reviewed correction to the same Draft #8056 with fresh exact-head ordinary/
native Actions. Fourteen whole service vectors, seven CLI cases, unchanged full
compatibility corpora, protected suites and actual delivery remain pending.

## Required forwarded keys before authored-key omission

Exact source `658cefc79eb044360a34ef4a822dd7ce89808c3d` completes native run
`37344084550` successfully, including the required seven whole CLI cases and
fourteen direct-service vectors. Independent complete artifact custody is still
pending. The ordinary strict Rust builder now passes, but tooling2
`111879229531` rejects `bar` in the unchanged upstream
`fallthroughAttributes_checkRequired` project; `fallthroughAttributes` now passes.
This ordinary failure remains a blocker and is never relabelled green.

The earlier strict-tail mapping is too late for the required path: a template-less
Basic component contributes an open string index to its declared prop surface.
`Omit<Surface, "foo">` loses the genuine unbound `bar` name before that mapping.
For explicit-strict required forwarding only, remove the broad string index
from the actual resolved surface before the existing authored-key Omit. Keep
property values/modifiers and all optional/native/default branches. The source
law compares both required branches and unchanged optional/native targets; the
authentic upstream whole-empty project remains the runtime compatibility oracle.
No original source/config/expectation, shared helper, mapper, stage or cap changes.

Publish this bounded correction to the same Draft only after independent source
review, then require fresh exact-head source/native Actions and unchanged full
corpora. Native source success does not grant protected/merge/release or stdio-RPC
credit. Full history and default replacement remain unfinished.
