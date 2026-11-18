## Summary

`vue/require-component-registration` missed registrations in a class component's `@Component({ components: { ... } })` decorator whenever the value was not a plain binding. In gridea's `BasicSetting.vue` (Real Project Matrix, #12336), `<v-nodes>` was reported as unregistered:

```ts
@Component({
  components: {
    VNodes: { functional: true, render: (h: any, ctx: any) => ctx.props.vnodes },
    ValidationProvider,
  },
})
export default class BasicSetting extends Vue {}
```

An inline object registration is still a registration: Vue registers whatever value the key names. The AST fallback that reads `export default { components }` and `defineComponent({ components })` now also reads the options object of a `@Component(...)` or `@Options(...)` decorator on the default-exported class. It matches keys, not values, as it already does for the other forms. Croquis is unchanged.

The same registrations now reach `vue/component-name-in-template-casing`, which checks registered components only. gridea writes those components in kebab case (`<language-setting>`, `<version>`), giving 29 new findings. eslint-plugin-vue reads registrations only from option objects, so the second commit classifies these findings in the lint matrix when an `@Component` or `@Options` components map registers the tag.

## Tests

- `component_registration::tests::reads_class_component_decorator_registrations_with_inline_values`: `v-nodes`, `VNodes` and `footer-box` are registered; `a-divider` is not; an unrelated `@Injectable({ components })` decorator registers nothing.
- Differential case `linter/decorator-inline-registration-12336`.
- `lint-divergence-component-registration-intentional.test.ts`: casing findings on decorator-registered tags are classified under both decorators; an unregistered tag and an option-object registration stay counted.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
