import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick } from "vue";

import Rating from "./rating.vue";
import type { RatingExpose, RatingItemSlotState, RatingSlotState } from "./rating-types.ts";
import { mountInteraction } from "../../../testing/mount.ts";

function ratingFormValue(form: HTMLFormElement, name = "score"): FormDataEntryValue | null {
  return new FormData(form).get(name);
}

test("renders native radio rating semantics with form and extension hooks", () => {
  const handle = mountInteraction(Rating, {
    props: {
      id: "movie-rating",
      name: "score",
      defaultValue: 3,
      min: 1,
      count: 5,
      clearable: true,
      required: true,
      dir: "rtl",
      itemLabel: "Score",
      ariaLabel: "Movie score",
      ariaDescribedby: "rating-help",
      ariaErrormessage: "rating-error",
      ariaInvalid: true,
    },
    slots: {
      default: (state: RatingSlotState) => ` value:${state.value} percent:${state.percent}`,
      item: (item: RatingItemSlotState) => `item-${item.value}-${item.active}`,
    },
  });
  const root = handle.getByRole("radiogroup", { name: "Movie score" });
  const third = handle.getByRole("radio", { name: "Score 3 of 5" }) as HTMLInputElement;
  const fourth = handle.getByRole("radio", { name: "Score 4 of 5" }) as HTMLInputElement;
  const thirdItem = third.closest("[data-vize-ui='rating-item']");
  const thirdIndicator = thirdItem?.querySelector("[data-vize-ui='rating-indicator']");

  assert.equal(root.tagName, "SPAN");
  assert.equal(root.id, "movie-rating");
  assert.equal(root.getAttribute("role"), "radiogroup");
  assert.equal(root.getAttribute("dir"), "rtl");
  assert.equal(root.getAttribute("aria-describedby"), "rating-help");
  assert.equal(root.getAttribute("aria-errormessage"), "rating-error");
  assert.equal(root.getAttribute("aria-invalid"), "true");
  assert.equal(root.getAttribute("aria-required"), "true");
  assert.equal(root.getAttribute("data-vize-ui"), "rating");
  assert.equal(root.getAttribute("part"), "root");
  assert.equal(root.getAttribute("data-state"), "invalid");
  assert.equal(root.getAttribute("data-value"), "3");
  assert.equal(root.getAttribute("data-min"), "1");
  assert.equal(root.getAttribute("data-max"), "5");
  assert.equal(root.getAttribute("data-count"), "5");
  assert.equal(root.getAttribute("data-dir"), "rtl");
  assert.equal(root.getAttribute("data-required"), "true");
  assert.equal(root.getAttribute("data-invalid"), "true");
  assert.equal(root.getAttribute("data-clearable"), "true");
  assert.equal(root.style.getPropertyValue("--vize-rating-value"), "3");
  assert.equal(root.style.getPropertyValue("--vize-rating-percent"), "60%");

  assert.equal(third.id, "movie-rating-item-3");
  assert.equal(third.type, "radio");
  assert.equal(third.name, "score");
  assert.equal(third.value, "3");
  assert.equal(third.checked, true);
  assert.equal(third.required, true);
  assert.equal(third.getAttribute("aria-invalid"), "true");
  assert.equal(third.getAttribute("part"), "control");
  assert.equal(third.getAttribute("data-state"), "checked");
  assert.equal(third.getAttribute("data-active"), "true");
  assert.equal(third.getAttribute("data-checked"), "true");
  assert.equal(thirdItem?.getAttribute("part"), "item");
  assert.equal(thirdIndicator?.getAttribute("part"), "indicator");
  assert.match(thirdIndicator?.textContent ?? "", /item-3-true/);
  assert.equal(fourth.checked, false);
  assert.match(root.textContent ?? "", /value:3 percent:60/);
  handle.unmount();
});

test("uncontrolled rating selects and clears through pointer activation", async () => {
  const recorded: [event: string, value: unknown, previous?: unknown, nativeEvent?: unknown][] = [];
  const FormProbe = defineComponent({
    setup: () => () =>
      h("form", [
        h(
          Rating,
          {
            ariaLabel: "Movie score",
            clearable: true,
            name: "score",
            onChange: (value: unknown, previous: unknown, nativeEvent: Event) =>
              recorded.push(["change", value, previous, nativeEvent]),
            onClear: (previous: number, nativeEvent: Event) =>
              recorded.push(["clear", previous, undefined, nativeEvent]),
            "onUpdate:modelValue": (value: unknown) => recorded.push(["update:modelValue", value]),
          },
          {
            item: (item: RatingItemSlotState) => String(item.value),
          },
        ),
      ]),
  });
  const handle = mountInteraction(FormProbe);
  const form = handle.root() as HTMLFormElement;
  const two = handle.getByRole("radio", { name: "Rating 2 of 5" }) as HTMLInputElement;
  const four = handle.getByRole("radio", { name: "Rating 4 of 5" }) as HTMLInputElement;

  assert.equal(ratingFormValue(form), null);
  await handle.click(four);
  assert.equal(four.checked, true);
  assert.equal(four.getAttribute("data-active"), "true");
  assert.equal(two.checked, false);
  assert.equal(ratingFormValue(form), "4");

  await handle.click(four);
  assert.equal(four.checked, false);
  assert.equal(ratingFormValue(form), null);
  assert.deepEqual(
    recorded.map(([event, value, previous]) => [event, value, previous]),
    [
      ["update:modelValue", 4, undefined],
      ["change", 4, null],
      ["update:modelValue", null, undefined],
      ["clear", 4, undefined],
      ["change", null, 4],
    ],
  );
  assert.ok(recorded[1]?.[3] instanceof Event);
  assert.ok(recorded[3]?.[3] instanceof Event);
  handle.unmount();
});

test("controlled value wins until the parent accepts the request", async () => {
  const handle = mountInteraction(Rating, {
    props: { ariaLabel: "Movie score", modelValue: 2, name: "score" },
    record: ["update:modelValue", "change"],
  });
  const two = handle.getByRole("radio", { name: "Rating 2 of 5" }) as HTMLInputElement;
  const five = handle.getByRole("radio", { name: "Rating 5 of 5" }) as HTMLInputElement;

  await handle.click(five);
  await nextTick();

  assert.deepEqual(
    handle.recorded().map((emit) => [emit.event, emit.payload[0], emit.payload[1]]),
    [
      ["update:modelValue", 5, undefined],
      ["change", 5, 2],
    ],
  );
  assert.equal(two.checked, true);
  assert.equal(five.checked, false);

  await handle.wrapper.setProps({ modelValue: 5 });
  assert.equal(two.checked, false);
  assert.equal(five.checked, true);
  handle.unmount();
});

test("defaultValue seeds state and native form reset restores it", async () => {
  const FormProbe = defineComponent({
    setup: () => () =>
      h("form", [h(Rating, { ariaLabel: "Movie score", defaultValue: 2, name: "score" })]),
  });
  const handle = mountInteraction(FormProbe);
  const form = handle.root() as HTMLFormElement;
  const two = handle.getByRole("radio", { name: "Rating 2 of 5" }) as HTMLInputElement;
  const five = handle.getByRole("radio", { name: "Rating 5 of 5" }) as HTMLInputElement;

  assert.equal(two.checked, true);
  assert.equal(ratingFormValue(form), "2");
  await handle.click(five);
  assert.equal(five.checked, true);
  assert.equal(ratingFormValue(form), "5");

  form.reset();
  await nextTick();
  assert.equal(two.checked, true);
  assert.equal(five.checked, false);
  assert.equal(ratingFormValue(form), "2");
  handle.unmount();
});

test("keyboard support honors native radio expectations and RTL direction", async () => {
  const handle = mountInteraction(Rating, {
    props: { ariaLabel: "Movie score", clearable: true, defaultValue: 3, dir: "rtl" },
  });
  const two = handle.getByRole("radio", { name: "Rating 2 of 5" }) as HTMLInputElement;
  const three = handle.getByRole("radio", { name: "Rating 3 of 5" }) as HTMLInputElement;
  const five = handle.getByRole("radio", { name: "Rating 5 of 5" }) as HTMLInputElement;

  three.focus();
  const right = await handle.press(three, "ArrowRight");
  assert.equal(right.keydownPrevented, true);
  assert.equal(two.checked, true);
  assert.ok(handle.activeElement() === two);

  const left = await handle.press(two, "ArrowLeft");
  assert.equal(left.keydownPrevented, true);
  assert.equal(three.checked, true);

  const end = await handle.press(three, "End");
  assert.equal(end.keydownPrevented, true);
  assert.equal(five.checked, true);

  const space = await handle.press(five, " ");
  assert.equal(space.keydownPrevented, true);
  assert.equal(space.activated, false);
  assert.equal(five.checked, false);
  assert.equal(handle.root().getAttribute("data-state"), "empty");
  handle.unmount();
});

test("disabled and read-only ratings keep availability and form semantics", async () => {
  const DisabledProbe = defineComponent({
    setup: () => () =>
      h("form", [
        h(Rating, {
          ariaLabel: "Movie score",
          defaultValue: 3,
          disabled: true,
          name: "score",
          required: true,
        }),
      ]),
  });
  const disabled = mountInteraction(DisabledProbe);
  const disabledForm = disabled.root() as HTMLFormElement;
  const disabledRoot = disabled.getByRole("radiogroup", { name: "Movie score" });
  const disabledThree = disabled.getByRole("radio", { name: "Rating 3 of 5" }) as HTMLInputElement;

  assert.equal(disabledRoot.getAttribute("aria-disabled"), "true");
  assert.equal(disabledRoot.getAttribute("data-state"), "disabled");
  assert.equal(disabledThree.disabled, true);
  assert.equal(ratingFormValue(disabledForm), null);
  assert.ok((await disabled.tab()) === null);
  disabled.unmount();

  const ReadOnlyProbe = defineComponent({
    setup: () => () =>
      h("form", [
        h(Rating, {
          ariaLabel: "Movie score",
          defaultValue: 3,
          name: "score",
          readOnly: true,
        }),
      ]),
  });
  const readOnly = mountInteraction(ReadOnlyProbe);
  const readOnlyForm = readOnly.root() as HTMLFormElement;
  const readOnlyRoot = readOnly.getByRole("radiogroup", { name: "Movie score" });
  const readOnlyThree = readOnly.getByRole("radio", {
    name: "Rating 3 of 5",
  }) as HTMLInputElement;
  const readOnlyFive = readOnly.getByRole("radio", { name: "Rating 5 of 5" }) as HTMLInputElement;

  assert.equal(readOnlyRoot.getAttribute("aria-readonly"), "true");
  assert.equal(readOnlyRoot.getAttribute("data-state"), "readonly");
  assert.equal(readOnlyThree.disabled, false);
  assert.equal(readOnlyThree.getAttribute("aria-readonly"), "true");
  assert.equal(ratingFormValue(readOnlyForm), "3");
  assert.ok((await readOnly.tab()) === readOnlyThree);

  await readOnly.click(readOnlyFive);
  await readOnly.press(readOnlyThree, "ArrowRight");
  assert.equal(readOnlyThree.checked, true);
  assert.equal(readOnlyFive.checked, false);
  assert.equal(readOnly.wrapper.emitted("change"), undefined);
  readOnly.unmount();
});

test("exposes focus, setValue, clear, reset, and normalized state", async () => {
  const handle = mountInteraction(Rating, {
    props: { ariaLabel: "Movie score", defaultValue: 2, max: 4 },
  });
  const two = handle.getByRole("radio", { name: "Rating 2 of 4" }) as HTMLInputElement;
  const four = handle.getByRole("radio", { name: "Rating 4 of 4" }) as HTMLInputElement;
  const exposed = handle.exposes<RatingExpose>();

  assert.equal(exposed.value, 2);
  assert.equal(exposed.min, 1);
  assert.equal(exposed.max, 4);
  assert.equal(exposed.count, 4);
  assert.deepEqual(exposed.items, [1, 2, 3, 4]);
  assert.equal(exposed.elements.length, 4);

  assert.equal(exposed.setValue(9), true);
  await nextTick();
  assert.equal(exposed.value, 4);
  assert.equal(four.checked, true);

  four.blur();
  exposed.focus();
  assert.ok(handle.activeElement() === four);

  assert.equal(exposed.clear(), true);
  await nextTick();
  assert.equal(exposed.value, null);
  assert.equal(four.checked, false);

  assert.equal(exposed.reset(), true);
  await nextTick();
  assert.equal(exposed.value, 2);
  assert.equal(two.checked, true);
  handle.unmount();
});
