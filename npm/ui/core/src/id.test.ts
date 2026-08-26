import assert from "node:assert/strict";

import { mount } from "@vue/test-utils";
import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick, ref } from "vue";
import { renderToString } from "vue/server-renderer";

import IdProvider from "./deterministic-id-provider.vue";
import {
  createDeterministicIdScope,
  deriveDeterministicId,
  toDeterministicId,
  useDeterministicId,
} from "./deterministic-id.ts";

const IdProbe = defineComponent({
  name: "IdProbe",
  props: {
    hint: { type: String, default: "id" },
    prefix: { type: String, default: undefined },
  },
  setup(props) {
    const id = useDeterministicId({
      hint: props.hint,
      ...(props.prefix === undefined ? {} : { prefix: props.prefix }),
    });
    return () => h("span", { id: id.value, "data-id-probe": props.hint }, id.value);
  },
});

test("creates immutable request-local scopes with independent sequences", () => {
  const scope = createDeterministicIdScope({ prefix: "checkout", seed: "request-42" });

  assert.equal(scope.prefix, "checkout");
  assert.equal(scope.namespace, "checkout-request-42");
  assert.equal(Object.isFrozen(scope), true);
  assert.equal(scope.nextId("label"), "checkout-request-42-label-0");
  assert.equal(scope.nextId("label"), "checkout-request-42-label-1");

  const child = scope.createChild({ seed: "dialog" });
  assert.equal(child.namespace, "checkout-request-42-scope-0-dialog");
  assert.equal(child.nextId("title"), "checkout-request-42-scope-0-dialog-title-0");
  assert.equal(scope.nextId("control"), "checkout-request-42-control-2");
});

test("keeps ID and child-scope allocation sequences independent", () => {
  const withoutChild = createDeterministicIdScope({ seed: "request" });
  assert.equal(withoutChild.nextId("before"), "vize-request-before-0");
  assert.equal(withoutChild.nextId("after"), "vize-request-after-1");

  const withChild = createDeterministicIdScope({ seed: "request" });
  assert.equal(withChild.nextId("before"), "vize-request-before-0");
  withChild.createChild({ seed: "nested" });
  assert.equal(withChild.nextId("after"), "vize-request-after-1");
});

test("supports a nested namespace prefix override", () => {
  const scope = createDeterministicIdScope({ prefix: "page", seed: 7 });
  const child = scope.createChild({ prefix: "dialog", seed: "settings" });

  assert.equal(child.prefix, "dialog");
  assert.equal(child.namespace, "dialog-page-7-scope-0-settings");
  assert.equal(child.nextId(), "dialog-page-7-scope-0-settings-id-0");
});

test("rejects namespace values that are unsafe to compose", () => {
  assert.throws(
    () => createDeterministicIdScope({ prefix: "9field", seed: "request" }),
    /VIZE_UI_ID_PREFIX/,
  );
  assert.throws(() => createDeterministicIdScope({ seed: "request 1" }), /VIZE_UI_ID_SEED/);
  assert.throws(
    () => createDeterministicIdScope({ seed: Number.POSITIVE_INFINITY }),
    /VIZE_UI_ID_SEED/,
  );
  assert.throws(() => createDeterministicIdScope({ seed: -1 }), /VIZE_UI_ID_SEED/);
  const scope = createDeterministicIdScope({ seed: "request" });
  assert.throws(() => scope.nextId("bad hint"), /VIZE_UI_ID_HINT/);
});

test("validates explicit IDs and derives semantic parts", () => {
  assert.equal(toDeterministicId("account.email"), "account.email");
  assert.equal(toDeterministicId("設定"), "設定");
  assert.equal(deriveDeterministicId("account.email", "description"), "account.email-description");
  assert.throws(() => toDeterministicId(""), /VIZE_UI_ID_VALUE/);
  assert.throws(() => toDeterministicId("two words"), /VIZE_UI_ID_VALUE/);
  assert.throws(() => toDeterministicId("line\nbreak"), /VIZE_UI_ID_VALUE/);
  assert.throws(() => deriveDeterministicId("field", "bad part"), /VIZE_UI_ID_PART/);
});

test("uses Vue's application ID sequence without a provider", () => {
  const wrapper = mount(IdProbe, { props: { hint: "control", prefix: "field" } });
  const id = wrapper.get("span").attributes("id");

  assert.ok(id);
  assert.ok(id.startsWith("field-v-"), `unexpected generated ID: ${id}`);
  assert.ok(id.endsWith("-control"), `unexpected generated ID: ${id}`);
  wrapper.unmount();
});

test("allocates descriptive IDs from the nearest provider", () => {
  const wrapper = mount(IdProvider, {
    props: { prefix: "form", seed: "profile" },
    slots: {
      default: () => [
        h(IdProbe, { hint: "label" }),
        h(IdProbe, { hint: "control" }),
        h(IdProbe, { hint: "description" }),
      ],
    },
  });

  assert.deepEqual(
    wrapper.findAll("[data-id-probe]").map((probe) => probe.attributes("id")),
    ["form-profile-label-0", "form-profile-control-1", "form-profile-description-2"],
  );
  wrapper.unmount();
});

test("preserves one fallback across reactive explicit ID changes", async () => {
  const explicit = ref<string | undefined>();
  const Probe = defineComponent({
    setup() {
      const id = useDeterministicId({ id: explicit, hint: "control", prefix: "field" });
      return () => h("input", { id: id.value });
    },
  });
  const wrapper = mount(Probe);
  const fallback = wrapper.get("input").attributes("id");

  explicit.value = "consumer-control";
  await nextTick();
  assert.equal(wrapper.get("input").attributes("id"), "consumer-control");

  explicit.value = undefined;
  await nextTick();
  assert.equal(wrapper.get("input").attributes("id"), fallback);
  wrapper.unmount();
});

test("rejects composable use outside component setup", () => {
  assert.throws(() => useDeterministicId(), /VIZE_UI_ID_SETUP/);
});

test("exposes the resolved namespace to its slot and public instance", () => {
  const wrapper = mount(IdProvider, {
    props: { prefix: "checkout", seed: 17 },
    slots: {
      default: (state: { namespace: string; prefix: string }) =>
        h("output", { "data-prefix": state.prefix }, state.namespace),
    },
  });

  assert.equal(wrapper.get("output").text(), "checkout-17");
  assert.equal(wrapper.get("output").attributes("data-prefix"), "checkout");
  assert.equal((wrapper.vm as unknown as { namespace: string }).namespace, "checkout-17");
  wrapper.unmount();
});

test("keeps duplicate nested provider seeds collision-free", () => {
  const wrapper = mount(IdProvider, {
    props: { seed: "page" },
    slots: {
      default: () => [
        h(IdProbe, { hint: "control" }),
        h(IdProvider, { seed: "dialog" }, { default: () => h(IdProbe, { hint: "control" }) }),
        h(IdProvider, { seed: "dialog" }, { default: () => h(IdProbe, { hint: "control" }) }),
      ],
    },
  });

  assert.deepEqual(
    wrapper.findAll("[data-id-probe]").map((probe) => probe.attributes("id")),
    [
      "vize-page-control-0",
      "vize-page-scope-0-dialog-control-0",
      "vize-page-scope-1-dialog-control-0",
    ],
  );
  wrapper.unmount();
});

function createSsrTree(seed: string) {
  return defineComponent({
    name: "DeterministicIdSsrTree",
    setup() {
      return () =>
        h(
          IdProvider,
          { seed },
          {
            default: () => [
              h(IdProbe, { hint: "label" }),
              h(IdProbe, { hint: "control" }),
              h(
                IdProvider,
                { seed: "nested" },
                { default: () => h(IdProbe, { hint: "description" }) },
              ),
            ],
          },
        );
    },
  });
}

async function renderSsrTree(seed: string): Promise<string> {
  return renderToString(createSSRApp(createSsrTree(seed)));
}

test("renders byte-stable IDs for repeated and concurrent SSR requests", async () => {
  const repeated = await Promise.all([renderSsrTree("request-a"), renderSsrTree("request-a")]);
  assert.equal(repeated[0], repeated[1]);
  assert.match(repeated[0], /id="vize-request-a-label-0"/);
  assert.match(repeated[0], /id="vize-request-a-control-1"/);
  assert.match(repeated[0], /id="vize-request-a-scope-0-nested-description-0"/);

  const [left, right] = await Promise.all([
    renderSsrTree("request-left"),
    renderSsrTree("request-right"),
  ]);
  assert.match(left, /vize-request-left-label-0/);
  assert.doesNotMatch(left, /request-right/);
  assert.match(right, /vize-request-right-label-0/);
  assert.doesNotMatch(right, /request-left/);
});

test("hydrates provider IDs without warnings or replacement", async () => {
  const Root = createSsrTree("hydrate");
  const serverHtml = await renderToString(createSSRApp(Root));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverIds = [...host.querySelectorAll<HTMLElement>("[id]")].map((element) => element.id);
  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(Root);

  try {
    app.mount(host);
    const hydratedIds = [...host.querySelectorAll<HTMLElement>("[id]")].map(
      (element) => element.id,
    );
    assert.deepEqual(hydratedIds, serverIds);
    assert.deepEqual(diagnostics, []);
  } finally {
    app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
