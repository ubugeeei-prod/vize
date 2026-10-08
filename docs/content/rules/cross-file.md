---
title: Cross-file rules
---

# Cross-file rules

Project checks need the complete analyzed component graph. Each linked page gives shared files and the exact Bad/Good changes; apply shared files to both examples.

Start with the Vite+ configuration below. `vp run lint` invokes Vize and Oxlint; built-in `vp lint` runs its own upstream checker.

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "cross-file": "warn" },
    },
  },
});
```

```sh
vp run lint
```

The public CLI exposes the same pass with `vize lint --cross-file`. Displayed `vize:croquis/cf/*` codes use `croquis/cf/*` in `lint.vize.rules` (omit `vize:`). Information/hint diagnostics become CLI warnings. Related locations explain the source/consumer relationship.

The 60 published cross-file codes have different support boundaries: 19 belong to the CLI pass (18 complete source pairs and one reactive-graph scenario); 16 have experimental Rust analyzer producers but are not individually emitted by that pass; 25 are published contracts without a current diagnostic producer. Enabling a rule ID does not activate an unavailable producer.

## Project-specific lint IDs

| Rule | Examples | Severity |
| --- | --- | --- |
| [`ecosystem/vue-router-unknown-route`](./project/ecosystem-vue-router-unknown-route.md) | [Bad](./project/ecosystem-vue-router-unknown-route.md#bad) · [Good](./project/ecosystem-vue-router-unknown-route.md#good) | error |
| [`ecosystem/vue-router-extra-param`](./project/ecosystem-vue-router-extra-param.md) | [Bad](./project/ecosystem-vue-router-extra-param.md#bad) · [Good](./project/ecosystem-vue-router-extra-param.md#good) | error |
| [`ecosystem/vue-router-param-type`](./project/ecosystem-vue-router-param-type.md) | [Bad](./project/ecosystem-vue-router-param-type.md#bad) · [Good](./project/ecosystem-vue-router-param-type.md#good) | error |
| [`ecosystem/vue-router-missing-param`](./project/ecosystem-vue-router-missing-param.md) | [Bad](./project/ecosystem-vue-router-missing-param.md#bad) · [Good](./project/ecosystem-vue-router-missing-param.md#good) | warning |
| [`html/cross-component-nesting`](./project/html-cross-component-nesting.md) | [Bad](./project/html-cross-component-nesting.md#bad) · [Good](./project/html-cross-component-nesting.md#good) | warning |
| [`vue/cross-file-attrs-fallthrough`](./project/vue-cross-file-attrs-fallthrough.md) | [Bad](./project/vue-cross-file-attrs-fallthrough.md#bad) · [Good](./project/vue-cross-file-attrs-fallthrough.md#good) | warning |

## Published analyzer codes

| Code | Examples | Status |
| --- | --- | --- |
| [`vize:croquis/cf/array-mutation`](./project/vize-croquis-cf-array-mutation.md) | [Bad](./project/vize-croquis-cf-array-mutation.md#bad) · [Good](./project/vize-croquis-cf-array-mutation.md#good) | Contract only; no current producer |
| [`vize:croquis/cf/async-boundary`](./project/vize-croquis-cf-async-boundary.md) | [Bad](./project/vize-croquis-cf-async-boundary.md#bad) · [Good](./project/vize-croquis-cf-async-boundary.md#good) | CLI |
| [`vize:croquis/cf/async-no-suspense`](./project/vize-croquis-cf-async-no-suspense.md) | [Bad](./project/vize-croquis-cf-async-no-suspense.md#bad) · [Good](./project/vize-croquis-cf-async-no-suspense.md#good) | Rust analyzer; CLI uses a different surface or disables this pass |
| [`vize:croquis/cf/browser-api-ssr`](./project/vize-croquis-cf-browser-api-ssr.md) | [Bad](./project/vize-croquis-cf-browser-api-ssr.md#bad) · [Good](./project/vize-croquis-cf-browser-api-ssr.md#good) | CLI |
| [`vize:croquis/cf/circular-dep`](./project/vize-croquis-cf-circular-dep.md) | [Bad](./project/vize-croquis-cf-circular-dep.md#bad) · [Good](./project/vize-croquis-cf-circular-dep.md#good) | Contract only; no current producer |
| [`vize:croquis/cf/circular-reactive-dependency`](./project/vize-croquis-cf-circular-reactive-dependency.md) | [Bad](./project/vize-croquis-cf-circular-reactive-dependency.md#bad) · [Good](./project/vize-croquis-cf-circular-reactive-dependency.md#good) | CLI |
| [`vize:croquis/cf/closure-captures-reactive`](./project/vize-croquis-cf-closure-captures-reactive.md) | [Bad](./project/vize-croquis-cf-closure-captures-reactive.md#bad) · [Good](./project/vize-croquis-cf-closure-captures-reactive.md#good) | Contract only; no current producer |
| [`vize:croquis/cf/composable-outside-setup`](./project/vize-croquis-cf-composable-outside-setup.md) | [Bad](./project/vize-croquis-cf-composable-outside-setup.md#bad) · [Good](./project/vize-croquis-cf-composable-outside-setup.md#good) | Contract only; no current producer |
| [`vize:croquis/cf/computed-side-effects`](./project/vize-croquis-cf-computed-side-effects.md) | [Bad](./project/vize-croquis-cf-computed-side-effects.md#bad) · [Good](./project/vize-croquis-cf-computed-side-effects.md#good) | Contract only; no current producer |
| [`vize:croquis/cf/deep-import`](./project/vize-croquis-cf-deep-import.md) | [Bad](./project/vize-croquis-cf-deep-import.md#bad) · [Good](./project/vize-croquis-cf-deep-import.md#good) | Contract only; no current producer |
| [`vize:croquis/cf/destructuring-breaks-reactivity`](./project/vize-croquis-cf-destructuring-breaks-reactivity.md) | [Bad](./project/vize-croquis-cf-destructuring-breaks-reactivity.md#bad) · [Good](./project/vize-croquis-cf-destructuring-breaks-reactivity.md#good) | CLI |
| [`vize:croquis/cf/di-outside-setup`](./project/vize-croquis-cf-di-outside-setup.md) | [Bad](./project/vize-croquis-cf-di-outside-setup.md#bad) · [Good](./project/vize-croquis-cf-di-outside-setup.md#good) | Contract only; no current producer |
| [`vize:croquis/cf/dom-access-without-next-tick`](./project/vize-croquis-cf-dom-access-without-next-tick.md) | [Bad](./project/vize-croquis-cf-dom-access-without-next-tick.md#bad) · [Good](./project/vize-croquis-cf-dom-access-without-next-tick.md#good) | Contract only; no current producer |
| [`vize:croquis/cf/duplicate-id`](./project/vize-croquis-cf-duplicate-id.md) | [Bad](./project/vize-croquis-cf-duplicate-id.md#bad) · [Good](./project/vize-croquis-cf-duplicate-id.md#good) | CLI |
| [`vize:croquis/cf/event-listener-leak`](./project/vize-croquis-cf-event-listener-leak.md) | [Bad](./project/vize-croquis-cf-event-listener-leak.md#bad) · [Good](./project/vize-croquis-cf-event-listener-leak.md#good) | Contract only; no current producer |
| [`vize:croquis/cf/event-modifier`](./project/vize-croquis-cf-event-modifier.md) | [Bad](./project/vize-croquis-cf-event-modifier.md#bad) · [Good](./project/vize-croquis-cf-event-modifier.md#good) | Rust analyzer; CLI uses a different surface or disables this pass |
| [`vize:croquis/cf/hydration-risk`](./project/vize-croquis-cf-hydration-risk.md) | [Bad](./project/vize-croquis-cf-hydration-risk.md#bad) · [Good](./project/vize-croquis-cf-hydration-risk.md#good) | CLI |
| [`vize:croquis/cf/inherit-attrs-unused`](./project/vize-croquis-cf-inherit-attrs-unused.md) | [Bad](./project/vize-croquis-cf-inherit-attrs-unused.md#bad) · [Good](./project/vize-croquis-cf-inherit-attrs-unused.md#good) | Rust analyzer; CLI uses a different surface or disables this pass |
| [`vize:croquis/cf/inject-without-symbol`](./project/vize-croquis-cf-inject-without-symbol.md) | [Bad](./project/vize-croquis-cf-inject-without-symbol.md#bad) · [Good](./project/vize-croquis-cf-inject-without-symbol.md#good) | CLI |
| [`vize:croquis/cf/injected-async-mutation-race`](./project/vize-croquis-cf-injected-async-mutation-race.md) | [Bad](./project/vize-croquis-cf-injected-async-mutation-race.md#bad) · [Good](./project/vize-croquis-cf-injected-async-mutation-race.md#good) | CLI |
| [`vize:croquis/cf/lifecycle-outside-setup`](./project/vize-croquis-cf-lifecycle-outside-setup.md) | [Bad](./project/vize-croquis-cf-lifecycle-outside-setup.md#bad) · [Good](./project/vize-croquis-cf-lifecycle-outside-setup.md#good) | Contract only; no current producer |
| [`vize:croquis/cf/lifecycle-without-cleanup`](./project/vize-croquis-cf-lifecycle-without-cleanup.md) | [Bad](./project/vize-croquis-cf-lifecycle-without-cleanup.md#bad) · [Good](./project/vize-croquis-cf-lifecycle-without-cleanup.md#good) | Rust analyzer; CLI uses a different surface or disables this pass |
| [`vize:croquis/cf/missing-required-prop`](./project/vize-croquis-cf-missing-required-prop.md) | [Bad](./project/vize-croquis-cf-missing-required-prop.md#bad) · [Good](./project/vize-croquis-cf-missing-required-prop.md#good) | Rust analyzer; CLI uses a different surface or disables this pass |
| [`vize:croquis/cf/missing-suspense`](./project/vize-croquis-cf-missing-suspense.md) | [Bad](./project/vize-croquis-cf-missing-suspense.md#bad) · [Good](./project/vize-croquis-cf-missing-suspense.md#good) | Contract only; no current producer |
| [`vize:croquis/cf/module-scope-reactive`](./project/vize-croquis-cf-module-scope-reactive.md) | [Bad](./project/vize-croquis-cf-module-scope-reactive.md#bad) · [Good](./project/vize-croquis-cf-module-scope-reactive.md#good) | Contract only; no current producer |
| [`vize:croquis/cf/multi-root-attrs`](./project/vize-croquis-cf-multi-root-attrs.md) | [Bad](./project/vize-croquis-cf-multi-root-attrs.md#bad) · [Good](./project/vize-croquis-cf-multi-root-attrs.md#good) | Rust analyzer; CLI uses a different surface or disables this pass |
| [`vize:croquis/cf/mutated-after-escape`](./project/vize-croquis-cf-mutated-after-escape.md) | [Bad](./project/vize-croquis-cf-mutated-after-escape.md#bad) · [Good](./project/vize-croquis-cf-mutated-after-escape.md#good) | Contract only; no current producer |
| [`vize:croquis/cf/non-reactive-provide`](./project/vize-croquis-cf-non-reactive-provide.md) | [Bad](./project/vize-croquis-cf-non-reactive-provide.md#bad) · [Good](./project/vize-croquis-cf-non-reactive-provide.md#good) | CLI |
| [`vize:croquis/cf/non-unique-id`](./project/vize-croquis-cf-non-unique-id.md) | [Bad](./project/vize-croquis-cf-non-unique-id.md#bad) · [Good](./project/vize-croquis-cf-non-unique-id.md#good) | CLI |
| [`vize:croquis/cf/object-identity-comparison`](./project/vize-croquis-cf-object-identity-comparison.md) | [Bad](./project/vize-croquis-cf-object-identity-comparison.md#bad) · [Good](./project/vize-croquis-cf-object-identity-comparison.md#good) | Contract only; no current producer |
| [`vize:croquis/cf/pinia-getter`](./project/vize-croquis-cf-pinia-getter.md) | [Bad](./project/vize-croquis-cf-pinia-getter.md#bad) · [Good](./project/vize-croquis-cf-pinia-getter.md#good) | Contract only; no current producer |
| [`vize:croquis/cf/prop-type-mismatch`](./project/vize-croquis-cf-prop-type-mismatch.md) | [Bad](./project/vize-croquis-cf-prop-type-mismatch.md#bad) · [Good](./project/vize-croquis-cf-prop-type-mismatch.md#good) | Rust analyzer; CLI uses a different surface or disables this pass |
| [`vize:croquis/cf/provide-inject-type`](./project/vize-croquis-cf-provide-inject-type.md) | [Bad](./project/vize-croquis-cf-provide-inject-type.md#bad) · [Good](./project/vize-croquis-cf-provide-inject-type.md#good) | CLI |
| [`vize:croquis/cf/provide-without-symbol`](./project/vize-croquis-cf-provide-without-symbol.md) | [Bad](./project/vize-croquis-cf-provide-without-symbol.md#bad) · [Good](./project/vize-croquis-cf-provide-without-symbol.md#good) | CLI |
| [`vize:croquis/cf/reactive-export`](./project/vize-croquis-cf-reactive-export.md) | [Bad](./project/vize-croquis-cf-reactive-export.md#bad) · [Good](./project/vize-croquis-cf-reactive-export.md#good) | Contract only; no current producer |
| [`vize:croquis/cf/reactivity-outside-setup`](./project/vize-croquis-cf-reactivity-outside-setup.md) | [Bad](./project/vize-croquis-cf-reactivity-outside-setup.md#bad) · [Good](./project/vize-croquis-cf-reactivity-outside-setup.md#good) | Contract only; no current producer |
| [`vize:croquis/cf/reassignment-breaks-reactivity`](./project/vize-croquis-cf-reassignment-breaks-reactivity.md) | [Bad](./project/vize-croquis-cf-reassignment-breaks-reactivity.md#bad) · [Good](./project/vize-croquis-cf-reassignment-breaks-reactivity.md#good) | CLI |
| [`vize:croquis/cf/reference-escapes-scope`](./project/vize-croquis-cf-reference-escapes-scope.md) | [Bad](./project/vize-croquis-cf-reference-escapes-scope.md#bad) · [Good](./project/vize-croquis-cf-reference-escapes-scope.md#good) | Contract only; no current producer |
| [`vize:croquis/cf/setup-context-violation`](./project/vize-croquis-cf-setup-context-violation.md) | [Bad](./project/vize-croquis-cf-setup-context-violation.md#bad) · [Good](./project/vize-croquis-cf-setup-context-violation.md#good) | Rust analyzer; CLI uses a different surface or disables this pass |
| [`vize:croquis/cf/shallow-deep-access`](./project/vize-croquis-cf-shallow-deep-access.md) | [Bad](./project/vize-croquis-cf-shallow-deep-access.md#bad) · [Good](./project/vize-croquis-cf-shallow-deep-access.md#good) | Contract only; no current producer |
| [`vize:croquis/cf/spread-breaks-reactivity`](./project/vize-croquis-cf-spread-breaks-reactivity.md) | [Bad](./project/vize-croquis-cf-spread-breaks-reactivity.md#bad) · [Good](./project/vize-croquis-cf-spread-breaks-reactivity.md#good) | CLI |
| [`vize:croquis/cf/suspense-no-fallback`](./project/vize-croquis-cf-suspense-no-fallback.md) | [Bad](./project/vize-croquis-cf-suspense-no-fallback.md#bad) · [Good](./project/vize-croquis-cf-suspense-no-fallback.md#good) | Contract only; no current producer |
| [`vize:croquis/cf/template-ref-timing`](./project/vize-croquis-cf-template-ref-timing.md) | [Bad](./project/vize-croquis-cf-template-ref-timing.md#bad) · [Good](./project/vize-croquis-cf-template-ref-timing.md#good) | Contract only; no current producer |
| [`vize:croquis/cf/toraw-mutation`](./project/vize-croquis-cf-toraw-mutation.md) | [Bad](./project/vize-croquis-cf-toraw-mutation.md#bad) · [Good](./project/vize-croquis-cf-toraw-mutation.md#good) | Contract only; no current producer |
| [`vize:croquis/cf/uncaught-error`](./project/vize-croquis-cf-uncaught-error.md) | [Bad](./project/vize-croquis-cf-uncaught-error.md#bad) · [Good](./project/vize-croquis-cf-uncaught-error.md#good) | CLI |
| [`vize:croquis/cf/undeclared-emit`](./project/vize-croquis-cf-undeclared-emit.md) | [Bad](./project/vize-croquis-cf-undeclared-emit.md#bad) · [Good](./project/vize-croquis-cf-undeclared-emit.md#good) | Rust analyzer; CLI uses a different surface or disables this pass |
| [`vize:croquis/cf/undeclared-prop`](./project/vize-croquis-cf-undeclared-prop.md) | [Bad](./project/vize-croquis-cf-undeclared-prop.md#bad) · [Good](./project/vize-croquis-cf-undeclared-prop.md#good) | Rust analyzer; CLI uses a different surface or disables this pass |
| [`vize:croquis/cf/undefined-slot`](./project/vize-croquis-cf-undefined-slot.md) | [Bad](./project/vize-croquis-cf-undefined-slot.md#bad) · [Good](./project/vize-croquis-cf-undefined-slot.md#good) | Contract only; no current producer |
| [`vize:croquis/cf/unhandled-event`](./project/vize-croquis-cf-unhandled-event.md) | [Bad](./project/vize-croquis-cf-unhandled-event.md#bad) · [Good](./project/vize-croquis-cf-unhandled-event.md#good) | Rust analyzer; CLI uses a different surface or disables this pass |
| [`vize:croquis/cf/unmatched-inject`](./project/vize-croquis-cf-unmatched-inject.md) | [Bad](./project/vize-croquis-cf-unmatched-inject.md#bad) · [Good](./project/vize-croquis-cf-unmatched-inject.md#good) | CLI |
| [`vize:croquis/cf/unmatched-listener`](./project/vize-croquis-cf-unmatched-listener.md) | [Bad](./project/vize-croquis-cf-unmatched-listener.md#bad) · [Good](./project/vize-croquis-cf-unmatched-listener.md#good) | Rust analyzer; CLI uses a different surface or disables this pass |
| [`vize:croquis/cf/unregistered-component`](./project/vize-croquis-cf-unregistered-component.md) | [Bad](./project/vize-croquis-cf-unregistered-component.md#bad) · [Good](./project/vize-croquis-cf-unregistered-component.md#good) | Rust analyzer; CLI uses a different surface or disables this pass |
| [`vize:croquis/cf/unresolved-import`](./project/vize-croquis-cf-unresolved-import.md) | [Bad](./project/vize-croquis-cf-unresolved-import.md#bad) · [Good](./project/vize-croquis-cf-unresolved-import.md#good) | Rust analyzer; CLI uses a different surface or disables this pass |
| [`vize:croquis/cf/unused-attrs`](./project/vize-croquis-cf-unused-attrs.md) | [Bad](./project/vize-croquis-cf-unused-attrs.md#bad) · [Good](./project/vize-croquis-cf-unused-attrs.md#good) | Rust analyzer; CLI uses a different surface or disables this pass |
| [`vize:croquis/cf/unused-emit`](./project/vize-croquis-cf-unused-emit.md) | [Bad](./project/vize-croquis-cf-unused-emit.md#bad) · [Good](./project/vize-croquis-cf-unused-emit.md#good) | Rust analyzer; CLI uses a different surface or disables this pass |
| [`vize:croquis/cf/unused-provide`](./project/vize-croquis-cf-unused-provide.md) | [Bad](./project/vize-croquis-cf-unused-provide.md#bad) · [Good](./project/vize-croquis-cf-unused-provide.md#good) | CLI |
| [`vize:croquis/cf/value-extraction-breaks-reactivity`](./project/vize-croquis-cf-value-extraction-breaks-reactivity.md) | [Bad](./project/vize-croquis-cf-value-extraction-breaks-reactivity.md#bad) · [Good](./project/vize-croquis-cf-value-extraction-breaks-reactivity.md#good) | CLI |
| [`vize:croquis/cf/watch-can-be-computed`](./project/vize-croquis-cf-watch-can-be-computed.md) | [Bad](./project/vize-croquis-cf-watch-can-be-computed.md#bad) · [Good](./project/vize-croquis-cf-watch-can-be-computed.md#good) | Contract only; no current producer |
| [`vize:croquis/cf/watcheffect-async`](./project/vize-croquis-cf-watcheffect-async.md) | [Bad](./project/vize-croquis-cf-watcheffect-async.md#bad) · [Good](./project/vize-croquis-cf-watcheffect-async.md#good) | CLI |
| [`vize:croquis/cf/watcher-outside-setup`](./project/vize-croquis-cf-watcher-outside-setup.md) | [Bad](./project/vize-croquis-cf-watcher-outside-setup.md#bad) · [Good](./project/vize-croquis-cf-watcher-outside-setup.md#good) | Contract only; no current producer |
