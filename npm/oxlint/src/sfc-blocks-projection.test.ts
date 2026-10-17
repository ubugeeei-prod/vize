import assert from "node:assert/strict";
import test from "node:test";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";

import type { NativeSfcDescriptor } from "./model.ts";
import { projectSfcBlocks } from "./sfc-blocks.ts";
import { captureFixtureInputs, createExternalFixtureDir } from "./test-support/fixture-dir.ts";

// Independently authored bytes: <template> is10; 😀 is4UTF-8/2UTF-16;
// <input/> is8; </template> is11. No parser participates in these controls.
const source = "<template>😀<input/></template>";
const descriptor: NativeSfcDescriptor = {
  source,
  template: {
    content: "😀<input/>",
    loc: {
      start: 10,
      end: 22,
      tagStart: 0,
      tagEnd: 33,
      startLine: 1,
      startColumn: 11,
      endLine: 1,
      endColumn: 23,
    },
  },
  script: null,
  scriptSetup: null,
  styles: [],
  customBlocks: [],
};

void test("original stock argv is ignored inside target and executes outside the repository", () => {
  const repository = path.resolve(import.meta.dirname, "../../..");
  const evidenceRoot = path.join(repository, "target/vize-tests");
  fs.mkdirSync(evidenceRoot, { recursive: true });
  const evidence = fs.mkdtempSync(
    path.join(evidenceRoot, "original-sfc-host-7004-7005-ignore-boundary-"),
  );
  const ignored = fs.mkdtempSync(path.join(repository, "target/original-host-ignored-"));
  const outside = createExternalFixtureDir();
  const binary = path.join(repository, "node_modules/oxlint/bin/oxlint");
  const args = [binary, "-f", "json", "input.js"];
  const records = [];
  try {
    for (const [name, cwd] of [
      ["ignored", ignored],
      ["outside", outside],
    ] as const) {
      fs.writeFileSync(path.join(cwd, "input.js"), "debugger;\n");
      fs.writeFileSync(path.join(cwd, ".oxlintrc.json"), '{"rules":{"no-debugger":"error"}}\n');
      const directory = path.join(evidence, name);
      const before = captureFixtureInputs(
        cwd,
        ["input.js", ".oxlintrc.json"],
        path.join(directory, "before"),
      );
      const result = spawnSync(process.execPath, args, { cwd, timeout: 30_000 });
      fs.writeFileSync(path.join(directory, "stdout.bin"), result.stdout);
      fs.writeFileSync(path.join(directory, "stderr.bin"), result.stderr);
      records.push({
        argv: [process.execPath, ...args],
        cwd,
        status: result.status,
        signal: result.signal,
        timeoutMs: 30_000,
        streams: captureFixtureInputs(
          directory,
          ["stdout.bin", "stderr.bin"],
          path.join(directory, "streams"),
        ),
      });
      fs.writeFileSync(
        path.join(evidence, "commands.json"),
        JSON.stringify(records, null, 2) + "\n",
      );
      assert.equal(result.error, undefined);
      assert.equal(result.signal, null);
      assert.equal(result.status, 1);
      assert.deepEqual(
        captureFixtureInputs(cwd, ["input.js", ".oxlintrc.json"], path.join(directory, "after")),
        before,
      );
      if (name === "ignored") assert.match(result.stdout.toString(), /No files found to lint/u);
      else {
        const report = JSON.parse(result.stdout.toString());
        assert.equal(report.number_of_files, 1);
        assert.deepEqual(
          report.diagnostics.map((row: { code: string }) => row.code),
          ["eslint(no-debugger)"],
        );
      }
    }
  } finally {
    fs.rmSync(ignored, { recursive: true, force: true });
    fs.rmSync(outside, { recursive: true, force: true });
  }
});

void test("fixture evidence refuses dependency symlinks and parent traversal", () => {
  const root = createExternalFixtureDir();
  try {
    fs.writeFileSync(path.join(root, "input.vue"), "<template>😀</template>\r\n");
    fs.symlinkSync(path.join(root, "input.vue"), path.join(root, "linked.vue"));
    assert.throws(
      () => captureFixtureInputs(root, ["linked.vue"], path.join(root, "archive")),
      /regular file/u,
    );
    assert.throws(() => captureFixtureInputs(root, ["../outside.vue"], path.join(root, "archive")));
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

void test("byte geometry is projected once into complete UTF-16 content geometry", () => {
  assert.deepEqual(projectSfcBlocks(source, descriptor), [
    {
      kind: "template",
      name: "template",
      content: "😀<input/>",
      contentStart: { line: 1, column: 11 },
      contentEnd: { line: 1, column: 21 },
    },
  ]);
  assert.deepEqual(
    descriptor.template?.loc,
    {
      start: 10,
      end: 22,
      tagStart: 0,
      tagEnd: 33,
      startLine: 1,
      startColumn: 11,
      endLine: 1,
      endColumn: 23,
    },
    "projection never mutates native geometry",
  );
});

for (const offset of [11, 12, 13]) {
  void test(`refuses native byte endpoint inside non-BMP scalar at${offset}`, () => {
    const changed = structuredClone(descriptor);
    changed.template!.loc.start = offset;
    assert.throws(() => projectSfcBlocks(source, changed), {
      message: "Vize SFC byte endpoint splits a UTF-8 scalar.",
    });
  });
}

const mutations: [string, (value: NativeSfcDescriptor) => void][] = [
  [
    "different complete source",
    (value) => {
      value.source += "\n";
    },
  ],
  [
    "missing original content",
    (value) => {
      value.template!.content = "<input/>";
    },
  ],
  [
    "fractional endpoint",
    (value) => {
      value.template!.loc.start = 10.5;
    },
  ],
  [
    "out-of-bounds tag end",
    (value) => {
      value.template!.loc.tagEnd = 34;
    },
  ],
  [
    "reversed content",
    (value) => {
      value.template!.loc.start = 23;
    },
  ],
  [
    "byte column replaced by UTF-16 column",
    (value) => {
      value.template!.loc.endColumn = 21;
    },
  ],
  [
    "wrong content line",
    (value) => {
      value.template!.loc.endLine = 2;
    },
  ],
  [
    "overlapping top-level ownership",
    (value) => {
      value.styles.push(structuredClone(value.template!));
    },
  ],
  [
    "script incorrectly marked setup",
    (value) => {
      value.script = { ...value.template!, setup: true };
      value.template = null;
    },
  ],
  [
    "setup incorrectly marked ordinary script",
    (value) => {
      value.scriptSetup = { ...value.template!, setup: false };
      value.template = null;
    },
  ],
];
for (const [name, mutate] of mutations) {
  void test(`refuses ${name} without guessing a source range`, () => {
    const changed = structuredClone(descriptor);
    mutate(changed);
    assert.throws(() => projectSfcBlocks(source, changed));
  });
}

// A sentinel entry verifies rejection before module execution. These child
// processes have no compiler implementation and earn no native/host credit.
const bindingOverrideKeys = [
  "NODE_OPTIONS",
  "NODE_PATH",
  "VIZE_OXLINT_NATIVE_CUSTODY",
  "VIZE_PREFER_WORKSPACE_BINDING",
  "VIZE_OXLINT_TEST_ENTRYPOINT",
  "NAPI_RS_NATIVE_LIBRARY_PATH",
  "NAPI_RS_FORCE_WASI",
] as const;
for (const key of bindingOverrideKeys) {
  void test(`public baseline rejects ambient${key} before importing a plugin`, () => {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "oxlint-public-import-refusal-"));
    const pluginPackage = path.join(root, "node_modules/oxlint-plugin-vize");
    const oxlintPackage = path.join(root, "node_modules/oxlint");
    const marker = path.join(root, "forbidden-import");
    try {
      fs.mkdirSync(pluginPackage, { recursive: true });
      fs.mkdirSync(oxlintPackage, { recursive: true });
      fs.writeFileSync(
        path.join(pluginPackage, "package.json"),
        JSON.stringify({
          name: "oxlint-plugin-vize",
          version: "0.458.0",
          main: "sentinel.mjs",
          bin: { "oxlint-vize": "sentinel.mjs" },
        }),
      );
      fs.writeFileSync(
        path.join(pluginPackage, "sentinel.mjs"),
        `import fs from 'node:fs'; fs.writeFileSync(${JSON.stringify(marker)}, 'unsafe import');`,
      );
      fs.writeFileSync(
        path.join(oxlintPackage, "package.json"),
        JSON.stringify({ name: "oxlint", version: "1.86.0", bin: { oxlint: "unused" } }),
      );
      const env = { ...process.env };
      for (const name of bindingOverrideKeys) delete env[name];
      env[key] = key === "NODE_OPTIONS" ? "--no-warnings" : "must-refuse-before-import";
      const options = {
        pluginPackage,
        oxlintPackage,
        output: path.join(root, "output"),
        expectation: "public-0.458",
      };
      const script = `import assert from 'node:assert/strict'; import {runOriginalSfcHostMatrix} from ${JSON.stringify(new URL("./test-support/original-sfc-host-matrix.ts", import.meta.url).href)}; await assert.rejects(runOriginalSfcHostMatrix(${JSON.stringify(options)}), {message: ${JSON.stringify(`public baseline cannot inherit${key}`)}});`;
      const result = spawnSync(process.execPath, ["--input-type=module", "--eval", script], {
        env,
        encoding: "utf8",
        timeout: 10_000,
      });
      assert.equal(result.error, undefined);
      assert.equal(result.signal, null);
      assert.equal(result.status, 0, result.stdout + result.stderr);
      assert.equal(fs.existsSync(marker), false);
    } finally {
      fs.rmSync(root, { recursive: true, force: true });
    }
  });
}
