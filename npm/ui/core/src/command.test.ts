import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { effectScope, ref } from "vue";

import { createCommandRouter, useCommandRouter } from "./command.ts";
import type { CommandDispatch, CommandExecution } from "./command.ts";

test("routes a dispatch to the registered command with a frozen execution context", () => {
  const router = createCommandRouter<"save" | "open">();
  const executions: CommandExecution<"save" | "open">[] = [];
  router.register({
    id: "save",
    title: "Save Document",
    run: (execution) => {
      executions.push(execution);
      return "saved";
    },
  });

  const dispatch = router.execute("save");
  assert.equal(dispatch.status, "executed");
  assert.equal(dispatch.value, "saved");
  assert.equal(dispatch.source, "imperative");
  assert.ok(Object.isFrozen(dispatch));
  assert.equal(executions.length, 1);
  assert.ok(Object.isFrozen(executions[0]));
  assert.equal(executions[0]?.id, "save");
  assert.equal(executions[0]?.payload, undefined);
  router.dispose();
});

test("payload and source thread through to the handler and observers", () => {
  const observed: CommandDispatch<"insert">[] = [];
  const router = createCommandRouter<"insert">({
    onDidExecute: (dispatch) => observed.push(dispatch),
  });
  router.register({
    id: "insert",
    run: (execution) => execution.payload,
  });

  const dispatch = router.execute("insert", { text: "hello" }, { source: "palette" });
  assert.equal(dispatch.status, "executed");
  assert.deepEqual(dispatch.value, { text: "hello" });
  assert.equal(dispatch.source, "palette");
  assert.deepEqual(observed, [dispatch]);
  router.dispose();
});

test("unknown identifiers report not-found without running observers", () => {
  const observed: unknown[] = [];
  const router = createCommandRouter({ onDidExecute: (dispatch) => observed.push(dispatch) });
  const dispatch = router.execute("missing");
  assert.equal(dispatch.status, "not-found");
  assert.equal(dispatch.value, undefined);
  assert.deepEqual(observed, []);
  assert.equal(router.has("missing"), false);
  router.dispose();
});

test("enablement gates the command and the whole router", () => {
  const enabled = ref(false);
  const disabled = ref(false);
  const router = createCommandRouter<"cut">({ isDisabled: disabled });
  let runs = 0;
  router.register({ id: "cut", when: enabled, run: () => (runs += 1) });

  assert.equal(router.isEnabled("cut"), false);
  assert.equal(router.execute("cut").status, "disabled");
  enabled.value = true;
  assert.equal(router.isEnabled("cut"), true);
  disabled.value = true;
  assert.equal(router.isEnabled("cut"), false);
  assert.equal(router.execute("cut").status, "disabled");
  disabled.value = false;
  assert.equal(router.execute("cut").status, "executed");
  assert.equal(runs, 1);
  router.dispose();
});

test("duplicate identifiers conflict until the registration is released", () => {
  const router = createCommandRouter<"save">();
  const release = router.register({ id: "save", title: "Save Document", run: () => undefined });
  assert.throws(
    () => router.register({ id: "save", run: () => undefined }),
    /VIZE_UI_COMMAND_CONFLICT.*Save Document/,
  );

  release();
  release();
  assert.equal(router.has("save"), false);
  router.register({ id: "save", title: "Save As", run: () => undefined });
  assert.equal(router.commands.value[0]?.title, "Save As");
  router.dispose();
});

test("commands publishes reactive frozen help metadata in registration order", () => {
  const enabled = ref(true);
  const router = createCommandRouter<"save" | "open">();
  router.register({
    id: "save",
    title: "Save Document",
    description: "Write the active document to disk",
    keywords: ["persist", "write"],
    group: "file",
    when: enabled,
    run: () => undefined,
  });
  const releaseOpen = router.register({ id: "open", run: () => undefined });

  const listed = router.commands.value;
  assert.ok(Object.isFrozen(listed));
  assert.deepEqual(
    listed.map((command) => command.id),
    ["save", "open"],
  );
  assert.equal(listed[0]?.title, "Save Document");
  assert.deepEqual(listed[0]?.keywords, ["persist", "write"]);
  assert.equal(listed[0]?.group, "file");
  assert.equal(listed[1]?.title, null);
  assert.equal(listed[0]?.isEnabled(), true);
  enabled.value = false;
  assert.equal(listed[0]?.isEnabled(), false);

  releaseOpen();
  assert.deepEqual(
    router.commands.value.map((command) => command.id),
    ["save"],
  );
  router.dispose();
});

test("a throwing handler surfaces its failure and reports no dispatch", () => {
  const observed: unknown[] = [];
  const router = createCommandRouter<"explode">({
    onDidExecute: (dispatch) => observed.push(dispatch),
  });
  router.register({
    id: "explode",
    run: () => {
      throw new Error("boom");
    },
  });
  assert.throws(() => router.execute("explode"), /boom/);
  assert.deepEqual(observed, []);
  router.dispose();
});

test("stable diagnostics reject malformed definitions and options", () => {
  assert.throws(() => createCommandRouter({ onDidExecute: 1 as never }), /VIZE_UI_COMMAND_OPTION/);
  const router = createCommandRouter();
  assert.throws(() => router.register({ id: "", run: () => undefined }), /VIZE_UI_COMMAND_OPTION/);
  assert.throws(() => router.register({ id: "a", run: null as never }), /VIZE_UI_COMMAND_OPTION/);
  assert.throws(
    () => router.register({ id: "a", run: () => undefined, keywords: [1] as never }),
    /VIZE_UI_COMMAND_OPTION/,
  );
  assert.throws(
    () => router.execute("a", undefined, { source: "webhook" as never }),
    /VIZE_UI_COMMAND_OPTION/,
  );
  const gate = ref<boolean | string>(false);
  const gated = createCommandRouter({ isDisabled: gate as never });
  gated.register({ id: "a", run: () => undefined });
  gate.value = "no";
  assert.throws(() => gated.isEnabled("a"), /VIZE_UI_COMMAND_OPTION/);
  router.dispose();
  gated.dispose();
});

test("dispose and Vue scope teardown clear registrations and become terminal", () => {
  const scope = effectScope();
  const router = scope.run(() => useCommandRouter<"save">())!;
  router.register({ id: "save", run: () => undefined });
  assert.equal(router.commands.value.length, 1);

  scope.stop();
  assert.equal(router.commands.value.length, 0);
  assert.throws(() => router.execute("save"), /VIZE_UI_COMMAND_DISPOSED/);
  assert.throws(() => router.has("save"), /VIZE_UI_COMMAND_DISPOSED/);
  assert.throws(
    () => router.register({ id: "save", run: () => undefined }),
    /VIZE_UI_COMMAND_DISPOSED/,
  );
  router.dispose();
  assert.throws(() => useCommandRouter(), /VIZE_UI_COMMAND_SETUP/);
});
