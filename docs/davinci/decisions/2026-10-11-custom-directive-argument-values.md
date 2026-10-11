# Custom directive argument values (#6879)

The frozen Element Plus matrix retained three TS2464 false positives at
`20298a21ae54a5417955373b80130ae7c548a6bb`: Cascader's `contentRef` and
Select/Select V2's `popperRef` are DOM element arguments to ClickOutside.
Canon projected these custom directive arguments as object computed keys,
which imposed an unrelated JavaScript `PropertyKey` restriction. Vue's real
`DirectiveBinding<Value, Modifiers, Arg>` defaults `Arg` to `any`.

Classify dynamic custom arguments from `DirectiveNode.name` and the parser's
exact argument span in the existing directive-table AST walk. Keep these
expressions in the existing mapped `void` expression path, preserving unknown
names and member errors. Keep every original directive hook/value check and
the computed-key path for built-in directives. No prefix heuristic, diagnostic
filter, new pipeline stage, budget adjustment, or golden change is introduced.

The minimized complete native corpus retains both upstream directive
spellings and genuine Vue bindings/refs. It requires the whole vector for
legal arguments, unknown arguments, invalid members, wrong handler values,
and each repair. The existing spread and dynamic-name native controls remain
required. Full original source/projection/diagnostic packets are retained in
the existing `native-source-recipe*` immutable artifact namespace.

The hosted replay now pins the actual main commit
`67e3d0e6775cc3a1f29bfb9309d991252f9e75ff`, which contains the merged strict
reach reader and the incoming production corrections. Restore the three
preexisting projection files to that commit's exact blobs and make the new
argument-metadata module absent. The complete compiled-source manifest requires
all other production/Cargo/lock bytes to equal that baseline; the label is
`67e3d0e production + candidate neutral test driver`, not an exact whole
baseline tree. The candidate neutral driver and build environment are identical
between phases. Keep the existing ci-opt target, native corpus and dependency
cache. Require the entire original three-row TS2464 vector, restore all four
candidate files, authenticate the exact original tracked HEAD and source hash,
and run every clean/negative/repair vector before all original native gates.
An exit trap restores the candidate on failure. No extra target/builder or
generic tolerated baseline failure is introduced.

The previous source `b32ae8b120501bcb166c91b5a9d1e4112b5cf294` qualified
its fixed `006c564` production replay and all native gates in
[run 38151468812](https://github.com/ubugeeei-prod/vize/actions/runs/38151468812).
The complete compiled-source manifests authenticate precisely four production
changes and an unchanged neutral driver. Artifact 11706873164 has ZIP SHA256
`78872674edf244095016ab4f2675635d4cc95c73898c63010e052ca177b058fd`.
The native executable is the official Linux x64 TypeScript 7.0.2 distribution,
SHA256 `4f2de678286401759b3fb4475bafe35b8f32b4b3a07d92642bbf37eadc9b34a4`.

The unchanged full original Element Plus typecheck matrix also qualified b32
in [run 38152547464, shard 2](https://github.com/ubugeeei-prod/vize/actions/runs/38152547464/job/114508798224):
all 224 input cases, baseline configuration, mutation oracle and diagnostic
budgets remain identical to the original RED run 38147644073. All three
TS2464 false positives become zero, false negatives stay zero, and all 26 shared
diagnostics and six existing message mismatches remain unchanged. Artifact
11707377284 ZIP SHA256 is
`2eedf5ae8a1884f334665706fd13a3f1f5ce675a7f3abee5cc1fe091696a80ee`.
Only the full Element Plus typecheck surface is green; other Matrix surfaces
failed and no overall Matrix or release acceptance is claimed.

The b32 default gate found one unreadable reach input. Its old aggregate-only
observer discarded the original path and OS error, so those remain unknown.
[Independent PR #8682](https://github.com/ubugeeei-prod/vize/pull/8682) migrated
only that sweep caller to the existing strict componentwise reader and added
physical alias/cycle/UTF-8 caller laws. It actually merged as
`d7a881c314d811b51587e8206348ada49e07816e` after
[full protected run 38157531089](https://github.com/ubugeeei-prod/vize/actions/runs/38157531089).
Every one of the original 44,367 complete input tuples remains identical;
unreadable is zero and all 21 reach laws pass. This proves strict full original
qualification, not an attribution of the missing old path or OS error.

The first unit input's self-closing non-void `div` produced the parser's real
ExtendPoint warning. Use an explicit closing tag while keeping the whole
no-error parser expectation and exact argument-span checks. Classification and
all original diagnostic/projection/budget/golden contracts are unchanged.

After restacking onto 67e3d0e, the production delta is still exactly the same
four projection files; all incoming main changes and native commands are retained.
The foundation remains 350 lines with only the reserved paragraph 347 appendix.
Fresh source/native/default qualification, full original 224-case typecheck
acceptance, protected actual merge and release acceptance remain pending.
