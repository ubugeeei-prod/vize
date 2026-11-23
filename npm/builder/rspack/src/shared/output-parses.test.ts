// #11583: output assembly reads the native `moduleShape` instead of parsing the
// emitted module again. A child process mocks `oxc-parser` before the output
// helpers load and counts parses over real native compiles.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { test } from "node:test";

void test("assembling real native output never parses the emitted module", () => {
  const child = spawnSync(
    process.execPath,
    [
      "--experimental-test-module-mocks",
      "--no-warnings",
      fileURLToPath(new URL("../../test/output-parses.ts", import.meta.url)),
    ],
    { cwd: new URL("../..", import.meta.url), encoding: "utf8" },
  );
  assert.equal(child.status, 0, child.stderr);
  const results: Record<string, { parses: number; code: string }> = JSON.parse(child.stdout);
  assert.equal(Object.keys(results).length, 13);
  for (const [name, { parses, code }] of Object.entries(results)) {
    assert.equal(parses, 0, `${name}: assembling the module must not re-parse it`);
    if (name.startsWith("/src/reexport")) {
      // Negative control: no `_sfc_main` is exported, so nothing is attached.
      assert.doesNotMatch(code, /_sfc_main/, name);
      assert.match(code, /export \{ default \} from "\.\/x\.js"/, name);
      continue;
    }
    const exported = code.indexOf("export default _sfc_main");
    assert.ok(exported >= 0, `${name}: the component stays the default export`);
    assert.match(code, /_sfc_main\.__scopeId = "data-v-[0-9a-f]{8}";|__scopeId: "data-v-/, name);
    assert.ok(code.indexOf("_sfc_main.__file") > exported, `${name}: appended after the export`);
  }
  assert.match(results["/src/template.ce.vue"].code, /_sfc_main\.styles = \[_style_0\];/);
  assert.match(results["/src/template.vue"].code, /block0\(_sfc_main\)/);
  assert.match(results["/src/setup.vue"].code, /_sfc_main\.__cssModules\["\$style"\]/);
  assert.match(results["/src/setupTs.vue"].code, /import _imports_0 from "\.\/logo\.png";/);
});
