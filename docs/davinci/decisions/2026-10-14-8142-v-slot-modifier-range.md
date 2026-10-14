# `vue/valid-v-slot` modifier ranges (#8142)

Issue: [#8142](https://github.com/ubugeeei-prod/vize/issues/8142).

eslint-plugin-vue 10.9.2 reports a disallowed `v-slot` modifier from the first modifier to the last (`name` in `#item.name`, `raw.value` in `v-slot:item.raw.value`). Vize reported the whole directive. The v0.452.0 Real Project Matrix (run 38515301568) therefore recorded the same 52 vuetify findings as both 52 false positives and 52 false negatives.

Vize now reports the modifier span for both modifier messages: a slot argument without `allowModifiers`, and any modifier on an argless `v-slot`. Other `vue/valid-v-slot` findings keep the directive range.

The recorded packets in `slot-directive-validity-8142`, `slot-dynamic-modifiers-8142` and `slot-parameter-bindings-8142` were rewritten to the new modifier spans by computing them from each case's source. The parameter-bindings oracle now derives CLI and LSP coordinates from each diagnostic's own byte range, and its preserved-input hashes follow the rewritten files.
