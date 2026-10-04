import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { repoRoot } from "../../_helpers/realworld-patch.ts";
import { resolveTsgoBinary, symlinkVueTypes } from "../../_helpers/realworld-typecheck.ts";
import type { VizeCheckJson } from "../../_helpers/vize-check.ts";

const corpus = path.join(repoRoot, "tests/fixtures/typechecker/pnpm-workspace-routes");
const sources = [
  "packages/a/src/index.ts",
  "packages/b/src/index.ts",
  "packages/c/src/Btn.vue",
  "packages/c/src/index.ts",
  "packages/c/src/util.ts",
];
const links = JSON.parse(fs.readFileSync(path.join(corpus, "links.json"), "utf8")) as {
  root: Record<string, string>;
  pnpm: Record<string, string>;
};
const cleanTypedButton = `<script setup lang="ts">
const count: number = 1
defineEmits<{ pick: [value: number] }>()
</script>
<template>{{ count }}</template>
`;
const typedConsumer = `<script setup lang="ts">
import { Btn } from '@x/a'
type IsAny<T> = 0 extends 1 & T ? true : false
const componentMustBeTyped: IsAny<typeof Btn> = false
void componentMustBeTyped
</script>
<template><Btn @pick="value => value.toFixed()" /></template>
`;

type Receipt = {
  args: string[];
  execution: number;
  cwd: string;
  cliPid: number;
  cliSha256: string;
  nativeSha256: string;
  sourceSha: string;
  input: Array<{ path: string; sha256: string }>;
  links: Array<{ path: string; target: string }>;
  report: VizeCheckJson;
  status: number | null;
  stderr: string;
  stdout: string;
};

let executions = 0;

function sha256(bytes: string | Buffer): string {
  return createHash("sha256").update(bytes).digest("hex");
}

function workspace(pnpm: boolean): string {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-pnpm-routes-")));
  fs.cpSync(corpus, root, { recursive: true });
  for (const [relative, target] of Object.entries({ ...links.root, ...(pnpm ? links.pnpm : {}) })) {
    const file = path.join(root, relative);
    fs.mkdirSync(path.dirname(file), { recursive: true });
    fs.symlinkSync(target, file, "dir");
    assert.equal(fs.readlinkSync(file), target, "retain the reported raw relative link");
  }
  symlinkVueTypes(root);
  return root;
}

function check(root: string, patterns: string[], servers: number): Receipt {
  assert.equal(process.env.VIZE_TEST_REQUIRE_TSGO, "1", "native runtime is mandatory");
  // The registered lane builds this exact path before the supervisor runs.
  // Cached Vite+ scripts do not forward VIZE_TEST_BIN; bind the build path
  // directly rather than accepting a PATH binary or invoking another build.
  const command = path.join(repoRoot, "target/ci/vize");
  assert.ok(fs.statSync(command).isFile(), "the Actions source-built CLI is required");
  const native = resolveTsgoBinary();
  const args = [
    "check",
    ...patterns,
    "--tsconfig",
    "tsconfig.json",
    "--format",
    "json",
    "--quiet",
    "--show-virtual-ts",
    "--servers",
    String(servers),
    "--corsa-path",
    native,
  ];
  const result = spawnSync(command, args, {
    cwd: root,
    encoding: "utf8",
    env: { ...process.env, LANG: "C", LC_ALL: "C" },
    maxBuffer: 64 * 1024 * 1024,
    timeout: 120_000,
  });
  if (result.error) throw result.error;
  assert.equal(result.signal, null, result.stderr);
  const input = fs
    .readdirSync(root, { recursive: true, withFileTypes: true })
    .filter((file) => file.isFile() && !file.parentPath.includes("node_modules"))
    .map((file) => {
      const absolute = path.join(file.parentPath, file.name);
      return { path: path.relative(root, absolute), sha256: sha256(fs.readFileSync(absolute)) };
    })
    .sort((a, b) => a.path.localeCompare(b.path));
  const receipt: Receipt = {
    args,
    execution: ++executions,
    cwd: root,
    cliPid: result.pid,
    cliSha256: sha256(fs.readFileSync(command)),
    nativeSha256: sha256(fs.readFileSync(native)),
    sourceSha: spawnSync("git", ["rev-parse", "HEAD"], {
      cwd: repoRoot,
      encoding: "utf8",
    }).stdout.trim(),
    input,
    links: Object.entries({
      ...links.root,
      ...(fs.existsSync(path.join(root, "packages/a/node_modules")) ? links.pnpm : {}),
    }).map(([relative]) => ({
      path: relative,
      target: fs.readlinkSync(path.join(root, relative)),
    })),
    report: JSON.parse(result.stdout) as VizeCheckJson,
    status: result.status,
    stderr: result.stderr,
    stdout: result.stdout,
  };
  const output = path.join(
    repoRoot,
    "target/vize-tests/metrics/check-fixtures-topology/pnpm-workspace-routes",
  );
  fs.mkdirSync(output, { recursive: true });
  const bytes = `${JSON.stringify(receipt, null, 2)}\n`;
  fs.writeFileSync(path.join(output, `${sha256(bytes)}.json`), bytes);
  return receipt;
}

function diagnosticVector(receipt: Receipt): unknown {
  return {
    status: receipt.status,
    errorCount: receipt.report.errorCount,
    warningCount: receipt.report.warningCount,
    fileCount: receipt.report.fileCount,
    files: receipt.report.files.map(({ file, diagnostics }) => ({ file, diagnostics })),
  };
}

function assertClean(receipt: Receipt, expected: string[], roots = expected): void {
  assert.deepEqual(diagnosticVector(receipt), {
    status: 0,
    errorCount: 0,
    warningCount: 0,
    fileCount: expected.length,
    files: expected.map((file) => ({ file, diagnostics: [] })),
  });
  assert.equal(receipt.report.programs.length, 1);
  assert.deepEqual(receipt.report.programs[0]?.files, roots);
  assert.ok(receipt.report.files.every((file) => typeof file.virtualTs === "string"));
}

for (const pnpm of [false, true]) {
  test(`one physical package keeps its class identity across source directories (${pnpm ? "pnpm" : "root"})`, () => {
    const root = workspace(pnpm);
    try {
      fs.appendFileSync(
        path.join(root, "packages/c/src/index.ts"),
        "export class Thing { private readonly brand!: void; }\n",
      );
      for (const directory of ["x", "y"]) {
        fs.mkdirSync(path.join(root, "packages/b/src", directory));
      }
      fs.writeFileSync(
        path.join(root, "packages/b/src/x/make.ts"),
        'import { Thing } from "@x/c";\nexport const makeThing = () => new Thing();\n',
      );
      fs.writeFileSync(
        path.join(root, "packages/b/src/y/use.ts"),
        'import { Thing } from "@x/c";\nexport const useThing = (value: Thing) => { void value; };\n',
      );
      fs.writeFileSync(
        path.join(root, "packages/b/src/index.ts"),
        'import { makeThing } from "./x/make";\nimport { useThing } from "./y/use";\nuseThing(makeThing());\nexport { makeThing };\nexport * from "@x/c";\n',
      );
      fs.writeFileSync(
        path.join(root, sources[0]!),
        'import { Thing } from "@x/c";\nimport { makeThing } from "@x/b";\nconst thing: Thing = makeThing();\nvoid thing;\nexport * from "@x/b";\n',
      );
      const expected = [...sources, "packages/b/src/x/make.ts", "packages/b/src/y/use.ts"].sort();
      for (const servers of [1, 2]) {
        // Private members reject two separately materialized module identities,
        // even when their declarations and physical package are identical.
        assertClean(check(root, [sources[0]!], servers), expected, [sources[0]!]);
      }
    } finally {
      fs.rmSync(root, { recursive: true, force: true });
    }
  });

  test(`original #6982/#7834 workspace links resolve (${pnpm ? "pnpm" : "root"})`, () => {
    const root = workspace(pnpm);
    try {
      for (const servers of [1, 2]) {
        assertClean(check(root, [sources[0]!], servers), sources, [sources[0]!]);
        assertClean(check(root, ["packages/c/src/index.ts"], servers), sources.slice(2));
        assertClean(check(root, [], servers), sources);
      }
    } finally {
      fs.rmSync(root, { recursive: true, force: true });
    }
  });

  test(`typed Vue barrel retains complete error vectors across repair (${pnpm ? "pnpm" : "root"})`, () => {
    const root = workspace(pnpm);
    const button = path.join(root, "packages/c/src/Btn.vue");
    const consumer = "src/App.vue";
    try {
      fs.writeFileSync(button, cleanTypedButton);
      fs.mkdirSync(path.join(root, "src"));
      fs.writeFileSync(path.join(root, consumer), typedConsumer);
      // Keep the original issue tsconfig for the literal case above; the
      // typed secondary variant explicitly includes its additional consumer.
      const config = JSON.parse(fs.readFileSync(path.join(root, "tsconfig.json"), "utf8"));
      config.include.push("src/**/*.vue");
      fs.writeFileSync(path.join(root, "tsconfig.json"), `${JSON.stringify(config)}\n`);
      const expected = [...sources, consumer];
      const clean = check(root, [consumer], 1);
      assertClean(clean, expected, [consumer]);
      assertClean(check(root, [consumer], 2), expected, [consumer]);

      fs.writeFileSync(button, cleanTypedButton.replace("= 1", "= 'broken'"));
      const broken = check(root, [consumer], 1);
      assert.deepEqual(diagnosticVector(broken), {
        status: 1,
        errorCount: 1,
        warningCount: 0,
        fileCount: expected.length,
        files: expected.map((file) => ({
          file,
          diagnostics:
            file === "packages/c/src/Btn.vue"
              ? ["error:2:7 [TS2322] Type 'string' is not assignable to type 'number'."]
              : [],
        })),
      });
      assert.deepEqual(diagnosticVector(check(root, [consumer], 2)), diagnosticVector(broken));

      // A clean callback alone cannot prove the event payload is not any.
      // Changing its declared type must expose this authored toFixed access.
      fs.writeFileSync(button, cleanTypedButton.replace("value: number", "value: string"));
      const event = check(root, [consumer], 1);
      const eventDiagnostics = event.report.files.find(
        (file) => file.file === consumer,
      )?.diagnostics;
      assert.equal(eventDiagnostics?.length, 1);
      const column = typedConsumer.split("\n")[6]!.indexOf("toFixed") + 1;
      assert.match(
        eventDiagnostics![0]!,
        new RegExp(
          `^error:7:${column} \\[TS(2339|2551)\\] Property 'toFixed' does not exist on type 'string'`,
        ),
      );
      assert.deepEqual(diagnosticVector(event), {
        status: 1,
        errorCount: 1,
        warningCount: 0,
        fileCount: expected.length,
        files: expected.map((file) => ({
          file,
          diagnostics: file === consumer ? eventDiagnostics : [],
        })),
      });
      assert.deepEqual(diagnosticVector(check(root, [consumer], 2)), diagnosticVector(event));

      fs.writeFileSync(button, cleanTypedButton);
      const repaired = check(root, [consumer], 2);
      assertClean(repaired, expected, [consumer]);
      assert.deepEqual(
        repaired.report,
        clean.report,
        "repair preserves full ordered virtual TS/report",
      );
      assert.deepEqual(repaired.input, clean.input, "all authored bytes restored");
      assert.deepEqual(repaired.links, clean.links, "legitimate pnpm links remain unchanged");
    } finally {
      fs.rmSync(root, { recursive: true, force: true });
    }
  });
}
