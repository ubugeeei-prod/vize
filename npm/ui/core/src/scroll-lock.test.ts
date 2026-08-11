import assert from "node:assert/strict";

import { effectScope, ref } from "vue";
import { test } from "vite-plus/test";

import { createScrollLock, useScrollLock } from "./scroll-lock.ts";
import { mockWindowMetrics, preserveDocumentPresentation } from "./scroll-lock-test-utils.ts";

test("locks root overflow and restores exact styles, priorities, attributes, and scroll", () => {
  const restorePresentation = preserveDocumentPresentation(document);
  const metrics = mockWindowMetrics(document, {
    clientWidth: 1180,
    innerWidth: 1200,
    scrollX: 7,
    scrollY: 41,
    supportsScrollbarGutter: true,
  });
  const root = document.documentElement;
  const body = document.body;
  root.style.setProperty("overflow", "scroll", "important");
  root.style.setProperty("overscroll-behavior", "contain");
  root.style.setProperty("scrollbar-gutter", "auto");
  root.setAttribute("data-vize-scroll-locked", "external");
  body.style.setProperty("position", "relative");
  const controller = createScrollLock({ document, strategy: "overflow" });
  try {
    controller.activate();
    assert.equal(controller.isLocked.value, true);
    assert.equal(controller.scrollbarGap.value, 20);
    assert.equal(controller.resolvedStrategy.value, "overflow");
    assert.equal(root.style.getPropertyValue("overflow"), "hidden");
    assert.equal(root.style.getPropertyPriority("overflow"), "important");
    assert.equal(root.style.getPropertyValue("overscroll-behavior"), "none");
    assert.equal(root.style.getPropertyValue("scrollbar-gutter"), "stable");
    assert.equal(root.style.getPropertyValue("--vize-scroll-lock-scrollbar-gap"), "20px");
    assert.equal(root.getAttribute("data-vize-scroll-locked"), "");
    assert.equal(body.style.getPropertyValue("position"), "relative");
    controller.deactivate();
    assert.equal(root.style.getPropertyValue("overflow"), "scroll");
    assert.equal(root.style.getPropertyPriority("overflow"), "important");
    assert.equal(root.style.getPropertyValue("overscroll-behavior"), "contain");
    assert.equal(root.style.getPropertyValue("scrollbar-gutter"), "auto");
    assert.equal(root.style.getPropertyValue("--vize-scroll-lock-scrollbar-gap"), "");
    assert.equal(root.getAttribute("data-vize-scroll-locked"), "external");
    assert.deepEqual(metrics.scrollCalls.at(-1), {
      behavior: "instant",
      left: 7,
      top: 41,
    });
  } finally {
    controller.dispose();
    metrics.restore();
    restorePresentation();
  }
});

test("nested locks compose the strongest strategy and unwind without an unlock gap", () => {
  const restorePresentation = preserveDocumentPresentation(document);
  const metrics = mockWindowMetrics(document, {
    clientWidth: 980,
    innerWidth: 1000,
    scrollX: 8,
    scrollY: 13,
    supportsScrollbarGutter: false,
  });
  const root = document.documentElement;
  const body = document.body;
  root.style.setProperty("padding-inline-end", "4px");
  const parent = createScrollLock({ document, strategy: "overflow" });
  const child = createScrollLock({ document, preserveScrollbarGap: false, strategy: "fixed" });
  try {
    parent.activate();
    assert.equal(root.style.getPropertyValue("padding-inline-end"), "24px");
    child.activate();
    assert.equal(parent.resolvedStrategy.value, "fixed");
    assert.equal(child.resolvedStrategy.value, "fixed");
    assert.equal(body.style.getPropertyValue("position"), "fixed");
    assert.equal(body.style.getPropertyValue("top"), "-13px");
    assert.equal(body.style.getPropertyValue("left"), "-8px");
    child.deactivate();
    assert.equal(parent.isLocked.value, true);
    assert.equal(parent.resolvedStrategy.value, "overflow");
    assert.equal(root.style.getPropertyValue("overflow"), "hidden");
    assert.equal(body.style.getPropertyValue("position"), "");
    parent.deactivate();
    assert.equal(root.style.getPropertyValue("padding-inline-end"), "4px");
    assert.equal(root.style.getPropertyValue("overflow"), "");
  } finally {
    child.dispose();
    parent.dispose();
    metrics.restore();
    restorePresentation();
  }
});

test("reactive document, enablement, strategy, gap, and restoration policies recompute", () => {
  const restorePresentation = preserveDocumentPresentation(document);
  const metrics = mockWindowMetrics(document, {
    clientWidth: 790,
    innerWidth: 800,
    scrollX: 0,
    scrollY: 25,
  });
  const documentRef = ref<Document | null>(null);
  const enabled = ref(true);
  const preserveGap = ref(true);
  const restoreScroll = ref(false);
  const strategy = ref<"fixed" | "overflow">("overflow");
  const controller = createScrollLock({
    document: documentRef,
    enabled,
    preserveScrollbarGap: preserveGap,
    restoreScroll,
    strategy,
  });
  try {
    controller.activate();
    assert.equal(controller.isLocked.value, false);
    documentRef.value = document;
    assert.equal(controller.isLocked.value, true);
    assert.equal(controller.scrollbarGap.value, 10);
    strategy.value = "fixed";
    assert.equal(document.body.style.getPropertyValue("position"), "fixed");
    preserveGap.value = false;
    assert.equal(document.documentElement.style.getPropertyValue("padding-inline-end"), "");
    const callsBeforeDisable = metrics.scrollCalls.length;
    enabled.value = false;
    assert.equal(controller.isLocked.value, false);
    assert.equal(document.documentElement.style.getPropertyValue("overflow"), "");
    assert.equal(metrics.scrollCalls.length, callsBeforeDisable);
    const callsBeforeDispose = metrics.scrollCalls.length;
    controller.dispose();
    assert.equal(metrics.scrollCalls.length, callsBeforeDispose);
  } finally {
    controller.dispose();
    metrics.restore();
    restorePresentation();
  }
});

test("auto strategy recognizes touch-capable iPad desktop mode without blocking zoom", () => {
  const restorePresentation = preserveDocumentPresentation(document);
  const metrics = mockWindowMetrics(document, {
    clientWidth: 800,
    innerWidth: 800,
    maxTouchPoints: 5,
    platform: "MacIntel",
    scrollX: 0,
    scrollY: 19,
    userAgent: "Desktop Safari",
  });
  const controller = createScrollLock({ document });
  try {
    controller.activate();
    assert.equal(controller.resolvedStrategy.value, "fixed");
    assert.equal(document.body.style.getPropertyValue("top"), "-19px");
    assert.equal(document.documentElement.style.getPropertyValue("touch-action"), "");
  } finally {
    controller.dispose();
    metrics.restore();
    restorePresentation();
  }
});

test("cross-document migration restores the old document before locking the new one", () => {
  const restorePresentation = preserveDocumentPresentation(document);
  const frame = document.createElement("iframe");
  document.body.append(frame);
  const frameDocument = frame.contentDocument;
  assert.ok(frameDocument);
  const documentRef = ref<Document | null>(document);
  const controller = createScrollLock({ document: documentRef, strategy: "fixed" });
  try {
    controller.activate();
    assert.equal(document.body.style.getPropertyValue("position"), "fixed");
    documentRef.value = frameDocument;
    assert.equal(document.body.style.getPropertyValue("position"), "");
    assert.equal(frameDocument.body.style.getPropertyValue("position"), "fixed");
    controller.deactivate();
    assert.equal(frameDocument.body.style.getPropertyValue("position"), "");
  } finally {
    controller.dispose();
    frame.remove();
    restorePresentation();
  }
});

test("refresh acquires a document that gains its body after activation", () => {
  const lateDocument = document.implementation.createHTMLDocument("late body");
  const body = lateDocument.body;
  body.remove();
  const controller = createScrollLock({ document: lateDocument, strategy: "overflow" });
  try {
    controller.activate();
    assert.equal(controller.isActive.value, true);
    assert.equal(controller.isLocked.value, false);
    assert.equal(lateDocument.documentElement.style.getPropertyValue("overflow"), "");

    lateDocument.documentElement.append(body);
    controller.refresh();
    assert.equal(controller.isLocked.value, true);
    assert.equal(lateDocument.documentElement.style.getPropertyValue("overflow"), "hidden");
  } finally {
    controller.dispose();
  }
  assert.equal(lateDocument.documentElement.style.getPropertyValue("overflow"), "");
});

test("runtime diagnostics, idempotence, and effect-scope disposal are explicit", () => {
  assert.throws(() => createScrollLock(null as never), /options must be an object/);
  assert.throws(
    () => createScrollLock({ document: "main" } as never),
    /VIZE_UI_SCROLL_LOCK_OPTION.*Document/,
  );
  assert.throws(
    () => createScrollLock({ document: null, strategy: "position" } as never),
    /VIZE_UI_SCROLL_LOCK_OPTION.*strategy/,
  );
  assert.throws(() => useScrollLock({ document }), /VIZE_UI_SCROLL_LOCK_SETUP/);
  const restorePresentation = preserveDocumentPresentation(document);
  const scope = effectScope();
  let controller!: ReturnType<typeof useScrollLock>;
  try {
    scope.run(() => {
      controller = useScrollLock({ document, strategy: "overflow" });
    });
    controller.activate();
    assert.equal(controller.isLocked.value, true);
    scope.stop();
    assert.equal(controller.isActive.value, false);
    assert.equal(document.documentElement.style.getPropertyValue("overflow"), "");
    controller.dispose();
    assert.throws(() => controller.refresh(), /VIZE_UI_SCROLL_LOCK_DISPOSED/);
  } finally {
    scope.stop();
    controller?.dispose();
    restorePresentation();
  }
});
