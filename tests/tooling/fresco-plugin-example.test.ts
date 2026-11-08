import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

const read = (relativePath: string) =>
  readFileSync(new URL(`../../${relativePath}`, import.meta.url), "utf8");

void test("the external Fresco provider example stays on public APIs", () => {
  const main = read("npm/fresco/examples/external-plugin/main.ts");
  const plugin = read("npm/fresco/examples/external-plugin/plugin.ts");
  const provider = read("npm/fresco/examples/external-plugin/provider.ts");
  const component = read("npm/fresco/examples/external-plugin/ExternalMeter.vue");
  const guide = read("docs/fresco/plugins.md");

  for (const source of [main, plugin, provider, guide]) {
    assert.match(source, /from "@vizejs\/fresco"/u);
    assert.doesNotMatch(source, /@vizejs\/fresco\/src\//u);
  }
  assert.ok(component.length > 0);
});
