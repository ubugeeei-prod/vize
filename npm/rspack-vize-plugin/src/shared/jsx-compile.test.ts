import { test } from "node:test";
import assert from "node:assert/strict";
import { compileJsxModule } from "./compiler.ts";

const SOURCE = "const App = () => <div>{message}</div>;\nexport default App;\n";

void test("compileJsxModule includes the runtime-helper preamble in the emitted module", () => {
  // The VDOM render code references `_createElementBlock` / `_toDisplayString`,
  // so the emitted module must import them — the preamble is no longer dropped
  // (#1533).
  const { code } = compileJsxModule("/src/App.tsx", SOURCE, { jsxMode: "vdom" });
  assert.match(
    code,
    /import \{[^}]*createElementBlock[^}]*\} from "vue"/,
    "the module imports its runtime helpers",
  );
  assert.match(code, /_createElementBlock\("div"/, "the render code uses the imported helper");
});

void test("compileJsxModule surfaces a v3 source map when sourceMap is requested", () => {
  const { map } = compileJsxModule("/src/App.tsx", SOURCE, { jsxMode: "vdom", sourceMap: true });
  assert.equal(typeof map, "string", "a source map is surfaced when requested");
  assert.match(String(map), /"version":\s*3/, "the source map is v3");
});

void test("compileJsxModule omits the source map when sourceMap is off", () => {
  const { map } = compileJsxModule("/src/App.tsx", SOURCE, { jsxMode: "vdom", sourceMap: false });
  assert.equal(map, null, "no source map unless requested");
});

void test("compileJsxModule has no source map for Vapor output", () => {
  // The Vapor backend does not emit a source map yet, so even with `sourceMap`
  // requested the result carries none (#1533).
  const { map } = compileJsxModule("/src/App.tsx", SOURCE, { vapor: true, sourceMap: true });
  assert.equal(map, null, "Vapor output reports no source map");
});
