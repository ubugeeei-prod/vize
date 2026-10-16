import assert from "node:assert/strict";
import { test } from "node:test";
import {
  createSSRApp,
  defineComponent,
  effectScope,
  h,
  nextTick,
  ref,
  shallowRef,
  useTemplateRef,
} from "vue";
import { renderToString } from "vue/server-renderer";

import { useTextareaAutosize } from "./textarea-autosize.ts";
import {
  asElement,
  FakeDocument,
  latestObserver,
  resetObservers,
  resizeObserverHost,
  trigger,
} from "./testing/fake-dom.ts";

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

for (const styleProp of ["height", "min-height"] as const) {
  for (const boxSizing of ["content-box", "border-box"] as const) {
    void test(`${styleProp} converts scrollHeight into a ${boxSizing} CSS size`, async () => {
      const document = new FakeDocument();
      const element = document.createElement();
      const target = asElement(element);
      let measuredHeight = 100;
      // These stand-ins control the box-model arithmetic and write ordering;
      // real browser geometry is verified separately, not simulated here.
      Object.defineProperty(document, "defaultView", {
        value: {
          getComputedStyle: (received: Element) => {
            assert.equal(received, target);
            return {
              boxSizing,
              paddingTop: "8.5px",
              paddingBottom: "7.5px",
              borderTopWidth: "2.5px",
              borderBottomWidth: "3.5px",
            };
          },
        },
      });
      Object.defineProperty(element, "scrollHeight", {
        // The runtime also reads this property in its numeric capability
        // guard. Only an auto-sized measurement reports the intended content;
        // retaining the old height would produce a different result.
        get: () => (element.styles.get(styleProp) === "auto" ? measuredHeight : 999),
      });
      const input = ref("First draft");
      const heights: number[] = [];
      const scope = effectScope();
      try {
        scope.run(() =>
          useTextareaAutosize(target, {
            input,
            styleProp,
            onResize: (height) => heights.push(height),
            host: () => null,
          }),
        );
        const initial = boxSizing === "border-box" ? 106 : 84;
        const smaller = boxSizing === "border-box" ? 70 : 48;
        assert.equal(element.styles.get(styleProp), `${initial}px`);
        assert.deepEqual(heights, [initial]);
        measuredHeight = 64;
        input.value = "Short";
        await nextTick();
        assert.equal(element.styles.get(styleProp), `${smaller}px`, "shorter content can shrink");
        assert.deepEqual(heights, [initial, smaller], "callback reports the written CSS height");
      } finally {
        scope.stop();
      }
    });
  }

  void test(`${styleProp} never writes a negative content height`, () => {
    const document = new FakeDocument();
    const element = document.createElement();
    element.scrollHeight = 8;
    Object.defineProperty(document, "defaultView", {
      value: {
        getComputedStyle: () => ({
          boxSizing: "content-box",
          paddingTop: "8px",
          paddingBottom: "8px",
        }),
      },
    });
    const scope = effectScope();
    try {
      scope.run(() => useTextareaAutosize(asElement(element), { styleProp, host: () => null }));
      assert.equal(element.styles.get(styleProp), "0px");
    } finally {
      scope.stop();
    }
  });
}

void test("SSR renders the declared textarea rows and escaped draft without measuring", async () => {
  resetObservers();
  const heights: number[] = [];
  const Probe = defineComponent({
    setup() {
      const target = useTemplateRef<HTMLTextAreaElement>("draft");
      const { input } = useTextareaAutosize(target, {
        input: ref("Release <notes>\nKeep & verify"),
        onResize: (height) => heights.push(height),
        host: resizeObserverHost(),
      });
      return () => h("textarea", { ref: "draft", rows: 3, value: input.value });
    },
  });
  const expected = '<textarea rows="3">Release &lt;notes&gt;\nKeep &amp; verify</textarea>';
  assert.equal(await renderToString(createSSRApp(Probe)), expected);
  assert.equal(await renderToString(createSSRApp(Probe)), expected);
  assert.deepEqual(heights, [], "no target is available to measure during SSR");
  assert.throws(latestObserver, /no observer was constructed/);
});

void test("scope disposal stops automatic input/target watching and disconnects width observation", async () => {
  resetObservers();
  const document = new FakeDocument();
  const first = document.createElement();
  first.scrollHeight = 40;
  const second = document.createElement();
  second.scrollHeight = 120;
  const target = shallowRef<Element | null>(asElement(first));
  const input = ref("Draft");
  const heights: number[] = [];
  const scope = effectScope();
  try {
    scope.run(() =>
      useTextareaAutosize(target, {
        input,
        onResize: (height) => heights.push(height),
        host: resizeObserverHost(),
      }),
    );
    const observer = latestObserver();
    assert.equal(observer.observed.has(target.value), true);
    first.scrollHeight = 60;
    trigger(observer, [{ contentRect: { width: 200 } }]);
    assert.equal(first.styles.get("height"), "60px");
    first.scrollHeight = 80;
    trigger(observer, [{ contentRect: { width: 200 } }]);
    assert.deepEqual(heights, [40, 60], "height-only notifications do not remeasure");
    trigger(observer, [{ contentRect: { width: 140 } }]);
    assert.deepEqual(heights, [40, 60, 80], "width changes remeasure content");

    scope.stop();
    assert.equal(observer.disconnected, true);
    assert.equal(observer.observed.size, 0);
    first.scrollHeight = 100;
    input.value = "A later draft";
    await nextTick();
    assert.equal(first.styles.get("height"), "80px");
    target.value = asElement(second);
    await nextTick();
    assert.equal(second.styles.has("height"), false);
    assert.equal(latestObserver(), observer, "no new observer follows the changed target");
    assert.deepEqual(heights, [40, 60, 80]);
  } finally {
    scope.stop();
  }
});
