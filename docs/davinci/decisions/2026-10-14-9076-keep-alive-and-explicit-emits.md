# False positives in two opt-in rules, then promotion

Issue [#9076](https://github.com/ubugeeei-prod/vize/issues/9076). The
[#9059](https://github.com/ubugeeei-prod/vize/issues/9059) corpus run kept two rules
out of the presets because every finding they reported was a false positive.

- `vue/no-non-component-keep-alive-child` read `ElementType::Element` as a plain
  wrapper. The lint parse leaves kebab-case components such as `<n-layout>` and
  `<router-view>` as elements, so they were reported (naive-ui 6,
  vue-element-admin 2). The rule now reports only native tags.
- `script/require-explicit-emits` counted every `*.$emit(...)` as the component's
  own event, including `picker.$emit('pick')` in n8n (3) and
  `(this.$datepicker as IDatepicker).$emit(…)` in buefy (1). It now counts
  `this.$emit` and the `defineEmits` binding, as before.

After the fixes, both rules report nothing across the 34-project corpus in any
lintable file type, so each joins a preset:

- `vue/no-non-component-keep-alive-child` joins `happy-path`. A plain element below
  `<KeepAlive>` silently disables caching, which is a correctness problem.
- `script/require-explicit-emits` is eslint-plugin-vue strongly-recommended and
  an API-contract convention, so it joins the other `opinionated`-only script rules.

n8n uses `preset: "incremental"` and enables neither rule.
