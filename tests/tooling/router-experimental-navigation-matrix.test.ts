import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { performance } from "node:perf_hooks";
import { test } from "node:test";
import { repoRoot } from "../_helpers/realworld-patch.ts";
import { resolveVizeLaunchCommand } from "./support/lsp/launch.ts";
import { prepareExperimentalRouterProject } from "./support/router-experimental-provider.ts";
import {
  authoredPages,
  classifyFormatProbe,
  formatProbes,
  invalidMatrixCases,
  parseCompilerDiagnostics,
  plainTypeScriptOracle,
  point,
  readNavigationFixture,
  relocateScannerOutput,
  scannerTransportCompiler,
  scannerTransportControls,
  type FormatStage,
} from "./support/router-experimental-navigation-matrix.ts";

type Packet = {
  label: string;
  command: string;
  args: string[];
  status: number | null;
  signal: string | null;
  error: string | null;
  stdout: string;
  stderr: string;
  elapsedMs: number;
};

// Identity and timing are retained in raw custody. Every actual process result
// field, including the entire unmodified stdout packet, participates in equality.
function result(packet: Packet) {
  return {
    status: packet.status,
    signal: packet.signal,
    error: packet.error,
    stdout: packet.stdout,
    stderr: packet.stderr,
  };
}
function equivalent(actual: Packet, expected: Packet) {
  assert.deepEqual(result(actual), result(expected), actual.label);
}

const sourceFiles = [
  "src/pages/index.vue",
  "src/pages/optional-repeat/[[ids=int]]+.vue",
  "src/pages/optional/[[id=int]].vue",
  "src/pages/products/[id=int].vue",
  "src/pages/repeat/[ids=int]+.vue",
  "src/params/date.ts",
  "src/router/index.ts",
];

function assertReport(
  packet: Packet,
  file?: string,
  location?: { line: number; column: number },
  code?: number,
) {
  const report = JSON.parse(packet.stdout);
  assert.equal(report.errorCount, file ? 1 : 0, packet.label);
  assert.equal(report.warningCount, 0, packet.label);
  assert.equal(report.fileCount, sourceFiles.length, packet.label);
  assert.deepEqual(
    report.files.map((row: { file: string }) => row.file),
    sourceFiles,
    packet.label,
  );
  for (const row of report.files) {
    assert.equal(row.diagnostics.length, row.file === file ? 1 : 0, packet.label);
    if (row.file === file)
      assert.ok(
        row.diagnostics[0].startsWith(`error:${location!.line}:${location!.column} [TS${code}] `),
        packet.label,
      );
  }
  return report;
}

function mutationControls(packet: Packet) {
  for (const change of [
    { status: 0 },
    { signal: "SIGTERM" },
    { error: "unexpected failure" },
    { stderr: "unexpected stderr" },
    { stdout: `${packet.stdout}\n` },
    { stdout: packet.stdout.replace(/\[TS\d+\]/u, "[TS9999]") },
    { stdout: packet.stdout.replace("not assignable", "assignable") },
    { stdout: packet.stdout.replace('"root": "."', '"root": "other"') },
  ]) {
    assert.notDeepEqual(change, {}, "each negative control must change the actual packet");
    assert.notEqual(
      JSON.stringify(result({ ...packet, ...change })),
      JSON.stringify(result(packet)),
    );
    assert.throws(() => equivalent({ ...packet, ...change }, packet));
  }
  let report = JSON.parse(packet.stdout);
  for (const mutate of [
    () => {
      report.programs[0].unexpected = true;
    },
    () => {
      report.files.find((row: { diagnostics: string[] }) => row.diagnostics.length).diagnostics =
        [];
    },
  ]) {
    report = JSON.parse(packet.stdout);
    mutate();
    assert.throws(() =>
      equivalent({ ...packet, stdout: `${JSON.stringify(report, null, 2)}\n` }, packet),
    );
  }
}

test("published navigation fixtures pin independent authorities and every authored family", () => {
  const { files } = readNavigationFixture();
  const pages = authoredPages(files);
  assert.equal(pages.size, 4);
  assert.deepEqual(
    formatProbes().map(({ id }) => id),
    ["vue-js", "ts", "tsx", "art", "pug", "jade"],
  );
  assert.deepEqual(
    invalidMatrixCases(pages).map(({ id }) => id),
    [
      "required-path",
      "required-query",
      "default-query",
      "array-query",
      "custom-date-query",
      "hash",
      "optional-path",
      "repeated-path",
      "optional-repeated-path",
      "navigation-push",
      "navigation-replace",
      "navigation-resolve",
      "router-link-target",
    ],
  );
});

await test(
  "published route families preserve whole native packets against the immutable released oracle",
  {
    skip:
      !process.env.VIZE_TEST_BIN && process.env.VIZE_TEST_ROUTER_MATRIX_REQUIRED !== "1"
        ? "requires the dedicated exact-source campaign"
        : false,
  },
  async () => {
    assert.equal(process.env.VIZE_LSP_REQUIRE_SOURCE_BUILD, "1");
    assert.ok(process.env.VIZE_TEST_BIN, "exact source CLI required");
    const output =
      process.env.VIZE_TEST_ROUTER_MATRIX_OUTPUT ??
      path.join(repoRoot, "target/differential/router-navigation");
    fs.mkdirSync(output, { recursive: true });
    const [binary] = resolveVizeLaunchCommand(undefined, process.env.VIZE_TEST_BIN, {
      required: true,
      repoRoot,
      onVerifiedLaunch(launch) {
        fs.writeFileSync(
          path.join(output, "source-launch.json"),
          `${JSON.stringify(launch, null, 2)}\n`,
        );
      },
    });
    const checker = process.env.VIZE_TEST_ROUTER_MATRIX_CHECKER;
    const publicCli = process.env.VIZE_TEST_ROUTER_MATRIX_PUBLIC_CLI;
    assert.ok(checker && path.isAbsolute(checker), "independently locked TypeScript host required");
    assert.ok(publicCli && path.isAbsolute(publicCli), "immutable published Vize oracle required");
    const { authority, files } = readNavigationFixture();
    const fixture = await prepareExperimentalRouterProject(output);
    const scannerCompiler = scannerTransportCompiler(path.join(fixture.root, "package.json"));
    scannerTransportControls(files, authority, scannerCompiler);
    const pages = authoredPages(files);
    const references = JSON.parse(files.get("reference-diagnostics.json")!) as Array<{
      id: string;
      sfc: { status: number; stdout: string; stderr: string };
    }>;
    const records: Packet[] = [];
    const checks: Array<{ id: string; check: () => void }> = [];
    const run = (label: string, command: string, args: string[]) => {
      const started = performance.now();
      const raw = spawnSync(command, args, {
        cwd: fixture.root,
        encoding: "utf8",
        timeout: 120_000,
        maxBuffer: 32 * 1024 * 1024,
        env: { ...process.env, LANG: "C", LC_ALL: "C" },
      });
      const packet: Packet = {
        label,
        command,
        args,
        status: raw.status,
        signal: raw.signal,
        error: raw.error?.message ?? null,
        stdout: raw.stdout ?? "",
        stderr: raw.stderr ?? "",
        elapsedMs: performance.now() - started,
      };
      records.push(packet);
      fs.writeFileSync(
        path.join(output, "processes.json"),
        `${JSON.stringify(records, null, 2)}\n`,
      );
      assert.equal(packet.error, null, label);
      assert.equal(packet.signal, null, label);
      return packet;
    };
    const installPages = (newline: string) => {
      for (const [relative, source] of pages) {
        const target = path.join(fixture.root, relative);
        fs.mkdirSync(path.dirname(target), { recursive: true });
        fs.writeFileSync(target, source.replaceAll("\n", newline));
      }
    };
    const checkArgs = [
      "check",
      "src",
      "--tsconfig",
      "tsconfig.json",
      "--format",
      "json",
      "--quiet",
      "--corsa-path",
      checker,
    ];
    const native = (label: string) => run(`${label}/native`, binary, checkArgs);
    const published = (label: string) => run(`${label}/released-0.562`, publicCli, checkArgs);
    const generated = (label: string) =>
      run(`${label}/whole-generated-source`, binary, [...checkArgs, "--show-virtual-ts"]);
    const reference = (label: string) =>
      run(`${label}/reference`, process.execPath, [
        path.join(fixture.root, "node_modules/vue-tsc/bin/vue-tsc.js"),
        "--noEmit",
        "--pretty",
        "false",
        "-p",
        "tsconfig.json",
      ]);
    const plainArgs = [
      "--noEmit",
      "--pretty",
      "false",
      "--strict",
      "--skipLibCheck",
      "--noErrorTruncation",
      "--target",
      "ESNext",
      "--module",
      "ESNext",
      "--moduleResolution",
      "Bundler",
      "--types",
      "vue-router/auto-routes",
      "independent-target.ts",
      "typed-router.d.ts",
    ];
    try {
      const version = run("independent-checker-version", checker, ["--version"]);
      assert.equal(version.status, 0);
      assert.equal(version.stdout, "Version 7.1.0-dev.20261017.1\n");
      const publicVersion = run("immutable-public-oracle-version", publicCli, ["--version"]);
      assert.equal(publicVersion.status, 0);
      assert.equal(publicVersion.stdout, "vize 0.562.0\n");
      fs.writeFileSync(
        path.join(fixture.root, "tsconfig.json"),
        files.get("project/tsconfig.json.txt")!,
      );
      installPages("\n");
      const scanner = run("extended-official-scanner", process.execPath, ["generate.mjs"]);
      assert.equal(scanner.status, 0);
      const scannerPackets: unknown[] = [];
      const scannerChecks: Array<() => void> = [];
      for (const name of ["typed-router.d.ts", "routes.generated.ts", "resolver.generated.ts"]) {
        const bytes = fs.readFileSync(path.join(fixture.root, name), "utf8");
        fs.writeFileSync(path.join(output, `${name}.txt`), bytes);
        const pinned = files.get(`${name}.txt`)!;
        const expected =
          name === "typed-router.d.ts"
            ? pinned
            : relocateScannerOutput(
                name,
                pinned,
                fs.realpathSync(fixture.root),
                authority,
                scannerCompiler,
              );
        fs.writeFileSync(path.join(output, `${name}.expected.txt`), expected);
        scannerPackets.push({
          name,
          originalSha256: createHash("sha256").update(pinned).digest("hex"),
          expectedSha256: createHash("sha256").update(expected).digest("hex"),
          actualSha256: createHash("sha256").update(bytes).digest("hex"),
        });
        scannerChecks.push(() => assert.equal(bytes, expected, `complete scanner output ${name}`));
        if (name !== "typed-router.d.ts") fs.rmSync(path.join(fixture.root, name));
      }
      fs.writeFileSync(
        path.join(output, "scanner-transport.json"),
        `${JSON.stringify({ authority, admittedRoot: fs.realpathSync(fixture.root), packets: scannerPackets }, null, 2)}\n`,
      );
      for (const check of scannerChecks) check();
      for (const [variant, newline] of [
        ["lf", "\n"],
        ["crlf", "\r\n"],
      ] as const) {
        installPages(newline);
        const initial = native(`${variant}/valid`);
        const initialPublic = published(`${variant}/valid`);
        const initialGenerated = generated(`${variant}/valid`);
        const initialReference = reference(`${variant}/valid`);
        for (const entry of invalidMatrixCases(pages)) {
          installPages(newline);
          const source = entry.source.replaceAll("\n", newline);
          fs.writeFileSync(path.join(fixture.root, entry.file), source);
          fs.writeFileSync(path.join(output, `${variant}-${entry.id}.vue.txt`), source);
          const label = `${variant}/${entry.id}`;
          const candidate = native(label);
          const released = published(label);
          generated(label);
          const independent = reference(label);
          fs.writeFileSync(
            path.join(fixture.root, "independent-target.ts"),
            plainTypeScriptOracle(entry.source, entry.file, entry.link),
          );
          const target = run(`${label}/plain-typescript-7`, checker, [
            "--ignoreConfig",
            ...plainArgs,
          ]);
          const targetReference = run(`${label}/plain-typescript-5`, process.execPath, [
            path.join(fixture.root, "node_modules/typescript/bin/tsc"),
            ...plainArgs,
          ]);
          checks.push({
            id: label,
            check() {
              const location = point(source, entry.anchor, entry.token);
              const expected = references.find(({ id }) => id === entry.id)!;
              assert.equal(independent.status, expected.sfc.status, label);
              assert.equal(
                independent.stdout,
                expected.sfc.stdout,
                `complete pinned reference ${label}`,
              );
              assert.equal(independent.stderr, expected.sfc.stderr, label);
              const referenceRows = parseCompilerDiagnostics(independent.stdout);
              assert.equal(referenceRows.length, 1, label);
              assert.deepEqual(
                { ...referenceRows[0], message: undefined },
                {
                  file: entry.file,
                  ...point(
                    source,
                    entry.anchor,
                    entry.id === "navigation-resolve" ? "resolve" : entry.token,
                  ),
                  code: entry.code,
                  message: undefined,
                },
                label,
              );
              for (const packet of [target, targetReference]) {
                assert.equal(packet.status, 2, packet.label);
                assert.equal(packet.stderr, "", packet.label);
                const rows = parseCompilerDiagnostics(packet.stdout);
                assert.equal(rows.length, 1, packet.label);
                assert.equal(rows[0].code, entry.code, packet.label);
              }
              assert.equal(released.status, 1, label);
              assert.equal(released.stderr, "", label);
              assertReport(released, entry.file, location, entry.code);
              equivalent(candidate, released);
              if (entry.id === "navigation-push") mutationControls(released);
            },
          });
        }
        installPages(newline);
        const repaired = native(`${variant}/repaired`);
        const repairedPublic = published(`${variant}/repaired`);
        const repairedGenerated = generated(`${variant}/repaired`);
        const repairedReference = reference(`${variant}/repaired`);
        checks.push({
          id: `${variant}/valid-repaired`,
          check() {
            assert.equal(initialPublic.status, 0, variant);
            assert.equal(initialPublic.stderr, "", variant);
            assertReport(initialPublic);
            equivalent(initial, initialPublic);
            equivalent(repaired, repairedPublic);
            equivalent(repaired, initial);
            equivalent(repairedGenerated, initialGenerated);
            assert.equal(initialReference.status, 0, variant);
            assert.equal(initialReference.stdout, "", variant);
            assert.equal(initialReference.stderr, "", variant);
            equivalent(repairedReference, initialReference);
          },
        });
      }
      const formatOutcomes: unknown[] = [];
      const configPath = path.join(fixture.root, "tsconfig.json");
      const config = JSON.parse(fs.readFileSync(configPath, "utf8"));
      config.compilerOptions = {
        ...config.compilerOptions,
        allowJs: true,
        checkJs: true,
        jsx: "preserve",
        jsxImportSource: "vue",
      };
      config.include.push("src/**/*.tsx");
      fs.writeFileSync(configPath, JSON.stringify(config));
      for (const probe of formatProbes()) {
        installPages("\n");
        const probePath = path.join(fixture.root, probe.file);
        const stages: FormatStage[] = [];
        for (const stage of ["invalid", "repaired"] as const) {
          const source =
            stage === "invalid"
              ? probe.source
              : probe.source.replaceAll("toUpperCase()", "toFixed()");
          fs.writeFileSync(probePath, source);
          fs.writeFileSync(path.join(output, `${probe.id}-${stage}.source.txt`), source);
          const candidate = native(`format/${probe.id}/${stage}`);
          const released = published(`format/${probe.id}/${stage}`);
          checks.push({
            id: `format/${probe.id}/${stage}/native-preservation`,
            check() {
              equivalent(candidate, released);
            },
          });
          generated(`format/${probe.id}/${stage}`);
          const independent =
            probe.reference === "vue-tsc"
              ? reference(`format/${probe.id}/${stage}`)
              : run(`format/${probe.id}/${stage}/reference`, process.execPath, [
                  path.join(fixture.root, "node_modules/typescript/bin/tsc"),
                  "--noEmit",
                  "--pretty",
                  "false",
                  "--strict",
                  "--skipLibCheck",
                  "--target",
                  "ESNext",
                  "--module",
                  "ESNext",
                  "--moduleResolution",
                  "Bundler",
                  "--jsx",
                  "preserve",
                  "--jsxImportSource",
                  "vue",
                  probe.file,
                  "typed-router.d.ts",
                ]);
          let rows: FormatStage["nativeReport"];
          try {
            rows = JSON.parse(candidate.stdout);
          } catch {
            rows = null;
          }
          stages.push({
            stage,
            nativeStatus: candidate.status,
            referenceStatus: independent.status,
            nativeReport: rows,
            referenceDiagnostics: independent.stdout,
            referenceStderr: independent.stderr,
          });
        }
        formatOutcomes.push({
          id: probe.id,
          file: probe.file,
          reference: probe.reference,
          limitation: probe.limitation,
          scope: "discovery-only",
          outcome: classifyFormatProbe(probe.file, stages),
          stages,
        });
        fs.rmSync(probePath);
      }
      fs.writeFileSync(
        path.join(output, "format-discovery.json"),
        `${JSON.stringify(formatOutcomes, null, 2)}\n`,
      );
      const outcomes = checks.map(({ id, check }) => {
        try {
          check();
          return { id, outcome: "pass" };
        } catch (error) {
          return { id, outcome: "gap", detail: String(error) };
        }
      });
      fs.writeFileSync(
        path.join(output, "outcomes.json"),
        `${JSON.stringify(outcomes, null, 2)}\n`,
      );
      assert.deepEqual(
        outcomes.filter(({ outcome }) => outcome !== "pass"),
        [],
        "complete check matrix gaps; raw packets retained",
      );
    } finally {
      fixture.cleanup();
    }
  },
);
