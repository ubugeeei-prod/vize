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

rejectOverrides();
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
const consumerHarness = [
  "installed.ts",
  "native.ts",
  "browser.ts",
  "gallery.ts",
  "browser_observation.ts",
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
          vite: "8.0.0",
          vue: "3.5.35",
          "vue-router": "4.5.1",
          playwright: "1.62.1",
        },
      },
      null,
      2,
    ) + "\n",
    { flag: "wx" },
  );
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
  const consumerSources = [
    "package.json",
    "preview.setup.ts",
    "preview.one-argument.ts",
    ...fixtures.map((fixture) => fixture.file.slice(fixturePrefix.length)),
  ].map((file) => ({ file, sha256: digest(readFileSync(path.join(consumer, file))) }));
  save("preparation.json", {
    source,
    consumer,
    version,
    fixtures,
    consumerSources,
    authority: authority(),
    toolingCommit: execFileSync("git", ["--no-replace-objects", "-C", root, "rev-parse", "HEAD"], {
      encoding: "utf8",
    }).trim(),
  });
  assert.ok(!/[\r\n]/u.test(consumer));
  appendFileSync(
    process.env.GITHUB_ENV!,
    `PUBLIC_CONSUMER=${consumer}\nPUBLIC_VERSION=${version}\n`,
  );
  console.log(JSON.stringify({ source, consumer, version, status: "prepared-public-observation" }));
} else if (process.argv[2] === "finish") {
  const preparation = load("preparation.json");
  assert.deepEqual(preparation.source, source);
  assert.deepEqual(authority(), preparation.authority);
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
    musea = load("musea.json");
  for (const receipt of [native, musea]) {
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
  const lock = readFileSync(path.join(preparation.consumer, "package-lock.json"));
  copyFileSync(
    path.join(preparation.consumer, "package-lock.json"),
    path.join(output, "package-lock.json"),
  );
  save("acceptance.json", {
    schema: "vize-public-third-party-release-v1",
    source,
    checkedAt: new Date().toISOString(),
    preparation,
    publication,
    native,
    musea,
    lockSha256: digest(lock),
    success: true,
    scope:
      "Postpublication exact public npm native/Musea cases and mandatory planned channel observations. Optional Open VSX is recorded separately. No cryptographic signature claim.",
  });
  console.log(JSON.stringify({ source, version: preparation.version, success: true }));
} else {
  throw new Error("use prepare or finish");
}
