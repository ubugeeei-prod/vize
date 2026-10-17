import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { rejectOverrides } from "./public_acceptance/installed.ts";
import { publicReader } from "./public_acceptance/http.ts";

const repository = "ubugeeei-prod/vize";
// #9122 carries #8940's Windows TypeScript project-path repair from #9085.
const fixSource = "8009e15b7cd2b279dfa2a8ad8b9dfa73e628bfd4";
const object = (value: unknown): Record<string, unknown> => {
  assert.ok(value && typeof value === "object" && !Array.isArray(value), "object required");
  return value as Record<string, unknown>;
};
const sha256 = (file: string) => createHash("sha256").update(fs.readFileSync(file)).digest("hex");

/** Registry/source observations; signature verification is not claimed here. */
export function verifyPackageIdentity(
  name: string,
  version: string,
  head: string,
  metadataValue: unknown,
  statementValue: unknown,
  installedValue: unknown,
) {
  const metadata = object(metadataValue),
    statement = object(statementValue);
  const installed = object(installedValue),
    dist = object(metadata.dist);
  assert.equal(metadata.name, name);
  assert.equal(metadata.version, version);
  assert.equal(installed.version, version);
  assert.equal(installed.link, undefined);
  assert.equal(installed.integrity, dist.integrity);
  assert.equal(installed.resolved, dist.tarball);
  assert.ok(typeof dist.tarball === "string");
  assert.equal(new URL(dist.tarball).origin, "https://registry.npmjs.org");
  assert.ok(typeof dist.integrity === "string");
  assert.match(dist.integrity, /^sha512-[A-Za-z0-9+/]{86}==$/u);
  assert.equal(statement.predicateType, "https://slsa.dev/provenance/v1");
  assert.ok(Array.isArray(statement.subject));
  assert.equal(statement.subject.length, 1);
  const subject = object(statement.subject[0]);
  assert.equal(decodeURIComponent(String(subject.name)), `pkg:npm/${name}@${version}`);
  assert.equal(
    object(subject.digest).sha512,
    Buffer.from(dist.integrity.slice(7), "base64").toString("hex"),
  );
  const predicate = object(statement.predicate);
  const definition = object(predicate.buildDefinition);
  const workflow = object(object(definition.externalParameters).workflow);
  assert.equal(workflow.repository, `https://github.com/${repository}`);
  assert.equal(workflow.path, ".github/workflows/release.yml");
  assert.equal(workflow.ref, `refs/heads/release/v${version}`);
  const dependencies = definition.resolvedDependencies;
  assert.ok(Array.isArray(dependencies));
  assert.ok(
    dependencies.some((value: unknown) => {
      const dependency = object(value);
      return (
        dependency.uri === `git+https://github.com/${repository}@refs/heads/release/v${version}` &&
        object(dependency.digest).gitCommit === head
      );
    }),
    "npm provenance must name the official tagged source",
  );
  return {
    name,
    version,
    tarball: dist.tarball,
    integrity: dist.integrity,
    source: head,
    invocation: object(object(predicate.runDetails).metadata).invocationId,
  };
}

async function main() {
  assert.equal(process.platform, "win32", "actual Windows host required");
  assert.ok(["22", "24"].includes(process.versions.node.split(".")[0]));
  assert.equal(process.arch, "x64");
  rejectOverrides();
  for (const name of [
    "NODE_PATH",
    "VIZE_BINDING_PATH",
    "VIZE_CORSA_DEV_PATHS",
    "CARGO_BIN_EXE_vize",
    "SOURCE_SHA",
    "VIZE_TEST_REQUIRE_TSGO",
    "VIZE_TEST_DISABLE_TSGO",
  ]) {
    assert.ok(!process.env[name], `runtime override must be absent: ${name}`);
  }
  const version = process.env.PUBLIC_VERSION ?? "0.457.0";
  assert.match(version, /^0\.[1-9][0-9]*\.0$/u);
  assert.ok(process.env.PUBLIC_OUTPUT, "PUBLIC_OUTPUT required");
  const output = path.resolve(process.env.PUBLIC_OUTPUT);
  fs.mkdirSync(output, { recursive: true });
  const save = (name: string, value: unknown) =>
    fs.writeFileSync(path.join(output, name), JSON.stringify(value, null, 2) + "\n");
  const github = async (endpoint: string) => {
    const response = await fetch(`https://api.github.com/repos/${repository}/${endpoint}`, {
      headers: {
        Accept: "application/vnd.github+json",
        ...(process.env.GITHUB_TOKEN
          ? { Authorization: `Bearer ${process.env.GITHUB_TOKEN}` }
          : {}),
      },
      signal: AbortSignal.timeout(30_000),
    });
    assert.ok(response.ok, `GitHub ${response.status}: ${endpoint}`);
    return object(await response.json());
  };
  const release = await github(`releases/tags/v${version}`);
  assert.equal(release.tag_name, `v${version}`);
  assert.equal(release.draft, false);
  assert.equal(release.prerelease, false);
  assert.equal(typeof release.published_at, "string");
  const tagged = await github(`commits/v${version}`);
  const head = typeof tagged.sha === "string" ? tagged.sha : "";
  assert.match(head, /^[0-9a-f]{40}$/u);
  const comparison = await github(`compare/${fixSource}...${head}`);
  assert.equal(comparison.behind_by, 0);
  assert.ok(comparison.status === "ahead" || comparison.status === "identical");
  save("published-source.json", {
    version,
    source: head,
    fixSource,
    releaseId: release.id,
    releaseUrl: release.html_url,
    publishedAt: release.published_at,
    scope: "Windows installed npm CLI hover; not full Release workflow acceptance",
  });
  const project = path.join(
    fs.mkdtempSync(path.join(process.env.RUNNER_TEMP ?? os.tmpdir(), "vize-public-hover-")),
    "vite vue 日本語",
  );
  fs.mkdirSync(project);
  assert.ok(!path.relative(process.cwd(), project).startsWith(`node_modules${path.sep}`));
  assert.ok(
    path.relative(process.cwd(), project).startsWith(".."),
    "fresh consumer must be outside checkout",
  );
  const fixtures: unknown = createRequire(import.meta.url)(
    "../compat/npm/smoke-release-init-shapes.mjs",
  );
  const shape = object(object(object(fixtures).PROJECT_SHAPES)["vite-vue-ts"]);
  assert.ok(typeof shape.files === "function");
  const authoredValue: unknown = shape.files({
    vue: "3.5.34",
    typescript: "6.0.3",
    vite: "8.0.0",
  });
  const authored = object(authoredValue);
  const write = (name: string, text: string) => {
    const target = path.join(project, name);
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.writeFileSync(target, text);
  };
  for (const [name, text] of Object.entries(authored)) {
    assert.ok(typeof text === "string");
    write(name, text);
  }
  save("authored-fixture.json", authored);
  const env = { ...process.env };
  delete env.GITHUB_TOKEN;
  delete env.GH_TOKEN;
  const run = (name: string, command: string, args: string[]) => {
    const result = spawnSync(command, args, {
      cwd: project,
      env,
      encoding: "utf8",
      timeout: 240_000,
      maxBuffer: 16 * 1024 * 1024,
    });
    save(`${name}.json`, {
      command,
      args,
      cwd: project,
      status: result.status,
      signal: result.signal,
      error: result.error?.message,
      stdout: result.stdout,
      stderr: result.stderr,
    });
    return result;
  };
  const npmCli = path.join(
    path.dirname(process.execPath),
    "node_modules",
    "npm",
    "bin",
    "npm-cli.js",
  );
  assert.ok(fs.existsSync(npmCli), "setup-node's installed npm CLI required");
  const install = run("install", process.execPath, [
    npmCli,
    "install",
    "--save-exact",
    "--ignore-scripts",
    "--no-audit",
    "--no-fund",
    "--registry=https://registry.npmjs.org",
    `vize@${version}`,
    `@vizejs/native@${version}`,
    `@vizejs/vite-plugin@${version}`,
  ]);
  assert.equal(install.status, 0, install.stderr);
  const cliManifest = path.join(project, "node_modules/vize/package.json");
  const cliRoot = fs.realpathSync(path.dirname(cliManifest));
  const require = createRequire(cliManifest);
  const nativeManifest = path.join(project, "node_modules/@vizejs/native/package.json");
  const nativeRequire = createRequire(nativeManifest);
  const platformName = `@vizejs/native-win32-${process.arch}-msvc`;
  const binding = fs.realpathSync(nativeRequire.resolve(platformName));
  assert.equal(path.extname(binding), ".node", "actual native provider binary required");
  const platformRoot = path.dirname(binding);
  const lock = object(JSON.parse(fs.readFileSync(path.join(project, "package-lock.json"), "utf8")));
  const locked = object(lock.packages);
  const reader = publicReader();
  const packages = [];
  for (const [name, directory] of [
    ["vize", cliRoot],
    ["@vizejs/native", path.dirname(nativeManifest)],
    [platformName, platformRoot],
    ["@vizejs/vite-plugin", path.join(project, "node_modules/@vizejs/vite-plugin")],
  ]) {
    assert.ok(fs.realpathSync(directory).startsWith(fs.realpathSync(project) + path.sep));
    const manifest = object(
      JSON.parse(fs.readFileSync(path.join(directory, "package.json"), "utf8")),
    );
    assert.equal(manifest.name, name);
    assert.equal(manifest.version, version);
    const metadata = await reader.json(
      `https://registry.npmjs.org/${encodeURIComponent(name)}/${version}`,
    );
    const attestations = await reader.json(
      `https://registry.npmjs.org/-/npm/v1/attestations/${encodeURIComponent(name)}@${version}`,
    );
    assert.ok(Array.isArray(attestations.attestations));
    const attestation = attestations.attestations
      .map(object)
      .find((value) => value.predicateType === "https://slsa.dev/provenance/v1");
    assert.ok(attestation, `source provenance required: ${name}`);
    const envelope = object(object(attestation.bundle).dsseEnvelope);
    assert.ok(typeof envelope.payload === "string");
    const statement: unknown = JSON.parse(Buffer.from(envelope.payload, "base64").toString("utf8"));
    const identity = verifyPackageIdentity(
      name,
      version,
      head,
      metadata,
      statement,
      locked[`node_modules/${name}`],
    );
    packages.push({ ...identity, directory });
    save("packages.json", packages);
  }
  const native = require("@vizejs/native");
  assert.equal(require.cache[binding]?.loaded, true, "public platform binding must actually load");
  assert.equal(require.cache[binding]?.exports, native);
  const cli = path.join(cliRoot, "bin/vize");
  const corsaManifest = require.resolve(
    `@typescript/typescript-win32-${process.arch}/package.json`,
  );
  const corsa = path.join(path.dirname(corsaManifest), "lib/tsc.exe");
  assert.ok(fs.existsSync(corsa));
  const corsaVersion = object(JSON.parse(fs.readFileSync(corsaManifest, "utf8"))).version;
  const cliPackage = object(JSON.parse(fs.readFileSync(cliManifest, "utf8")));
  assert.equal(
    corsaVersion,
    object(cliPackage.optionalDependencies)[`@typescript/typescript-win32-${process.arch}`],
  );
  const identity = {
    version,
    source: head,
    project,
    node: process.version,
    platform: process.platform,
    arch: process.arch,
    cli: { path: cli, sha256: sha256(cli) },
    native: { path: binding, sha256: sha256(binding) },
    corsa: { path: corsa, version: corsaVersion, sha256: sha256(corsa) },
  };
  save("installed-identity.json", identity);
  assert.ok(
    Array.isArray(shape.initFlags) &&
      shape.initFlags.every((value: unknown) => typeof value === "string"),
  );
  const initialized = run("init", process.execPath, [
    cli,
    "init",
    ...shape.initFlags,
    "--no-install",
  ]);
  assert.equal(initialized.status, 0, initialized.stderr);
  const check = (name: string, expected: number) => {
    const result = run(name, process.execPath, [cli, "check", "--format", "json", "--quiet"]);
    assert.equal(result.status, expected ? 1 : 0, result.stderr);
    const report = object(JSON.parse(result.stdout));
    assert.equal(report.errorCount, expected);
    assert.ok(Array.isArray(report.files));
    const files = report.files.map(object).filter((file) => {
      assert.ok(Array.isArray(file.diagnostics));
      return file.diagnostics.length > 0;
    });
    assert.deepEqual(
      files,
      expected
        ? [
            {
              file: path.join("src", "App.vue"),
              diagnostics: ["error:9:7 [TS2322] Type 'number' is not assignable to type 'string'."],
            },
          ]
        : [],
      "complete diagnostic vector required",
    );
    assert.equal(report.warningCount, 0);
  };
  check("clean", 0);
  const app = authored["src/App.vue"];
  assert.ok(typeof app === "string");
  const broken = app.replace("</script>", "const rejected: string = 42;\n</script>");
  assert.notEqual(broken, app);
  save("diagnostic-control-source.json", { clean: app, broken });
  write("src/App.vue", broken);
  check("broken", 1);
  write("src/App.vue", app);
  check("repaired", 0);
  env.VIZE_SMOKE_LSP_LOG_DIR = path.join(output, "lsp");
  const oracle = fileURLToPath(
    new URL("../compat/npm/smoke-release-lsp-session.ts", import.meta.url),
  );
  const session = run("lsp", process.execPath, [oracle, project]);
  assert.equal(session.status, 0, `${session.stdout}\n${session.stderr}`);
  save("accepted.json", {
    ...identity,
    packages,
    oracle: { path: oracle, sha256: sha256(oracle) },
    diagnosticControls: ["clean", "TS2322", "repaired"],
    success: true,
  });
  console.log(JSON.stringify({ ...identity, success: true }));
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) await main();
