/** Capture the real current-source baseline over all original 27 #11023 cases. */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import { execFileSync } from "node:child_process";
import { fileURLToPath, pathToFileURL } from "node:url";
import { identify, run, save } from "../../../benchmarks/scripts/canon-instance-directives-io.ts";
import { packet } from "../../../benchmarks/scripts/canon-instance-directives-model.ts";
import { captureSlotPacket } from "../../../benchmarks/scripts/canon-slot-constructor-protocol.ts";
import {
  original27Inputs,
  original27Root,
  writeOriginal27,
  sha256,
} from "./constructor-overload-original27.ts";

const declarations = [
  "virtual-ts-globals.d.ts",
  "virtual-ts-patterns.d.ts",
  "virtual-ts.d.ts",
  "virtual-ts.d.mts",
  "virtual-ts.d.cts",
  "content-mapper-helpers.d.ts",
  "content-mapper-helpers.d.mts",
  "content-mapper-helpers.d.cts",
  "generated/content-mapper-types.d.ts",
];
const sourceRoot = process.cwd();
const [mode, source, outputArg, providerArg] = process.argv.slice(2);
assert.ok(source && outputArg);
assert.match(source, /^[a-f0-9]{40}$/u);
const output = path.resolve(outputArg);
const git = (...argv: string[]) => execFileSync("git", argv, { cwd: sourceRoot }).toString().trim();
assert.equal(git("rev-parse", "HEAD"), source);
assert.equal(git("status", "--porcelain", "--untracked-files=no"), "");
const tree = git("rev-parse", "HEAD^{tree}");
const driverRoot = fileURLToPath(new URL("../../../../", import.meta.url));
const driver = {
  source: execFileSync("git", ["rev-parse", "HEAD"], { cwd: driverRoot }).toString().trim(),
  tree: execFileSync("git", ["rev-parse", "HEAD^{tree}"], { cwd: driverRoot }).toString().trim(),
  script: identify(fileURLToPath(import.meta.url)),
};

if (mode === "build") {
  const messages = fs
    .readFileSync(path.join(output, "cargo.jsonl"), "utf8")
    .split("\n")
    .filter(Boolean)
    .map((line) => JSON.parse(line) as Record<string, unknown>);
  assert.deepEqual(messages.at(-1), { reason: "build-finished", success: true });
  type Artifact = {
    reason: string;
    target: { name: string; kind: string[] };
    executable?: string;
    features: string[];
    fresh: boolean;
  };
  const artifacts = messages.filter(
    (value) => value.reason === "compiler-artifact",
  ) as unknown as Artifact[];
  const images = artifacts.filter(
    (value) =>
      value.target.name === "vize" && value.target.kind.join() === "bin" && value.executable,
  );
  assert.equal(images.length, 1);
  save(path.join(output, "build.json"), {
    source,
    tree,
    sourceRoot,
    driver,
    cli: identify(images[0].executable!),
    cargoArtifact: images[0],
    argv: [
      "cargo",
      "build",
      "--locked",
      "--profile",
      "ci",
      "-p",
      "vize",
      "--bin",
      "vize",
      "--message-format=json",
    ],
    overlay: false,
    cacheReuseAllowed: true,
    timingUsable: false,
    tools: {
      node: process.version,
      rustc: execFileSync("rustc", ["-Vv"]).toString(),
      cargo: execFileSync("cargo", ["-V"]).toString(),
    },
    environment: Object.fromEntries(
      [
        "PATH",
        "RUSTC",
        "RUSTFLAGS",
        "CARGO_ENCODED_RUSTFLAGS",
        "CARGO_INCREMENTAL",
        "CARGO_BUILD_JOBS",
        "CARGO_TARGET_DIR",
        "GITHUB_SHA",
        "GITHUB_RUN_ID",
        "GITHUB_RUN_ATTEMPT",
      ].map((key) => [key, process.env[key] ?? null]),
    ),
  });
} else {
  assert.equal(mode, "capture");
  assert.ok(providerArg);
  const provider = path.resolve(providerArg);
  const build = JSON.parse(fs.readFileSync(path.join(output, "build.json"), "utf8")) as {
    source: string;
    tree: string;
    cli: { path: string; bytes: number; sha256: string };
  };
  assert.equal(build.source, source);
  assert.equal(build.tree, tree);
  assert.deepEqual(identify(build.cli.path), build.cli);
  const inputs = original27Inputs();
  save(path.join(output, "original-manifest.json"), inputs.manifest);
  fs.copyFileSync(
    path.join(original27Root, "historical-public-summary.json"),
    path.join(output, "historical-public-summary.json"),
  );
  const inputRoot = path.join(output, "inputs");
  for (const [file, body] of inputs.files) {
    fs.mkdirSync(path.dirname(path.join(inputRoot, file)), { recursive: true });
    fs.writeFileSync(path.join(inputRoot, file), body);
  }
  const versions: Record<string, unknown> = {};
  for (const name of ["vize", "vue", "typescript", "typescript-native", "vue-tsc"])
    versions[name] = JSON.parse(
      fs.readFileSync(path.join(provider, "node_modules", name, "package.json"), "utf8"),
    ).version;
  assert.deepEqual(versions, {
    vize: "0.602.0",
    vue: "3.5.42",
    typescript: "6.0.3",
    "typescript-native": "7.1.0-beta",
    "vue-tsc": "3.3.13",
  });
  const providerIdentities = inputs.historical.identities.map(({ file, bytes, sha256 }) => {
    const actual = { file, ...identify(path.join(provider, file)) };
    if (file !== "package-lock.json")
      assert.deepEqual({ bytes: actual.bytes, sha256: actual.sha256 }, { bytes, sha256 }, file);
    return actual;
  });
  const nativeModule = (await import(
    pathToFileURL(path.join(provider, "node_modules/typescript-native/lib/getExePath.js")).href
  )) as { default: () => string };
  const native = nativeModule.default(),
    vueTsc = path.join(provider, "node_modules/vue-tsc/bin/vue-tsc.js"),
    stock = path.join(provider, "node_modules/typescript/lib/tsc.js");
  const requireVueTsc = createRequire(vueTsc);
  assert.equal(
    fs.realpathSync(requireVueTsc.resolve("typescript/package.json")),
    fs.realpathSync(path.join(provider, "node_modules/typescript/package.json")),
  );
  const nativeIdentity = identify(native);
  for (const [file, expected] of [
    [native, inputs.historical.checker],
    [stock, inputs.historical.stock],
    [vueTsc, inputs.historical.vueTsc],
  ] as const) {
    const actual = identify(file);
    assert.deepEqual(
      { bytes: actual.bytes, sha256: actual.sha256 },
      { bytes: expected.bytes, sha256: expected.sha256 },
      file,
    );
  }
  const publicProject = path.join(output, "published-project");
  writeOriginal27(publicProject);
  fs.symlinkSync(
    path.join(provider, "node_modules"),
    path.join(publicProject, "node_modules"),
    "dir",
  );
  const currentProject = path.join(output, "current-source-project");
  writeOriginal27(currentProject);
  const currentModules = path.join(currentProject, "node_modules");
  fs.mkdirSync(currentModules);
  for (const name of fs.readdirSync(path.join(provider, "node_modules"))) {
    if (name === "vize" || name === ".bin") continue;
    fs.symlinkSync(
      path.join(provider, "node_modules", name),
      path.join(currentModules, name),
      "dir",
    );
  }
  const currentPackage = path.join(currentModules, "vize");
  fs.mkdirSync(currentPackage);
  const packageBytes = fs.readFileSync(path.join(sourceRoot, "npm/cli/package.json"));
  const manifest = JSON.parse(packageBytes.toString()) as {
    name: string;
    version: string;
    typescript: { contentMapper: Record<string, unknown> };
  };
  const sourceManifest = {
    ...manifest,
    version: `0.0.0-source-${source}`,
    typescript: {
      ...manifest.typescript,
      contentMapper: {
        ...manifest.typescript.contentMapper,
        exec: [build.cli.path, "content-mapper"],
      },
    },
  };
  save(path.join(currentPackage, "package.json"), sourceManifest);
  const shipped = declarations.map((file) => {
    const bytes = fs.readFileSync(path.join(sourceRoot, "npm/cli", file)),
      destination = path.join(currentPackage, file);
    fs.mkdirSync(path.dirname(destination), { recursive: true });
    fs.writeFileSync(destination, bytes);
    const retained = path.join(output, "source-package", file);
    fs.mkdirSync(path.dirname(retained), { recursive: true });
    fs.writeFileSync(retained, bytes);
    return { file, bytes: bytes.length, sha256: sha256(bytes) };
  });
  save(path.join(output, "source-package/package.json"), sourceManifest);
  save(path.join(output, "providers.json"), {
    source,
    tree,
    build,
    versions,
    providerIdentities,
    native: nativeIdentity,
    stock: identify(stock),
    vueTsc: identify(vueTsc),
    sourcePackage: {
      manifestBeforeSha256: sha256(packageBytes),
      manifestAfter: sourceManifest,
      declarations: shipped,
    },
    originalProviderLockSha256: "404c0044722de82488d559210af047bda408b5f353d74b8e333c512d15375cc7",
    freshLockMatchesHistorical:
      identify(path.join(provider, "package-lock.json")).sha256 ===
      "404c0044722de82488d559210af047bda408b5f353d74b8e333c512d15375cc7",
  });
  const tscDiagnostics = (stdout: Buffer) => {
    const text = stdout.toString(),
      diagnostics: {
        file: string;
        line: number;
        column: number;
        code: number;
        message: string;
        continuation: string[];
      }[] = [];
    for (const line of text.trimEnd().split("\n").filter(Boolean)) {
      const match = /^(.*?)\((\d+),(\d+)\): error TS(\d+): (.*)$/u.exec(line);
      if (match)
        diagnostics.push({
          file: match[1].replaceAll("\\", "/"),
          line: +match[2],
          column: +match[3],
          code: +match[4],
          message: match[5],
          continuation: [],
        });
      else {
        assert.ok(
          diagnostics.length > 0 && /^\s/u.test(line),
          `unclassified compiler output: ${line}`,
        );
        diagnostics.at(-1)!.continuation.push(line);
      }
    }
    return diagnostics;
  };
  const h = run(
    output,
    "stock-h",
    process.execPath,
    [stock, "--pretty", "false", "--project", "tsconfig.h.json"],
    publicProject,
  );
  const official = run(
    output,
    "official-vue-tsc",
    process.execPath,
    [vueTsc, "--pretty", "false", "--project", "tsconfig.json"],
    publicProject,
  );
  const officialDiagnostics = tscDiagnostics(official.stdout);
  const headers = (diagnostics: ReturnType<typeof tscDiagnostics>) =>
    diagnostics.map(({ continuation: _continuation, ...diagnostic }) => diagnostic);
  assert.equal(h.status, 2);
  assert.deepEqual(headers(tscDiagnostics(h.stdout)), inputs.historical.hDiagnostics);
  assert.equal(official.status, officialDiagnostics.length ? 2 : 0);
  for (const row of inputs.historical.byCase)
    assert.deepEqual(
      headers(officialDiagnostics.filter((value) => value.file.startsWith(row.id + "/"))),
      row.official,
      row.id,
    );
  const routes: Record<string, unknown> = {};
  const byCase = inputs.rows.map(({ id, requiresAuthoredError }) => ({
    id,
    requiresAuthoredError,
    official: officialDiagnostics.filter((value) => value.file.startsWith(id + "/")),
    routes: {} as Record<string, unknown>,
  }));
  for (const [role, project, binary] of [
    ["current-source", currentProject, build.cli.path],
    ["published", publicProject, path.join(provider, "node_modules/vize/bin/vize")],
  ]) {
    const mapper = run(
      output,
      `${role}-mapper`,
      native,
      [
        "--runExternalCode",
        "--checkers",
        "1",
        "--noEmit",
        "--pretty",
        "false",
        "--project",
        "tsconfig.mapper.json",
      ],
      project,
    );
    const mapperDiagnostics = tscDiagnostics(mapper.stdout);
    assert.equal(mapper.status, mapperDiagnostics.length ? 2 : 0);
    const cli = run(
      output,
      `${role}-cli`,
      binary,
      [
        "check",
        "--no-config",
        "--format",
        "json",
        "--show-virtual-ts",
        "--tsconfig",
        "tsconfig.json",
        "--corsa-path",
        native,
      ],
      project,
    );
    const cliPacket = packet(cli.stdout, cli.status, project);
    save(path.join(output, `${role}.json`), {
      source,
      tree,
      mapperStatus: mapper.status,
      mapperDiagnostics,
      cliStatus: cli.status,
      cliPacket,
    });
    if (role === "published")
      for (const row of inputs.historical.byCase) {
        assert.deepEqual(
          headers(mapperDiagnostics.filter((value) => value.file.startsWith(row.id + "/"))),
          row.mapper,
          row.id,
        );
        assert.deepEqual(
          cliPacket.diagnostics.filter((value) => value.file.startsWith(row.id + "/")),
          row.cli,
          row.id,
        );
      }
    routes[role] = {
      mapperStatus: mapper.status,
      mapperDiagnostics,
      cliStatus: cli.status,
      cliPacket,
    };
    for (const row of byCase)
      row.routes[role] = {
        mapper: mapperDiagnostics.filter((value) => value.file.startsWith(row.id + "/")),
        cli: cliPacket.diagnostics.filter((value) => value.file.startsWith(row.id + "/")),
      };
    for (const row of inputs.rows)
      captureSlotPacket(
        binary,
        project,
        row.source,
        path.join(output, "protocol", role, row.id),
        `${row.id}/src/Tiny.vue`,
      );
  }
  assert.deepEqual(identify(build.cli.path), build.cli);
  assert.deepEqual(identify(native), nativeIdentity);
  for (const before of providerIdentities)
    assert.deepEqual(identify(path.join(provider, before.file)), {
      path: before.path,
      bytes: before.bytes,
      sha256: before.sha256,
    });
  for (const project of [publicProject, currentProject])
    for (const [file, body] of inputs.files)
      assert.equal(fs.readFileSync(path.join(project, file), "utf8"), body, file);
  save(path.join(output, "summary.json"), {
    schema: 1,
    role: "real current-source baseline over original27; not repair or release qualification",
    source,
    tree,
    driver,
    originalFixtureSource: "190257dbe52a7d4942be4496403a55b32c24426f",
    originalFrameworkSource: "0fb9c041bb5e4bfaa375a0d2dbc6e64508730ddb",
    historicalSummarySha256: inputs.historicalSummarySha256,
    manifest: inputs.manifest,
    cases: 27,
    files: 58,
    stockH: { status: h.status, diagnostics: tscDiagnostics(h.stdout) },
    official: { status: official.status, diagnostics: officialDiagnostics },
    routes,
    byCase,
    directProtocolTransactions: 54,
    timingUsable: false,
    repairQualified: false,
    releaseQualified: false,
  });
}
