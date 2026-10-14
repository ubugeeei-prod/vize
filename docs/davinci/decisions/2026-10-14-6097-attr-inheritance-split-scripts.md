# Attribute inheritance with two script blocks (#6097)

Issue: [#6097](https://github.com/ubugeeei-prod/vize/issues/6097).

`script/no-duplicate-attr-inheritance` skipped every SFC that had both `<script>` and `<script setup>`. The rule is invoked once per block, and `inheritAttrs` can be declared in either one, so neither invocation alone could decide. eslint-plugin-vue reads both blocks. The v0.452.0 Real Project Matrix (run 38515301568) recorded 37 false negatives across 7 projects, 28 of them in reka-ui components that keep their props interface in a plain `<script>`.

The SFC script dispatcher now passes each invocation the other block's already parsed program (`SfcScriptContext::sibling_program`), without an extra parse. The `<script setup>` invocation owns the whole-file conclusion. It reads `inheritAttrs` from both blocks and reports `v-bind="$attrs"` spreads once. A `false` or non-literal `inheritAttrs` in either block suppresses the finding, as with a single block. If the sibling did not parse cleanly, the rule stays silent.
