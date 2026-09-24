/**
 * Server-rendering contract for the ref helpers and state factories:
 * rendering twice yields identical markup that equals the client's first
 * render, no timer starts, and no state leaks between requests.
 */
import assert from "node:assert/strict";
import { test } from "node:test";
import { createSSRApp, defineComponent, h, reactive, ref, shallowRef } from "vue";
import type { Component } from "vue";
import { renderToString } from "vue/server-renderer";

import { computedAsync } from "./computed-async.ts";
import { computedWithControl } from "./computed-with-control.ts";
import { createEventHook } from "./create-event-hook.ts";
import { createGlobalState } from "./create-global-state.ts";
import { createInjectionState } from "./create-injection-state.ts";
import { createSharedComposable } from "./create-shared-composable.ts";
import { reactify } from "./reactify.ts";
import { refAutoReset } from "./ref-auto-reset.ts";
import { refDebounced } from "./ref-debounced.ts";
import { refDefault } from "./ref-default.ts";
import { syncRef } from "./sync-ref.ts";
import { FakeTimeouts } from "./testing/fake-timeouts.ts";
import { toReactive } from "./to-reactive.ts";
import { useIdGenerator } from "./use-id-generator.ts";
import { useMounted } from "./use-mounted.ts";
import { useSupported } from "./use-supported.ts";
import { useVModel } from "./use-v-model.ts";

async function renderTwice(component: () => Component): Promise<string> {
  const first = await renderToString(createSSRApp(component()));
  const second = await renderToString(createSSRApp(component()));
  assert.equal(first, second, "server output must be deterministic");
  return first;
}

void test("ref helpers render their synchronous state without timers", async () => {
  const timers = new FakeTimeouts();
  const html = await renderTwice(() =>
    defineComponent({
      setup() {
        const status = refAutoReset("idle", 100, { scheduler: timers });
        status.value = "saved";
        const query = shallowRef("a");
        const debounced = refDebounced(query, 300, { scheduler: timers });
        query.value = "ab";
        const name = refDefault(ref<string | undefined>(), "guest");
        const left = ref(1);
        const right = ref(0);
        syncRef(left, right);
        const view = toReactive(ref({ label: "view" }));
        const sum = reactify((a: number, b: number) => a + b)(left, 2);
        const version = shallowRef(0);
        const controlled = computedWithControl(version, () => `v${version.value}`);
        const data = computedAsync(async () => "loaded", "pending");
        return () =>
          h(
            "output",
            [
              status.value,
              debounced.value,
              name.value,
              right.value,
              view.label,
              sum.value,
              controlled.value,
              data.value,
            ].join("|"),
          );
      },
    }),
  );
  assert.equal(html, "<output>saved|ab|guest|1|view|3|v0|pending</output>");
  assert.equal(timers.timers.size, 0);
});

void test("v-model helpers render the prop value", async () => {
  const html = await renderTwice(() =>
    defineComponent({
      props: { modelValue: { type: String, default: "from-prop" } },
      emits: ["update:modelValue"],
      setup(props, { emit }) {
        const model = useVModel(props, "modelValue", emit);
        const passive = useVModel(props, "modelValue", emit, { passive: true });
        return () => h("output", `${model.value}|${passive.value}`);
      },
    }),
  );
  assert.equal(html, "<output>from-prop|from-prop</output>");
});

void test("injection state is per request, shared composables are per call", async () => {
  const [useProvideCart, useCart] = createInjectionState(() => reactive({ items: 0 }));
  let created = 0;
  const useShared = createSharedComposable(() => ({ id: ++created }));
  const Child = defineComponent({
    setup() {
      const cart = useCart();
      return () => h("i", String(cart?.items));
    },
  });
  const html = await renderTwice(() =>
    defineComponent({
      setup() {
        const cart = useProvideCart();
        cart.items += 1;
        useShared();
        useShared();
        return () => h(Child);
      },
    }),
  );
  assert.equal(html, "<i>1</i>");
  assert.equal(created, 4, "no instance is shared across server calls");
});

void test("global state and event hooks are inert data during render", async () => {
  const useGlobal = createGlobalState(() => shallowRef("global"));
  const html = await renderTwice(() =>
    defineComponent({
      setup() {
        // Server component scopes are not disposed after rendering, so
        // request code must use request-local hooks, never module-level ones.
        const hook = createEventHook<[string]>();
        hook.on(() => undefined);
        return () => h("output", `${useGlobal().value}|${hook.size()}`);
      },
    }),
  );
  assert.equal(html, "<output>global|1</output>");
});

void test("module-level event hooks skip server setup listeners instead of piling up", async () => {
  const shared = createEventHook<[string]>();
  const opted = createEventHook<[string]>({ serverListeners: "register" });
  const warnings: unknown[] = [];
  const originalWarn = console.warn;
  console.warn = (...args: unknown[]) => {
    warnings.push(args[0]);
  };
  try {
    for (let request = 0; request < 3; request += 1) {
      await renderToString(
        createSSRApp(
          defineComponent({
            setup() {
              const skipped = shared.on(() => undefined);
              skipped.off();
              opted.on(() => undefined);
              return () => h("i");
            },
          }),
        ),
      );
    }
  } finally {
    console.warn = originalWarn;
  }
  assert.equal(shared.size(), 0, "no listener accumulates across requests");
  assert.equal(opted.size(), 3, "opted-in hooks register and leave cleanup to the caller");
  assert.equal(warnings.length, 1, "the development warning fires once per hook");
  assert.match(String(warnings[0]), /VIZE_COMPOSE_EVENT_HOOK_SERVER_LISTENER/);
  shared.on(() => undefined);
  assert.equal(shared.size(), 1, "registrations outside server setup are unaffected");
});

void test("mount-gated state renders the client's pre-mount fallback", async () => {
  const html = await renderTwice(() =>
    defineComponent({
      setup() {
        const mounted = useMounted();
        const supported = useSupported(() => true);
        const nextId = useIdGenerator({ prefix: "field" });
        return () => h("label", { for: nextId("input") }, `${mounted.value}|${supported.value}`);
      },
    }),
  );
  assert.match(html, /^<label for="field-v-[\w-]+-input-0">false\|false<\/label>$/);
});
