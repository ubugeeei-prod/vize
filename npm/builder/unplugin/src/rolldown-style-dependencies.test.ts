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

void test("real tsdown/Rolldown refuses original style src and import instead of dropping CSS", (t) => {
  const requireFromWorkspace = createRequire(path.join(workspaceRoot, "package.json"));
  const tsdown = requireFromWorkspace.resolve("tsdown");
  const cli = requireFromWorkspace.resolve("tsdown/run");
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
  const cases = [
    { id: "original-src", source: input.content.toString("utf8"), expectedError: "<style src=" },
    {
      id: "inline-import",
      source: input.content
        .toString("utf8")
        .replace(
          '<style scoped src="./foo.css"></style>',
          '<style scoped>@import "./foo.css";</style>',
        ),
      expectedError: "CSS @import",
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
      fs.writeFileSync(path.join(cwd, "tsdown.config.ts"), adaptedConfig);
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
        assert.ok(`${actual.stdout}${actual.stderr}`.includes(actual.expectedError), actual.id);
        assert.match(`${actual.stdout}${actual.stderr}`, /not supported by @vizejs\/unplugin/);
      } else {
        assert.equal(actual.status, 0, actual.stderr);
        const emitted = actual.output.map((file) => file.content).join("\n");
        assert.match(emitted, /color:\s*red/);
        assert.match(emitted, /data-v-/);
        assert.match(emitted, /document\.createElement\("style"\)/);
      }
    }
  } finally {
    fs.rmSync(temp, { recursive: true, force: true });
  }
});
