# Accessible image headings and conditional landmarks

Issues: [#7332](https://github.com/ubugeeei-prod/vize/issues/7332),
[#7328](https://github.com/ubugeeei-prod/vize/issues/7328).

Heading content includes descendant images with a nonempty static or bound
`alt`. Empty literal alternatives, missing alternatives, and descendants of
`aria-hidden="true"` remain inaccessible. The same rule body serves Vue and
direct/lowered JSX, with cross-backend regression assertions.

Landmarks record the choices along their conditional ancestry. Only elements
whose choices can coexist participate in duplicate-main, missing-label, and
duplicate-label diagnostics. Raw sibling `v-if` chains and transformed `If`
nodes retain this information, including nested branches. Independent chains,
multiple landmarks within a branch, outside landmarks, and `v-show` still
report actual duplicates.

The 23-case public SFC corpus in
`tests/_fixtures/differential/lint-accessible-content/cases.json` exercises
positive and negative cases. Authored image and conditional SFCs also enter the
repository differential sweep. Validation runs through the PR and merge-queue
Actions; unrelated native migration and fix-history completion stay open.
