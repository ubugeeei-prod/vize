# Narrowing inside an enclosing template branch

These cases retain the branch shapes behind three App check regressions:

- Elk `CommandPanel.vue`: a search/command union inside two `v-for` scopes and
  an enclosing loading/result branch.
- Elk `NotificationPaginator.vue`: a slot union excludes two grouped variants
  before binding the remaining notification, in both outer branches.
- Misskey `search.note.vue`: a nullable user is checked before the `v-else`
  branch passes it to a component inside an enclosing search-scope branch.

The complete original files are retained under `original/`, with repository,
immutable revision, original path and SHA256 in `original/sources.json`. Elk is
MIT licensed; Misskey is AGPL-3.0-only. Their original licenses are included.
The existing App check suite continues to check the complete pinned projects.

The six adjacent `.txt` files form an independently written executable
reduction. Copy them to `src/` without `.txt`, with strict TypeScript,
`target: ES2022`, `module: ESNext`, `moduleResolution: bundler` and `noEmit`.
`vue-tsc` 3.3.11, TypeScript 6.0.3 and Vue 3.6.0-beta.10 report no diagnostics.

The native Canon regression checks all six files as one real project and
asserts its entire diagnostic vector. Three separate negative controls change
only the receiving child's prop type to `number`:

| Changed prop | Expected authored errors |
| --- | --- |
| `cmd` | One TS2322 in `Command.vue` |
| `notification` | Two TS2322 in `Notification.vue` |
| `user` | One TS2322 in `User.vue` |

All four controls were also executed with the same `vue-tsc` oracle. The
negative controls keep every condition, component binding and source file.

The faulty generated chain repeated the parent condition in each branch:
`if (parent && first) … else if (parent && second) … else if (parent) …`.
TypeScript cannot infer the excluded child variants from that final parent
condition. Nesting the linear chain once under `if (parent)` retains them.
`oracle/branch-shape.ts.txt` preserves this direct TypeScript discriminator.
With strict checking and target ES2022, both TypeScript 6.0.3 and native
TypeScript 7.0.2 report only lines 7 (TS2339), 18 (TS2322), 29 (TS2322), and
40 (the genuine wrong-type TS2322); the corrected branches are clean.
The separate `flow-depth-isolation` corpus still guards against reverting to
quadratic repeated negations and the resulting cross-file TS2563 failure.
