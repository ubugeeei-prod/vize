# Handoff: feat/jsx-l2-and-value-arms (PR #13409, J2b part 1)

Delete before merge. Stacked on #13382, which is stacked on #13400 (both in the merge queue at pause).

## State
- Value ternary arms lower natively and emit from L2, byte-identical to Relief. `clean_text` no longer copies every line; its helpers are byte loops.
- Allocation budgets tightened: 399→350, 338→310, 531→503.
- Tests pass: vize_atelier_jsx 476, vize_l1_to_l2 964. Clippy and the assertion lint are clean. The `fmt-rust` failure is fixed by the last style commit.
- Release profile: fat base run 44508605066, fat head run 44510668327 (`measure/jsx-fat-j2b`, head c29353b6bd). The 101 existing rows are identical; JSX rows fall (vdom −0.60%, babel −0.37%, vapor −0.20%). The PR body has the table.
- Twins in flight at pause:
  - base twin 44514652948 (`measure/jsx-fat-base`, done);
  - head twin 44514658304 (`measure/jsx-fat-j2b`).
  
  Download the `instruction-counts-*` artifacts and compare each with its first run. Both should be identical.

## Next
1. Add the twin results and the PR performance observation summary to the #13409 body. Remove this file, then enable auto-merge after #13382 merges (rebase onto main first).
2. J2b part 2 (`ui.and`), per the maintainer: keep today's bytes and replace the Relief path. This needs an `AndOp` fact for the falsy rendering (raw child vs keyed text/comment IIFE) and its DOM emission in `vize_l4` (vif.rs `emit_and`), which belongs to lane B, so sequence it with them. The bytes to reproduce: `(() => { const _jsx_valueN = LEFT; return (_jsx_valueN) ? <branch key 0> : (typeof … "number"/"string"/"bigint") ? Fragment{key:1}[createTextVNode(toDisplayString(_jsx_valueN),1)] : _createCommentVNode("v-if", true) })()`.
3. S5 follow-up (not started): replace `ScriptTables::from_croquis` in vize_atelier_jsx (compile.rs, vdom.rs, vapor.rs, ssr.rs and tests) with lane A's Q3 queries (branch `feat/6871-query-vue`, not merged yet). It needs reactive sources, SetupConst bindings and used component names for a JSX module, and the output must stay byte-identical.
4. Unopened branch `perf/l1-to-l2-line-comment-memchr` (memchr for `contains("//")`). It removed a ci-opt-only shift; ci-opt is now informational, so open it only if a release-profile run shows a gain.
