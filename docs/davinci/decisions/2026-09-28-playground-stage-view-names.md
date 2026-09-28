# Playground stage view naming (#6832)

The stage view already lives under `playground/src/features/stages/`. Its active
source and e2e file names now use stage/dump terms: `StagePlayground`,
`useStageLadder`, `DumpView`, `DumpDiffView`, `dumpLines`, and
`stage-ladder.test.ts`. Matching component, composable, line and token
identifiers change with the paths. The first commit moves nine files without
editing their contents; `tools/support/levels/rename-playground-stage-view.py`
replays the moves and reference updates after a conflict.

This is a source identity change. The stage page `kind` values and feed JSON,
the `Davinci` UI tab's route key and label, its preset source, user-visible
page/remark text, and VRT PNG bytes are unchanged. The generated Vapor test
snapshot updates only the renamed App component binding and import. The natural
v-on corpus shard changes two path cells for `DumpView.vue`; spellings and
counts stay the same. Verify
focused playground tests, typechecking, the browser stage ladder, ordinary
Actions, and the protected merge queue before treating this slice as landed.

Remaining #6832 naming inventory is explicit: the active
`playground/src/shared/presets/davinci.ts` path and its exported names; the
`Davinci` tab key, label, query parameter and VRT snapshot names; and the
`.davinci-*` CSS/DOM class names across the stage view. Those are not approved
exceptions. A later PR must rename them with URL/VRT migration evidence, or
the maintainer must approve a scoped exception. The shared production capture
and CLI work remain open independently.
