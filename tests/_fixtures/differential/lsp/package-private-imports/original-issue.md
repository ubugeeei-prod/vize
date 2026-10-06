## Summary

#7031 is fixed for `vize check`: a subpath import from `package.json` `"imports"` that maps to a `.ts` file now type-checks clean. In `vize lsp` on the same project, both the `.vue` and the `.ts` importer still get `TS2307 Cannot find module '#lib/util.ts'`. Everything imported through it is then `any` in the editor, which hides real type errors.

Adding the same mapping to tsconfig `paths` hides it, as it did in #7031.

## Environment

- vize 0.432.0 (npm)
- macOS 26.4.1 arm64, Node v26.10.0, npm 11.19.1
- typescript 7.0.2, vue 3.5.43

## Reproduction

```sh
mkdir repro && cd repro
cat > package.json <<'EOF'
{ "private": true, "type": "module", "imports": { "#lib/*": "./src/lib/*" } }
EOF
npm i -D vize@0.432.0 typescript@7.0.2 vue@3.5.43
mkdir -p src/lib
cat > tsconfig.json <<'EOF'
{
  "compilerOptions": { "module": "ESNext", "moduleResolution": "Bundler", "strict": true, "noEmit": true, "allowImportingTsExtensions": true, "types": [] },
  "include": ["src/**/*.ts", "src/**/*.vue"]
}
EOF
echo 'export const greet = (name: string): string => `Hello, ${name}`;' > src/lib/util.ts
printf '<script setup lang="ts">\nimport { greet } from "#lib/util.ts";\n\nconst message = greet("x");\n</script>\n\n<template>\n  <p>{{ message }}</p>\n</template>\n' > src/App.vue
printf 'import { greet } from "#lib/util.ts";\n\nexport const message: string = greet("y");\n' > src/main.ts

npx vize check
node lsp-diag.mjs . src/App.vue src/main.ts   # script below: didOpen both files, print publishDiagnostics
```

<details><summary>lsp-diag.mjs (minimal stdio client)</summary>

```js
// node lsp-diag.mjs <workspace-root> <file>... : opens the files in `vize lsp`, waits 20 s, prints the last diagnostics.
import { spawn } from "node:child_process";
import { readFileSync } from "node:fs";
import { relative, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const [root, ...files] = process.argv.slice(2).map((p) => resolve(p));
const server = spawn(resolve("node_modules/.bin/vize"), ["lsp", "--stdio"], { cwd: root, stdio: ["pipe", "pipe", "ignore"] });
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
const request = (method, params) => new Promise((done) => { pending.set(++nextId, done); write({ id: nextId, method, params }); });
const notify = (method, params) => write({ method, params });

await request("initialize", { processId: process.pid, rootUri: pathToFileURL(root).href, capabilities: {} });
notify("initialized", {});
for (const file of files) {
  const languageId = file.endsWith(".vue") ? "vue" : "typescript";
  notify("textDocument/didOpen", { textDocument: { uri: pathToFileURL(file).href, languageId, version: 1, text: readFileSync(file, "utf8") } });
}
await new Promise((r) => setTimeout(r, 20000));
for (const [uri, list] of diagnostics) {
  for (const d of list) console.log(`${relative(root, fileURLToPath(uri))}:${d.range.start.line + 1}:${d.range.start.character + 1} [${d.source} ${d.code ?? ""}] ${d.message.split("\n")[0]}`);
}
await request("shutdown", null);
notify("exit", null);
```

</details>

## Actual

`vize check`:

```
✓ Type checked 3 files
  No type errors found!
```

`vize lsp`:

```
src/App.vue:2:23 [vize/types 2307] Cannot find module '#lib/util.ts' or its corresponding type declarations.
src/main.ts:1:23 [vize/types 2307] Cannot find module '#lib/util.ts' or its corresponding type declarations.
```

Go to definition on `"#lib/util.ts"` does open `src/lib/util.ts`. Only the type diagnostics fail to resolve it.

## Expected

The editor resolves `#lib/*` through `package.json` `imports` as `vize check` (and `tsc` / vue-tsc) do, with no TS2307.
