# Production stage capture

Decision for #6832, 2026-09-28.

The existing `ladder_run` is an independent L1 → L2 → L3 execution. DOM
production emission takes an optimized L1 → L2 → L4 route, while the accepted
SSR and Vapor bridges execute L1 → L2 → L3 → L4. Pages from `ladder_run` cannot
be presented as the stages that emitted a product module.

L0 owns a target-neutral `CaptureSink` contract and an owned `StageCapture`
sidecar. The ordinary compile uses `NoCapture`; page, timing, remark and
outcome builders are lazy, so they do no dump work or clock reading there.
The ordinary DOM host retains its original `try_emit_l2` and `emit_l2` bodies,
and the L1→L2 emitter retains its original parse, lower, transform and emit
body. A compile-time `CaptureSink::RECORDING` switch sends `NoCapture` through
those bodies and an observing sink through the same-run page path. This was
required by the protected instruction gate: candidate `8f364d` against the
green predecessor `cf8086` changed only the six full DOM compile probes
(+162, +779, +772, +1251, +23 and +2121 instructions); the DOM transform,
codegen and other product probes stayed unchanged. The captured generic body
changed LLVM's optimization of the ordinary DOM emitter despite lazy page
builders. Restoring the L1→L2 body alone left only the small (+28) and wide
(+48) probes above their ceilings; Callgrind showed the remaining ordinary
host calls still passed through the generic captured wrappers. No instruction
ceiling is raised. After restoring the host's direct ordinary calls, a
fresh-main diagnostic `f707e07c9` left four full DOM compile probes exactly
three instructions above their ceilings; all other 96 passed. Callgrind
showed identical ordinary host and L1→L2 emitter calls, with the remaining
three instructions at the benchmark closure's call to the generic DOM root
with a `NoCapture` argument. The ordinary DOM root now keeps its original
signature and direct compile body; the opt-in captured root is a sibling.
The next exact-candidate instruction run must verify all 100 probes before
the PR is updated.

The same cost law applies to SSR and Vapor. Protected #7132 candidate
`93c71ca` exceeded unchanged ceilings on four SSR full-compile, five Vapor
lowering and two Vapor full-compile probes. Three Callgrind repetitions
matched exactly:
SSR paid two instructions in the ordinary compile call and four in its L4
lower/emit call for `NoCapture` argument setup; a seventh argument to the
L2-to-L3 selector spilled beyond the x86-64 integer registers. The ordinary
SSR compile, source selector and L2-to-L3 selector therefore retain their
no-sink signatures and bodies, with captured siblings only for observation.
Vapor's lowerer was unchanged, but its existing element-template helper was
outlined by the changed compile layout, adding 15 instructions per dynamic
element; that helper is forced inline at its existing call sites. These are
code generation remedies, not new stages or budget changes. A fresh 100-probe
diagnostic must pass before #7132 is updated.
The first fresh-main diagnostic `70fcdbd6e` reduced the 11 protected misses
to four SSR full-compile probes (+1 or +3 instructions) and one Vapor
generate-large probe (+2); the ordinary Vapor lowering and compilation
probes all passed. Callgrind attributed SSR's common +1 to the extracted
ordinary compile body, with deep's extra +2 in `memcpy`; Vapor's +2 was
inside `memcmp` with exactly the same comparison calls. Restore the SSR
ordinary body in its original compile module, keep the captured sibling,
and keep the existing Vapor lowering helper inline. A second diagnostic
`f777c645d` made all SSR probes pass, but changing the retained expression
trim algorithm pushed eight Vapor generate/compile probes above their
ceilings, so that experimental trim change is withdrawn. The next exact
100-probe diagnostic must pass before the PR head changes.
The isolated `fdb9e04ec` diagnostic then passed 99 of 100 probes; only Vapor
generate-large remained two instructions over the unchanged ceiling.
Callgrind showed the same 23 comparisons in its retained-expression resolver
and two extra instructions inside libc `memcmp`. The resolver already has the
trimmed subslice, so its byte offset from the original expression supplies
the retained span adjustment without scanning leading whitespace again. This
is a local ordinary-path optimization to be measured on all 100 probes;
it changes neither parsed expressions nor output bytes.
Each product records only boundaries it actually executed. Provisional pages
are committed only after final product selection, including DOM source-map
parity. A compatibility map mismatch returns the compatibility module and
discards native pages; selection accounting reports that final lane. A legacy
selection, no-template result or rejected compile likewise discards pages. The sidecar
records the target, effective options and explicit outcome so an empty feed
cannot be mistaken for a native run with no changes.
`timings_observed` and `remarks_observed` are false until a producer observes
the entire respective channel; an empty unobserved channel is unavailable,
not a claim that zero events occurred.

DOM records its actual surface parse, lowering, resulting fact tables and render
emission. These facts describe the current transitional DOM emitter; static
and hoist decisions remain L3 target ownership. The production emitter folds
preserving facts rather than executing the separate ladder's pass plan, and
does not lower L3. SSR and Vapor record
the L3 artifact that their accepted backends consume. L4 pages represent the
documented backend emission boundary; later SFC script assembly is a distinct
host step. The current SFC adapter must carry the capture from the compile
that produced its returned module, including explicit fallback status. It
must not infer selection from global profiler counters or rerun a ladder.
Compatibility parser work that produces the existing public AST/diagnostics is
outside these native module-producing pages and remains separately reported.
The opt-in native Vue container splitter keeps authored byte spans. Its
interpolation boundary scan balances nested JS braces and skips quoted text,
comments and recognizable regex literals so their `}}` and
`</template>` bytes cannot become a false root close. Prefix versus postfix
`!`, `++` and `--` retain operand state so a following `/` is scanned as
division when Vue+TS or JS grammar makes it one. A nested template-literal expression remains explicitly
uncertain and supplies no trusted close span. SFC tests compare the observed
result with the ordinary compile for both script plus template and template
errors; fallback discards all provisional pages. The full container grammar
remains #6837.

This adds no replacement route for legacy compiler behavior. #6880 still
gates any such replacement. The CLI and playground consume this sidecar in
the next #6832 change; their old independent ladder feed is not production
evidence. `vize_l0::dump::capture` moves with the L0 substrate in #6833;
the full product SFC descriptor and tokenizer/dialect moves retain their own
issue boundaries.

## Product feed and CLI input

The CLI and Playground use the same host serializer for a version 2 product
feed. It includes the chosen target, effective options, accepted/legacy/
unavailable/rejected outcome, executed level pages, and explicit availability
for timings and remarks. A non-accepted outcome has no native pages. An empty
unobserved timing or remark array means unavailable, not zero events. The
version 1 inspector ladder feed remains distinct and cannot fill missing
product pages.

The opt-in CLI uses the L1 Vue container to locate authored SFC block byte
spans, including a nested template and quoted attributes. Its bounded scope
is top-level blocks; non-template blocks end at the first literal matching
close tag. The product SFC descriptor remains authoritative for script
compilation. Before labeling a template span as authored, the CLI compares
the native template bytes, language and offsets with that descriptor and
rejects a mismatch. An unclosed interpolation or malformed block is reported
instead of yielding an attributed page. Full SFC descriptor and tokenizer
migration remains #6837 and #6835. Ordinary compilation never calls this
CLI reader.

Raw Pug input is parsed and derived with the native L1/L1-to-L2 Pug code;
the emitted product compile consumes the derived Vue template. The feed
distinguishes authored Pug from compiled Vue syntax. Until a source-map
consumer is wired, it does not claim that the derived page's spans index
the authored Pug text. The CLI pipeline selector accepts only real DOM,
SSR and Vapor product backends; unsupported source/backend pairs and
arbitrary named no-op passes are rejected. Page files use level and step
names with a dump extension; historical fixture bytes remain unchanged.
The CLI passes the real input filename to both descriptor parsing and product
compilation, preserving scoped-style identity and relative script type-import
resolution. Standalone dump uses its documented default compile options; build
has its own explicit script, template and style options. External SFC block
`src` inputs fail closed until the native CLI can resolve them. Reusing a dump
directory removes only page files whose generated names and bytes match the
previous version 2 `vize-dump` feed. A modified page or a colliding file stops
the export without deleting user content; unrelated files remain untouched.

## Build command dump

`vize build --dump-dir` observes the same SFC compile that returns each built
module. `--dump-after-change` filters consecutive equal page text without
another compile; the version 2 feed still records every executed page. The former `--folio-dir` and `--folio-after-change` names have
no aliases. Ordinary builds keep the non-observing compiler entry and do no
dump work. A stats-only build bypasses its content cache only when capture is
requested, so each reported file has an executed compile.

Each source gets a version 2 `stages.json` feed through the shared product
serializer. Only accepted native template stages write `.dump` pages; legacy
or unavailable selection records its actual outcome and zero pages. This does
not claim native SFC script/style work or add native acceptance credit.
Each Rayon worker writes and releases its own capture, so batch memory does
not retain all dump pages. A dump write failure fails that file without
emitting a fallback module, including under `--continue-on-error`.

On reuse, build validates the prior feed's version, command and source, plus
each existing page filename and exact bytes before deleting owned files.
Unknown and modified files remain untouched; filename collisions and symlinked
source directories fail the dump instead of redirecting writes or deletion.
Cleanup runs before the next observed compile, so a read, parse or compile
failure under `--continue-on-error` leaves no stale accepted feed beside its
fallback module. The authored syntax label is a lowercase slug; an unannotated
Vue SFC template uses `vue-template`, matching `vize dump`. Pug is desugared
first, while Jade is passed through unchanged. Every inline template is handed
to the Vue template parser, so its compiled syntax is `vue-template`. External
template, script or style `src` makes an otherwise successful capture
unavailable because the product compile does not inline those external blocks
into the observed stages.

## Playground product stage view

The Playground derives stage identity from the product feed's `(level, step)`
pair. L2 facts and provenance are not transform trees; L3 partition and values
are their own observed pages. The stage timeline counts only executed stages
with the appropriate semantic kind. The actual L4 emit page remains visible
beside the later assembled SFC module, with distinct labels; assembly is not
presented as an L4 backend decision. A rejected DOM target does not suppress
independent SSR or Vapor results. The stage view observes the same adapter
compile that supplies its displayed assembled module. The WASM result still
also computes a separate legacy `template` field; this duplicate work is an
unresolved #6832 performance follow-up, not another native stage or
evidence for the displayed module.

## Merge sequence

Dependent product capture slices are registered as one GitHub native Stack
after each conventional PR has the true parent branch as its base. Check the
Stack number and ordered positions through the GitHub API. Individual layers
do not use auto-merge. Merge a contiguous exact-head-green prefix through the
protected queue with `gh stack merge <highest-ready-PR> --yes --squash`; a red
later layer stays out. Verify the prefix's actual merge, then rebase and
retarget remaining layers on fresh main, rerun Actions, and verify Stack
membership before the next prefix. Independent PRs use ordinary squash
auto-merge after their checks pass.
