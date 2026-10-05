## Summary

Hovering the name of a bound attribute on a native element (`:for="…"`, `:id="…"`) shows the right DOM property, but `Hover.range` is not mapped back to the SFC. It is `line 5, character 17` (zero-based) in every file I tried, whatever line the attribute is on, and its length is that of the DOM property in the generated code: 7 for `:for` (`htmlFor`), 2 for `:id`.

Editors use the range to mark the hovered word and to decide when to hide the hover, so the hover is attached to an unrelated span of the file. Hovering the attribute **value** (`"id"`) returns a correct range. #3326 (closed) added `Hover.range`; this case seems to have been left out.

## Environment

- vize 0.432.0 (npm)
- macOS 26.4.1 arm64, Node v26.10.0, npm 11.19.1
- typescript 7.0.2, vue 3.5.43

## Reproduction

```sh
mkdir repro && cd repro
npm i -D vize@0.432.0 typescript@7.0.2 vue@3.5.43
printf '{\n  "compilerOptions": { "module": "ESNext", "moduleResolution": "Bundler", "strict": true, "noEmit": true }\n}\n' > tsconfig.json
cat > Field.vue <<'EOF'
<script setup lang="ts">
const id = "field";
</script>

<template>
  <label :for="id">Name</label>
  <input :id="id" />
</template>
EOF
# zero-based positions: the `for` of :for, the `id` of :id, the value "id" of :id
node lsp-req.mjs . Field.vue textDocument/hover 5:10 6:10 6:14
```

<details><summary>lsp-req.mjs (minimal stdio client)</summary>

```js
// node lsp-req.mjs <folder> <file> <method> [<line>:<character>...] [newName]
// Starts `vize lsp` on <folder>, opens every .vue file in it, waits 10 s, then sends <method> for <file> at each
// zero-based position and prints the result. textDocument/rename takes the new name as the last argument;
// textDocument/codeAction is sent once per published diagnostic of <file>; completion prints the item labels.
import { spawn } from "node:child_process";
import { readFileSync, readdirSync } from "node:fs";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";

const [folderArg, fileArg, method, ...rest] = process.argv.slice(2);
const newName = method === "textDocument/rename" ? rest.pop() : undefined;
const folder = resolve(folderArg);
const server = spawn(resolve("node_modules/.bin/vize"), ["lsp", "--stdio"], { cwd: folder, stdio: ["pipe", "pipe", "ignore"] });
const write = (msg) => {
  const body = JSON.stringify({ jsonrpc: "2.0", ...msg });
  server.stdin.write(`Content-Length: ${Buffer.byteLength(body)}\r\n\r\n${body}`);
};
const pending = new Map();
const diagnostics = new Map();
let nextId = 0;
let buffer = Buffer.alloc(0);
server.stdout.on("data", (chunk) => {
  buffer = Buffer.concat([buffer, chunk]);
  for (let end; (end = buffer.indexOf("\r\n\r\n")) >= 0; ) {
    const length = Number(/Content-Length: (\d+)/i.exec(buffer.subarray(0, end))[1]);
    if (buffer.length < end + 4 + length) break;
    const msg = JSON.parse(buffer.subarray(end + 4, end + 4 + length));
    buffer = buffer.subarray(end + 4 + length);
    if (msg.method && msg.id !== undefined) write({ id: msg.id, result: null });
    else if (msg.id !== undefined) pending.get(msg.id)?.(msg);
    else if (msg.method === "textDocument/publishDiagnostics") diagnostics.set(msg.params.uri, msg.params.diagnostics);
  }
});
const request = (m, params) => new Promise((done) => { pending.set(++nextId, done); write({ id: nextId, method: m, params }); });
const notify = (m, params) => write({ method: m, params });

await request("initialize", { processId: process.pid, rootUri: pathToFileURL(folder).href, capabilities: {} });
notify("initialized", {});
for (const name of readdirSync(folder).filter((n) => n.endsWith(".vue"))) {
  const path = resolve(folder, name);
  notify("textDocument/didOpen", { textDocument: { uri: pathToFileURL(path).href, languageId: "vue", version: 1, text: readFileSync(path, "utf8") } });
}
await new Promise((r) => setTimeout(r, 10000));
const uri = pathToFileURL(resolve(folder, fileArg)).href;
const show = (label, result) => {
  const items = Array.isArray(result) ? result : result?.items;
  const text = method === "textDocument/completion" && items ? items.map((i) => i.label).join(", ") : JSON.stringify(result ?? null);
  console.log(`${label}: ${text.replaceAll(pathToFileURL(folder).href + "/", "")}`);
};
if (method === "textDocument/codeAction") {
  for (const d of diagnostics.get(uri) ?? []) {
    const { result } = await request(method, { textDocument: { uri }, range: d.range, context: { diagnostics: [d] } });
    show(`${d.range.start.line}:${d.range.start.character} ${d.code}`, result?.map((a) => a.title));
  }
} else {
  for (const p of rest) {
    const [line, character] = p.split(":").map(Number);
    const context = method === "textDocument/references" ? { includeDeclaration: true } : undefined;
    const params = { textDocument: { uri }, position: { line, character }, context, newName };
    const response = await request(method, params);
    show(`${method} @ ${p}`, response.error ? { error: response.error } : (response.result ?? null));
  }
}
await request("shutdown", null);
notify("exit", null);
```

</details>

## Actual

| hovered | contents | range |
| --- | --- | --- |
| `for` (5:10) | `(property) HTMLLabelElement.htmlFor: string` | 5:17-5:24 (`">Name<`) |
| `id` (6:10) | `(property) Element.id: string` | 5:17-5:19, on the line above |
| value `id` (6:14) | `const id: "field"` | 6:14-6:16 (correct) |

## Expected

The range covers the hovered attribute name: 5:10-5:13 for `for`, 6:10-6:12 for `id`.

