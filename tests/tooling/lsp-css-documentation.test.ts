import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import {
  cssSession,
  observeCss,
  completionItems,
  documentation,
} from "./support/lsp/css-documentation.ts";
import { root } from "./support/lsp/paths.ts";

await test("ordinary CSS completion/resolve/hover uses static docs without typechecking", async () => {
  const binary = path.join(root, "target/ci", process.platform === "win32" ? "vize.exe" : "vize");
  const observations = await observeCss(binary, true, true);
  const output = path.join(root, "target/differential/css-documentation-observations.json");
  fs.mkdirSync(path.dirname(output), { recursive: true });
  fs.writeFileSync(
    output,
    `${JSON.stringify({ binary, scope: "actual stdio latency observations; no instruction/allocation or speedup claim", observations }, null, 2)}\n`,
  );
  const client = await cssSession(binary, true, false);
  try {
    const source = "<template><div /></template>\n<style>.demo { col }</style>";
    client.open(source, 1);
    const result = await client.session.request("textDocument/completion", {
      textDocument: { uri: client.uri },
      position: { line: 1, character: source.split("\n")[1]!.indexOf("col") + 3 },
    });
    const item = completionItems(result).find((item) => item.label === "color");
    assert.ok(item);
    assert.equal(item.data, undefined);
    const text = documentation(item.documentation);
    assert.ok(text.includes("**color**"));
    assert.ok(text.includes("<color>"));
    assert.ok(text.includes("developer.mozilla.org"));
  } finally {
    await client.close();
  }
});
