import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick } from "vue";

import type { CollapsibleContentExpose, CollapsibleRootExpose } from "./collapsible.ts";
import CollapsibleContent from "./collapsible-content.vue";
import CollapsibleRoot from "./collapsible-root.vue";
import CollapsibleTrigger from "./collapsible-trigger.vue";
import { mountInteraction } from "../../../testing/mount.ts";

function mountCollapsible(
  props: Record<string, unknown> = {},
  triggerProps: Record<string, unknown> = {},
  contentProps: Record<string, unknown> = {},
) {
  return mountInteraction(CollapsibleRoot, {
    props,
    slots: {
      default: () => [
        h(
          CollapsibleTrigger,
          { ariaLabel: "Filters", ...triggerProps },
          { default: ({ state }) => h("span", { "data-trigger-state": state }, "Filters") },
        ),
        h(
          CollapsibleContent,
          { ariaDescribedby: "filters-help", ...contentProps },
          {
            default: ({ state }) =>
              h("p", { id: "filters-help", "data-content-state": state }, "Filter controls"),
          },
        ),
      ],
    },
  });
}

test("renders native disclosure semantics with deterministic ids and slots", () => {
  const handle = mountCollapsible({ defaultOpen: true, id: "filters" });
  const root = handle.root();
  const trigger = handle.getByRole("button", { name: "Filters" }) as HTMLButtonElement;
  const content = handle.getByRole("region", { name: "Filters" }) as HTMLDivElement;

  assert.equal(root.id, "filters");
  assert.equal(root.getAttribute("data-vize-ui"), "collapsible-root");
  assert.equal(root.getAttribute("part"), "root");
  assert.equal(root.getAttribute("data-state"), "open");
  assert.equal(trigger.id, "filters-trigger");
  assert.equal(trigger.type, "button");
  assert.equal(trigger.getAttribute("aria-expanded"), "true");
  assert.equal(trigger.getAttribute("aria-controls"), "filters-content");
  assert.equal(trigger.getAttribute("data-vize-ui"), "collapsible-trigger");
  assert.equal(trigger.getAttribute("part"), "trigger");
  assert.equal(
    trigger.querySelector("[data-trigger-state]")?.getAttribute("data-trigger-state"),
    "open",
  );
  assert.equal(content.id, "filters-content");
  assert.equal(content.hidden, false);
  assert.equal(content.getAttribute("aria-labelledby"), "filters-trigger");
  assert.equal(content.getAttribute("aria-describedby"), "filters-help");
  assert.equal(content.getAttribute("data-vize-ui"), "collapsible-content");
  assert.equal(content.getAttribute("part"), "content");
  assert.equal(
    content.querySelector("[data-content-state]")?.getAttribute("data-content-state"),
    "open",
  );

  handle.unmount();
});

test("uncontrolled disclosure toggles through pointer and native keyboard activation", async () => {
  const handle = mountCollapsible();
  const trigger = handle.getByRole("button", { name: "Filters" });
  const content = handle.getByRole("region", { name: "Filters" }) as HTMLDivElement;

  assert.equal(trigger.getAttribute("aria-expanded"), "false");
  assert.equal(content.hidden, true);

  await handle.click(trigger);
  assert.equal(trigger.getAttribute("aria-expanded"), "true");
  assert.equal(content.hidden, false);

  const enter = await handle.press(trigger, "Enter");
  assert.equal(enter.activated, true);
  assert.equal(trigger.getAttribute("aria-expanded"), "false");
  assert.equal(content.hidden, true);

  const space = await handle.press(trigger, " ");
  assert.equal(space.activated, true);
  assert.equal(trigger.getAttribute("aria-expanded"), "true");
  assert.equal(content.hidden, false);
  assert.equal(handle.wrapper.emitted("update:open")?.length, 3);
  assert.equal(handle.wrapper.emitted("open-change")?.length, 3);

  handle.unmount();
});

test("controlled open state waits for the parent to accept a request", async () => {
  const handle = mountCollapsible({ open: false, id: "controlled" });
  const trigger = handle.getByRole("button", { name: "Filters" });
  const content = handle.getByRole("region", { name: "Filters" }) as HTMLDivElement;

  await handle.click(trigger);

  assert.equal(trigger.getAttribute("aria-expanded"), "false");
  assert.equal(content.hidden, true);
  assert.deepEqual(handle.wrapper.emitted("update:open"), [[true]]);
  const openChange = handle.wrapper.emitted("open-change");
  assert.equal(openChange?.length, 1);
  assert.deepEqual(openChange?.[0]?.slice(0, 2), [true, false]);
  assert.ok(openChange?.[0]?.[2] instanceof MouseEvent);

  await handle.wrapper.setProps({ open: true });
  assert.equal(trigger.getAttribute("aria-expanded"), "true");
  assert.equal(content.hidden, false);

  handle.unmount();
});

test("trigger click is preventable before state changes", async () => {
  const handle = mountCollapsible(
    {},
    {
      onClick: (event: MouseEvent) => event.preventDefault(),
    },
  );
  const trigger = handle.getByRole("button", { name: "Filters" });
  const content = handle.getByRole("region", { name: "Filters" }) as HTMLDivElement;

  await handle.click(trigger);

  assert.equal(trigger.getAttribute("aria-expanded"), "false");
  assert.equal(content.hidden, true);
  assert.equal(handle.wrapper.emitted("update:open"), undefined);
  assert.equal(handle.wrapper.emitted("open-change"), undefined);

  handle.unmount();
});

test("root and trigger disabled states block user activation with native button semantics", async () => {
  const rootDisabled = mountCollapsible({ disabled: true });
  const rootTrigger = rootDisabled.getByRole("button", { name: "Filters" }) as HTMLButtonElement;
  const rootContent = rootDisabled.getByRole("region", { name: "Filters" }) as HTMLDivElement;

  assert.equal(rootTrigger.disabled, true);
  assert.equal(rootTrigger.getAttribute("data-disabled"), "true");
  await rootDisabled.click(rootTrigger);
  assert.equal(rootTrigger.getAttribute("aria-expanded"), "false");
  assert.equal(rootContent.hidden, true);
  assert.equal(rootDisabled.wrapper.emitted("update:open"), undefined);
  rootDisabled.unmount();

  const triggerDisabled = mountCollapsible({}, { disabled: true });
  const localTrigger = triggerDisabled.getByRole("button", {
    name: "Filters",
  }) as HTMLButtonElement;

  assert.equal(localTrigger.disabled, true);
  assert.equal(localTrigger.getAttribute("data-disabled"), "true");
  await triggerDisabled.click(localTrigger);
  assert.equal(localTrigger.getAttribute("aria-expanded"), "false");
  assert.equal(triggerDisabled.wrapper.emitted("update:open"), undefined);
  triggerDisabled.unmount();
});

test("root and content expose typed state and programmatic controls", async () => {
  const seen: string[] = [];
  let rootExpose: CollapsibleRootExpose | null = null;
  let contentExpose: CollapsibleContentExpose | null = null;
  const Probe = defineComponent({
    name: "CollapsibleExposeProbe",
    setup: () => () =>
      h(
        CollapsibleRoot,
        {
          ref: (value) => {
            rootExpose = value as CollapsibleRootExpose | null;
          },
        },
        {
          default: (state) => {
            seen.push(`${state.state}:${state.open}:${state.disabled}`);
            return h(
              CollapsibleContent,
              {
                ref: (value) => {
                  contentExpose = value as CollapsibleContentExpose | null;
                },
                tabindex: "-1",
              },
              () => h("output", state.state),
            );
          },
        },
      ),
  });
  const handle = mountInteraction(Probe);

  if (rootExpose === null || contentExpose === null) {
    assert.fail("Collapsible refs must expose root and content state");
  }

  const root = rootExpose;
  const content = contentExpose;

  assert.equal(root.open, false);
  assert.equal(root.state, "closed");
  assert.match(root.triggerId, /^vize-v-\d+-collapsible-trigger$/);
  assert.match(root.contentId, /^vize-v-\d+-collapsible-content$/);
  assert.equal(root.expand(), true);
  await nextTick();
  assert.equal(root.open, true);
  assert.equal(content.open, true);
  assert.equal(content.state, "open");
  content.focusContent();
  assert.ok(handle.activeElement() === content.element);
  assert.equal(root.toggle(), true);
  await nextTick();
  assert.equal(root.open, false);
  assert.equal(root.collapse(), false);
  assert.ok(seen.includes("closed:false:false"));
  assert.ok(seen.includes("open:true:false"));

  handle.unmount();
});

test("content can opt out of the region role and default trigger label", () => {
  const handle = mountCollapsible({ defaultOpen: true }, {}, { role: null, ariaLabelledby: null });
  const content = handle.root().querySelector<HTMLElement>("[data-vize-ui='collapsible-content']");

  assert.ok(content);
  assert.equal(content.getAttribute("role"), null);
  assert.equal(content.getAttribute("aria-labelledby"), null);
  assert.equal(handle.queryByRole("region"), null);

  handle.unmount();
});

test("trigger and content require a matching root provider", () => {
  assert.throws(() => mountInteraction(CollapsibleTrigger), /VIZE_UI_CONTEXT_MISSING/);
  assert.throws(() => mountInteraction(CollapsibleContent), /VIZE_UI_CONTEXT_MISSING/);
});
