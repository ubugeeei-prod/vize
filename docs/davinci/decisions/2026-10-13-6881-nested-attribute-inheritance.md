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
