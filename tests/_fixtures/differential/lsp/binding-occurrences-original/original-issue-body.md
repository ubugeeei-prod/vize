## Summary

`textDocument/documentHighlight` on a `<script setup>` binding highlights every template token that spells the same word, not just the uses of that binding:

- the tag names `<label>` / `</label>` for a binding called `label`
- the attribute **names** of `:id="…"` for a binding called `id`
- the word inside string text, e.g. `hint` in `` `${id}-hint` ``, for a binding called `hint`

`textDocument/references` on the same positions is correct, so the editor highlights tokens that "Find references" does not list.

The code lens titles have the same miscount (`label` → "3 template/style references", `id` → "5", `hint` → "2"; the right counts are 1, 3 and 1). That count also disagreed with references in #6980 (closed).

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
const label = "Name";
const hint = "Shown below the field";
const id = "field";
</script>

<template>
  <label :for="id">{{ label }}</label>
  <input :id="id" />
  <p :id="`${id}-hint`">{{ hint }}</p>
</template>
EOF
# positions are zero-based: the declarations of label, hint and id
node lsp-req.mjs . Field.vue textDocument/documentHighlight 1:6 2:6 3:6
node lsp-req.mjs . Field.vue textDocument/references 1:6 2:6 3:6
node lsp-req.mjs . Field.vue textDocument/codeLens 0:0
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

documentHighlight (zero-based `line:start-end`):

| binding | highlighted | of which wrong |
| --- | --- | --- |
| `label` | 1:6-11, 7:3-8, 7:22-27, 7:32-37 | 7:3-8 `<label`, 7:32-37 `</label>` |
| `hint` | 2:6-10, 9:17-21, 9:27-31 | 9:17-21, the `hint` inside `` `${id}-hint` `` |
| `id` | 3:6-8, 7:15-17, 8:10-12, 8:14-16, 9:6-8, 9:13-15 | 8:10-12 and 9:6-8, the attribute names of `:id="…"` |

references (correct):

| binding | locations |
| --- | --- |
| `label` | 1:6-11, 7:22-27 |
| `hint` | 2:6-10, 9:27-31 |
| `id` | 3:6-8, 7:15-17, 8:14-16, 9:13-15 |

codeLens: `3 template/style references` on line 1, `2 …` on line 2, `5 …` on line 3.

## Expected

documentHighlight and the code lens count cover the same locations as references (1, 1 and 3 template uses), and no tag names, attribute names or string text.
