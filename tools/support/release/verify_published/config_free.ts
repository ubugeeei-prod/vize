// Post-publish check that the published packages need no `vize.config.*`: a
// fresh project, a migration off a retired dedicated config and an npm
// workspace monorepo all read `vue:` from vite.config.ts and tsconfig.json.
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { collectOutput, println, run, runMain, writeStderr } from "../command_io.ts";
import { cliArgs } from "../command_runtime.ts";
import { uriOf, watchDiagnostics, type Diagnostics } from "./lsp_client.ts";

// Violates vue/require-v-for-key and vue/no-v-html, both on by default.
const APP = `<script setup lang="ts">
const items = ["<b>1</b>"]
</script>

<template>
  <ul><li v-for="item in items" v-html="item" /></ul>
</template>
`;
// TS7006 only when tsconfig.json enables noImplicitAny (strict).
const IMPLICIT = `<script setup lang="ts">
function echo(value) {
  return value
}
echo(1)
</script>
`;
const QUOTES = `<script setup lang="ts">
const label = "x"
</script>

<template>
  <p>{{ label }}</p>
</template>
`;
const KEY = "vue/require-v-for-key";
const HTML = "vue/no-v-html";
const npm = process.platform === "win32" ? "npm.cmd" : "npm";

type Step = [name: string, run: () => Promise<string | null>];

function write(path: string, text: string): void {
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, text);
}

function viteConfig(vue: object | null, plugin = false): string {
  const imports = ['import { defineConfig } from "vite";'];
  if (plugin) imports.push('import vize from "@vizejs/vite-plugin";');
  const fields = [];
  if (plugin) fields.push("plugins: [vize()]");
  if (vue) fields.push(`vue: ${JSON.stringify(vue)}`);
  return `${imports.join("\n")}\n\nexport default defineConfig({ ${fields.join(", ")} });\n`;
}

function tsconfig(strict: boolean): string {
  const compilerOptions = { strict, noEmit: true, module: "ESNext", moduleResolution: "Bundler" };
  return `${JSON.stringify({ compilerOptions, include: ["src/**/*"] }, null, 2)}\n`;
}

function retired(dir: string): string[] {
  return readdirSync(dir).filter((name) => name.startsWith("vize.config."));
}

async function install(dir: string, packages: string[]): Promise<string | null> {
  const args = ["install", "--no-audit", "--no-fund", ...packages];
  return (await run(npm, args, { cwd: dir })) === 0 ? null : "npm install failed";
}

function vize(bin: string, cwd: string, args: string[]) {
  return collectOutput(process.execPath, [bin, ...args], { cwd });
}

/** Lints src/App.vue from `cwd`; fails unless exactly `expected` rules report. */
async function lintRules(bin: string, cwd: string, expected: string[]): Promise<string | null> {
  const out = await vize(bin, cwd, ["lint", "src/App.vue", "--format", "json"]);
  let rules: string[];
  try {
    rules = JSON.parse(out.stdout).flatMap((file: { messages: { ruleId: string }[] }) =>
      file.messages.map((message) => message.ruleId),
    );
  } catch {
    return `lint printed no report (status ${out.status}): ${out.stderr}`;
  }
  const actual = [...new Set(rules)].sort();
  return JSON.stringify(actual) === JSON.stringify([...expected].sort())
    ? null
    : `expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`;
}

async function implicitAny(bin: string, cwd: string, expected: boolean): Promise<string | null> {
  const out = await vize(bin, cwd, ["check"]);
  const reported = `${out.stdout}${out.stderr}`.includes("TS7006");
  return reported === expected ? null : `TS7006 ${reported ? "reported" : "missing"}`;
}

/** Whether every file's latest diagnostics include `present` and exclude `absent`. */
function rulesSettled(latest: Diagnostics, expected: Map<string, [string, string]>) {
  for (const [uri, [present, absent]] of expected) {
    const text = JSON.stringify(latest.get(uri) ?? []);
    if (!text.includes(present)) return undefined;
    if (text.includes(absent)) return `${uri} still reports ${absent}`;
  }
  return null;
}

function freshProject(dir: string, version: string): Step[] {
  const bin = join(dir, "node_modules/vize/bin/vize");
  const packages = [`vize@${version}`, `@vizejs/vite-plugin@${version}`, "vite", "vue"];
  const vue = { linter: { rules: { [KEY]: "off" } }, formatter: { singleQuote: true } };
  return [
    ["install", () => install(dir, packages)],
    ["defaults without any config", () => lintRules(bin, dir, [KEY, HTML])],
    [
      "vize init writes no dedicated config",
      async () => {
        write(join(dir, "vite.config.ts"), viteConfig(vue));
        const out = await vize(bin, dir, [
          "init",
          "-y",
          "--no-install",
          "--no-editor",
          "--no-lint",
        ]);
        if (out.status !== 0) return `init exited ${out.status}: ${out.stderr}`;
        const config = readFileSync(join(dir, "vite.config.ts"), "utf8");
        if (!config.includes("vize()") || !config.includes(KEY))
          return `vite.config.ts lost vize() or vue: settings:\n${config}`;
        if (!existsSync(join(dir, "tsconfig.json"))) return "no tsconfig.json";
        return retired(dir).length === 0 ? null : `created ${retired(dir).join(", ")}`;
      },
    ],
    ["lint reads vue.linter", () => lintRules(bin, dir, [HTML])],
    [
      "fmt reads vue.formatter",
      async () => {
        write(join(dir, "src/Quotes.vue"), QUOTES);
        const out = await vize(bin, dir, ["fmt", "--write", "src/Quotes.vue"]);
        const text = readFileSync(join(dir, "src/Quotes.vue"), "utf8");
        return text.includes("'x'") ? null : `status ${out.status}, output:\n${text}`;
      },
    ],
    [
      "check reads tsconfig.json",
      async () => {
        write(join(dir, "src/Implicit.vue"), IMPLICIT);
        const strict = await implicitAny(bin, dir, true);
        if (strict) return `strict: ${strict}`;
        write(join(dir, "tsconfig.json"), tsconfig(false));
        const loose = await implicitAny(bin, dir, false);
        return loose && `strict false: ${loose}`;
      },
    ],
    [
      "lsp reads vue.linter",
      () =>
        watchDiagnostics({
          bin,
          root: dir,
          files: [join(dir, "src/App.vue")],
          version,
          settle: (latest) =>
            rulesSettled(latest, new Map([[uriOf(join(dir, "src/App.vue")), [HTML, KEY]]])),
        }),
    ],
  ];
}

function migration(dir: string, version: string): Step[] {
  const bin = join(dir, "node_modules/vize/bin/vize");
  const dedicated = join(dir, "vize.config.json");
  const stale = `${JSON.stringify({ linter: { rules: { [HTML]: "off", [KEY]: "off" } } })}\n`;
  return [
    ["install", () => install(dir, [`vize@${version}`, `@vizejs/vite-plugin@${version}`, "vite"])],
    [
      "a lone vize.config.json is refused with migration guidance",
      async () => {
        write(dedicated, stale);
        const out = await vize(bin, dir, ["lint", "src/App.vue"]);
        if (out.status === 0) return "lint accepted a retired config";
        return `${out.stdout}${out.stderr}`.includes("vite.config")
          ? null
          : `no migration guidance: ${out.stderr}`;
      },
    ],
    [
      "vize init keeps the retired file",
      async () => {
        const out = await vize(bin, dir, [
          "init",
          "-y",
          "--no-install",
          "--no-editor",
          "--no-lint",
        ]);
        if (!existsSync(dedicated) || readFileSync(dedicated, "utf8") !== stale)
          return "init changed or removed vize.config.json";
        return `${out.stdout}${out.stderr}`.includes("vize.config")
          ? null
          : `init did not explain the migration (status ${out.status})`;
      },
    ],
    [
      "vite.config.ts wins over a stale vize.config.json",
      () => {
        write(
          join(dir, "vite.config.ts"),
          viteConfig({ linter: { rules: { [KEY]: "off" } } }, true),
        );
        return lintRules(bin, dir, [HTML]);
      },
    ],
    [
      "same result once vize.config.json is removed",
      () => {
        rmSync(dedicated);
        return lintRules(bin, dir, [HTML]);
      },
    ],
  ];
}

function monorepo(dir: string, version: string): Step[] {
  const bin = join(dir, "node_modules/vize/bin/vize");
  const a = join(dir, "packages/a");
  const b = join(dir, "packages/b");
  const packages = [`vize@${version}`, `@vizejs/vite-plugin@${version}`, "vite", "vue"];
  return [
    [
      "install",
      () => {
        write(join(dir, "package.json"), '{ "private": true, "workspaces": ["packages/*"] }\n');
        for (const [root, name, rule, strict] of [
          [a, "a", KEY, true],
          [b, "b", HTML, false],
        ] as const) {
          write(join(root, "package.json"), `{ "name": "${name}", "private": true }\n`);
          write(join(root, "vite.config.ts"), viteConfig({ linter: { rules: { [rule]: "off" } } }));
          write(join(root, "tsconfig.json"), tsconfig(strict));
          write(join(root, "src/App.vue"), APP);
          write(join(root, "src/Implicit.vue"), IMPLICIT);
        }
        return install(dir, packages);
      },
    ],
    ["lint from packages/a", () => lintRules(bin, a, [HTML])],
    ["lint from packages/b", () => lintRules(bin, b, [KEY])],
    ["check from packages/a (strict)", () => implicitAny(bin, a, true)],
    ["check from packages/b (not strict)", () => implicitAny(bin, b, false)],
    [
      "lsp at the workspace root applies each package's config",
      () =>
        watchDiagnostics({
          bin,
          root: dir,
          files: [join(a, "src/App.vue"), join(b, "src/App.vue")],
          version,
          settle: (latest) =>
            rulesSettled(
              latest,
              new Map([
                [uriOf(join(a, "src/App.vue")), [HTML, KEY]],
                [uriOf(join(b, "src/App.vue")), [KEY, HTML]],
              ]),
            ),
        }),
    ],
  ];
}

export async function verifyConfigFree(version: string): Promise<number> {
  const root = mkdtempSync(join(tmpdir(), "vize-config-free-"));
  let ok = true;
  try {
    for (const [label, scenario] of [
      ["fresh", freshProject],
      ["migration", migration],
      ["monorepo", monorepo],
    ] as const) {
      const dir = join(root, label);
      write(join(dir, "package.json"), `{ "name": "${label}", "private": true }\n`);
      write(join(dir, "src/App.vue"), APP);
      for (const [name, step] of scenario(dir, version)) {
        const failure = await step();
        println(
          `${failure === null ? "ok  " : "FAIL"} ${label}: ${name}${failure ? `: ${failure}` : ""}`,
        );
        ok &&= failure === null;
        if (failure !== null && name === "install") break;
      }
    }
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
  return ok ? 0 : 1;
}

if (import.meta.main) {
  runMain(async () => {
    const [version] = cliArgs();
    if (version === undefined) {
      writeStderr("Usage: verify_published/config_free.ts <version>\n");
      return 1;
    }
    return verifyConfigFree(version);
  });
}
