import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { isDiagnosticsForUri, offsetToPosition } from "./support/lsp/assertions.ts";
import { testOutputRoot } from "./support/lsp/paths.ts";
import type { PublishDiagnosticsParams } from "./support/lsp/protocol.ts";
import { LspSession } from "./support/lsp/session.ts";

type Range = {
  start: { line: number; character: number };
  end: { line: number; character: number };
};
type Location = { uri: string; range: Range };

const art = (isolate: string) => `<script setup lang="ts"${isolate} generic="T extends string">
const props = defineProps<{ value: T }>();
const local: T = props.value;
const missing = qzxUndeclared;
</script>
<art title="Generic"><variant name="Default"><div>{{ local }}</div></variant></art>
`;

function at(text: string, needle: string, skip: number, length: number): Range {
  const start = text.indexOf(needle);
  assert.notEqual(start, -1, needle);
  return {
    start: offsetToPosition(text, start + skip),
    end: offsetToPosition(text, start + skip + length),
  };
}

const locations = (response: unknown): Location[] =>
  response == null ? [] : Array.isArray(response) ? response : [response as Location];

test("art script setup type parameters resolve and navigate from the generic attribute", async () => {
  fs.mkdirSync(testOutputRoot, { recursive: true });
  const workspace = fs.mkdtempSync(path.join(testOutputRoot, "art-generic-attribute-"));
  const session = new LspSession();
  try {
    fs.writeFileSync(
      path.join(workspace, "tsconfig.json"),
      JSON.stringify({
        compilerOptions: { strict: true, target: "ESNext", module: "ESNext", noEmit: true },
        include: ["./**/*"],
      }),
    );
    await session.initialize(workspace, { editor: true, lint: false, typecheck: true });
    for (const [index, isolate] of ["", ' isolate="false"'].entries()) {
      const source = art(isolate);
      const file = path.join(workspace, `Generic${index}.art.vue`);
      fs.writeFileSync(file, source);
      const uri = pathToFileURL(file).href;
      session.notify("textDocument/didOpen", {
        textDocument: { uri, languageId: "art-vue", version: 1, text: source },
      });
      const published = (await session.waitForNotification(
        "textDocument/publishDiagnostics",
        (params) => isDiagnosticsForUri(params, uri),
        60_000,
      )) as PublishDiagnosticsParams;
      // Only the genuinely undefined name is unresolved.
      assert.deepEqual(
        published.diagnostics.map(({ code, range }) => ({ code, range })),
        [{ code: 2304, range: at(source, "qzxUndeclared", 0, "qzxUndeclared".length) }],
        `diagnostics${isolate}`,
      );

      const request = (method: string, range: Range) =>
        session.request(method, { textDocument: { uri }, position: range.start });
      const declaration = at(source, 'generic="T', 9, 1);
      const usages = [at(source, "value: T", 7, 1), at(source, "local: T", 7, 1)];
      const usageHover = (await request("textDocument/hover", usages[0])) as {
        contents: { value?: string };
      } | null;
      assert.ok(usageHover, `usage hover${isolate}`);
      assert.match(usageHover.contents.value ?? "", /\(type parameter\) T extends string/);
      assert.deepEqual(await request("textDocument/hover", declaration), {
        contents: usageHover.contents,
        range: declaration,
      });
      for (const query of [declaration, ...usages]) {
        assert.deepEqual(
          locations(await request("textDocument/definition", query)),
          [{ uri, range: declaration }],
          `definition${isolate}`,
        );
      }
      // The attribute name is not TypeScript.
      assert.equal(await request("textDocument/hover", at(source, 'generic="', 1, 1)), null);
    }
  } finally {
    await session.shutdown();
    fs.rmSync(workspace, { recursive: true, force: true });
  }
});
