// #11483: style blocks the plugin hands to the host's CSS pipeline (preprocessor
// and CSS module blocks) must reach that pipeline under tsdown + @tsdown/css,
// and a host without one must fail loudly instead of parsing CSS as JavaScript.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import vizeRolldown from "./rolldown.ts";
import { writeFakePreprocessors } from "./test/fake-preprocessors.ts";
import { workspaceRoot } from "./test/helpers.ts";

const requireFromWorkspace = createRequire(path.join(workspaceRoot, "package.json"));
const tsdown = requireFromWorkspace.resolve("tsdown");
const cli = requireFromWorkspace.resolve("tsdown/run");
const requireFromTsdown = createRequire(tsdown);
const rolldownEntry = requireFromTsdown.resolve("rolldown");
const requireFromCssPlugin = createRequire(requireFromTsdown.resolve("@tsdown/css"));
const hasSass = ["sass-embedded", "sass"].some((name) => {
  try {
    requireFromCssPlugin.resolve(name);
    return true;
  } catch {
    return false;
  }
});
const plugin = path.join(import.meta.dirname, "rolldown.ts");

// The reporter's tsdown.config.ts and src/index.ts, with the plugin and tsdown
// imported from this checkout.
const config = `import Vize from ${JSON.stringify(pathToFileURL(plugin).href)};
import { defineConfig } from ${JSON.stringify(pathToFileURL(tsdown).href)};

export default defineConfig({
  deps: { neverBundle: ["vue"] },
  dts: false,
  entry: { index: "src/index.ts" },
  format: "esm",
  outDir: "dist-vize",
  plugins: [Vize({ isProduction: true })],
});
`;
const index = `export { default as Foo } from "./Foo.vue";\n`;
const component = (template: string, style: string) =>
  `<template>\n  ${template}\n</template>\n\n${style}\n`;
const button = '<button class="foo">Foo</button>';

function writeProject(cwd: string, source: string): void {
  fs.mkdirSync(path.join(cwd, "src"), { recursive: true });
  writeFakePreprocessors(cwd);
  fs.writeFileSync(path.join(cwd, "package.json"), '{ "private": true, "type": "module" }\n');
  fs.writeFileSync(path.join(cwd, "tsdown.config.ts"), config);
  fs.writeFileSync(path.join(cwd, "src/index.ts"), index);
  fs.writeFileSync(path.join(cwd, "src/Foo.vue"), source);
}

function readOutput(dir: string): Record<string, string> {
  if (!fs.existsSync(dir)) return {};
  return Object.fromEntries(
    fs.readdirSync(dir).map((file) => [file, fs.readFileSync(path.join(dir, file), "utf8")]),
  );
}

interface Case {
  id: string;
  source: string;
  /** Matched against the emitted CSS file; null when the build must fail. */
  css: RegExp | null;
  /** Matched against the build log when the build must fail. */
  error?: RegExp;
}

const cases: Case[] = [
  {
    // The reopen comment's component.
    id: "scss-scoped",
    source: component(button, '<style scoped lang="scss">\n.foo {\n  color: red;\n}\n</style>'),
    css: /\.foo\[data-v-[0-9a-f]{8}\]\s*\{\s*color:\s*red/,
  },
  {
    // An unscoped preprocessor block is compiled by the host's preprocessor.
    id: "scss",
    source: component(button, '<style lang="scss">\n$c: red;\n.foo {\n  color: $c;\n}\n</style>'),
    css: hasSass ? /\.foo\s*\{\s*color:\s*red/ : null,
    error: hasSass ? undefined : /Preprocessor dependency "sass" not found/,
  },
  {
    id: "module",
    source: component(
      '<button :class="$style.foo">Foo</button>',
      "<style module>\n.foo {\n  color: green;\n}\n</style>",
    ),
    css: /\.[\w-]*foo[\w-]*\s*\{\s*color:\s*green/,
  },
  {
    // Negative control: an authored CSS error is still reported, by the CSS
    // pipeline rather than the JavaScript parser.
    id: "module-invalid",
    source: component(
      '<button :class="$style.foo">Foo</button>',
      "<style module>\n.foo {\n  color: green;\n}}\n</style>",
    ),
    css: null,
    error:
      /\[plugin @tsdown\/css\] \S+\/src\/Foo\.vue\?vue&type=style&index=0&module&lang\.module\.css:\d+:\d+\nSyntaxError/,
  },
];

void test("#11483 tsdown + @tsdown/css builds delegated style blocks into a CSS file", (t) => {
  const temp = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-delegated-11483-")));
  try {
    for (const item of cases) {
      const cwd = path.join(temp, item.id);
      writeProject(cwd, item.source);
      const result = spawnSync(process.execPath, [cli, "--config", "tsdown.config.ts"], {
        cwd,
        encoding: "utf8",
        timeout: 60_000,
        maxBuffer: 8 * 1024 * 1024,
      });
      const log = `${result.stdout}${result.stderr}`;
      const output = readOutput(path.join(cwd, "dist-vize"));
      t.diagnostic(JSON.stringify({ id: item.id, status: result.status, log, output }));
      assert.equal(result.error, undefined, item.id);
      assert.doesNotMatch(log, /PARSE_ERROR/, item.id);
      if (!item.css) {
        assert.notEqual(result.status, 0, item.id);
        assert.match(log, item.error!, item.id);
        continue;
      }
      assert.equal(result.status, 0, `${item.id}: ${log}`);
      const cssFiles = Object.keys(output).filter((file) => file.endsWith(".css"));
      assert.deepEqual(cssFiles, ["style.css"], item.id);
      assert.match(output["style.css"], item.css, item.id);
      const js = Object.entries(output)
        .filter(([file]) => !file.endsWith(".css"))
        .map(([, code]) => code)
        .join("\n");
      assert.doesNotMatch(js, /document\.createElement\("style"\)/, item.id);
    }
  } finally {
    fs.rmSync(temp, { recursive: true, force: true });
  }
});

void test("#11483 Rolldown without a CSS pipeline rejects a delegated style block", async () => {
  const { rolldown } = (await import(pathToFileURL(rolldownEntry).href)) as {
    rolldown: (options: object) => Promise<{ generate(options: object): Promise<unknown> }>;
  };
  const temp = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-delegated-11483-")));
  try {
    writeProject(temp, cases[0].source);
    await assert.rejects(
      rolldown({
        input: path.join(temp, "src/index.ts"),
        cwd: temp,
        external: ["vue"],
        plugins: [vizeRolldown({ isProduction: true, root: temp })],
      }).then((bundle) => bundle.generate({ format: "esm" })),
      // Rolldown's own stylesheet error, not a JavaScript parse error.
      /Bundling CSS is no longer supported/,
    );
  } finally {
    fs.rmSync(temp, { recursive: true, force: true });
  }
});
