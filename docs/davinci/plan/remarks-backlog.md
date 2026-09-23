# Missed-remarks backlog (C-13)

> [!NOTE]
> Generated; do not edit. Mined from the TS-32 corpus baseline
> (`tests/_fixtures/davinci-remarks-baseline.folio`) by
> `crates/vize_s1_to_s2/tests/davinci_remarks_corpus.rs`, which pins this file
> and rewrites it under `UPDATE_REMARKS_BASELINE=1`. Each item is one missed
> reason (a remark's arguments after its subject, per
> [remarks-format.md](./remarks-format.md)), ranked by corpus hits: the
> optimization backlog the P3-13 remarks mine for continuous task C-13.

Corpus: 442 files, 1024 remarks (152 applied, 872 missed), 24 missed reasons.

1. `s2.hoist-static static-subtree` `blocker="child" op="ui.interpolation"` - 185 hits in 81 files; first at `tests/_fixtures/_projects/class-component/src/App.vue` @354:376
2. `s2.hoist-static static-props` `blocker="binding" op="ui.on"` - 154 hits in 60 files; first at `tests/_fixtures/_projects/class-component/src/HelloDecorator.vue` @544:608
3. `s2.hoist-static static-props` `blocker="binding" op="ui.bind" rule="non-constant"` - 97 hits in 57 files; first at `tests/_fixtures/_projects/compiler-macros/src/DefineModelType.vue` @179:208
4. `s2.hoist-static static-subtree` `blocker="binding" op="ui.on"` - 91 hits in 32 files; first at `tests/_fixtures/_projects/class-component/src/HelloDecorator.vue` @544:608
5. `s2.hoist-static static-props` `blocker="binding" op="ui.model"` - 41 hits in 16 files; first at `tests/_fixtures/_projects/ecosystem-products/src/ComposablesAndRouting.vue` @3622:3772
6. `s2.hoist-static static-props` `blocker="binding" op="ui.slot-content"` - 41 hits in 26 files; first at `tests/_fixtures/_projects/ecosystem-products/src/UiLibraries.vue` @3108:3144
7. `s2.hoist-static static-subtree` `blocker="binding" op="ui.bind" rule="non-constant"` - 41 hits in 24 files; first at `tests/_fixtures/_projects/compiler-macros/src/DefineModelType.vue` @179:208
8. `s2.hoist-static static-subtree` `blocker="binding" op="ui.slot-content"` - 32 hits in 20 files; first at `tests/_fixtures/_projects/ecosystem-products/src/UiLibraries.vue` @3108:3144
9. `s2.hoist-static static-props` `blocker="binding" op="vue.directive"` - 30 hits in 13 files; first at `tests/_fixtures/_projects/generic-build/src/DirectiveBuiltins.vue` @393:534
10. `s2.hoist-static static-subtree` `blocker="child" op="ui.element"` - 28 hits in 18 files; first at `tests/_fixtures/_projects/class-component/src/App.vue` @343:422
11. `s2.hoist-static static-subtree` `blocker="binding" op="vue.directive"` - 23 hits in 10 files; first at `tests/_fixtures/_projects/generic-build/src/DirectiveBuiltins.vue` @393:534
12. `s2.hoist-static static-subtree` `blocker="child" op="ui.component"` - 23 hits in 17 files; first at `tests/_fixtures/_projects/ecosystem-products/src/ComposablesAndRouting.vue` @4057:4291
13. `s2.hoist-static static-props` `blocker="ref-attribute"` - 17 hits in 12 files; first at `tests/_fixtures/_projects/generic-build/src/BindingPatchFlags.vue` @660:1412
14. `s2.hoist-static static-subtree` `blocker="child" op="ui.for"` - 11 hits in 8 files; first at `tests/_fixtures/_projects/compiler-macros/src/DefinePropsGeneric.vue` @190:268
15. `s2.hoist-static static-props` `blocker="binding" op="ui.bind" rule="reserved-key"` - 10 hits in 9 files; first at `tests/_fixtures/_projects/generic-build/src/NormalScriptBindings.vue` @370:556
16. `s2.hoist-static static-subtree` `blocker="binding" op="ui.bind" rule="reserved-key"` - 10 hits in 9 files; first at `tests/_fixtures/_projects/generic-build/src/NormalScriptBindings.vue` @370:556
17. `s2.hoist-static static-subtree` `blocker="ref-attribute"` - 10 hits in 7 files; first at `tests/_fixtures/_projects/generic-build/src/BindingPatchFlags.vue` @660:1412
18. `s2.hoist-static static-subtree` `blocker="binding" op="ui.model"` - 7 hits in 4 files; first at `tests/_fixtures/_projects/ecosystem-products/src/ComposablesAndRouting.vue` @3622:3772
19. `s2.hoist-static static-subtree` `blocker="child" op="ui.slot"` - 7 hits in 7 files; first at `tests/_fixtures/_projects/compiler-macros/src/DefineSlotsType.vue` @188:272
20. `s2.hoist-static static-props` `blocker="binding" op="ui.bind" rule="dynamic-name"` - 6 hits in 6 files; first at `tests/_fixtures/vue-language-tools/upstream/test-workspace/tsc/#2166/main.vue` @13:60
21. `s2.hoist-static static-subtree` `blocker="binding" op="ui.bind" rule="dynamic-name"` - 5 hits in 5 files; first at `tests/_fixtures/vue-language-tools/upstream/test-workspace/tsc/#2166/main.vue` @13:60
22. `s2.hoist-static static-props` `blocker="binding" op="vue.once"` - 1 hits in 1 files; first at `tests/_fixtures/vue-language-tools/upstream/test-workspace/tsc/#4827/child.vue` @12:46
23. `s2.hoist-static static-subtree` `blocker="binding" op="vue.once"` - 1 hits in 1 files; first at `tests/_fixtures/vue-language-tools/upstream/test-workspace/tsc/#4827/child.vue` @12:46
24. `s2.hoist-static static-subtree` `blocker="child" op="ui.if"` - 1 hits in 1 files; first at `tests/_fixtures/vue-language-tools/upstream/test-workspace/tsc/#3295/main.vue` @12:68
