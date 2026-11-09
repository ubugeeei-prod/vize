import assert from "node:assert/strict";

/** Observe Teleport destinations outside the app's mount container. */
export function mountedScope(window, host, targetIds, observeChildren) {
  assert.ok(Array.isArray(targetIds), "external target IDs must be an array");
  assert.equal(new Set(targetIds).size, targetIds.length, "duplicate external target ID");
  const targets = targetIds.map((id) => {
    assert.equal(typeof id, "string", "external target ID must be a string");
    assert.match(id, /^[A-Za-z][A-Za-z0-9-]*$/u, "unsupported external target ID");
    const element = window.document.createElement("div");
    element.id = id;
    window.document.body.append(element);
    return element;
  });
  return {
    elements(selector = "*") {
      return [host, ...targets].flatMap((root) => [...root.querySelectorAll(selector)]);
    },
    observations() {
      return targets.length
        ? {
            targets: Object.fromEntries(
              targets.map((target) => [target.id, observeChildren(target)]),
            ),
          }
        : {};
    },
    assertUnmounted() {
      for (const target of targets)
        assert.equal(target.childNodes.length, 0, `unmount left Teleport nodes in ${target.id}`);
    },
    dispose() {
      for (const target of targets) target.remove();
    },
  };
}
