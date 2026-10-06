## Summary

`vize lint --fix` rewrites `v-bind:title="title"` → `:title="title"` → `:title`. In the editor, `textDocument/codeAction` for the same diagnostics returns `null`: no quick fix, and not the "Suppress with @vize:forget" action that the server offers for other rules (`vue/no-v-html` in the same file gets it).

So in the editor these warnings can only be fixed by hand or by running the CLI. Related: #6876 (unify diagnostic fixes, code actions and lint autofixes).

## Environment

- vize 0.432.0 (npm)
- macOS 26.4.1 arm64, Node v26.10.0, npm 11.19.1
- typescript 7.0.2, vue 3.5.43

## Reproduction

```sh
mkdir repro && cd repro
npm i -D vize@0.432.0 typescript@7.0.2 vue@3.5.43
printf '{\n  "compilerOptions": { "module": "ESNext", "moduleResolution": "Bundler", "strict": true, "noEmit": true }\n}\n' > tsconfig.json
echo '{ "linter": { "rules": { "vue/v-bind-style": "warn", "vue/prefer-props-shorthand": "warn" } } }' > vize.config.json
cat > Child.vue <<'EOF'
<script setup lang="ts">
const { title } = defineProps<{ title: string }>();
</script>

<template>
  <p>{{ title }}</p>
</template>
EOF
cat > Parent.vue <<'EOF'
<script setup lang="ts">
import Child from "./Child.vue";

const title = "Hello";
const html = "<b>Hello</b>";
</script>

<template>
  <Child v-bind:title="title" />
  <Child :title="title" />
  <div v-html="html" />
</template>
EOF
# one textDocument/codeAction request per published diagnostic of Parent.vue (range and context.diagnostics = that diagnostic)
node lsp-req.mjs . Parent.vue textDocument/codeAction
cp Parent.vue Parent.vue.orig && npx vize lint --fix Parent.vue >/dev/null; npx vize lint --fix Parent.vue >/dev/null; diff Parent.vue.orig Parent.vue
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

```
8:9 vue/prefer-props-shorthand: null
8:9 vue/v-bind-style: null
9:9 vue/prefer-props-shorthand: null
10:7 vue/no-v-html: ["Suppress with @vize:forget (vue/no-v-html)"]
```

`vize lint --fix` (run twice, see #7906):

```diff
<   <Child v-bind:title="title" />
<   <Child :title="title" />
---
>   <Child :title />
>   <Child :title />
```

## Expected

A `quickfix` code action with the same edit as `--fix` for each fixable diagnostic (`:title="title"`, then `:title`), plus the suppress action offered for the other rules.
