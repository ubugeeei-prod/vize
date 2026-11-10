import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { musea } from "./lazy.js";

void test("lazy root plugin delegates its first Vite hook", async (t) => {
  const root = await fs.promises.mkdtemp(path.join(os.tmpdir(), "musea-lazy-plugin-"));
  t.after(() => fs.promises.rm(root, { recursive: true, force: true }));
  await fs.promises.mkdir(path.join(root, "src"));

  const [plugin] = musea({ include: ["src/**/*.art.vue"] });
  const configHook = plugin?.config;
  assert.equal(typeof configHook, "function");
  const config = await Reflect.apply(configHook as CallableFunction, {}, [
    { root },
    { command: "build", mode: "production" },
  ]);

  assert.match(config.resolve.alias[0].replacement, /vue\.esm-bundler\.js$/);
});
