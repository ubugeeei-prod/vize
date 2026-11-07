import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { resolveFixturePath, workspaceRoot } from "./test/helpers.ts";

type OriginalFile = { path: string; storedPath: string; bytes: number; sha256: string };
const fixtureRoot = resolveFixturePath("style-src-11456", "");
const custody = JSON.parse(fs.readFileSync(path.join(fixtureRoot, "custody.json"), "utf8")) as {
  files: OriginalFile[];
};

function filesUnder(
  root: string,
): { path: string; bytes: number; sha256: string; content: string }[] {
  if (!fs.existsSync(root)) return [];
  return fs.readdirSync(root, { withFileTypes: true }).flatMap((entry) => {
    const file = path.join(root, entry.name);
    if (entry.isDirectory()) {
      return filesUnder(file).map((item) => ({ ...item, path: `${entry.name}/${item.path}` }));
    }
    const bytes = fs.readFileSync(file);
    return [
      {
        path: entry.name,
        bytes: bytes.length,
        sha256: createHash("sha256").update(bytes).digest("hex"),
        content: bytes.toString("utf8"),
      },
    ];
  });
}

void test("#11456 real tsdown/Rolldown builds original style src CSS into a CSS file", (t) => {
  const requireFromUnplugin = createRequire(
    path.join(workspaceRoot, "npm/builder/unplugin/package.json"),
  );
  const tsdown = requireFromUnplugin.resolve("tsdown");
  const cli = requireFromUnplugin.resolve("tsdown/run");
  const requireFromTsdown = createRequire(tsdown);
  const rolldown = requireFromTsdown.resolve("rolldown");
  const plugin = path.join(import.meta.dirname, "rolldown.ts");
  const original = custody.files.map((file) => ({
    ...file,
    content: fs.readFileSync(path.join(fixtureRoot, file.storedPath)),
  }));
  const input = original.find((file) => file.path === "src/Foo.vue");
  const config = original.find((file) => file.path === "tsdown.config.ts");
  assert.ok(input && config);
  for (const file of original) {
    assert.equal(file.content.length, file.bytes);
    assert.equal(createHash("sha256").update(file.content).digest("hex"), file.sha256);
  }

  const adaptedConfig = config.content
    .toString("utf8")
    .replace('"@vizejs/unplugin/rolldown"', JSON.stringify(pathToFileURL(plugin).href))
    .replace('"tsdown"', JSON.stringify(pathToFileURL(tsdown).href));
  const replaceStyle = (style: string) =>
    input.content.toString("utf8").replace('<style scoped src="./foo.css"></style>', style);
  const cases: Array<{
    id: string;
    source: string;
    expectedError: RegExp | null;
    /** Defaults to the block's red rule. */
    expectedCss?: RegExp;
    scoped?: boolean;
    /** `foo.css` moved to a package or an aliased directory (#11482). */
    cssAt?: string;
    alias?: boolean;
    /** Listed in `dependencies`, which tsdown keeps external (#11494). */
    external?: string;
    /** The import the output keeps of the external module. */
    externalImport?: RegExp;
  }> = [
    {
      id: "original-src",
      source: input.content.toString("utf8"),
      expectedError: null,
      scoped: true,
    },
    {
      id: "plain-src",
      source: replaceStyle('<style src="./foo.css"></style>'),
      expectedError: null,
      scoped: false,
    },
    {
      id: "missing-src",
      source: replaceStyle('<style scoped src="./missing.css"></style>'),
      expectedError: /<style src="\.\/missing\.css"> not found/,
    },
    {
      id: "package-src",
      source: replaceStyle('<style scoped src="fake-css/foo.css"></style>'),
      expectedError: null,
      scoped: true,
      cssAt: "node_modules/fake-css/foo.css",
    },
    {
      id: "alias-src",
      source: replaceStyle('<style scoped src="@styles/foo.css"></style>'),
      expectedError: null,
      scoped: true,
      cssAt: "styles/foo.css",
      alias: true,
    },
    {
      id: "internal-package-plain-src",
      source: replaceStyle('<style src="fake-css/foo.css"></style>'),
      expectedError: null,
      scoped: false,
      cssAt: "node_modules/fake-css/foo.css",
    },
    {
      id: "external-package-src",
      source: replaceStyle('<style src="fake-css/foo.css"></style>'),
      expectedError: null,
      cssAt: "node_modules/fake-css/foo.css",
      external: "fake-css",
      externalImport: /import "fake-css\/foo\.css";/,
    },
    {
      id: "external-package-script-src",
      source: `<script src="fake-css/foo.js"></script>\n${replaceStyle("")}`,
      expectedError: null,
      cssAt: "node_modules/fake-css/foo.css",
      external: "fake-css",
      externalImport: /import \w+ from "fake-css\/foo\.js";/,
    },
    {
      id: "external-package-scoped-src",
      source: replaceStyle('<style scoped src="fake-css/foo.css"></style>'),
      expectedError:
        /<style scoped src="fake-css\/foo\.css"> resolved to an external module; a scoped style needs its contents/,
      cssAt: "node_modules/fake-css/foo.css",
      external: "fake-css",
    },
    {
      id: "missing-package-src",
      source: replaceStyle('<style scoped src="missing-css/foo.css"></style>'),
      expectedError: /<style src="missing-css\/foo\.css"> not found/,
    },
    {
      // @tsdown/css passes the @import of a virtual stylesheet through, as it
      // does for unplugin-vue's.
      id: "inline-import",
      source: input.content
        .toString("utf8")
        .replace(
          '<style scoped src="./foo.css"></style>',
          '<style scoped>@import "./foo.css";</style>',
        ),
      expectedError: null,
      expectedCss: /^@import "\.\/foo\.css";/,
    },
    {
      id: "inline-scoped",
      source: input.content
        .toString("utf8")
        .replace(
          '<style scoped src="./foo.css"></style>',
          "<style scoped>.foo { color: red; }</style>",
        ),
      expectedError: null,
      scoped: true,
    },
  ];
  const temp = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-style-src-11456-")));
  try {
    const outcomes = cases.map((item) => {
      const cwd = path.join(temp, item.id);
      for (const file of original) {
        const target = path.join(cwd, file.path);
        fs.mkdirSync(path.dirname(target), { recursive: true });
        fs.writeFileSync(target, file.content);
      }
      fs.writeFileSync(path.join(cwd, "src/Foo.vue"), item.source);
      if (item.external) {
        const manifest = JSON.parse(fs.readFileSync(path.join(cwd, "package.json"), "utf8"));
        manifest.dependencies = { [item.external]: "*" };
        fs.writeFileSync(path.join(cwd, "package.json"), JSON.stringify(manifest));
      }
      if (item.cssAt) {
        fs.mkdirSync(path.dirname(path.join(cwd, item.cssAt)), { recursive: true });
        fs.renameSync(path.join(cwd, "src/foo.css"), path.join(cwd, item.cssAt));
      }
      // Rolldown resolves the alias through `this.resolve`.
      const aliasConfig = `  alias: { "@styles": ${JSON.stringify(path.join(cwd, "styles"))} },\n  plugins:`;
      fs.writeFileSync(
        path.join(cwd, "tsdown.config.ts"),
        item.alias ? adaptedConfig.replace("  plugins:", aliasConfig) : adaptedConfig,
      );
      const result = spawnSync(process.execPath, [cli, "--config", "tsdown.config.ts"], {
        cwd,
        encoding: "utf8",
        timeout: 60_000,
        maxBuffer: 8 * 1024 * 1024,
      });
      return {
        ...item,
        cwd,
        sourceRevision: process.env.GITHUB_SHA ?? null,
        provider: { node: process.version, tsdown, rolldown, cli, plugin },
        originalFiles: custody.files,
        config: adaptedConfig,
        argv: [process.execPath, cli, "--config", "tsdown.config.ts"],
        status: result.status,
        signal: result.signal,
        error: result.error
          ? { name: result.error.name, message: result.error.message, stack: result.error.stack }
          : null,
        stdout: result.stdout,
        stderr: result.stderr,
        output: filesUnder(path.join(cwd, "dist")),
      };
    });
    for (const outcome of outcomes) {
      t.diagnostic(
        JSON.stringify({ schema: "vize.unplugin.rolldownStyleDependencies.v1", outcome }),
      );
    }
    for (const actual of outcomes) {
      assert.equal(actual.error, null);
      assert.equal(actual.signal, null);
      if (actual.expectedError) {
        assert.notEqual(actual.status, 0, actual.id);
        assert.match(`${actual.stdout}${actual.stderr}`, actual.expectedError, actual.id);
      } else if (actual.external) {
        // Kept as an import of the external stylesheet, as @vitejs/plugin-vue does.
        assert.equal(actual.status, 0, actual.stderr);
        const emitted = actual.output.map((file) => file.content).join("\n");
        assert.ok(actual.externalImport, actual.id);
        assert.match(emitted, actual.externalImport, actual.id);
        assert.doesNotMatch(emitted, /color:\s*red/, actual.id);
      } else {
        assert.equal(actual.status, 0, actual.stderr);
        // The host's CSS pipeline writes the stylesheet; nothing is injected.
        assert.deepEqual(
          actual.output.map((file) => file.path),
          ["index.mjs", "style.css"],
          actual.id,
        );
        const [js, css] = actual.output.map((file) => file.content);
        assert.doesNotMatch(js, /document\.createElement\("style"\)/, actual.id);
        assert.match(css, actual.expectedCss ?? /color:\s*red/, actual.id);
        if (!actual.expectedCss) {
          assert.equal(/\.foo\[data-v-[0-9a-f]{8}\]/.test(css), actual.scoped, actual.id);
        }
      }
    }
  } finally {
    fs.rmSync(temp, { recursive: true, force: true });
  }
});
