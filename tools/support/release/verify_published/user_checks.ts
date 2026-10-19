// What a user does first with a freshly installed `vize`: compile, lint, format
// and open the project in an editor (LSP). Each check runs the published bits.
import { spawn } from "node:child_process";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { pathToFileURL } from "node:url";
import { collectOutput, run } from "../command_io.ts";

const LIST = `<script setup lang="ts">
const items = [1, 2]
</script>

<template>
  <ul><li v-for="item in items">{{ item }}</li></ul>
</template>
`;
const UNFORMATTED = `<template><div   class="a">x</div></template>\n`;
const COMPILE = "<script setup>const n = 1</script><template><p>{{ n }}</p></template>";
const MISSING_KEY = "vue/require-v-for-key";

type Check = { name: string; run: () => Promise<string | null> };

/** Runs the installed CLI; returns null on success or a failure description. */
function cli(dir: string, args: string[]) {
  return collectOutput(process.execPath, [join(dir, "node_modules/vize/bin/vize"), ...args], {
    cwd: dir,
  });
}

/** One LSP session: initialize, open a file, wait for its diagnostics. */
function lspDiagnostics(dir: string, version: string): Promise<string | null> {
  return new Promise((resolve) => {
    const server = spawn(process.execPath, [join(dir, "node_modules/vize/bin/vize"), "lsp"], {
      cwd: dir,
      stdio: ["pipe", "pipe", "inherit"],
    });
    const done = (result: string | null) => {
      clearTimeout(timer);
      server.kill();
      resolve(result);
    };
    const timer = setTimeout(() => done("no diagnostics within 60s"), 60_000);
    const send = (message: object) => {
      const body = JSON.stringify({ jsonrpc: "2.0", ...message });
      server.stdin.write(`Content-Length: ${Buffer.byteLength(body)}\r\n\r\n${body}`);
    };
    let buffer = Buffer.alloc(0);
    server.stdout.on("data", (chunk: Buffer) => {
      buffer = Buffer.concat([buffer, chunk]);
      for (;;) {
        const header = buffer.indexOf("\r\n\r\n");
        if (header < 0) return;
        const length = Number(
          /Content-Length: (\d+)/i.exec(buffer.subarray(0, header).toString())?.[1],
        );
        if (buffer.length < header + 4 + length) return;
        const message = JSON.parse(buffer.subarray(header + 4, header + 4 + length).toString());
        buffer = buffer.subarray(header + 4 + length);
        if (message.id === 1) {
          if (message.result?.serverInfo?.version !== version)
            return done(`serverInfo.version is ${message.result?.serverInfo?.version}`);
          send({ method: "initialized", params: {} });
          const uri = pathToFileURL(join(dir, "src/App.vue")).href;
          send({
            method: "textDocument/didOpen",
            params: { textDocument: { uri, languageId: "vue", version: 1, text: LIST } },
          });
        }
        if (
          message.method === "textDocument/publishDiagnostics" &&
          JSON.stringify(message.params?.diagnostics ?? []).includes(MISSING_KEY)
        )
          return done(null);
      }
    });
    server.on("exit", (code) => done(`vize lsp exited with ${code}`));
    send({
      id: 1,
      method: "initialize",
      params: { processId: process.pid, rootUri: pathToFileURL(dir).href, capabilities: {} },
    });
  });
}

export function userChecks(dir: string, version: string): Check[] {
  return [
    {
      name: "vize --version",
      run: async () => {
        const out = await cli(dir, ["--version"]);
        return out.stdout.trim() === `vize ${version}` ? null : `printed ${out.stdout.trim()}`;
      },
    },
    {
      name: "compileSfc from @vizejs/native",
      run: async () => {
        const script = `const { compileSfc } = require(require.resolve("@vizejs/native", { paths: [${JSON.stringify(dir)}] }));
          if (!JSON.stringify(compileSfc(${JSON.stringify(COMPILE)}, { filename: "Smoke.vue" })).includes("createElementBlock")) process.exit(1);`;
        return (await run(process.execPath, ["-e", script])) === 0 ? null : "unexpected output";
      },
    },
    {
      name: "vize lint",
      run: async () => {
        const out = await cli(dir, ["lint", "src/App.vue", "--format", "json"]);
        return out.stdout.includes(MISSING_KEY) ? null : `missing ${MISSING_KEY}: ${out.stderr}`;
      },
    },
    {
      name: "vize fmt",
      run: async () => {
        const path = join(dir, "src/Fmt.vue");
        writeFileSync(path, UNFORMATTED);
        if ((await cli(dir, ["fmt", "--check", "src/Fmt.vue"])).status === 0)
          return "--check accepted unformatted input";
        if ((await cli(dir, ["fmt", "--write", "src/Fmt.vue"])).status !== 0) return "fmt failed";
        return readFileSync(path, "utf8") === UNFORMATTED ? "fmt left the file unchanged" : null;
      },
    },
    { name: "vize lsp diagnostics", run: () => lspDiagnostics(dir, version) },
  ];
}

export function writeProject(dir: string): void {
  mkdirSync(join(dir, "src"), { recursive: true });
  writeFileSync(join(dir, "src/App.vue"), LIST);
}
