import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  appendFileSync,
  copyFileSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  realpathSync,
  writeFileSync,
} from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { publicInputs } from "./inputs.ts";
import { rejectOverrides } from "./installed.ts";
import { derivePublicationPlan, readRawBlob } from "./plan.ts";
import { verifyPublication } from "./registry.ts";
import { materializeVrtInputs, vrtHarness } from "./vrt_fixtures.ts";
import { assertVrtCustody } from "./vrt_artifacts.ts";
import { materializeLiteralVrtInputs } from "./vrt_literal.ts";
import { assertLspSourceRoot, prepareLspSource } from "./lsp_source.ts";
import { archiveValidatorSha256, assertInstalledLsp } from "./lsp.ts";
import { rejectLspOverrides } from "./lsp_assets.ts";
import {
  assertActorIdentity,
  assertActorAuthority,
  assertAxeArchive,
  axePolicy,
  installedAxe,
  retainInstalledInputs,
} from "./a11y.ts";
import { readArchive } from "../consumer_recovery/archive.ts";

rejectOverrides();
rejectLspOverrides();
const source = publicInputs(process.env);
assert.equal(process.env.GITHUB_REPOSITORY, "ubugeeei-prod/vize");
assert.equal(process.env.GITHUB_REF, "refs/heads/main", "dispatch reviewed protected main only");
const root = realpathSync(process.cwd());
const toolDirectory = fileURLToPath(new URL("./", import.meta.url));
const digest = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
const output = path.resolve(process.env.PUBLIC_OUTPUT!);
const save = (name: string, packet: unknown) =>
  writeFileSync(path.join(output, name), JSON.stringify(packet, null, 2) + "\n", { flag: "wx" });
const load = (name: string) => JSON.parse(readFileSync(path.join(output, name), "utf8"));
const authority = () =>
  readdirSync(toolDirectory)
    .filter((file) => file.endsWith(".ts"))
    .sort()
    .map((file) => ({ file, sha256: digest(readFileSync(path.join(toolDirectory, file))) }));
function actorPolicy(consumer: string) {
  const head = execFileSync("git", ["--no-replace-objects", "-C", root, "rev-parse", "HEAD"], {
    encoding: "utf8",
  }).trim();
  const identity = assertActorIdentity(head, process.env);
  const actorAuthority = authority();
  assertActorAuthority(actorAuthority, (file) =>
    readRawBlob(root, head, `tools/support/release/public_acceptance/${file}`),
  );
  const workflow = ".github/workflows/release-public-acceptance.yml";
  const workflowSha256 = digest(readRawBlob(root, head, workflow));
  assert.equal(digest(readFileSync(path.join(root, workflow))), workflowSha256);
  return {
    identity,
    workflow: { file: workflow, sha256: workflowSha256 },
    frozenProductSource: source,
    frozenInputAuthority:
      "fixtures, vrtInputs, trustedInputs and lsp retain raw frozen H provenance",
    generatedConsumerRecipe: {
      commit: head,
      files: ["package.json", "preview.setup.ts", "preview.one-argument.ts"],
      requestedPackageSha256: digest(readFileSync(path.join(consumer, "package.json"))),
      dependencyDelta: { ...axePolicy, reason: "required by the positive hosted A11y control" },
      reviewedDependencyLock: {
        file: "pnpm-lock.yaml",
        sha256: digest(readRawBlob(root, head, "pnpm-lock.yaml")),
        importer: "npm/builder/vite-musea",
      },
    },
  };
}
const consumerHarness = [
  "a11y.ts",
  "installed.ts",
  "native.ts",
  "browser.ts",
  "gallery.ts",
  "browser_observation.ts",
  "vrt_secrets.ts",
  "vrt_retained.ts",
  "vrt_retained_scan.ts",
  ...vrtHarness,
  "vrt_tunnel.ts",
  "vrt_tunnel_process.ts",
  "vrt_service_announcement.ts",
  "vrt_service_process.ts",
  "vrt_trusted_surface.ts",
  "vrt_literal.ts",
  "vrt_trusted_network.ts",
  "vrt_trusted_ui.ts",
  "vrt_trusted_observation.ts",
  "vrt_trusted_report_isolation.ts",
  "vrt_trusted_failures.ts",
  "vrt_trusted_gallery.ts",
  "vrt_trusted_color_build.ts",
  "vrt_trusted_color_capture.ts",
  "vrt_trusted_colors.ts",
];

if (process.argv[2] === "prepare") {
  mkdirSync(output, { recursive: true });
  save("request.json", { source, requestedAt: new Date().toISOString() });
  const plan = derivePublicationPlan(root, source.head);
  const publication = await verifyPublication(plan, source);
  save("publication.json", publication);
  const temporary = realpathSync(process.env.RUNNER_TEMP!);
  const consumer = mkdtempSync(path.join(temporary, "vize-public-consumer-"));
  assert.ok(!consumer.startsWith(root + path.sep));
  const version = plan.version;
  const lsp = prepareLspSource(root, source.head, consumer, output);
  assert.equal(lsp.metadata.package.version, version);
  const dependencies = Object.fromEntries(
    [
      "vize",
      "@vizejs/native",
      "@vizejs/native-linux-x64-gnu",
      "@vizejs/vite-plugin",
      "@vizejs/vite-plugin-musea",
      "@vizejs/unplugin",
    ].map((name) => [name, version]),
  );
  writeFileSync(
    path.join(consumer, "package.json"),
    JSON.stringify(
      {
        private: true,
        type: "module",
        dependencies: {
          ...dependencies,
          [lsp.metadata.provider.name]: lsp.metadata.provider.version,
          vite: "8.0.0",
          vue: "3.5.35",
          "vue-router": "4.5.1",
          playwright: "1.62.1",
          [axePolicy.name]: axePolicy.version,
        },
      },
      null,
      2,
    ) + "\n",
    { flag: "wx" },
  );
  copyFileSync(path.join(consumer, "package.json"), path.join(output, "consumer-package.json"));
  const fixturePrefix = "examples/vite-musea/";
  const paths = execFileSync(
    "git",
    [
      "--no-replace-objects",
      "-C",
      root,
      "ls-tree",
      "-r",
      "--name-only",
      source.head,
      fixturePrefix + "src",
    ],
    { encoding: "utf8" },
  )
    .trim()
    .split("\n");
  paths.push(fixturePrefix + "musea.preview.ts", fixturePrefix + "index.html");
  const fixtures = paths.map((file) => {
    assert.ok(file.startsWith(fixturePrefix));
    const bytes = readRawBlob(root, source.head, file);
    const target = path.join(consumer, file.slice(fixturePrefix.length));
    mkdirSync(path.dirname(target), { recursive: true });
    writeFileSync(target, bytes, { flag: "wx" });
    return { file, sha256: digest(bytes) };
  });
  writeFileSync(
    path.join(consumer, "preview.setup.ts"),
    `import setup from './musea.preview.ts';
export default function publicSetup(app, context) {
  if (!Reflect.has(window, '__publicDocumentToken'))
    Reflect.set(window, '__publicDocumentToken', crypto.randomUUID());
  Reflect.set(window, '__publicSetupCalls', (Reflect.get(window, '__publicSetupCalls') || 0) + 1);
  if (!Reflect.has(window, '__publicFirstGlobals'))
    Reflect.set(window, '__publicFirstGlobals', context ? { ...context.globals.value } : null);
  setup(app, context);
}
`,
    { flag: "wx" },
  );
  writeFileSync(
    path.join(consumer, "preview.one-argument.ts"),
    `export default function setup(app) {
  Reflect.set(window, '__publicLegacySetupCalls', (Reflect.get(window, '__publicLegacySetupCalls') || 0) + 1);
  Reflect.set(window, '__publicLegacyArguments', arguments.length);
  app.provide('public-legacy-setup', true);
}
`,
    { flag: "wx" },
  );
  for (const name of consumerHarness) {
    copyFileSync(path.join(toolDirectory, name), path.join(consumer, name));
  }
  const vrtInputs = materializeVrtInputs(consumer, (file) => readRawBlob(root, source.head, file));
  const trustedInputs = materializeLiteralVrtInputs(consumer, (file) =>
    readRawBlob(root, source.head, file),
  );
  const consumerSources = [
    "package.json",
    "preview.setup.ts",
    "preview.one-argument.ts",
    ...fixtures.map((fixture) => fixture.file.slice(fixturePrefix.length)),
    ...vrtInputs.map((entry) => entry.file),
    ...trustedInputs.map((entry) => entry.file),
  ].map((file) => ({ file, sha256: digest(readFileSync(path.join(consumer, file))) }));
  const policy = actorPolicy(consumer);
  save("preparation.json", {
    source,
    consumer,
    version,
    lsp,
    archiveValidatorSha256: archiveValidatorSha256(),
    fixtures,
    vrtInputs,
    trustedInputs,
    consumerSources,
    authority: authority(),
    actorPolicy: policy,
    toolingCommit: policy.identity.commit,
  });
  assert.ok(!/[\r\n]/u.test(consumer));
  appendFileSync(
    process.env.GITHUB_ENV!,
    `PUBLIC_CONSUMER=${consumer}\nPUBLIC_VERSION=${version}\nPUBLIC_LSP_TOOLCHAIN=${lsp.metadata.toolchain}\n`,
  );
  console.log(JSON.stringify({ source, consumer, version, status: "prepared-public-observation" }));
} else if (process.argv[2] === "installed") {
  const preparation = load("preparation.json");
  assert.deepEqual(preparation.source, source);
  const consumer = realpathSync(preparation.consumer);
  assert.equal(consumer, preparation.consumer);
  assert.ok(!consumer.startsWith(root + path.sep));
  const inputs = retainInstalledInputs(consumer, output);
  try {
    assert.deepEqual(actorPolicy(consumer), preparation.actorPolicy);
    const axe = installedAxe(consumer);
    const archive = await readArchive(axePolicy.resolved, axePolicy.integrity);
    assertAxeArchive(archive, axe);
    assert.equal(
      axe.requestedSha256,
      preparation.consumerSources.find((entry: { file: string }) => entry.file === "package.json")
        ?.sha256,
    );
    save("installation.json", { inputs, axe, archive, success: true });
  } catch (error) {
    save("installation.json", { inputs, error: String(error), success: false });
    throw error;
  }
} else if (process.argv[2] === "finish") {
  const preparation = load("preparation.json");
  assert.deepEqual(preparation.source, source);
  assert.deepEqual(actorPolicy(preparation.consumer), preparation.actorPolicy);
  assert.deepEqual(authority(), preparation.authority);
  assertLspSourceRoot(preparation.lsp, { checkout: root, consumer: preparation.consumer, output });
  for (const entry of preparation.consumerSources) {
    assert.equal(
      digest(readFileSync(path.join(preparation.consumer, entry.file))),
      entry.sha256,
      `consumer source unchanged: ${entry.file}`,
    );
  }
  for (const file of consumerHarness) {
    assert.equal(
      digest(readFileSync(path.join(preparation.consumer, file))),
      preparation.authority.find((entry: { file: string }) => entry.file === file)?.sha256,
      `executed consumer harness unchanged: ${file}`,
    );
  }
  const native = load("native.json"),
    musea = load("musea.json"),
    vrt = load("vrt.json");
  const lsp = await assertInstalledLsp(output);
  for (const receipt of [native, musea, vrt]) {
    assert.equal(receipt.version, preparation.version);
    assert.equal(receipt.success, true);
  }
  const publication = load("publication.json");
  for (const item of native.packages) {
    const publicItem = publication.npm.find((entry: { name: string }) => entry.name === item.name);
    assert.ok(publicItem, item.name);
    assert.equal(item.integrity, publicItem.integrity);
    assert.equal(item.resolved, publicItem.tarball);
  }
  assert.deepEqual(musea.packages, native.packages, "both probes use the same public installation");
  assert.deepEqual(
    musea.loaded,
    native.loaded,
    "both probes loaded the same native provider bytes",
  );
  assertVrtCustody(vrt, native);
  const lock = readFileSync(path.join(preparation.consumer, "package-lock.json"));
  const installation = load("installation.json");
  assert.equal(installation.success, true);
  assert.equal(digest(lock), installation.axe.lockSha256);
  assert.deepEqual(installedAxe(preparation.consumer), installation.axe);
  assert.equal(digest(readFileSync(path.join(output, "package-lock.json"))), digest(lock));
  save("acceptance.json", {
    schema: "vize-public-third-party-release-v1",
    source,
    checkedAt: new Date().toISOString(),
    preparation,
    publication,
    native,
    musea,
    vrt,
    lsp,
    lockSha256: digest(lock),
    success: true,
    scope:
      "Postpublication exact public native/Musea/VRT and original 32 owned event-rename contexts, plus mandatory planned channels. Stock-host null remains a separate limitation. Optional Open VSX is recorded separately. No cryptographic signature claim.",
  });
  console.log(JSON.stringify({ source, version: preparation.version, success: true }));
} else {
  throw new Error("use prepare, installed or finish");
}
