import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick } from "vue";

import type {
  StepperContentExpose,
  StepperItemExpose,
  StepperListExpose,
  StepperRootExpose,
  StepperTriggerExpose,
} from "./stepper.ts";
import StepperContent from "./stepper-content.vue";
import StepperItem from "./stepper-item.vue";
import StepperList from "./stepper-list.vue";
import StepperRoot from "./stepper-root.vue";
import StepperTrigger from "./stepper-trigger.vue";
import { mountInteraction } from "./testing/mount.ts";

test("exposes typed state and imperative focus/value controls", async () => {
  let rootExpose: StepperRootExpose | null = null;
  let listExpose: StepperListExpose | null = null;
  let itemExpose: StepperItemExpose | null = null;
  let triggerExpose: StepperTriggerExpose | null = null;
  let contentExpose: StepperContentExpose | null = null;
  const Probe = defineComponent({
    name: "StepperExposeProbe",
    setup: () => () =>
      h(
        StepperRoot,
        {
          ref: (value) => {
            rootExpose = value as StepperRootExpose | null;
          },
        },
        () => [
          h(
            StepperList,
            {
              ref: (value) => {
                listExpose = value as StepperListExpose | null;
              },
            },
            () => [
              h(StepperItem, { completed: true, value: "shipping" }, () =>
                h(StepperTrigger, () => "Shipping"),
              ),
              h(
                StepperItem,
                {
                  ref: (value) => {
                    itemExpose = value as StepperItemExpose | null;
                  },
                  value: "billing",
                },
                () =>
                  h(
                    StepperTrigger,
                    {
                      ref: (value) => {
                        triggerExpose = value as StepperTriggerExpose | null;
                      },
                    },
                    () => "Billing",
                  ),
              ),
            ],
          ),
          h(StepperContent, { value: "shipping" }, () => "Shipping panel"),
          h(
            StepperContent,
            {
              ref: (value) => {
                contentExpose = value as StepperContentExpose | null;
              },
              value: "billing",
            },
            () => "Billing panel",
          ),
        ],
      ),
  });
  const handle = mountInteraction(Probe);
  await nextTick();

  if (
    rootExpose === null ||
    listExpose === null ||
    itemExpose === null ||
    triggerExpose === null ||
    contentExpose === null
  ) {
    assert.fail("Stepper refs must expose root, list, item, trigger, and content state");
  }

  rootExpose.focus();
  assert.ok(handle.activeElement() === handle.getByRole("button", { name: "Shipping" }));
  assert.equal(rootExpose.next(), true);
  await nextTick();
  assert.equal(rootExpose.value, "billing");
  assert.equal(itemExpose.current, true);
  assert.equal(triggerExpose.current, true);
  contentExpose.focusContent();
  assert.ok(handle.activeElement() === contentExpose.element);
  assert.equal(rootExpose.previous(), true);
  await nextTick();
  assert.equal(rootExpose.value, "shipping");
  assert.equal(triggerExpose.select(), true);
  await nextTick();
  assert.equal(rootExpose.value, "billing");
  listExpose.focus();
  assert.ok(handle.activeElement() === triggerExpose.element);
  assert.equal(itemExpose.focus(), true);
  assert.ok(handle.activeElement() === triggerExpose.element);
  assert.equal(rootExpose.reset(), true);
  await nextTick();
  assert.equal(rootExpose.value, "shipping");
  handle.unmount();
});

test("content can opt out of the region role and default trigger label", () => {
  const Probe = defineComponent({
    name: "StepperPlainContentProbe",
    setup: () => () =>
      h(StepperRoot, { defaultValue: "shipping" }, () => [
        h(StepperList, () =>
          h(StepperItem, { value: "shipping" }, () => h(StepperTrigger, () => "Shipping")),
        ),
        h(
          StepperContent,
          { ariaLabelledby: null, role: null, value: "shipping" },
          () => "Plain panel",
        ),
      ]),
  });
  const plain = mountInteraction(Probe);
  const content = plain.root().querySelector<HTMLElement>("[data-vize-ui='stepper-content']");

  assert.ok(content);
  assert.equal(content.getAttribute("role"), null);
  assert.equal(content.getAttribute("aria-labelledby"), null);
  assert.equal(plain.queryByRole("region"), null);
  plain.unmount();
});

test("compound parts require matching Stepper providers", () => {
  assert.throws(() => mountInteraction(StepperList), /VIZE_UI_CONTEXT_MISSING/);
  assert.throws(
    () =>
      mountInteraction(StepperItem, {
        props: { value: "orphan" },
        slots: { default: () => "Orphan" },
      }),
    /VIZE_UI_CONTEXT_MISSING/,
  );
  assert.throws(() => mountInteraction(StepperTrigger), /VIZE_UI_CONTEXT_MISSING/);
  assert.throws(
    () => mountInteraction(StepperContent, { props: { value: "orphan" } }),
    /VIZE_UI_CONTEXT_MISSING/,
  );
});
