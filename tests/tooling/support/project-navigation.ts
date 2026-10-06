import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { LspWire } from "../../differential/lsp-wire.ts";
import { initialize, observation, open, save, uri } from "./editor-jsconfig.ts";

export const carriers = ["App.vue", "Banner.vue", "notify.ts", "useToast.ts"];
const positions: Record<string, [number, number, boolean][]> = {
  "App.vue": [
    [1, 9, true],
    [4, 17, false],
  ],
  "Banner.vue": [
    [1, 9, true],
    [3, 17, false],
  ],
  "notify.ts": [
    [0, 9, true],
    [2, 24, false],
  ],
  "useToast.ts": [[0, 16, true]],
};
export function locations(directory: string, declaration: boolean, stock = false, dirty = false) {
  return Object.entries(positions)
    .flatMap(([file, occurrences]) =>
      occurrences
        .filter(([, , binding]) => declaration || !binding)
        .map(([line, character]) => {
          if (dirty && file === "notify.ts" && line === 2) character += 6;
          return {
            uri: uri(path.join(directory, "src", stock ? file.replace(/\.vue$/, ".ts") : file)),
            range: { start: { line, character }, end: { line, character: character + 8 } },
          };
        }),
    )
    .sort(order);
}
export function order(a: any, b: any) {
  return (
    a.uri.localeCompare(b.uri) ||
    a.range.start.line - b.range.start.line ||
    a.range.start.character - b.range.start.character ||
    a.range.end.line - b.range.end.line ||
    a.range.end.character - b.range.end.character
  );
}
export async function references(
  wire: LspWire,
  directory: string,
  declaration: boolean,
  stock = false,
  dirty = false,
) {
  const reply = await wire.request("textDocument/references", {
    textDocument: { uri: uri(path.join(directory, "src", stock ? "App.ts" : "App.vue")) },
    position: { line: 4, character: 19 },
    context: { includeDeclaration: declaration },
  });
  const expected = locations(directory, declaration, stock, dirty);
  save(`${stock ? "stock" : "vize"}-references-${wire.child.pid}-${wire.nextId}`, {
    expected,
    reply,
    wire: observation(wire),
  });
  assert.equal(reply.error, undefined);
  assert(Array.isArray(reply.result));
  // Keep the complete native reply before sorting: the canonical public service
  // orders all locations; native reference groups may have a different order.
  assert.deepEqual([...reply.result].sort(order), expected);
  if (!stock) assert.deepEqual(reply.result, expected);
}
export async function stock(directory: string, inputs: Record<string, string>, runtime: string) {
  const projected: Record<string, string> = {
    "tsconfig.json": inputs["tsconfig.json"],
    "src/useToast.ts": inputs["src/useToast.ts"],
    "src/notify.ts": inputs["src/notify.ts"],
    // A separate stock project keeps the exact authored script text/line spans.
    // The ambient Vue import has no bearing on the useToast symbol identity.
    "src/vue.d.ts":
      'declare module "*.vue" { const component: unknown; export default component; }\n',
  };
  for (const file of ["App.vue", "Banner.vue"]) {
    const source = inputs["src/" + file];
    projected["src/" + file.replace(".vue", ".ts")] = source.slice(
      source.indexOf(">") + 1,
      source.indexOf("</script>"),
    );
  }
  for (const [file, source] of Object.entries(projected)) {
    fs.mkdirSync(path.dirname(path.join(directory, file)), { recursive: true });
    fs.writeFileSync(path.join(directory, file), source);
  }
  save("stock-inputs", { original: inputs, projected });
  const wire = new LspWire(runtime, ["--lsp", "--stdio"], directory);
  try {
    await initialize(wire, directory, true);
    for (const [file, source] of Object.entries(projected))
      if (file.startsWith("src/") && !file.endsWith(".d.ts"))
        open(wire, path.join(directory, file), source, "typescript");
    for (const file of ["App.ts", "Banner.ts", "notify.ts", "useToast.ts"]) {
      const reply = await wire.request("textDocument/diagnostic", {
        textDocument: { uri: uri(path.join(directory, "src", file)) },
      });
      const spans =
        file === "App.ts"
          ? [
              [2, 7, 13, "Banner"],
              [4, 8, 12, "show"],
            ]
          : file === "Banner.ts"
            ? [[3, 8, 12, "show"]]
            : [];
      const expected = {
        jsonrpc: "2.0",
        id: wire.nextId,
        result: {
          kind: "full",
          items: spans.map(([line, start, end, name]) => ({
            code: 6133,
            severity: 4,
            source: "ts",
            message: `'${name}' is declared but its value is never read.`,
            range: { start: { line, character: start }, end: { line, character: end } },
          })),
        },
      };
      save("stock-diagnostics-" + file, { reply, expected, projected, wire: observation(wire) });
      assert.deepEqual(reply, expected);
    }
    await references(wire, directory, true, true);
    await references(wire, directory, false, true);
    const reply = await wire.request("shutdown");
    assert.deepEqual(reply, { jsonrpc: "2.0", id: wire.nextId, result: null });
    wire.child.stdin.end();
    await wire.stop(false);
    assert.equal(wire.exitStatus, 0);
    assert.equal(wire.signal, null);
    assert.equal(wire.processError, null);
    for (const message of wire.messages.filter(
      (message) => message.method === "textDocument/publishDiagnostics",
    ))
      assert.deepEqual(message, {
        jsonrpc: "2.0",
        method: "textDocument/publishDiagnostics",
        params: { uri: uri(path.join(directory, "tsconfig.json")), diagnostics: [] },
      });
  } finally {
    await wire.stop();
    save("stock-terminal", { projected, wire: observation(wire) });
  }
}
