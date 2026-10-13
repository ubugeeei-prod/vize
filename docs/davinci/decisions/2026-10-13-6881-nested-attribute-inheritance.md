# Attribute inheritance in nested and scriptless SFCs

Issue: [#6881](https://github.com/ubugeeei-prod/vize/issues/6881)

The v0.448.0 Matrix (38331707741, source
`2817a0aa13518dd46c16bc8bf56ac831f68c24dc`) retained mismatched directive
ranges and missing nested/scriptless `vue/no-duplicate-attr-inheritance`
findings. Native comparison keeps its zero false-positive and false-negative
ceilings; rule selection, severity and the comparator remain unchanged.

The native SFC rule reports the first unbound `$attrs` reference in each
argumentless `v-bind`, including nested elements. It uses the existing cached,
validated template AST. Template-only SFCs enter the existing script stage
through the actual configured builtin, without synthetic scripts or another
SFC parse. Disabled rules, script overrides and registered namesakes retain
their existing dispatch.

Ordinary fragments, explicit disabled/unknown inheritance and foreign loop,
slot or lexical bindings remain conservative. A conditional root group stays
comparable. Standalone JavaScript retains its existing redundant-true result.

The durable corpus preserves seven whole publicly pinned SFCs, all eight
original reference tuples, the original native mismatches and 20 additional
scope controls. GUI's historical baseline parse failure remains `UNUSABLE`;
its independently parsed finite inputs do not qualify the original project.
The installed eslint-plugin-vue 10.9.2 reference passed all 27 cases; native
qualification belongs to Actions. Full native packets include severity,
message, byte range, help, labels, fixes and counts.

The initial source-plan run 38337620437 refused a stale Patina source inventory.
Regenerating that existing inventory changes only its Patina shard; all checks
remain required on the successor head. The original failure is retained.

The namesake control lives in an integration test outside the production rule
catalog. Actual Actions then exposed a borrowed temporary and repository
string/slicing checks; those bounded source repairs retain the same rule laws.
Current source bookkeeping updates three existing rows and their whole-manifest
pins, retaining all 206 historical bodies and the remaining 320 current rows.

Native run 38339272169 retained an escaped-loop-alias false positive. The raw
AST can lack structural aliases, so its fallback now uses the existing shared
loop-scope parser: decoded aliases bind, collection expressions remain reads.
The adjacent `$attrs` collection-read control prevents blanket suppression.
Executable SFC Bad/Good examples and their bilingual explanations, generated
pages and three explain snapshots now describe this actual SFC behavior.

Source qualification of `3f5dcc8001dbd4a258aaac3a2a8e8bcaedec190e`
exposed two historical documentation custody failures. The
[named whole-body successor](../../../tests/_fixtures/differential/linter/attribute-inheritance-reference/documentation-successor.json.gz)
preserves the original 317-rule fence inventory and private-import explain
contract. Two complete bilingual documents and three complete explain bodies
permit only the authored inheritance Bad/Good replacements, their paragraphs,
and the implementation line link. The original private-import transformation,
whole snapshot hashes and lengths still run before that finite successor.
Mutation, omission, unknown-path and neighboring-rule controls retain strict
refusals; generated documentation is not a reason to accept arbitrary output.

The same receipt composes main
`bd5702c0a4d5fb3bb64487794fc45f6f876e6e38` with that exact PR source.
All 323 current source rows are compared as complete identities: three
inheritance rows come from the PR and one JSX component row from main. Every
other row, both complete historical source frames and all previous manifest
fields remain unchanged. The two helper/law checksum edits each have a
complete before/after inverse from both parents; every assertion is preserved.
The receipt grants source custody only. It grants no native, performance,
merge, release, or complete historical Matrix qualification.

The cumulative native Stack counterpart uses exact parent #8804
`2b8c61a3fdb46fb5117c4730b8ec65080ffa6ec2`. Eleven differing source identities
retain complete bodies from both parents; the two owned callers preserve PR
bytes, all other production bytes come from the parent, and four inventory
edits have complete forward/inverse witnesses with generator verification.
The sealed proof remains historical. The unchanged original physical/refusal
family qualifies later live sources, rather than freezing later children to a
past helper or manifest. Live documentation still authenticates the original
317-rule/private-import fixtures and the complete five named documents.
Four explicit `void` registrations repair the actual configured lint refusal
in source run 38355192945 without changing any assertion.
