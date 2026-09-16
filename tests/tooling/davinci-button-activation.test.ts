import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import test from "node:test";

const runner = fileURLToPath(new URL("./support/davinci-mounted-trace.mjs", import.meta.url));

function run(steps: unknown[], buttons = 1) {
  return spawnSync(process.execPath, [runner], {
    input: JSON.stringify({
      backend: "vdom",
      code: `import { createElementVNode as h } from "vue";
        export function render(ctx) {
          return h("main", null, Array.from({ length: ${buttons} }, () =>
            h("button", { disabled: ctx.locked, onClick: ctx.save }, "Save")));
        }`,
      context: { locked: true },
      steps,
    }),
    encoding: "utf8",
    timeout: 30_000,
  });
}

function snapshot(disabled: boolean, events: string[]) {
  return {
    tree: [
      {
        tag: "main",
        attributes: {},
        children: [
          {
            tag: "button",
            attributes: disabled ? { disabled: "" } : {},
            children: ["Save"],
            disabled,
          },
        ],
      },
    ],
    events,
  };
}

test("activation respects disabled state before and after reactive updates", () => {
  const result = run([
    { activate: "button" },
    { patch: { locked: false } },
    { activate: "button" },
    { patch: { locked: true } },
    { activate: "button" },
  ]);
  assert.ifError(result.error);
  assert.equal(result.status, 0, result.stderr);
  assert.deepEqual(JSON.parse(result.stdout), [
    snapshot(true, []),
    snapshot(true, []),
    snapshot(false, []),
    snapshot(false, ["save"]),
    snapshot(true, ["save"]),
    snapshot(true, ["save"]),
    { tree: [], events: ["save"] },
  ]);
});

test("activation does not silently redefine synthetic event dispatch", () => {
  const result = run([{ activate: "button" }, { event: "click", selector: "button" }]);
  assert.ifError(result.error);
  assert.equal(result.status, 0, result.stderr);
  assert.deepEqual(JSON.parse(result.stdout), [
    snapshot(true, []),
    snapshot(true, []),
    snapshot(true, ["save"]),
    { tree: [], events: ["save"] },
  ]);
});

test("activation rejects ambiguous fields and unsupported targets", () => {
  for (const step of [
    { activate: "button", patch: {} },
    { activate: "button", event: "click" },
    { activate: "button", ignored: true },
    { activate: "missing" },
    { activate: false },
    { activate: null },
  ]) {
    const result = run([step]);
    assert.ifError(result.error);
    assert.equal(result.status, 1, JSON.stringify(step));
    assert.match(
      result.stderr,
      /AssertionError.*(?:unexpected activation fields|unsupported activation target)/u,
    );
    assert.equal(result.stdout, "");
  }
});

test("activation requires exactly one mounted target", () => {
  for (const buttons of [0, 2]) {
    const result = run([{ activate: "button" }], buttons);
    assert.ifError(result.error);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /AssertionError.*expected one activation target/u);
    assert.equal(result.stdout, "");
  }
});
