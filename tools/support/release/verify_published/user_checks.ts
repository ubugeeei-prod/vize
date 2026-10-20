// What a user does first with a freshly installed `vize`: compile, lint, format
// and open the project in an editor (LSP). Each check runs the published bits.
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { collectOutput, run } from "../command_io.ts";
import { watchDiagnostics } from "./lsp_client.ts";

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
    {
      name: "vize lsp diagnostics",
      run: () =>
        watchDiagnostics({
          bin: join(dir, "node_modules/vize/bin/vize"),
          root: dir,
          files: [join(dir, "src/App.vue")],
          version,
          settle: (latest) =>
            JSON.stringify([...latest.values()]).includes(MISSING_KEY) ? null : undefined,
        }),
    },
  ];
}

export function writeProject(dir: string): void {
  mkdirSync(join(dir, "src"), { recursive: true });
  writeFileSync(join(dir, "src/App.vue"), LIST);
}
