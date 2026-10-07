import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { expectedBuildIdentity, validateBuildReceipt } from "../differential/build-receipt.mjs";
import {
  nativeHistoryDirectory,
  nativeHistoryReceipt,
  validateNativeHistoryBuild,
} from "../../npm/native/scripts/formatter-history-build.mjs";
import {
  checkPluginReport,
  cliExpected,
  nativeExpected,
  pluginExpected,
} from "./support/art-lint-public-expectations.mjs";

const root = fileURLToPath(new URL("../../", import.meta.url));
const corpus = path.join(root, "tests/_fixtures/differential/linter/art-public-boundaries-7900");
const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");
const require = createRequire(import.meta.url);

await test("original Art bindings remain correct through CLI, source NAPI and Oxlint plugin", () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-art-public-7900-"));
  const artifacts = path.join(root, "target/differential/art-public-boundaries-7900");
  fs.mkdirSync(artifacts, { recursive: true });
  const evidence = {
    schema: "vize.art-public-boundaries-7900",
    version: 1,
    complete: false,
    runs: [],
  };
  const persist = () =>
    fs.writeFileSync(
      path.join(artifacts, "observations.json"),
      JSON.stringify(evidence, null, 2) + "\n",
    );
  const observe = (value) => {
    evidence.runs.push(value);
    persist();
    return value;
  };
  try {
    const manifest = JSON.parse(fs.readFileSync(path.join(corpus, "cases.json")));
    assert.equal(manifest.schema, evidence.schema);
    assert.equal(manifest.version, 1);
    assert.equal(manifest.cases.length, 6);
    for (const pin of manifest.files) {
      const bytes = fs.readFileSync(path.join(corpus, pin.file));
      assert.equal(bytes.length, pin.bytes, pin.file);
      assert.equal(sha256(bytes), pin.sha256, pin.file);
    }
    const original = JSON.parse(
      fs.readFileSync(
        path.join(root, "tests/_fixtures/differential/linter/art-setup-bindings-7940/cases.json"),
      ),
    );
    const rows = manifest.cases.map((row) => {
      const source = fs.readFileSync(path.join(corpus, row.file), "utf8");
      assert.equal(Buffer.byteLength(source), row.sourceBytes);
      assert.equal(sha256(source), row.sourceSha256);
      if (row.originalServiceCase) {
        const old = original.cases.find((entry) => entry.id === row.originalServiceCase);
        assert.equal(source, old.source);
        assert.deepEqual(row.service, old.service);
      }
      return { ...row, source };
    });
    evidence.corpus = {
      manifestSha256: sha256(fs.readFileSync(path.join(corpus, "cases.json"))),
      manifest,
      rows,
    };
    const build = expectedBuildIdentity(root);
    const cliReceipt = JSON.parse(
      fs.readFileSync(path.join(root, build.binaryPath + ".differential-build.json")),
    );
    validateBuildReceipt(cliReceipt, build);
    evidence.cliBuild = cliReceipt;
    const nativeDir = path.join(root, "npm/native");
    const receipt = JSON.parse(fs.readFileSync(nativeHistoryReceipt(nativeDir)));
    validateNativeHistoryBuild(nativeDir, receipt);
    assert.equal(receipt.source.head, build.sourceRevision);
    if (process.env.GITHUB_SHA) assert.equal(receipt.source.head, process.env.GITHUB_SHA);
    evidence.nativeBuild = receipt;
    for (const name of [
      "build-receipt.json",
      "build-process.json",
      "cargo.stdout.bin",
      "cargo.stderr.bin",
    ])
      fs.copyFileSync(
        path.join(nativeHistoryDirectory(nativeDir), name),
        path.join(artifacts, name),
      );
    const env = { ...process.env, VIZE_PREFER_WORKSPACE_BINDING: "1" };
    delete env.NAPI_RS_NATIVE_LIBRARY_PATH;
    delete env.NAPI_RS_FORCE_WASI;
    delete env.GITHUB_ACTIONS;
    process.env.VIZE_PREFER_WORKSPACE_BINDING = "1";
    process.env.NAPI_RS_NATIVE_LIBRARY_PATH = receipt.frozen.path;
    delete process.env.NAPI_RS_FORCE_WASI;
    const binding = require(path.join(nativeDir, "index.js"));
    delete process.env.NAPI_RS_NATIVE_LIBRARY_PATH;
    assert.ok(
      require.cache[fs.realpathSync(receipt.frozen.path)],
      "the public NAPI loaded the authenticated physical addon",
    );
    const run = (
      producer,
      executable,
      argv,
      expectedStatus = 0,
      environment = env,
      cwd = directory,
    ) => {
      const result = spawnSync(executable, argv, {
        cwd,
        env: environment,
        encoding: "utf8",
        timeout: 120_000,
        maxBuffer: 8 * 1024 * 1024,
      });
      const observation = observe({
        producer,
        executable,
        argv,
        status: result.status,
        signal: result.signal,
        error: result.error?.message ?? null,
        stdout: result.stdout ?? "",
        stderr: result.stderr ?? "",
      });
      assert.equal(observation.error, null, JSON.stringify(observation));
      assert.equal(observation.signal, null);
      assert.equal(observation.status, expectedStatus, JSON.stringify(observation));
      assert.doesNotMatch(
        observation.stdout + observation.stderr,
        /Error running JS plugin|RangeError|Failed to load the Vize native binding/u,
      );
      return observation;
    };
    run("source-plugin-build", "vp", ["run", "--filter", "./npm/oxlint", "build"], 0, env, root);
    const plugin = path.join(root, "npm/oxlint/dist/index.mjs");
    const wrapper = path.join(root, "npm/oxlint/dist/cli.mjs");
    const oxlint = fs
      .readdirSync(path.join(root, "node_modules/.pnpm"))
      .filter((name) => name.startsWith("oxlint@"))
      .sort()
      .map((name) => path.join(root, "node_modules/.pnpm", name, "node_modules/oxlint/bin/oxlint"))
      .find((file) => fs.existsSync(file));
    assert.ok(oxlint, "the genuine pinned Oxlint CLI is required");
    assert.match(
      run("oxlint-version", process.execPath, [oxlint, "--version"]).stdout,
      /\b\d+\.\d+\.\d+\b/u,
    );
    evidence.pluginBuild = {
      pluginSha256: sha256(fs.readFileSync(plugin)),
      wrapperSha256: sha256(fs.readFileSync(wrapper)),
      oxlint,
      launcherSha256: sha256(fs.readFileSync(oxlint)),
    };
    fs.mkdirSync(path.join(directory, ".git"));
    fs.mkdirSync(path.join(directory, "node_modules/oxlint/bin"), { recursive: true });
    fs.symlinkSync(oxlint, path.join(directory, "node_modules/oxlint/bin/oxlint"));
    for (const name of ["Card.vue", "Button.vue", "Panel.vue"])
      fs.copyFileSync(path.join(corpus, name + ".txt"), path.join(directory, name));
    const nativeCalls = path.join(artifacts, "plugin-native-calls.jsonl");
    fs.writeFileSync(nativeCalls, "");
    const pluginEnv = {
      ...env,
      VIZE_ART_SOURCE_BINDING: receipt.frozen.path,
      VIZE_ART_NATIVE_DIRECTORY: nativeDir,
      VIZE_ART_SOURCE_BINDING_SHA256: receipt.frozen.sha256,
      VIZE_ART_NATIVE_CALLS: nativeCalls,
      NODE_OPTIONS: `${env.NODE_OPTIONS ?? ""} --require=${JSON.stringify(path.join(root, "tests/tooling/support/art-lint-source-binding.cjs"))}`,
    };
    for (const row of rows) {
      fs.writeFileSync(path.join(directory, row.filename), row.source);
      const config =
        row.id === "original-7900"
          ? fs.readFileSync(path.join(corpus, "vize.config.json.txt"), "utf8")
          : JSON.stringify({
              linter: {
                preset: "incremental",
                rules: Object.fromEntries(row.rules.map((name) => [name, "warn"])),
              },
            });
      fs.writeFileSync(path.join(directory, "vize.config.json"), config);
      if (row.service.diagnostics.length === 0) {
        const plain = run("vize-original-plain", path.join(root, build.binaryPath), [
          "lint",
          "-f",
          "plain",
          "--help-level",
          "none",
          row.filename,
        ]);
        assert.equal(plain.stderr, "");
        assert.equal(plain.stdout, "Patina lint report: No problems found in 1 file(s)\n");
      }
      const cli = run("vize-whole-json", path.join(root, build.binaryPath), [
        "lint",
        "-f",
        "json",
        "--help-level",
        "full",
        "--locale",
        "en",
        row.filename,
      ]);
      assert.equal(cli.stderr, "");
      assert.deepEqual(JSON.parse(cli.stdout), cliExpected(row));
      const native = binding.lintPatinaSfc(row.source, {
        filename: row.filename,
        preset: "incremental",
        enabledRules: row.rules,
        helpLevel: "full",
        locale: "en",
      });
      observe({
        producer: "public-napi",
        id: row.id,
        actual: native,
        expected: nativeExpected(row),
      });
      assert.deepEqual(native, nativeExpected(row));
      fs.writeFileSync(
        path.join(directory, ".oxlintrc.json"),
        JSON.stringify({
          plugins: ["vue"],
          jsPlugins: [plugin],
          settings: { vize: { preset: "incremental", helpLevel: "none", locale: "en" } },
          rules: {
            "no-unused-vars": "off",
            ...Object.fromEntries(row.rules.map((name) => [`vize/${name}`, "warn"])),
          },
        }),
      );
      for (const [producer, entry, originalLocations] of [
        ["oxlint-direct-plugin", oxlint, false],
        ["oxlint-vize-wrapper", wrapper, true],
      ]) {
        const before = fs.readFileSync(nativeCalls, "utf8").length;
        const actual = run(
          producer,
          process.execPath,
          [entry, "-c", ".oxlintrc.json", "-f", "json", row.filename],
          0,
          pluginEnv,
        );
        checkPluginReport(JSON.parse(actual.stdout), pluginExpected(row, originalLocations));
        const events = fs
          .readFileSync(nativeCalls, "utf8")
          .slice(before)
          .trim()
          .split("\n")
          .map((line) => JSON.parse(line));
        const loads = new Set(
          events.filter((event) => event.kind === "load").map((event) => event.pid),
        );
        const calls = events.filter((event) => event.kind === "call");
        assert.ok(calls.length > 0, `${row.id}: plugin must actually call source NAPI`);
        for (const call of calls) {
          assert.ok(loads.has(call.pid));
          assert.equal(call.binary, fs.realpathSync(receipt.frozen.path));
          assert.equal(call.sha256, receipt.frozen.sha256);
          assert.equal(
            call.args[0],
            row.source,
            "plugin must lint the complete original Art source",
          );
          assert.equal(path.basename(call.args[1].filename), row.filename);
        }
        observe({ producer: `${producer}-source-calls`, id: row.id, events });
      }
      assert.equal(fs.readFileSync(path.join(directory, row.filename), "utf8"), row.source);
      fs.unlinkSync(path.join(directory, row.filename));
    }
    // Preserve the SSR comment's exact two-file invocation as well as isolated rows.
    for (const row of rows.slice(1, 3))
      fs.writeFileSync(path.join(directory, row.filename), row.source);
    fs.writeFileSync(
      path.join(directory, "vize.config.json"),
      JSON.stringify({
        linter: { preset: "incremental", rules: { "ssr/no-browser-globals-in-ssr": "warn" } },
      }),
    );
    const pair = run("vize-original-ssr-pair", path.join(root, build.binaryPath), [
      "lint",
      "-f",
      "plain",
      "--help-level",
      "none",
      "Panel.art.vue",
      "PanelHost.vue",
    ]);
    assert.equal(pair.stderr, "");
    assert.equal(pair.stdout, "Patina lint report: No problems found in 2 file(s)\n");
    validateNativeHistoryBuild(nativeDir, receipt);
    assert.deepEqual(expectedBuildIdentity(root), build);
    evidence.complete = true;
    persist();
    console.log(
      "VIZE_ART_PUBLIC_BOUNDARIES",
      JSON.stringify({
        source: receipt.source,
        cases: rows.length,
        producers: ["CLI", "public NAPI", "direct Oxlint plugin", "oxlint-vize"],
      }),
    );
  } finally {
    persist();
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
