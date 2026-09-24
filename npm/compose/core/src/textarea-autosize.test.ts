import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, nextTick, ref } from "vue";

import { useTextareaAutosize } from "./textarea-autosize.ts";
import { asElement, FakeDocument } from "./testing/fake-dom.ts";

void test("writes the measured scroll height on input changes", async () => {
  const element = new FakeDocument().createElement();
  element.scrollHeight = 40;
  const heights: number[] = [];
  const input = ref("a");
  const scope = effectScope();
  const autosize = scope.run(() =>
    useTextareaAutosize(asElement(element), {
      input,
      onResize: (height) => heights.push(height),
      host: () => null,
    }),
  );
  assert.equal(element.styles.get("height"), "40px");

  element.scrollHeight = 80;
  input.value = "a\nb";
  await nextTick();
  assert.equal(element.styles.get("height"), "80px");
  element.scrollHeight = 20;
  autosize?.triggerResize();
  assert.deepEqual(heights, [40, 80, 20]);
  scope.stop();
});

void test("supports min-height and an internal input ref", () => {
  const element = new FakeDocument().createElement();
  element.scrollHeight = 33;
  const autosize = useTextareaAutosize(asElement(element), {
    styleProp: "min-height",
    host: () => null,
  });
  assert.equal(autosize.input.value, "");
  assert.equal(element.styles.get("min-height"), "33px");
});
