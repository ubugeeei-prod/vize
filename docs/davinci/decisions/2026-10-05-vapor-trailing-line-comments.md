# Vapor expression line-comment boundary (#7894)

Initial source `ca740`/Check37260156198 failed before runtime: the shared
converter was exported through a private module, and bare rustfmt used the
wrong edition for the new test imports. Expose only the helper through the
existing public codegen facade and format with edition 2024; unchanged original
inputs, oracle, counts and ceilings require fresh exact-head Actions.

Independent exact `b8` review caught two new unit expectations omitting the
converter's established preserved leading comment space. Correct only those
two strings to `/*  note */`; production and original corpus bytes stay exact.

The reported original App.vue is retained exactly: 138 bytes,
SHA-256 b7f2f1bea9c5157e0473ea690adf6a9572c6ba86536d177cac50936b4f41408f.
Its reporter is GitHub ubugeeei, verified public ID 71201308.
The implementation originally started from main ef506; the current genuine
rebase starts from signed actual main a2712e78968e9112c89cbc2111b108bd0e51959c,
which delivered #7853. Complete incoming canonical clauses and all five
production blobs were preserved before the isolated spread correction.

Vapor trims expression trivia, then callers append closing punctuation.
A final line comment consequently swallowed that punctuation and broke the
complete module. End the parse-only parenthesis wrapper with a newline,
keeping identifier offsets exactly one byte, and convert actual line comments
after all span rewrites with the established DOM comment converter. The
retained differential uses that same parse boundary and rewrite contract.
The shared lexer must recognize full Unicode identifiers and keyword-named
properties before deciding division versus regex; an independent source peer
found a quoted-slash corruption in the initial reuse, before any acceptance.
Preserve quoted strings, regex, template literals and comment-free output.
Exact954 peer review withdrew the earlier lexical clearance after executing
valid `async () => [...await /[//]/.exec('/')]`: three spread dots must begin
an operand, rather than make `await` a property name. Correct that one
transition and add the unchanged authored async expression to a complete SFC
event control. Click it, await its real Promise, require `/`, and compare all
three actual mounted states against official output. All fourteen prior
control sources and the original138 bytes remain exact; counts only increase.

The corpus contains the original plus fifteen fixed controls. It covers
reactive if/else-if/show/title/class, same-line comments, a block terminator
inside comment text, Unicode/property division with trailing comments and
quoted slashes, and working event/html/regex-string controls. Source-built
full SFCs run in development and production: 32 complete public results,
32 independent whole-module trace pairs, each with three rendered states,
exact diagnostics/interactions and complete unmount. Capture every public
field, compare the full setup-binding map with the independent compiler,
and require CSS/map absence plus empty errors/warnings/macro artifacts.

The oracle is the repository's exact official Vue 3.6.0-rc.9 compiler/runtime
graph, not the report's uninstalled rc.10. Its public compileScript compiles
the unchanged original descriptor with explicit Vapor selection and inline
rendering. The reported TS module uses the existing locked Vite TS transform.
Babel parses complete actual modules and changes only real Vue imports for
loading; no emitted render is reconstructed. The existing mounted observer
compares all rendered text/attributes/form values while excluding backend
anchor comments; setup creates the original genuine reactive state.

Original interpolation/v-for failures reported upstream remain separately
unfinished and outside this finite qualification. Actual Vapor SSR still uses
the existing explicit standard-SSR fallback; this change provides no native
product, Vapor SSR, Chromium, all-fixture or performance acceptance.

Actual source dbd/Check37261528918 passed lexer/format/JS/browser/tooling/Nuxt,
but its whole-runtime Rust test rejected the first reactive control's public
binding map: imported reactive was setup-maybe-ref rather than official rc9
setup-const. Preserve that full-map assertion and every original control.
The initial uncommitted global producer proposal would change immutable retained
DOM/SSR complete-result maps; withdraw that scope before publishing. Pass the
existing real is_vapor target into the same ScriptCompileContext statement walk,
without an extra AST walk, pipeline stage or metadata postpass. Only actual
Vapor named-value imports from parsed exact source vue are const; retained
DOM/SSR public maps and default context methods keep historical maybe-ref.
Other named sources remain maybe-ref and declaration/specifier type-only skips
remain exact. A whole parsed TS SFC checks both real target maps, Vue aliases,
an external named value and both type-only boundaries.
The source-backed public result authority is official
[rc9 compileScript lines756-765](https://github.com/vuejs/core/blob/v3.6.0-rc.9/packages/compiler-sfc/src/compileScript.ts#L756).
Independent stable3.5.35 execution corroborates that boundary, with no rc9
execution inferred; a local rc9 probe had no installed runtime alias and did
not run. Keep the Croquis/type34 template-fact model separate, with no broader
import/template/native acceptance. The retained DOM/SSR discrepancy remains
unfixed. Grandfathered parse/context/compile files do not grow; helper278.
Failed dbd has no complete32-pair acceptance; fresh source is required.

Independent preflight review found the existing extract_all_macros caller still
used the old parser signature after 4cbc was pushed. Correct that public
default-context entry point to explicit false, preserving retained semantics.
No source or runtime acceptance belongs to 4cbc; require fresh complete Actions.

Fresh main advanced to signed7364 while 4a was Draft; GitHub refused mergeability
and ordinary PR Actions did not start. Genuinely rebase all six owned commits
on that literal main, preserving their authors/messages/reporter trailers.
All eight other production blobs and the whole runtime inputs/helper/test stay
exact; shared compile.rs retains incoming Nuxt page-meta ownership, with only
the existing target-aware analysis call as this branch delta. Preserve every
incoming canonical byte at350; move only the own Vapor clause to the order
overview to avoid repeated unrelated changes in the Nuxt history paragraph.
Update the manifest base to the true new origin. No previous source result
transfers to this replay; fresh exact-head Actions remain required.

Actual6ae source Check37264729832 fails tooling workers1/3 only on the generated
SFC non-product BindingType census: existing131 sites versus actual133 after
the producer helper. Run the repository generator directly through its genuine
Node backend (the Rust command is a launcher), with no install/native build.
Only that derived row changes; resolved product sites, production/runtime
sources, fixture bytes, full assertions, caps and native qualifications stay
exact. Actual four source Rust shards complete15,906 executions with zero
failures/skips, including the strict32 whole-module runtime law and real
dual-target binding boundary. Their authenticated receipt remains source-only
evidence from the overall failed run; fresh complete Actions are required.

Current d7d Check37265487540 and all six associated workflows complete success.
Four authenticated Rust artifacts execute15,938 with zero failures/skips/errors,
including strict32 runtime4.459s and dual-target map0.012s; all census workers
pass. Its current-source receipt is distinct from census-red6ae. Genuine
prospective composition with latest signed4e main conflicts only on the order
overview, now also owned by the incoming CSS decision. Rebase all eight commits
on literal4e, preserving every original author/date/message/reporter trailer,
all nine production blobs and whole runtime input/helper/test/reference bytes.
Preserve all incoming canonical bytes at350; only the own Vapor clause moves
to the existing compiler fixture-sweep row. Bind the manifest to actual4e.
The changed head requires fresh exact Actions before Ready/auto/protected
admission; no green d7d result is transferred as current-head acceptance.

Fresh d728 source Check37270516589 and associated workflows pass, with15,977
successful executions and strict32 runtime/dual-target map tests. Independent
matched admission produces protected d52/Check37271681194, whose instruction
job111639821369 fails only SSR stress interpolation862525>850681. Immediately
dequeue #7967 and verify queue/auto null; it has not merged. No red admission
or source-only performance acceptance is retained.

Authenticate genuine literal0034 protected Check37271074703 raw artifact
11327713734 and candidate artifact11328553568: both100 ceilings measured three
times, all self-cost/edge/call maps identical within each head, fixture/caps/
windows/toolchain/guest/allocator unchanged. Parent842173 grows by20,352.
The change is concentrated in document appends: push_spanned now calls
CompactString push_str1002 times and push_escaped502, instead of inlined
reserve/copy paths; allocation-copy counts remain501. SSR production source
is unchanged, and the comment-free fixture never calls the guarded converter.
The authenticated failure packet SHA-256 is
51d3eeca846db50586d133ce31a2308759dbd0e4a7ea2052b6d9070416ace242.

Both real production callers already require contains("//"); preserve those
checks and the entire scanner, but declare the shared converter cold and
out of line. This is a narrow compilation hint, not measured savings. Replay
all nine commits genuinely onto signed literal0034, preserving original
source authors/messages/trailers, all eight other production blobs and all
whole-runtime custody. Incoming SSR/CSS repairs and complete canonical350
remain present; bind the manifest to the true origin. Fresh exact source and
all104 actual measurements must establish the result before re-admission.
No caps, fixtures, native qualification or strict32 assertions are waived.

TODO: exact source Actions including whole-module runtime, root peer review,
protected full suites and all 104 immutable instruction ceilings, signed
actual main with reporter trailer/issue closure, then a verified frequent cut.
