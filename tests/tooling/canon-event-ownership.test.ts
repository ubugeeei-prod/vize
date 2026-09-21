import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { check, workspace } from "./support/upstream/vue-language-tools.ts";

test("handler references and authored callbacks preserve lexical $event ownership", async () => {
  const directory = workspace("event-ownership-");
  fs.writeFileSync(
    path.join(directory, "tsconfig.json"),
    JSON.stringify({
      compilerOptions: {
        strict: true,
        noEmit: true,
        module: "esnext",
        moduleResolution: "bundler",
        skipLibCheck: true,
      },
      include: ["*.vue"],
    }),
  );
  const file = path.join(directory, "App.vue");
  const errors = async (script: string, handler: string) => {
    fs.writeFileSync(
      file,
      `<script setup lang="ts">\n${script}\n</script>\n<template><button @click="${handler}" /></template>`,
    );
    return check(directory);
  };
  try {
    for (const handler of ["$event", "$event.handler", "($event.handler)", "$event?.handler"]) {
      const script =
        handler === "$event"
          ? "const $event = (_event: PointerEvent) => {};"
          : "const $event = { handler: (_event: PointerEvent) => {} };";
      assert.deepEqual(await errors(script, handler), [], handler);
    }
    for (const handler of ["() => consume($event)", "function () { consume($event); }"]) {
      assert.deepEqual(
        await errors("const $event = 'authored'; function consume(value: string) {}", handler),
        [],
        handler,
      );
      // An authored callback owns its parameters: `$event` is a template name
      // like any other there, and nothing on the instance provides it.
      assert.deepEqual(
        (await errors("function consume(value: string) {}", handler)).map((d) => d.code),
        [2551],
        handler,
      );
    }
    assert.deepEqual(
      await errors(
        "const $event = 'authored'; function consume(value: PointerEvent) {}",
        "consume($event)",
      ),
      [],
    );
    assert.deepEqual(
      (
        await errors(
          "const $event = 'authored'; function consume(value: string) {}",
          "consume($event)",
        )
      ).map((d) => d.code),
      [2345],
    );
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
