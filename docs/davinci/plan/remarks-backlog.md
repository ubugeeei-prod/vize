# Missed-remarks backlog (C-13)

> [!NOTE]
> Generated; do not edit. Mined from the TS-32 corpus baseline
> (`tests/_fixtures/davinci-remarks-baseline.folio`) by
> `davinci/vize_l1_to_l2/tests/davinci_remarks_corpus.rs`, which pins this file
> and rewrites it under `UPDATE_REMARKS_BASELINE=1`. Each item is one missed
> reason (a remark's arguments after its subject, per
> [remarks-format.md](./remarks-format.md)), ranked by corpus hits: the
> optimization backlog the P3-13 remarks mine for continuous task C-13.

Corpus: 551 files, 1228 remarks (233 applied, 995 missed), 27 missed reasons.

1. `s2.hoist-static static-subtree` `blocker="child" op="ui.interpolation"` - 188 hits in 84 files; first at `tests/_fixtures/_projects/class-component/src/App.vue` @354:376
2. `s2.hoist-static static-props` `blocker="binding" op="ui.on"` - 158 hits in 62 files; first at `tests/_fixtures/_projects/class-component/src/HelloDecorator.vue` @544:608
3. `s2.hoist-static static-props` `blocker="binding" op="ui.bind" rule="non-constant"` - 109 hits in 66 files; first at `tests/_fixtures/_projects/compiler-macros/src/DefineModelType.vue` @179:208
4. `s2.hoist-static static-subtree` `blocker="binding" op="ui.on"` - 93 hits in 34 files; first at `tests/_fixtures/_projects/class-component/src/HelloDecorator.vue` @544:608
5. `s2.hoist-static static-subtree` `blocker="binding" op="ui.bind" rule="non-constant"` - 48 hits in 29 files; first at `tests/_fixtures/_projects/compiler-macros/src/DefineModelType.vue` @179:208
6. `s2.hoist-static static-props` `blocker="binding" op="ui.model"` - 47 hits in 18 files; first at `tests/_fixtures/_projects/ecosystem-products/src/ComposablesAndRouting.vue` @3622:3772
7. `s2.hoist-static static-props` `blocker="binding" op="ui.slot-content"` - 45 hits in 30 files; first at `tests/_fixtures/_projects/ecosystem-products/src/UiLibraries.vue` @3108:3144
8. `s2.hoist-static static-subtree` `blocker="child" op="ui.element"` - 39 hits in 28 files; first at `tests/_fixtures/_projects/class-component/src/App.vue` @343:422
9. `s2.hoist-static static-subtree` `blocker="binding" op="ui.slot-content"` - 36 hits in 24 files; first at `tests/_fixtures/_projects/ecosystem-products/src/UiLibraries.vue` @3108:3144
10. `s2.hoist-static static-props` `blocker="binding" op="ui.bind" rule="dynamic-name"` - 34 hits in 28 files; first at `tests/_fixtures/differential/lint/attrs-expression-ranges-8090/sources/FalseOptions.vue` @87:113
11. `s2.hoist-static static-props` `blocker="binding" op="vue.directive"` - 31 hits in 14 files; first at `tests/_fixtures/_projects/generic-build/src/DirectiveBuiltins.vue` @393:534
12. `s2.hoist-static static-subtree` `blocker="child" op="ui.component"` - 29 hits in 22 files; first at `tests/_fixtures/_projects/ecosystem-products/src/ComposablesAndRouting.vue` @4057:4291
13. `s2.hoist-static static-subtree` `blocker="binding" op="ui.bind" rule="dynamic-name"` - 27 hits in 23 files; first at `tests/_fixtures/differential/lint/attrs-expression-ranges-8090/sources/FalseOptions.vue` @87:113
14. `s2.hoist-static static-subtree` `blocker="binding" op="vue.directive"` - 24 hits in 11 files; first at `tests/_fixtures/_projects/generic-build/src/DirectiveBuiltins.vue` @393:534
15. `s2.hoist-static static-props` `blocker="ref-attribute"` - 17 hits in 12 files; first at `tests/_fixtures/_projects/generic-build/src/BindingPatchFlags.vue` @660:1412
16. `s2.hoist-static static-subtree` `blocker="child" op="ui.for"` - 12 hits in 9 files; first at `tests/_fixtures/_projects/compiler-macros/src/DefinePropsGeneric.vue` @190:268
17. `s2.hoist-static static-props` `blocker="binding" op="ui.bind" rule="reserved-key"` - 11 hits in 10 files; first at `tests/_fixtures/_projects/generic-build/src/NormalScriptBindings.vue` @370:556
18. `s2.hoist-static static-subtree` `blocker="binding" op="ui.bind" rule="reserved-key"` - 11 hits in 10 files; first at `tests/_fixtures/_projects/generic-build/src/NormalScriptBindings.vue` @370:556
19. `s2.hoist-static static-subtree` `blocker="ref-attribute"` - 10 hits in 7 files; first at `tests/_fixtures/_projects/generic-build/src/BindingPatchFlags.vue` @660:1412
20. `s2.hoist-static static-subtree` `blocker="child" op="ui.slot"` - 9 hits in 9 files; first at `tests/_fixtures/_projects/compiler-macros/src/DefineSlotsType.vue` @188:272
21. `s2.hoist-static static-subtree` `blocker="binding" op="ui.model"` - 7 hits in 4 files; first at `tests/_fixtures/_projects/ecosystem-products/src/ComposablesAndRouting.vue` @3622:3772
22. `s2.hoist-static static-subtree` `blocker="svg-directive"` - 4 hits in 4 files; first at `tests/_fixtures/differential/lint/attrs-expression-ranges-8090/sources/Slot.vue` @92:115
23. `s2.hoist-static static-subtree` `blocker="child" op="ui.if"` - 2 hits in 2 files; first at `tests/_fixtures/differential/lint/transition-custom-child-6881/sources/OriginalHome.vue` @164:1201
24. `s2.hoist-static static-props` `blocker="binding" op="vue.once"` - 1 hits in 1 files; first at `tests/_fixtures/vue-language-tools/upstream/test-workspace/tsc/#4827/child.vue` @12:46
25. `s2.hoist-static static-props` `blocker="binding" op="vue.show"` - 1 hits in 1 files; first at `tests/_fixtures/differential/lint/transition-custom-child-6881/sources/native-toggle-show.vue` @59:78
26. `s2.hoist-static static-subtree` `blocker="binding" op="vue.once"` - 1 hits in 1 files; first at `tests/_fixtures/vue-language-tools/upstream/test-workspace/tsc/#4827/child.vue` @12:46
27. `s2.hoist-static static-subtree` `blocker="binding" op="vue.show"` - 1 hits in 1 files; first at `tests/_fixtures/differential/lint/transition-custom-child-6881/sources/native-toggle-show.vue` @59:78
