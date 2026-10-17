# App check diagnostic changes and nested branch narrowing

Related: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830),
[#9005](https://github.com/ubugeeei-prod/vize/pull/9005),
[the original provider audit](./2026-10-11-app-check-provider-baseline-audit.md),
and [generic call inference](./2026-10-11-6879-generic-call-inference.md).

## Observed whole-report changes

App E2E [run 38866347055](https://github.com/ubugeeei-prod/vize/actions/runs/38866347055)
retains complete actual check reports. Comparing every original file object
and diagnostic multiset against the committed goldens gives:

| Project | Reported files before / after | Diagnostics before / after | Changed file objects |
| ------- | ----------------------------- | -------------------------- | -------------------- |
| Elk     | 279 / 279                     | 3 / 6                      | 2                    |
| ReKa UI | 928 / 928                     | 5 / 4                      | 1                    |
| Misskey | 794 / 794                     | 552 / 3                    | 109                  |

The file-name sets are identical. This proves reported membership; it does not
substitute for an authenticated native program or provider-read manifest.

## Confirmed regression: parent guards lose child exclusions

Commit `14d5bd91208f26b412159bdbb279847231fea107` replaced independently
repeated branch guards with a linear `if` / `else if` chain. It fixed a real
control-flow depth failure: TS2563 in generated template code stopped useful
flow typing in later files. That linearization remains necessary.

For a branch nested under a parent condition, however, it emitted:

```ts
if (parent && item.type === "search") {
  useSearch(item);
} else if (parent) {
  useCommand(item.cmd);
}
```

TypeScript does not infer the rejected search variant when the parent
condition is repeated in the final branch. The same issue affects a grouped
notification union and a nullable user. The corrected output nests the linear
chain under one parent guard:

```ts
if (parent) {
  if (item.type === "search") {
    useSearch(item);
  } else {
    useCommand(item.cmd);
  }
}
```

The emitter now retains the enclosing guard across adjacent branch blocks,
including repeated checks in one branch. Each branch condition still appears
once. Non-adjacent checks retain their complete original guards.

The confirmed affected bindings are:

- Elk `app/components/command/CommandPanel.vue:224`, `item.cmd`: TS2339.
- Elk `app/components/notification/NotificationPaginator.vue:201` and `:219`,
  `item`: two TS2322 errors.
- Misskey `packages/frontend/src/pages/search.note.vue:71`, `user`: TS2322.

The complete pinned files, source hashes and licenses are retained in
`tests/_fixtures/differential/typechecker/nested-branch-chain/original/`.
The existing full App corpus and authored conditions remain unchanged.

## Reference and regression evidence

TypeScript 6.0.3 and the native TypeScript 7.0.2 compiler both reproduce the
three kinds of false positives in the flattened conditional form. Both accept
the nested form and retain a genuine wrong-prop refusal.

The executable Vue corpus preserves the nested `v-for`, typed slot and
nullable prop boundaries. With `vue-tsc` 3.3.11, TypeScript 6.0.3 and Vue
3.6.0-beta.10, its complete valid project has no diagnostics. Changing each
receiving prop to `number` independently gives exactly one command, two
notification and one user TS2322 errors. The Canon integration test asserts
those complete vectors through the native batch checker.

Emitter unit laws additionally cover repeated nested checks and a hundred
nested branches, requiring each condition to occur once. The existing full
`flow-depth-isolation` fixture and CLI regression remain unchanged, preserving
the cross-file contextual-typing guarantee from #9005.

## Other diagnostic reductions require separate classification

ReKa's single removed row is the PinInput text-model false positive addressed
by `e127ba6bce` in #8614. The authored generic has no default; inventing an
`any` default chose the numeric branch of its conditional prop type. The
existing generic-call native contract retains the original text case and
genuine wrong-number and authored-`any` refusals. Independent TypeScript
6.0.3 and native 7.0.2 signature controls agree with that distinction.

Misskey removes 493 TS7006, 47 TS7031, three TS2578 and seven TS2345 rows,
and adds the nullable-user regression above. The earlier #9005 investigation
directly established cross-file implicit-`any` and unused-directive false
positives caused by TS2563. This explains the mechanism behind most of the
current reduction, but its earlier 95-file report does not independently
qualify all 109 changed file objects here. The seven argument-type rows in
`src/preferences/manager.ts` reject `key as keyof typeof PREF_DEF` when the
receiving function requires `K extends keyof PREF`, although the authored
alias is `type PREF = typeof PREF_DEF`. Native TypeScript 7.0.2 accepts all
seven call shapes against an equivalent mapped generic provider and still
rejects an unknown key. That control establishes the expected generic
semantics; it is not a replay of the entire original provider graph. A
separate forced-TS2563 control did not reproduce those seven rows, so their
individual cause is not claimed as directly proved here.
The separate Misskey SDK declaration-preparation change is provider evidence,
not proof of the cause of every removed diagnostic.

## Reviewed baseline proposal

The ReKa baseline removes only the exact PinInput text-model row and changes
the error count from five to four. All 928 original file objects and their
order remain. The resulting whole snapshot is byte-identical to the retained
actual report, SHA256
`ea3906a151a6a447008581d0ded1d6b791beb8e32ab9b2098ccb7bfcd60cf838`.
The App check has no PinInput-specific assertion to remove: it asserts the
whole diagnostic snapshot. ReKa remains in the original intentional-error
injection suite, and the native generic contract retains wrong-number checks.

The Misskey proposal starts from the old golden, preserving all 794 file
objects and their order. It removes only the exact 550 diagnostic identities
absent from the retained actual report, after checking the category counts
above and locating all seven TS2345 rows in `src/preferences/manager.ts`.
It does not add the new nullable-user false positive fixed by this patch.
The two retained original diagnostic strings are unchanged: TS7006 at
`src/pages/admin-file.vue:7:58` and TS7016 at
`src/pages/admin/modlog.ModLog.vue:232:26`. The resulting snapshot SHA256 is
`0008722c162efd00f9953601805920f46ca69c9689effa3af159c7436945812e`.
Elk's original three-diagnostic golden remains unchanged.

These proposals distinguish inherited main improvements from this patch's
four branch-narrowing corrections. They suppress no product diagnostics,
remove no source files, and alter no intentional-error control. Fresh
exact-source App Actions must confirm the whole original report scopes.
Complete original-provider reference parity remains a separate qualification;
the focused oracles and baseline proposals do not claim that it is finished.
