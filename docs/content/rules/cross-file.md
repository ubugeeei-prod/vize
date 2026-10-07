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

| Rule | Severity |
| --- | --- |
| [`ecosystem/vue-router-unknown-route`](./project/ecosystem-vue-router-unknown-route.md) | error |
| [`ecosystem/vue-router-extra-param`](./project/ecosystem-vue-router-extra-param.md) | error |
| [`ecosystem/vue-router-param-type`](./project/ecosystem-vue-router-param-type.md) | error |
| [`ecosystem/vue-router-missing-param`](./project/ecosystem-vue-router-missing-param.md) | warning |
| [`html/cross-component-nesting`](./project/html-cross-component-nesting.md) | warning |
| [`vue/cross-file-attrs-fallthrough`](./project/vue-cross-file-attrs-fallthrough.md) | warning |

## Published analyzer codes

| Code | Status |
| --- | --- |
| [`vize:croquis/cf/array-mutation`](./project/vize-croquis-cf-array-mutation.md) | Contract only; no current producer |
| [`vize:croquis/cf/async-boundary`](./project/vize-croquis-cf-async-boundary.md) | CLI |
| [`vize:croquis/cf/async-no-suspense`](./project/vize-croquis-cf-async-no-suspense.md) | Rust analyzer; CLI uses a different surface or disables this pass |
| [`vize:croquis/cf/browser-api-ssr`](./project/vize-croquis-cf-browser-api-ssr.md) | CLI |
| [`vize:croquis/cf/circular-dep`](./project/vize-croquis-cf-circular-dep.md) | Contract only; no current producer |
| [`vize:croquis/cf/circular-reactive-dependency`](./project/vize-croquis-cf-circular-reactive-dependency.md) | CLI |
| [`vize:croquis/cf/closure-captures-reactive`](./project/vize-croquis-cf-closure-captures-reactive.md) | Contract only; no current producer |
| [`vize:croquis/cf/composable-outside-setup`](./project/vize-croquis-cf-composable-outside-setup.md) | Contract only; no current producer |
| [`vize:croquis/cf/computed-side-effects`](./project/vize-croquis-cf-computed-side-effects.md) | Contract only; no current producer |
| [`vize:croquis/cf/deep-import`](./project/vize-croquis-cf-deep-import.md) | Contract only; no current producer |
| [`vize:croquis/cf/destructuring-breaks-reactivity`](./project/vize-croquis-cf-destructuring-breaks-reactivity.md) | CLI |
| [`vize:croquis/cf/di-outside-setup`](./project/vize-croquis-cf-di-outside-setup.md) | Contract only; no current producer |
| [`vize:croquis/cf/dom-access-without-next-tick`](./project/vize-croquis-cf-dom-access-without-next-tick.md) | Contract only; no current producer |
| [`vize:croquis/cf/duplicate-id`](./project/vize-croquis-cf-duplicate-id.md) | CLI |
| [`vize:croquis/cf/event-listener-leak`](./project/vize-croquis-cf-event-listener-leak.md) | Contract only; no current producer |
| [`vize:croquis/cf/event-modifier`](./project/vize-croquis-cf-event-modifier.md) | Rust analyzer; CLI uses a different surface or disables this pass |
| [`vize:croquis/cf/hydration-risk`](./project/vize-croquis-cf-hydration-risk.md) | CLI |
| [`vize:croquis/cf/inherit-attrs-unused`](./project/vize-croquis-cf-inherit-attrs-unused.md) | Rust analyzer; CLI uses a different surface or disables this pass |
| [`vize:croquis/cf/inject-without-symbol`](./project/vize-croquis-cf-inject-without-symbol.md) | CLI |
| [`vize:croquis/cf/injected-async-mutation-race`](./project/vize-croquis-cf-injected-async-mutation-race.md) | CLI |
| [`vize:croquis/cf/lifecycle-outside-setup`](./project/vize-croquis-cf-lifecycle-outside-setup.md) | Contract only; no current producer |
| [`vize:croquis/cf/lifecycle-without-cleanup`](./project/vize-croquis-cf-lifecycle-without-cleanup.md) | Rust analyzer; CLI uses a different surface or disables this pass |
| [`vize:croquis/cf/missing-required-prop`](./project/vize-croquis-cf-missing-required-prop.md) | Rust analyzer; CLI uses a different surface or disables this pass |
| [`vize:croquis/cf/missing-suspense`](./project/vize-croquis-cf-missing-suspense.md) | Contract only; no current producer |
| [`vize:croquis/cf/module-scope-reactive`](./project/vize-croquis-cf-module-scope-reactive.md) | Contract only; no current producer |
| [`vize:croquis/cf/multi-root-attrs`](./project/vize-croquis-cf-multi-root-attrs.md) | Rust analyzer; CLI uses a different surface or disables this pass |
| [`vize:croquis/cf/mutated-after-escape`](./project/vize-croquis-cf-mutated-after-escape.md) | Contract only; no current producer |
| [`vize:croquis/cf/non-reactive-provide`](./project/vize-croquis-cf-non-reactive-provide.md) | CLI |
| [`vize:croquis/cf/non-unique-id`](./project/vize-croquis-cf-non-unique-id.md) | CLI |
| [`vize:croquis/cf/object-identity-comparison`](./project/vize-croquis-cf-object-identity-comparison.md) | Contract only; no current producer |
| [`vize:croquis/cf/pinia-getter`](./project/vize-croquis-cf-pinia-getter.md) | Contract only; no current producer |
| [`vize:croquis/cf/prop-type-mismatch`](./project/vize-croquis-cf-prop-type-mismatch.md) | Rust analyzer; CLI uses a different surface or disables this pass |
| [`vize:croquis/cf/provide-inject-type`](./project/vize-croquis-cf-provide-inject-type.md) | CLI |
| [`vize:croquis/cf/provide-without-symbol`](./project/vize-croquis-cf-provide-without-symbol.md) | CLI |
| [`vize:croquis/cf/reactive-export`](./project/vize-croquis-cf-reactive-export.md) | Contract only; no current producer |
| [`vize:croquis/cf/reactivity-outside-setup`](./project/vize-croquis-cf-reactivity-outside-setup.md) | Contract only; no current producer |
| [`vize:croquis/cf/reassignment-breaks-reactivity`](./project/vize-croquis-cf-reassignment-breaks-reactivity.md) | CLI |
| [`vize:croquis/cf/reference-escapes-scope`](./project/vize-croquis-cf-reference-escapes-scope.md) | Contract only; no current producer |
| [`vize:croquis/cf/setup-context-violation`](./project/vize-croquis-cf-setup-context-violation.md) | Rust analyzer; CLI uses a different surface or disables this pass |
| [`vize:croquis/cf/shallow-deep-access`](./project/vize-croquis-cf-shallow-deep-access.md) | Contract only; no current producer |
| [`vize:croquis/cf/spread-breaks-reactivity`](./project/vize-croquis-cf-spread-breaks-reactivity.md) | CLI |
| [`vize:croquis/cf/suspense-no-fallback`](./project/vize-croquis-cf-suspense-no-fallback.md) | Contract only; no current producer |
| [`vize:croquis/cf/template-ref-timing`](./project/vize-croquis-cf-template-ref-timing.md) | Contract only; no current producer |
| [`vize:croquis/cf/toraw-mutation`](./project/vize-croquis-cf-toraw-mutation.md) | Contract only; no current producer |
| [`vize:croquis/cf/uncaught-error`](./project/vize-croquis-cf-uncaught-error.md) | CLI |
| [`vize:croquis/cf/undeclared-emit`](./project/vize-croquis-cf-undeclared-emit.md) | Rust analyzer; CLI uses a different surface or disables this pass |
| [`vize:croquis/cf/undeclared-prop`](./project/vize-croquis-cf-undeclared-prop.md) | Rust analyzer; CLI uses a different surface or disables this pass |
| [`vize:croquis/cf/undefined-slot`](./project/vize-croquis-cf-undefined-slot.md) | Contract only; no current producer |
| [`vize:croquis/cf/unhandled-event`](./project/vize-croquis-cf-unhandled-event.md) | Rust analyzer; CLI uses a different surface or disables this pass |
| [`vize:croquis/cf/unmatched-inject`](./project/vize-croquis-cf-unmatched-inject.md) | CLI |
| [`vize:croquis/cf/unmatched-listener`](./project/vize-croquis-cf-unmatched-listener.md) | Rust analyzer; CLI uses a different surface or disables this pass |
| [`vize:croquis/cf/unregistered-component`](./project/vize-croquis-cf-unregistered-component.md) | Rust analyzer; CLI uses a different surface or disables this pass |
| [`vize:croquis/cf/unresolved-import`](./project/vize-croquis-cf-unresolved-import.md) | Rust analyzer; CLI uses a different surface or disables this pass |
| [`vize:croquis/cf/unused-attrs`](./project/vize-croquis-cf-unused-attrs.md) | Rust analyzer; CLI uses a different surface or disables this pass |
| [`vize:croquis/cf/unused-emit`](./project/vize-croquis-cf-unused-emit.md) | Rust analyzer; CLI uses a different surface or disables this pass |
| [`vize:croquis/cf/unused-provide`](./project/vize-croquis-cf-unused-provide.md) | CLI |
| [`vize:croquis/cf/value-extraction-breaks-reactivity`](./project/vize-croquis-cf-value-extraction-breaks-reactivity.md) | CLI |
| [`vize:croquis/cf/watch-can-be-computed`](./project/vize-croquis-cf-watch-can-be-computed.md) | Contract only; no current producer |
| [`vize:croquis/cf/watcheffect-async`](./project/vize-croquis-cf-watcheffect-async.md) | CLI |
| [`vize:croquis/cf/watcher-outside-setup`](./project/vize-croquis-cf-watcher-outside-setup.md) | Contract only; no current producer |
