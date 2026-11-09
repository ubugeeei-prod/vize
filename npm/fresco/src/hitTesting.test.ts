import assert from "node:assert/strict";
import { test } from "vitest";
import { h } from "vue";

import { firstChild, mountFresco } from "./testing/mount.js";

test("hit testing selects the topmost node within ancestor overflow clips", () => {
  const mounted = mountFresco(() =>
    h("box", { style: { overflow: "hidden" } }, [
      h("text", { text: "under" }),
      h("box", [h("text", { text: "outside" })]),
    ]),
  );
  const root = firstChild(mounted);
  const under = root.children[0];
  const overlay = root.children[1];
  const outside = overlay?.children[0];
  assert.ok(under && overlay && outside);

  const layouts = [
    { id: root.id, x: 0, y: 0, width: 4, height: 3 },
    { id: under.id, x: 1, y: 1, width: 3, height: 1 },
    { id: overlay.id, x: 2, y: 1, width: 4, height: 2 },
    { id: outside.id, x: 4, y: 1, width: 2, height: 1 },
  ];

  assert.equal(mounted.renderer.hitTest(root, 2, 1, layouts), overlay);
  assert.equal(mounted.renderer.hitTest(root, 1, 1, layouts), under);
  assert.equal(
    mounted.renderer.hitTest(root, 4, 1, layouts),
    undefined,
    "a descendant outside hidden overflow cannot receive pointer input",
  );
  assert.equal(mounted.renderer.hitTest(root, Number.NaN, 1, layouts), undefined);
  mounted.unmount();

  const visible = mountFresco(() =>
    h("box", { style: { overflow: "visible" } }, [h("text", { text: "outside" })]),
  );
  const visibleRoot = firstChild(visible);
  const visibleChild = visibleRoot.children[0];
  assert.ok(visibleChild);
  assert.equal(
    visible.renderer.hitTest(visibleRoot, 4, 0, [
      { id: visibleRoot.id, x: 0, y: 0, width: 4, height: 1 },
      { id: visibleChild.id, x: 4, y: 0, width: 2, height: 1 },
    ]),
    visibleChild,
    "visible overflow keeps out-of-bounds descendants targetable",
  );
  visible.unmount();

  const visibleOverride = mountFresco(() =>
    h("box", { style: { overflow: "visible", overflowX: "hidden" } }, [
      h("text", { text: "outside" }),
    ]),
  );
  const visibleOverrideRoot = firstChild(visibleOverride);
  const visibleOverrideChild = visibleOverrideRoot.children[0];
  assert.ok(visibleOverrideChild);
  assert.equal(
    visibleOverride.renderer.hitTest(visibleOverrideRoot, 4, 0, [
      { id: visibleOverrideRoot.id, x: 0, y: 0, width: 4, height: 1 },
      { id: visibleOverrideChild.id, x: 4, y: 0, width: 2, height: 1 },
    ]),
    visibleOverrideChild,
    "explicit visible overflow takes precedence over axis aliases",
  );
  visibleOverride.unmount();

  const axisAlias = mountFresco(() =>
    h("box", { style: { overflowX: "hidden" } }, [h("text", { text: "below" })]),
  );
  const axisRoot = firstChild(axisAlias);
  const below = axisRoot.children[0];
  assert.ok(below);
  assert.equal(
    axisAlias.renderer.hitTest(axisRoot, 0, 1, [
      { id: axisRoot.id, x: 0, y: 0, width: 2, height: 1 },
      { id: below.id, x: 0, y: 1, width: 2, height: 1 },
    ]),
    undefined,
    "native overflow aliases clip both paint and input axes",
  );
  axisAlias.unmount();
});

test("hit testing follows sibling z-index before source order", () => {
  const mounted = mountFresco(() =>
    h("box", [
      h("text", { text: "front", style: { zIndex: 2 } }),
      h("text", { text: "back", style: { zIndex: -1 } }),
    ]),
  );
  const root = firstChild(mounted);
  const front = root.children[0];
  const back = root.children[1];
  assert.ok(front && back);

  assert.equal(
    mounted.renderer.hitTest(root, 0, 0, [
      { id: root.id, x: 0, y: 0, width: 5, height: 1 },
      { id: front.id, x: 0, y: 0, width: 5, height: 1 },
      { id: back.id, x: 0, y: 0, width: 5, height: 1 },
    ]),
    front,
  );
  mounted.unmount();
});

test("equal sibling z-index keeps reverse source order", () => {
  const mounted = mountFresco(() =>
    h("box", [
      h("text", { text: "under", style: { zIndex: 2 } }),
      h("text", { text: "front", style: { zIndex: 2 } }),
    ]),
  );
  const root = firstChild(mounted);
  const under = root.children[0];
  const front = root.children[1];
  assert.ok(under && front);

  assert.equal(
    mounted.renderer.hitTest(root, 0, 0, [
      { id: root.id, x: 0, y: 0, width: 5, height: 1 },
      { id: under.id, x: 0, y: 0, width: 5, height: 1 },
      { id: front.id, x: 0, y: 0, width: 5, height: 1 },
    ]),
    front,
  );
  mounted.unmount();
});

test("explicit null z-index does not fall through to the native alias", () => {
  const mounted = mountFresco(() =>
    h("box", [
      h("text", { text: "under", style: { zIndex: null, z_index: 1 } }),
      h("text", { text: "front" }),
    ]),
  );
  const root = firstChild(mounted);
  const under = root.children[0];
  const front = root.children[1];
  assert.ok(under && front);

  assert.equal(
    mounted.renderer.hitTest(root, 0, 0, [
      { id: root.id, x: 0, y: 0, width: 5, height: 1 },
      { id: under.id, x: 0, y: 0, width: 5, height: 1 },
      { id: front.id, x: 0, y: 0, width: 5, height: 1 },
    ]),
    front,
  );
  mounted.unmount();
});

test("default stacking skips full style normalization for untouched siblings", () => {
  const mounted = mountFresco(() =>
    h("box", [h("text", { text: "under" }), h("text", { text: "front" })]),
  );
  const root = firstChild(mounted);
  const under = root.children[0];
  const front = root.children[1];
  assert.ok(under && front);

  (under.props as Record<string, unknown>).style = new Proxy(
    { zIndex: 0 },
    {
      get(target, property, receiver) {
        assert.ok(property === "zIndex" || property === "z_index");
        return Reflect.get(target, property, receiver);
      },
    },
  );

  assert.equal(
    mounted.renderer.hitTest(root, 0, 0, [
      { id: root.id, x: 0, y: 0, width: 5, height: 1 },
      { id: under.id, x: 0, y: 0, width: 5, height: 1 },
      { id: front.id, x: 0, y: 0, width: 5, height: 1 },
    ]),
    front,
  );
  mounted.unmount();
});
