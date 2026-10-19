// #8099: all CLI consumers use the same receipted current executable, normal config lookup.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import {
  expectedBuildIdentity,
  validateBuildReceipt,
} from "../../../../tests/differential/build-receipt.ts";
import { asRecord, asString, parseJson, readJson } from "../../json-value.ts";
import { lexical, save, sha } from "./javascript-workspace-project.ts";
import type { WorkspaceContext } from "./javascript-workspace-project.ts";
import { expectedCheckMembership, projectEvidence } from "./javascript-workspace-config.ts";
import { packageDeclarations } from "./javascript-workspace-declarations.ts";

export interface CommandResult {
  status: number | null;
  stdout: string;
  stderr: string;
}
export type Command = (
  binary: string,
  args: readonly string[],
  cwd: string,
  artifacts: string,
  name: string,
  environment?: NodeJS.ProcessEnv,
) => CommandResult;
interface CheckCase {
  id: string;
  file: string;
  source: string;
  needle?: string;
  code?: number;
  message?: string;
}

export function sourceCli(root: string, artifacts: string): string {
  const identity = expectedBuildIdentity(root);
  assert.equal(identity.sourceRevision, process.env.GITHUB_SHA);
  const binary = path.join(root, identity.binaryPath);
  const receipt = readJson(`${binary}.differential-build.json`);
  validateBuildReceipt(receipt, identity);
  save(artifacts, "cli-source.json", { identity, receipt });
  return binary;
}
export const command: Command = (binary, args, cwd, artifacts, name, environment = {}) => {
  const started = performance.now();
  const result = spawnSync(binary, args, {
    cwd,
    env: { ...process.env, ...environment, NO_COLOR: "1", LANG: "C", LC_ALL: "C" },
    maxBuffer: 16 * 1024 * 1024,
    timeout: 120_000,
  });
  const wallMs = performance.now() - started;
  fs.writeFileSync(path.join(artifacts, `${name}.stdout.bin`), result.stdout ?? Buffer.alloc(0));
  fs.writeFileSync(path.join(artifacts, `${name}.stderr.bin`), result.stderr ?? Buffer.alloc(0));
  save(artifacts, `${name}-process.json`, {
    binary,
    binarySha256: sha(fs.readFileSync(binary)),
    args,
    cwd,
    status: result.status,
    signal: result.signal,
    error: result.error?.message ?? null,
    wallMs,
  });
  assert.equal(result.error, undefined);
  assert.equal(result.signal, null);
  return {
    status: result.status,
    stdout: result.stdout.toString(),
    stderr: result.stderr.toString(),
  };
};
export function position(source: string, needle: string): { line: number; character: number } {
  const offset = source.indexOf(needle);
  assert.ok(offset >= 0);
  assert.equal(source.indexOf(needle, offset + 1), -1);
  const prefix = source.slice(0, offset).split("\n");
  return { line: prefix.length - 1, character: (prefix.at(-1) ?? "").length };
}
export const missingMessage =
  "Cannot find module '@workspace/ui/Missing.vue' or its corresponding type declarations.";
export const argumentMessage =
  "Argument of type 'string' is not assignable to parameter of type 'number'.";
export async function cliProducts(root: string, context: WorkspaceContext, provider: string) {
  const binary = sourceCli(root, context.artifacts);
  const app = path.join(context.project, "apps", context.cohort.nuxt ? "nuxt" : "vite");
  const main = path.join(app, "src", context.cohort.nuxt ? "pages/index.vue" : "App.vue");
  const shared = path.join(context.project, "packages/ui/src/BadgeCard.vue");
  const label = path.join(context.project, "packages/pricing/src/label.mjs");
  const pricing = path.join(context.project, "packages/pricing/src/index.js");
  const ownFiles = [main, shared, label, pricing].sort(lexical);
  const lint = command(
    binary,
    ["lint", ...ownFiles, "--format", "json"],
    context.project,
    context.artifacts,
    "lint-clean",
  );
  assert.equal(lint.status, 0);
  assert.deepEqual(
    parseJson(lint.stdout),
    ownFiles.map((file) => ({ file, messages: [], errorCount: 0, warningCount: 0 })),
  );
  const lintFile = path.join(context.project, "lint-options.js");
  const lintSource = fs.readFileSync(
    path.join(
      root,
      "tests/_fixtures/differential/compat/javascript-workspace-products/lint-options.js.txt",
    ),
    "utf8",
  );
  fs.writeFileSync(lintFile, lintSource);
  try {
    const negative = command(
      binary,
      ["lint", lintFile, "--format", "json"],
      context.project,
      context.artifacts,
      "lint-negative",
    );
    const at = position(lintSource, "value() {}");
    const expected = [
      {
        file: lintFile,
        messages: [
          {
            ruleId: "script/no-dupe-keys",
            ruleDocsPath: "docs/content/rules/type-and-script.md",
            severity: 2,
            message: "[vize:script/no-dupe-keys] Duplicated key 'value'",
            line: at.line + 1,
            column: at.character + 1,
            endLine: at.line + 1,
            endColumn: at.character + 6,
            help: "props, data, computed, methods, setup, and inject share the component instance namespace; give each member a unique name.",
          },
        ],
        errorCount: 1,
        warningCount: 0,
      },
    ];
    save(context.artifacts, "lint-negative-expected.json", expected);
    assert.equal(negative.status, 1);
    assert.deepEqual(parseJson(negative.stdout), expected);
  } finally {
    fs.unlinkSync(lintFile);
  }
  const oldLabel = fs.readFileSync(label, "utf8");
  const oldShared = fs.readFileSync(shared, "utf8");
  try {
    fs.writeFileSync(
      label,
      oldLabel.replace(
        "export function badgeLabel(application) {\n  return `${application}: workspace exports`;\n}",
        "export function badgeLabel(application){return `${application}: workspace exports`}",
      ),
    );
    fs.writeFileSync(
      shared,
      oldShared.replace(
        '<template>\n  <aside data-test="shared">{{ label }}</aside>\n</template>',
        '<template><aside data-test="shared">{{label}}</aside></template>',
      ),
    );
    assert.notEqual(fs.readFileSync(label, "utf8"), oldLabel);
    assert.notEqual(fs.readFileSync(shared, "utf8"), oldShared);
    assert.equal(
      command(
        binary,
        ["fmt", label, shared, "--write"],
        context.project,
        context.artifacts,
        "fmt-write",
      ).status,
      0,
    );
    assert.equal(fs.readFileSync(label, "utf8"), oldLabel);
    assert.equal(fs.readFileSync(shared, "utf8"), oldShared);
    assert.equal(
      command(
        binary,
        ["fmt", label, shared, "--check"],
        context.project,
        context.artifacts,
        "fmt-idempotent",
      ).status,
      0,
    );
  } finally {
    fs.writeFileSync(label, oldLabel);
    fs.writeFileSync(shared, oldShared);
  }
  const compiled: { backend: string; packets: Record<string, unknown>[] }[] = [];
  for (const backend of ["client", "ssr"]) {
    const output = path.join(context.artifacts, `cli-${backend}`);
    const build = command(
      binary,
      [
        "build",
        main,
        shared,
        "--format",
        "json",
        "--output",
        output,
        ...(backend === "ssr" ? ["--ssr"] : []),
      ],
      context.project,
      context.artifacts,
      `compile-${backend}`,
    );
    assert.equal(build.status, 0);
    const packets = fs
      .readdirSync(output, { recursive: true, encoding: "utf8" })
      .filter((name) => name.endsWith(".json"))
      .sort(lexical)
      .map((name) => asRecord(readJson(path.join(output, name)), name));
    assert.equal(packets.length, 2);
    for (const packet of packets) {
      assert.deepEqual(Object.keys(packet).sort(lexical), [
        "code",
        "css",
        "errors",
        "filename",
        "macro_artifacts",
        "script_lang",
        "warnings",
      ]);
      assert.deepEqual(packet.errors, []);
      assert.deepEqual(packet.warnings, []);
      assert.equal(packet.css, null);
      assert.deepEqual(packet.macro_artifacts, []);
      assert.equal(packet.script_lang, "js");
      assert.ok(asString(packet.code, "packet.code").length > 0);
    }
    assert.deepEqual(
      packets.map((packet) => asString(packet.filename, "packet.filename")).sort(lexical),
      [path.basename(main), path.basename(shared)].sort(lexical),
    );
    compiled.push({ backend, packets });
  }
  save(context.artifacts, "compiled-whole-packets.json", compiled);
  packageDeclarations(root, context, command);
  const { original, stockPrograms, missingModuleFiles } = projectEvidence(root, context, app);
  const oldMain = fs.readFileSync(main, "utf8");
  const oldPricing = fs.readFileSync(pricing, "utf8");
  const records: unknown[] = [];
  const testsRequire = createRequire(path.join(root, "tests/package.json"));
  const vueTscManifestPath = testsRequire.resolve("vue-tsc/package.json");
  const vueTscManifest = readJson(vueTscManifestPath);
  const vueTsc = path.join(path.dirname(vueTscManifestPath), "bin/vue-tsc.js");
  const stockTypescriptManifestPath = createRequire(vueTsc).resolve("typescript/package.json");
  save(context.artifacts, "stock-checker-provider.json", {
    vueTsc: fs.realpathSync(vueTsc),
    vueTscManifestPath: fs.realpathSync(vueTscManifestPath),
    vueTscManifest,
    typescriptManifestPath: fs.realpathSync(stockTypescriptManifestPath),
    typescriptManifest: readJson(stockTypescriptManifestPath),
    vueManifest: readJson(path.join(context.project, "node_modules/vue/package.json")),
  });
  const wrongPrice = oldPricing.replace("unitPrice = 6", 'unitPrice = "wrong"');
  const cases: CheckCase[] = [
    { id: "clean", file: main, source: oldMain },
    {
      id: "wrong-argument",
      file: main,
      source: oldMain.replace("formatTotal(count.value)", 'formatTotal("two")'),
      needle: '"two"',
      code: 2345,
      message: argumentMessage,
    },
    { id: "repair-argument", file: main, source: oldMain },
    {
      id: "missing-module",
      file: main,
      source: oldMain.replace("@workspace/ui/BadgeCard.vue", "@workspace/ui/Missing.vue"),
      needle: '"@workspace/ui/Missing.vue"',
      code: 2307,
      message: missingMessage,
    },
    { id: "repair-module", file: main, source: oldMain },
    {
      id: "ordinary-js-wrong-type",
      file: pricing,
      source: wrongPrice,
      needle: 'unitPrice = "wrong"',
      code: 2322,
      message: "Type 'string' is not assignable to type 'number'.",
    },
    { id: "repair-ordinary-js", file: pricing, source: oldPricing },
  ];
  try {
    const mismatches: { id: string; error: string }[] = [];
    for (const testCase of cases) {
      try {
        fs.writeFileSync(testCase.file, testCase.source);
        const stock: CommandResult[] = [];
        for (const [index, program] of stockPrograms.entries())
          stock.push(
            command(
              process.execPath,
              [vueTsc, "--noEmit", "--pretty", "false", "-p", program.tsconfig],
              app,
              context.artifacts,
              `stock-${testCase.id}-${index}`,
            ),
          );
        const actual = command(
          binary,
          ["check", "--format", "json", "--quiet", "--corsa-path", provider],
          app,
          context.artifacts,
          `check-${testCase.id}`,
        );
        const expected = expectedCheckMembership(original, missingModuleFiles, testCase.id);
        const needle = testCase.needle ?? "";
        if (testCase.code) {
          const at = position(testCase.source, needle);
          const file = testCase.file.startsWith(app + path.sep)
            ? path.relative(app, testCase.file).split(path.sep).join("/")
            : testCase.file;
          let item = expected.files.find((row) => row.file === file);
          if (!item) {
            item = { file, diagnostics: [] };
            expected.files.push(item);
          }
          item.diagnostics.push(
            `error:${at.line + 1}:${at.character + 1} [TS${testCase.code}] ${testCase.message ?? ""}`,
          );
          expected.errorCount = 1;
        }
        const report = parseJson(actual.stdout);
        let stockExpected = "";
        if (testCase.code) {
          const at = position(testCase.source, needle);
          stockExpected = `${path.relative(app, testCase.file).split(path.sep).join("/")}(${at.line + 1},${at.character + 1}): error TS${testCase.code}: ${testCase.message ?? ""}\n`;
        }
        records.push({
          id: testCase.id,
          source: testCase.source,
          actual: report,
          expected,
          status: actual.status,
        });
        save(context.artifacts, "whole-typecheck-rows.json", records);
        save(context.artifacts, `stock-${testCase.id}-expectation.json`, { stockExpected, stock });
        assert.equal(stock.map((row) => row.stdout).join(""), stockExpected);
        assert.ok(stock.every((row) => row.stderr === ""));
        assert.equal(
          stock.reduce((total, row) => total + (row.status === 0 ? 0 : 1), 0),
          testCase.code ? 1 : 0,
        );
        assert.equal(actual.status, testCase.code ? 1 : 0);
        assert.deepEqual(report, expected);
      } catch (error) {
        mismatches.push({
          id: testCase.id,
          error: error instanceof Error ? (error.stack ?? error.message) : String(error),
        });
        save(context.artifacts, "typecheck-mismatches.json", mismatches);
      }
    }
    assert.deepEqual(mismatches, [], "all seven complete check vectors must qualify");
  } finally {
    fs.writeFileSync(main, oldMain);
    fs.writeFileSync(pricing, oldPricing);
  }
  return { binary, app, main, records, compiled };
}
