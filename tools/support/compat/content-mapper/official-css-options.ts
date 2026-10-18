import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import { fileURLToPath, pathToFileURL } from "node:url";
import {
  loadOfficialFixture,
  officialCases,
  officialDiagnostics,
  assertOfficialDiagnostics,
  officialRunOptions,
  verifyPublicInstallation,
  resolveOfficialNativeImage,
  officialActorInputs,
  type Case,
  type Fixture,
} from "./official-fixtures.ts";
import { preparePublishManifest } from "../../release/smoke_release_install/prepare.ts";
import { derivePublicationPlan, readRawBlob } from "../../release/public_acceptance/plan.ts";
import { verifyPublication } from "../../release/public_acceptance/registry.ts";

const root = fileURLToPath(new URL("../../../../", import.meta.url));
const fixturePath = path.join(root, "tests/_fixtures/content-mapper/escaton-css-options.json");
const fixture = loadOfficialFixture(fixturePath);
const digest = (bytes: string | Uint8Array) => createHash("sha256").update(bytes).digest("hex");
const identity = (file: string) => ({
  path: file,
  bytes: fs.statSync(file).size,
  sha256: digest(fs.readFileSync(file)),
});
const writeJson = (file: string, value: unknown) =>
  fs.writeFileSync(file, JSON.stringify(value, null, 2) + "\n");
const cleanEnv = Object.fromEntries(
  Object.entries(process.env).filter(
    ([key]) =>
      !/^(?:VIZE_|CORSA_|TSGO_|NAPI_RS_)/u.test(key) &&
      key !== "NODE_PATH" &&
      key !== "NODE_OPTIONS",
  ),
);

function run(command: string, args: string[], cwd: string, evidence: string, timeout = 120_000) {
  fs.mkdirSync(evidence, { recursive: true });
  const stdoutFile = path.join(evidence, "stdout.bin"),
    stderrFile = path.join(evidence, "stderr.bin");
  const stdout = fs.openSync(stdoutFile, "w"),
    stderr = fs.openSync(stderrFile, "w");
  let result;
  try {
    result = spawnSync(command, args, {
      cwd,
      env: cleanEnv,
      stdio: ["ignore", stdout, stderr],
      timeout,
    });
  } finally {
    fs.closeSync(stdout);
    fs.closeSync(stderr);
  }
  const outcome = {
    command,
    args,
    cwd,
    status: result.status,
    signal: result.signal,
    error: result.error
      ? Object.fromEntries(
          Object.getOwnPropertyNames(result.error).map((key) => [
            key,
            Reflect.get(result.error!, key),
          ]),
        )
      : null,
    stdout: identity(stdoutFile),
    stderr: identity(stderrFile),
  };
  writeJson(path.join(evidence, "process.json"), outcome);
  return {
    ...outcome,
    stdoutText: fs.readFileSync(stdoutFile, "utf8"),
    stderrText: fs.readFileSync(stderrFile, "utf8"),
  };
}
function successful(result: ReturnType<typeof run>) {
  assert.equal(
    result.status,
    0,
    `failed process: ${result.command}; full evidence ${result.stderr.path}`,
  );
  assert.equal(result.signal, null);
  assert.equal(result.error, null);
}

export function officialCssCases(): Case[] {
  return officialCases(fixture);
}

function check(install: string, row: Case, output: string, suite: Fixture, baseline = false) {
  const project = path.join(output, row.name);
  fs.mkdirSync(project, { recursive: true });
  for (const [file, source] of Object.entries(row.files)) {
    const target = path.join(project, file);
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.writeFileSync(target, source);
  }
  const dependencyLink = path.join(project, "node_modules");
  fs.symlinkSync(
    path.join(install, "node_modules"),
    dependencyLink,
    process.platform === "win32" ? "junction" : "dir",
  );
  writeJson(path.join(project, "inputs.json"), {
    originalIssue: row.original,
    baseline,
    files: Object.fromEntries(
      Object.keys(row.files).map((file) => [file, identity(path.join(project, file))]),
    ),
  });
  const tsc = path.join(install, "node_modules/typescript/bin/tsc");
  const args = row.build
    ? [tsc, "--build", "tsconfig.json", "--runExternalCode", "--pretty", "false"]
    : [tsc, "--runExternalCode", "--noEmit", "-p", "tsconfig.json", "--pretty", "false"];
  const result = run(process.execPath, args, project, path.join(project, "process"));
  const original = suite.originals.find((original) => original.issue === row.original)!;
  const diagnostics = officialDiagnostics(result.stdoutText);
  writeJson(path.join(project, "diagnostics.json"), diagnostics);
  assertOfficialDiagnostics(row, original, baseline, result, diagnostics);
  return {
    name: row.name,
    original: row.original,
    status: result.status,
    diagnostics,
    evidence: project,
  };
}

function install(directory: string, dependencies: Record<string, string>, evidence: string) {
  fs.mkdirSync(directory, { recursive: true });
  writeJson(path.join(directory, "package.json"), { private: true, type: "module", dependencies });
  fs.writeFileSync(path.join(directory, ".npmrc"), "registry=https://registry.npmjs.org/\n");
  successful(
    run(
      "npm",
      [
        "install",
        "--ignore-scripts",
        "--no-audit",
        "--no-fund",
        "--registry=https://registry.npmjs.org",
      ],
      directory,
      evidence,
      300_000,
    ),
  );
  return directory;
}
export async function installedIdentity(directory: string, output: string, probeNative = true) {
  const require = createRequire(path.join(directory, "package.json"));
  const records = ["typescript", "vue", "vize", "@vizejs/native"].map((name) => {
    const file = path.join(directory, "node_modules", name, "package.json");
    const packageRoot = fs.realpathSync.native(path.dirname(file));
    const entry = fs.realpathSync.native(require.resolve(name));
    const relative = path.relative(packageRoot, entry);
    assert.ok(
      relative && !relative.startsWith("..") && !path.isAbsolute(relative),
      `${name} must resolve inside the declared fresh dependency`,
    );
    const manifest = JSON.parse(fs.readFileSync(file, "utf8"));
    fs.mkdirSync(path.join(output, "manifests"), { recursive: true });
    fs.copyFileSync(file, path.join(output, "manifests", name.replaceAll("/", "-") + ".json"));
    return { name, version: manifest.version as string, manifest: identity(file) };
  });
  assert.equal(records[0].version, fixture.typescript);
  assert.equal(records[1].version, fixture.vue);
  const cliManifest = JSON.parse(
    fs.readFileSync(path.join(directory, "node_modules/vize/package.json"), "utf8"),
  );
  const mapperExec = cliManifest.typescript.contentMapper.exec;
  assert.deepEqual(mapperExec, ["node", "./bin/vize", "content-mapper"]);
  const cli = fs.realpathSync.native(path.join(directory, "node_modules/vize", mapperExec[1]));
  const nativeEntry = fs.realpathSync.native(require.resolve("@vizejs/native"));
  const cliNativeEntry = fs.realpathSync.native(createRequire(cli).resolve("@vizejs/native"));
  assert.equal(
    cliNativeEntry,
    nativeEntry,
    "official mapper CLI must resolve the candidate native wrapper",
  );
  const typescript = path.dirname(require.resolve("typescript/package.json"));
  const getExePath = (await import(pathToFileURL(path.join(typescript, "lib/getExePath.js")).href))
    .default as () => string;
  const compiler = getExePath();
  const compilerManifest = path.join(path.dirname(compiler), "../package.json");
  assert.equal(JSON.parse(fs.readFileSync(compilerManifest, "utf8")).version, fixture.typescript);
  fs.copyFileSync(compilerManifest, path.join(output, "manifests", "official-platform.json"));
  const version = run(
    process.execPath,
    [path.join(typescript, "bin/tsc"), "--version"],
    directory,
    path.join(output, "compiler-version"),
  );
  successful(version);
  assert.equal(version.stdoutText, `Version ${fixture.typescript}\n`);
  let nativeImage: string;
  if (probeNative) {
    const probe = run(
      process.execPath,
      [
        "--input-type=commonjs",
        "-e",
        "require('@vizejs/native'); process.stdout.write(JSON.stringify(Object.keys(require.cache).filter(p => p.endsWith('.node'))))",
      ],
      directory,
      path.join(output, "native-load"),
    );
    successful(probe);
    const images = JSON.parse(probe.stdoutText) as string[];
    const nativeImages = images.filter((file) => path.basename(file).startsWith("vize-vitrine."));
    assert.equal(nativeImages.length, 1);
    nativeImage = fs.realpathSync.native(nativeImages[0]);
  } else {
    nativeImage = resolveOfficialNativeImage(directory);
    for (const file of ["native-binding.js", "native-targets.js"])
      fs.copyFileSync(
        path.join(directory, "node_modules/@vizejs/native", file),
        path.join(output, file),
      );
  }
  const target = /^vize-vitrine\.([a-z0-9-]+)\.node$/u.exec(path.basename(nativeImage));
  assert.ok(target, "loaded image must identify its platform package");
  const platformName = `@vizejs/native-${target[1]}`;
  const nativeRequire = createRequire(cliNativeEntry);
  const platformImage = fs.realpathSync.native(nativeRequire.resolve(platformName));
  assert.equal(
    platformImage,
    nativeImage,
    "CLI native wrapper must resolve the loaded platform image",
  );
  const platformManifest = nativeRequire.resolve(`${platformName}/package.json`);
  const platform = JSON.parse(fs.readFileSync(platformManifest, "utf8"));
  assert.equal(platform.name, platformName);
  assert.equal(
    platform.version,
    records.find((record) => record.name === "@vizejs/native")!.version,
  );
  fs.copyFileSync(platformManifest, path.join(output, "manifests", "vize-platform.json"));
  const packet = {
    packages: records,
    mapper: { exec: mapperExec, cli: identity(cli), nativeEntry: identity(cliNativeEntry) },
    platform: {
      name: platformName,
      manifest: identity(platformManifest),
      image: identity(platformImage),
    },
    compiler: identity(compiler),
    nativeImage: identity(nativeImage),
    lock: identity(path.join(directory, "package-lock.json")),
    qualification: probeNative
      ? "Actual installed module resolution and loaded image; no OS child PID or loader instrumentation claim"
      : "Installed/resolved native image and loader text only; loaded-image identity UNAVAILABLE; no direct native require probe",
  };
  writeJson(path.join(output, "installed-identity.json"), packet);
  return packet;
}

function packCurrentPackages(output: string) {
  const directories = ["npm/native/npm/linux-x64-gnu", "npm/native", "npm/cli"].map((file) =>
    path.join(root, file),
  );
  const packs = path.join(output, "packs");
  fs.mkdirSync(packs);
  const dependencies: Record<string, string> = {};
  for (const directory of directories) {
    const before = identity(path.join(directory, "package.json"));
    preparePublishManifest(directory);
    const manifest = JSON.parse(fs.readFileSync(path.join(directory, "package.json"), "utf8"));
    const result = run(
      "npm",
      ["pack", "--ignore-scripts", "--json", "--pack-destination", packs],
      directory,
      path.join(output, "pack-" + manifest.name.replaceAll("/", "-")),
    );
    successful(result);
    const packet = JSON.parse(result.stdoutText);
    assert.equal(packet.length, 1);
    const tarball = path.join(packs, packet[0].filename);
    dependencies[manifest.name] = `file:${tarball}`;
    writeJson(path.join(output, "manifest-" + manifest.name.replaceAll("/", "-") + ".json"), {
      before,
      after: identity(path.join(directory, "package.json")),
      tarball: identity(tarball),
    });
  }
  return dependencies;
}

function failure(name: string, error: unknown) {
  return {
    name,
    error:
      error instanceof Error
        ? Object.fromEntries(
            Object.getOwnPropertyNames(error).map((key) => [key, Reflect.get(error, key)]),
          )
        : String(error),
  };
}

export async function main(args = process.argv.slice(2)) {
  const options = officialRunOptions(args, root, fixturePath);
  const output = path.resolve(
    options.output ?? path.join(root, "target/vize-tests/official-css-options"),
  );
  assert.ok(
    !fs.existsSync(output) || fs.readdirSync(output).length === 0,
    "refuse overwriting previous campaign evidence",
  );
  fs.mkdirSync(output, { recursive: true });
  const pins = { typescript: fixture.typescript, vue: fixture.vue };
  const { fixturePaths, publicVersion } = options;
  const authority = options.candidateOnly
    ? officialActorInputs(JSON.parse(fs.readFileSync(options.publicInputs!, "utf8")))
    : undefined;
  const subjectFixtures: { path: string; sha256: string; head: string }[] = [];
  const suites = fixturePaths.map((file) => {
    if (!authority) return { file, fixture: loadOfficialFixture(file) };
    const relative = path.relative(root, file).split(path.sep).join("/");
    const bytes = readRawBlob(root, authority.subject.head, relative);
    assert.equal(
      digest(fs.readFileSync(file)),
      digest(bytes),
      "F fixture must equal selected H raw source",
    );
    subjectFixtures.push({ path: relative, sha256: digest(bytes), head: authority.subject.head });
    return { file, fixture: loadOfficialFixture(file, bytes.toString("utf8")) };
  });
  const allCases = suites.flatMap((suite) => officialCases(suite.fixture));
  assert.equal(new Set(allCases.map((row) => row.name)).size, allCases.length);
  assert.ok(allCases.length, "nonempty owned fixture cases required");
  if (options.candidateOnly) {
    const { subject: inputs, actor } = authority!;
    assert.equal(inputs.tag, `v${publicVersion}`);
    const head = run("git", ["rev-parse", "HEAD"], root, path.join(output, "public-source"));
    successful(head);
    assert.equal(head.stdoutText.trim(), actor, "checkout must be trusted tooling actor F");
    successful(
      run(
        "git",
        ["--no-replace-objects", "merge-base", "--is-ancestor", inputs.head, actor],
        root,
        path.join(output, "subject-ancestry"),
      ),
    );
    const status = run("git", ["status", "--porcelain"], root, path.join(output, "public-clean"));
    successful(status);
    assert.equal(status.stdoutText, "", "public actor requires its clean exact source");
    successful(
      run(
        "git",
        ["ls-files", "--error-unmatch", "--", ...fixturePaths],
        root,
        path.join(output, "public-owned-fixtures"),
      ),
    );
    const publication = await verifyPublication(derivePublicationPlan(root, inputs.head), inputs);
    writeJson(path.join(output, "public-publication.json"), publication);
    writeJson(path.join(output, "selection.json"), {
      mode: "published-explicit-candidate-only",
      baseline: "NOT EXECUTED: no historical failure credit",
      inputs,
      actor,
      subjectFixtures,
      fixtures: suites.map((suite) => identity(suite.file)),
      expectedCases: allCases.length,
    });
  }
  const baselineVersions = new Map<string, string>();
  const baselineResults = [],
    baselineFailures = [];
  for (const suite of suites) {
    assert.equal(suite.fixture.typescript, pins.typescript);
    assert.equal(suite.fixture.vue, pins.vue);
    if (options.candidateOnly) continue;
    const version = suite.fixture.baselineVize;
    let baseline = baselineVersions.get(version);
    if (!baseline) {
      const suffix = baselineVersions.size ? "-" + version : "";
      baseline = install(
        path.join(output, "baseline-install" + suffix),
        { ...pins, vize: version },
        path.join(output, "baseline-install-process" + suffix),
      );
      const observed = await installedIdentity(
        baseline,
        path.join(output, "baseline-identity" + suffix),
      );
      assert.equal(observed.packages.find((pkg) => pkg.name === "vize")!.version, version);
      baselineVersions.set(version, baseline);
    }
    for (const original of suite.fixture.originals) {
      const row = officialCases(suite.fixture).find(
        (row) => row.name === `issue-${original.issue}`,
      )!;
      try {
        baselineResults.push(
          check(baseline, row, path.join(output, "baseline"), suite.fixture, true),
        );
      } catch (error) {
        baselineFailures.push(failure(row.name, error));
      }
    }
  }
  writeJson(path.join(output, "baseline-results.json"), baselineResults);
  writeJson(path.join(output, "baseline-failures.json"), baselineFailures);
  assert.deepEqual(baselineFailures, [], "all original public failures must reproduce");
  if (options.baselineOnly) return;
  const source = run("git", ["rev-parse", "HEAD"], root, path.join(output, "source-identity"));
  successful(source);
  if (!publicVersion)
    assert.equal(
      source.stdoutText.trim(),
      process.env.EXPECTED_SOURCE_SHA,
      "packed candidate requires its exact reviewed source",
    );
  else assert.match(publicVersion, /^\d+\.\d+\.\d+$/u);
  const candidate = install(
    path.join(output, "candidate-install"),
    { ...pins, ...(publicVersion ? { vize: publicVersion } : packCurrentPackages(output)) },
    path.join(output, "candidate-install-process"),
  );
  const candidateIdentity = await installedIdentity(
    candidate,
    path.join(output, "candidate-identity"),
    !options.candidateOnly,
  );
  if (options.candidateOnly) await verifyPublicInstallation(candidate, output, candidateIdentity);
  if (publicVersion)
    assert.equal(
      candidateIdentity.packages.find((pkg) => pkg.name === "vize")!.version,
      publicVersion,
    );
  else
    assert.equal(
      candidateIdentity.nativeImage.sha256,
      identity(path.join(root, "npm/native/vize-vitrine.linux-x64-gnu.node")).sha256,
      "fresh packed mapper must load the source-built image",
    );
  const results = [],
    failures = [];
  for (const suite of suites)
    for (const row of officialCases(suite.fixture)) {
      try {
        results.push(check(candidate, row, path.join(output, "candidate"), suite.fixture));
      } catch (error) {
        failures.push(failure(row.name, error));
      }
    }
  writeJson(path.join(output, "results.json"), {
    source: source.stdoutText.trim(),
    ...(authority ? { actor: authority.actor, subject: authority.subject, subjectFixtures } : {}),
    kind: publicVersion ? "released-package" : "source-built-packed-candidate",
    fixture: identity(fixturePaths[0]),
    fixtures: suites.map((suite) => identity(suite.file)),
    results,
    failures,
  });
  assert.deepEqual(
    failures,
    [],
    "official host matrix failed; preserve the complete output before review",
  );
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main().catch((error) => {
    console.error(error);
    process.exitCode = 1;
  });
}
