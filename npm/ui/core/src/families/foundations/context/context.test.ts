import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createApp } from "vue";

import { createContext } from "./context.ts";

test("creates an immutable context with an integration key", () => {
  const context = createContext<{ value: number }>(" Selection ");

  assert.equal(context.name, "Selection");
  assert.equal(context.key.description, "Selection");
  assert.equal(Object.isFrozen(context), true);
});

test("resolves application-provided values", () => {
  const context = createContext<{ value: number }>("Selection");
  const value = { value: 42 };
  const app = createApp({});
  app.provide(context.key, value);

  assert.equal(app.runWithContext(context.use), value);
  assert.equal(app.runWithContext(context.useOptional), value);
});

test("distinguishes an explicit undefined value from a missing provider", () => {
  const context = createContext<string | undefined>("NullableSelection");
  const provided = createApp({});
  provided.provide(context.key, undefined);

  assert.equal(provided.runWithContext(context.use), undefined);

  const missing = createApp({});
  assert.equal(missing.runWithContext(context.useOptional), undefined);
  assert.throws(
    () => missing.runWithContext(context.use),
    /VIZE_UI_CONTEXT_MISSING: NullableSelection requires a matching provider/,
  );
});

test("rejects names that cannot produce actionable diagnostics", () => {
  assert.throws(() => createContext("   "), /VIZE_UI_CONTEXT_NAME/);
});
