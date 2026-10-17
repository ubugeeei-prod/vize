import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import vm from "node:vm";

test("rich host custody preserves the whole completion API result before Markdown assertions", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-rich-custody-"));
  try {
    const source = fs.readFileSync(
      new URL("../../editors/vscode/test/suite/real-rich-hover-custody.cjs", import.meta.url),
      "utf8",
    );
    const module = { exports: {} as Record<string, (...args: unknown[]) => unknown> };
    const require = createRequire(import.meta.url);
    const retained = path.join(root, "retained-packets.json");
    vm.runInNewContext(source, {
      module,
      process: { env: { ...process.env, VIZE_RICH_CUSTODY_OUTPUT: retained } },
      require: (name: string) =>
        name === "./real-server-support.cjs"
          ? { getWorkspaceFolderPath: () => root }
          : require(name),
    });
    const authored = "<template>{{ invoice. }}</template>";
    const request = module.exports.beginCompletion(
      {
        getText: () => authored,
        uri: { toString: () => "file:///project/App.vue" },
        version: 7,
        languageId: "vue",
      },
      { line: 0, character: 23 },
      { expression: "invoice.total", label: "total" },
      16,
    );
    // The actual VSCode MarkdownString uses non-enumerable public getters.
    // Preserve that boundary: serializing the instance itself yields {}.
    class MarkdownString {
      get value() {
        return "**JPY**";
      }
      get isTrusted() {
        return false;
      }
      get supportThemeIcons() {
        return true;
      }
      get supportHtml() {
        return false;
      }
      get baseUri() {
        return new URL("file:///project/docs/");
      }
    }
    const actual = {
      isIncomplete: false,
      items: [
        { label: "total", detail: "total: number", data: { original: [1, "opaque"] } },
        { label: "formatTotal", documentation: new MarkdownString() },
      ],
    };
    module.exports.retainCompletion(request, actual);
    const result = JSON.parse(
      fs.readFileSync(path.join(root, "node_modules/.vize/rich-hover.json"), "utf8"),
    );
    assert.deepEqual(JSON.parse(fs.readFileSync(retained, "utf8")), result);
    assert.equal(result.packets.length, 1);
    const packet = result.packets[0];
    assert.deepEqual(packet.actual, JSON.parse(JSON.stringify(actual)));
    assert.equal(packet.source, authored);
    assert.equal(packet.sourceSha256, createHash("sha256").update(authored).digest("hex"));
    assert.deepEqual(packet.document, {
      uri: "file:///project/App.vue",
      version: 7,
      languageId: "vue",
    });
    assert.deepEqual(packet.request, {
      method: "vscode.executeCompletionItemProvider",
      position: { line: 0, character: 23 },
      itemResolveCount: 16,
    });
    const expectedDocumentation: readonly Record<string, unknown>[] = [
      { label: "total" },
      {
        label: "formatTotal",
        raw: {},
        constructor: "MarkdownString",
        value: "**JPY**",
        isTrusted: false,
        supportThemeIcons: true,
        supportHtml: false,
        baseUri: "file:///project/docs/",
      },
    ];
    assert.deepEqual(packet.publicDocumentation, expectedDocumentation);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});
