import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

import { vueVaporBrowserRuntime, vueVaporVersion } from "./support/vue-vapor-release.mjs";

const source = fs.readFileSync(
  new URL("../../davinci/vize_l4/src/runtime/vue.rs", import.meta.url),
  "utf8",
);
const fromUi = createRequire(new URL("../../npm/ui/package.json", import.meta.url));
const fromVue = createRequire(fromUi.resolve("vue/package.json"));

function names(table: string): string[] {
  const content = source.match(
    new RegExp(String.raw`const ${table}: &\[&str\] = &\[([^]*?)\];`, "u"),
  );
  assert.ok(content, `missing runtime table ${table}`);
  const exports = [...(content[1] ?? "").matchAll(/"(\w+)"/gu)].map((match) => match[1]);
  assert.ok(exports.length > 0 && exports.length <= 128);
  assert.equal(new Set(exports).size, exports.length);
  return exports;
}

function exported(runtime: Record<string, unknown>, table: string): void {
  for (const name of names(table)) {
    assert.ok(Object.hasOwn(runtime, name), `${table}: missing export ${name}`);
    assert.notEqual(runtime[name], null, `${table}: compat-only null export ${name}`);
    assert.notEqual(runtime[name], undefined, `${table}: undefined export ${name}`);
  }
}

test("L4 DOM and SSR names are real exports of the selected stable release", () => {
  assert.equal(fromVue("./package.json").version, "3.5.35");
  const fromServer = createRequire(fromVue.resolve("@vue/server-renderer/package.json"));
  assert.equal(fromServer("./package.json").version, "3.5.35");
  exported(fromVue("vue"), "DOM_NAMES");
  exported(fromServer("@vue/server-renderer"), "SERVER_NAMES");
});

test("L4 Vapor names are real exports of the selected rc.9 dual-renderer build", async () => {
  assert.equal(vueVaporVersion, "3.6.0-rc.9");
  const runtime = await import(pathToFileURL(vueVaporBrowserRuntime).href);
  exported(runtime, "VAPOR_NAMES");
  assert.ok(!names("VAPOR_NAMES").includes("prepend"));
  assert.ok(names("VAPOR_NAMES").includes("withVaporModifiers"));
  assert.ok(names("VAPOR_NAMES").includes("withVaporKeys"));
});
