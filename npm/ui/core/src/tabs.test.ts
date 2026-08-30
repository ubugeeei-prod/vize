import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick } from "vue";

import type {
  TabsContentExpose,
  TabsListExpose,
  TabsRootExpose,
  TabsTriggerExpose,
} from "./tabs.ts";
import TabsContent from "./tabs-content.vue";
import TabsList from "./tabs-list.vue";
import TabsRoot from "./tabs-root.vue";
import TabsTrigger from "./tabs-trigger.vue";
import { mountInteraction } from "./testing/mount.ts";

function mountTabs(
  props: Record<string, unknown> = {},
  detailsProps: Record<string, unknown> = {},
) {
  return mountInteraction(TabsRoot, {
    props,
    record: ["update:modelValue", "change"],
    slots: {
      default: (state) => [
        h("output", { "data-root-state": state.state }, String(state.value ?? "")),
        h(TabsList, { ariaLabel: "Product sections" }, () => [
          h(
            TabsTrigger,
            { value: "overview" },
            {
              default: ({ state }) => h("span", { "data-overview-state": state }, "Overview"),
              indicator: ({ selected }) =>
                h("span", { "data-overview-indicator": selected ? "true" : "false" }),
            },
          ),
          h(TabsTrigger, { value: "details", ...detailsProps }, () => "Details"),
          h(TabsTrigger, { value: "billing" }, () => "Billing"),
        ]),
        h(TabsContent, { value: "overview" }, ({ state }) =>
          h("p", { "data-overview-panel": state }, "Overview panel"),
        ),
        h(TabsContent, { value: "details" }, () => "Details panel"),
        h(TabsContent, { value: "billing" }, () => "Billing panel"),
      ],
    },
  });
}

test("renders accessible tab semantics with deterministic ids and slots", () => {
  const handle = mountTabs({ defaultValue: "overview", id: "product-tabs" });
  const root = handle.root();
  const list = handle.getByRole("tablist", { name: "Product sections" });
  const overview = handle.getByRole("tab", { name: "Overview" }) as HTMLButtonElement;
  const details = handle.getByRole("tab", { name: "Details" }) as HTMLButtonElement;
  const panel = handle.getByRole("tabpanel", { name: "Overview" }) as HTMLDivElement;
  const hiddenPanel = root.querySelector<HTMLDivElement>("#product-tabs-content-value-details");

  assert.equal(root.id, "product-tabs");
  assert.equal(root.getAttribute("data-vize-ui"), "tabs-root");
  assert.equal(root.getAttribute("data-state"), "selected");
  assert.equal(
    root.querySelector("[data-root-state]")?.getAttribute("data-root-state"),
    "selected",
  );
  assert.equal(list.id, "product-tabs-list");
  assert.equal(list.getAttribute("aria-orientation"), "horizontal");
  assert.equal(list.getAttribute("part"), "list");
  assert.equal(overview.id, "product-tabs-trigger-value-overview");
  assert.equal(overview.type, "button");
  assert.equal(overview.getAttribute("aria-selected"), "true");
  assert.equal(overview.getAttribute("aria-controls"), "product-tabs-content-value-overview");
  assert.equal(overview.getAttribute("tabindex"), "0");
  assert.equal(overview.getAttribute("part"), "trigger");
  assert.equal(details.getAttribute("aria-selected"), "false");
  assert.equal(details.getAttribute("tabindex"), "-1");
  assert.equal(
    overview.querySelector("[data-overview-state]")?.getAttribute("data-overview-state"),
    "active",
  );
  assert.equal(
    overview.querySelector("[data-overview-indicator]")?.getAttribute("data-overview-indicator"),
    "true",
  );
  assert.equal(panel.id, "product-tabs-content-value-overview");
  assert.equal(panel.hidden, false);
  assert.equal(panel.tabIndex, 0);
  assert.equal(panel.getAttribute("aria-labelledby"), "product-tabs-trigger-value-overview");
  assert.equal(panel.getAttribute("data-state"), "active");
  assert.equal(hiddenPanel?.hidden, true);
  assert.equal(hiddenPanel?.getAttribute("data-state"), "inactive");
  handle.unmount();
});

test("automatic activation follows roving focus and skips disabled triggers", async () => {
  const handle = mountTabs({ defaultValue: "overview" }, { disabled: true });
  const overview = handle.getByRole("tab", { name: "Overview" });
  const billing = handle.getByRole("tab", { name: "Billing" });
  const details = handle.getByRole("tab", { name: "Details" }) as HTMLButtonElement;

  assert.ok((await handle.tab()) === overview);
  const result = await handle.press(overview, "ArrowRight");
  assert.equal(result.keydownPrevented, true);
  assert.ok(handle.activeElement() === billing);
  assert.equal(billing.getAttribute("aria-selected"), "true");
  assert.equal(details.disabled, true);
  assert.equal(details.getAttribute("tabindex"), null);
  assert.deepEqual(handle.wrapper.emitted("update:modelValue"), [["billing"]]);
  assert.deepEqual(handle.wrapper.emitted("change")?.[0]?.slice(0, 2), ["billing", "overview"]);
  assert.ok(handle.wrapper.emitted("change")?.[0]?.[2] instanceof KeyboardEvent);
  handle.unmount();
});

test("manual activation waits for keyboard or pointer activation", async () => {
  const handle = mountTabs({ activationMode: "manual", defaultValue: "overview" });
  const overview = handle.getByRole("tab", { name: "Overview" });
  const details = handle.getByRole("tab", { name: "Details" });
  const detailsPanel = handle
    .root()
    .querySelector<HTMLDivElement>("[data-vize-ui='tabs-content'][data-value='details']");

  overview.focus();
  await handle.press(overview, "ArrowRight");
  assert.ok(handle.activeElement() === details);
  assert.equal(overview.getAttribute("aria-selected"), "true");
  assert.equal(details.getAttribute("aria-selected"), "false");
  assert.equal(detailsPanel?.hidden, true);

  const space = await handle.press(details, " ");
  assert.equal(space.activated, true);
  assert.equal(details.getAttribute("aria-selected"), "true");
  assert.equal(detailsPanel?.hidden, false);
  assert.deepEqual(handle.wrapper.emitted("update:modelValue"), [["details"]]);
  handle.unmount();
});

test("controlled value wins until the parent accepts the request", async () => {
  const handle = mountTabs({ modelValue: "overview" });
  const overview = handle.getByRole("tab", { name: "Overview" });
  const details = handle.getByRole("tab", { name: "Details" });

  await handle.click(details);
  assert.deepEqual(handle.wrapper.emitted("update:modelValue"), [["details"]]);
  assert.deepEqual(handle.wrapper.emitted("change")?.[0]?.slice(0, 2), ["details", "overview"]);
  assert.equal(overview.getAttribute("aria-selected"), "true");
  assert.equal(details.getAttribute("aria-selected"), "false");

  await handle.wrapper.setProps({ modelValue: "details" });
  assert.equal(overview.getAttribute("aria-selected"), "false");
  assert.equal(details.getAttribute("aria-selected"), "true");
  handle.unmount();
});

test("disabled roots and triggers suppress activation and focus", async () => {
  const rootDisabled = mountTabs({ defaultValue: "overview", disabled: true });
  const root = rootDisabled.root();
  const overview = rootDisabled.getByRole("tab", { name: "Overview" }) as HTMLButtonElement;
  const billing = rootDisabled.getByRole("tab", { name: "Billing" }) as HTMLButtonElement;

  assert.equal(root.getAttribute("data-state"), "disabled");
  assert.equal(overview.disabled, true);
  assert.equal(billing.disabled, true);
  assert.notEqual(await rootDisabled.tab(), overview);
  assert.notEqual(rootDisabled.activeElement(), billing);
  await rootDisabled.click(billing);
  assert.equal(overview.getAttribute("aria-selected"), "true");
  assert.equal(rootDisabled.wrapper.emitted("update:modelValue"), undefined);
  rootDisabled.unmount();

  const triggerDisabled = mountTabs({ defaultValue: "overview" }, { disabled: true });
  const details = triggerDisabled.getByRole("tab", { name: "Details" }) as HTMLButtonElement;
  await triggerDisabled.click(details);
  assert.equal(details.disabled, true);
  assert.equal(details.getAttribute("aria-selected"), "false");
  assert.equal(triggerDisabled.wrapper.emitted("update:modelValue"), undefined);
  triggerDisabled.unmount();
});

test("exposes typed state and imperative focus/value controls", async () => {
  let rootExpose: TabsRootExpose | null = null;
  let listExpose: TabsListExpose | null = null;
  let triggerExpose: TabsTriggerExpose | null = null;
  let contentExpose: TabsContentExpose | null = null;
  const Probe = defineComponent({
    name: "TabsExposeProbe",
    setup: () => () =>
      h(
        TabsRoot,
        {
          defaultValue: "overview",
          ref: (value) => {
            rootExpose = value as TabsRootExpose | null;
          },
        },
        () => [
          h(
            TabsList,
            {
              ref: (value) => {
                listExpose = value as TabsListExpose | null;
              },
            },
            () => [
              h(TabsTrigger, { value: "overview" }, () => "Overview"),
              h(
                TabsTrigger,
                {
                  ref: (value) => {
                    triggerExpose = value as TabsTriggerExpose | null;
                  },
                  value: "details",
                },
                () => "Details",
              ),
            ],
          ),
          h(TabsContent, { value: "overview" }, () => "Overview panel"),
          h(
            TabsContent,
            {
              ref: (value) => {
                contentExpose = value as TabsContentExpose | null;
              },
              value: "details",
            },
            () => "Details panel",
          ),
        ],
      ),
  });
  const handle = mountInteraction(Probe);

  if (!rootExpose || !listExpose || !triggerExpose || !contentExpose) {
    assert.fail("Tabs refs must expose root, list, trigger, and content state");
  }
  rootExpose.focus();
  assert.ok(handle.activeElement() === handle.getByRole("tab", { name: "Overview" }));
  assert.equal(rootExpose.setValue("details"), true);
  await nextTick();
  assert.equal(rootExpose.value, "details");
  assert.equal(triggerExpose.selected, true);
  contentExpose.focusContent();
  assert.ok(handle.activeElement() === contentExpose.element);
  assert.equal(rootExpose.reset(), true);
  await nextTick();
  assert.equal(rootExpose.value, "overview");
  listExpose.focus();
  assert.ok(handle.activeElement() === handle.getByRole("tab", { name: "Overview" }));
  handle.unmount();
});

test("compound parts require a matching root provider", () => {
  assert.throws(() => mountInteraction(TabsList), /VIZE_UI_CONTEXT_MISSING/);
  assert.throws(
    () => mountInteraction(TabsTrigger, { props: { value: "overview" } }),
    /VIZE_UI_CONTEXT_MISSING/,
  );
  assert.throws(
    () => mountInteraction(TabsContent, { props: { value: "overview" } }),
    /VIZE_UI_CONTEXT_MISSING/,
  );
});
