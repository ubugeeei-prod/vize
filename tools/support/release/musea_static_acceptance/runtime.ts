import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { publicNative } from "../public_acceptance/installed.ts";
import {
  assertFiles,
  assertSuiteCensus,
  authenticate,
  digest,
  rejectSubstitution,
} from "./custody.ts";

rejectSubstitution();
const root = fs.realpathSync(process.cwd()),
  output = path.resolve(process.env.PUBLIC_OUTPUT!);
const read = (file: string) => JSON.parse(fs.readFileSync(path.join(output, file), "utf8"));
const preparation = read("preparation.json"),
  publication = read("publication.json");
assert.equal(root, preparation.consumer);
assert.equal(publication.plan.head, preparation.source.head);
assert.equal(publication.plan.parentCut, preparation.source.cut);
assert.equal(publication.plan.version, preparation.version);
assert.equal(digest(fs.readFileSync(path.join(root, "package.json"))), preparation.manifestSha256);
const files = [
  ...preparation.fixtures.records,
  ...preparation.fixtures.adapters,
  ...preparation.harness,
];
assertFiles(root, files);
const lockBytes = fs.readFileSync(path.join(root, "package-lock.json"));
fs.writeFileSync(path.join(output, "package-lock.json"), lockBytes);
const receipt: Record<string, unknown> = {
  schema: "vize-public-musea-static-names-v1",
  source: preparation.source,
  version: preparation.version,
  observer: preparation.observer,
  platform: process.platform,
  arch: process.arch,
  preparation,
  success: false,
  scope:
    "Installed public Linux native/compiler/plugin, built HTTP colon/collision and ordinary previews plus six Self variants/globals/actual Document identity. No aggregate, literal-object-key, VRT, hosted companion, LSP or broader n8n credit.",
};
const persist = () =>
  fs.writeFileSync(path.join(output, "receipt.json"), JSON.stringify(receipt, null, 2) + "\n");
persist();
try {
  const custody = await authenticate(preparation.version, output);
  for (const record of custody.packages)
    assert.ok(
      publication.plan.npm.some(
        (target: { name: string; version: string }) =>
          target.name === record.name && target.version === record.version,
      ),
    );
  fs.writeFileSync(
    path.join(output, "archive-custody.json"),
    JSON.stringify(custody, null, 2) + "\n",
  );
  receipt.custody = custody;
  persist();
  await new Promise<void>((resolve, reject) => {
    const log = fs.openSync(path.join(output, "original-suite.log"), "w");
    const child = spawn(
      process.execPath,
      ["--test", "--test-reporter=tap", "--test-concurrency=1", "musea_static_acceptance/suite.ts"],
      {
        cwd: root,
        env: { ...process.env, VIZE_MUSEA_NATIVE_BROWSER_TESTS: "1" },
        stdio: ["ignore", log, log],
        timeout: 360_000,
      },
    );
    fs.closeSync(log);
    child.once("error", reject);
    child.once("exit", (code, signal) =>
      code === 0 && signal === null
        ? resolve()
        : reject(new Error(`Original installed suite failed: code=${code}, signal=${signal}`)),
    );
  });
  const suiteLog = fs.readFileSync(path.join(output, "original-suite.log"), "utf8");
  assertSuiteCensus(suiteLog);
  const artifactRoot = path.join(root, "authority/artifacts");
  const observations = (directory: string) =>
    JSON.parse(fs.readFileSync(path.join(artifactRoot, directory, "observations.json"), "utf8"));
  const inline = observations("musea-native-static"),
    binding = observations("musea-native-bindings"),
    ordinary = observations("musea-static-ordinary");
  assert.deepEqual(
    inline.observations.map((row: { phase: string }) => row.phase),
    [
      "built-native-defaults",
      "built-native-globals-no-remount",
      "built-native-deep-link-reload",
      "built-native-literal-colon",
    ],
  );
  assert.deepEqual(
    inline.observations.map((row: { frames: unknown[] }) => row.frames.length),
    [6, 6, 6, 2],
  );
  for (const row of inline.observations)
    for (const frame of row.frames)
      assert.match(
        frame.html ?? frame.rendered?.html ?? "",
        /^<html\b/u,
        "complete observed document",
      );
  assert.equal(binding.observations.length, 4);
  assert.deepEqual(
    binding.observations.map((row: { kind: string; rendered: { variant: string } }) => [
      row.kind,
      row.rendered.variant,
    ]),
    [
      ["collision", "State enabled"],
      ["collision", "State-enabled"],
      ["ordinary", "State enabled"],
      ["ordinary", "State disabled"],
    ],
  );
  assert.deepEqual(
    ordinary.observations.map((row: { rendered: { variant: string } }) => row.rendered.variant),
    ["Default", "State enabled"],
  );
  const executed = read("executed-provider.json");
  assert.equal(executed.complete, true);
  assert.deepEqual(executed.loaded, custody.loaded);
  assert.deepEqual(publicNative(preparation.version).loaded, custody.loaded);
  for (const record of custody.packages) assertFiles(root, record.files);
  assertFiles(root, files);
  assert.deepEqual(fs.readFileSync(path.join(root, "package-lock.json")), lockBytes);
  receipt.observations = { inline, binding, ordinary };
  receipt.executed = executed;
  receipt.denominator = {
    reportedAndOrdinaryStaticPreviews: 8,
    selfVariants: 6,
    selfObservationPhases: 3,
    originalBrowserLaws: 2,
    additiveOrdinaryLaw: 1,
    skipped: 0,
  };
  receipt.lockSha256 = digest(lockBytes);
  receipt.success = true;
  persist();
  console.log(
    JSON.stringify({ source: preparation.source, denominator: receipt.denominator, success: true }),
  );
} catch (error) {
  receipt.failure = {
    message: String(error),
    stack: error instanceof Error ? error.stack : undefined,
  };
  persist();
  throw error;
} finally {
  const artifacts = path.join(root, "authority/artifacts");
  if (fs.existsSync(artifacts))
    fs.cpSync(artifacts, path.join(output, "physical-observations"), { recursive: true });
  fs.cpSync(path.join(root, "originals"), path.join(output, "original-source-laws"), {
    recursive: true,
  });
  fs.cpSync(path.join(root, "drivers"), path.join(output, "adapted-source-laws"), {
    recursive: true,
  });
  fs.cpSync(path.join(root, "authority/examples"), path.join(output, "original-examples"), {
    recursive: true,
  });
  fs.cpSync(path.join(root, "authority/tests"), path.join(output, "original-fixtures"), {
    recursive: true,
  });
}
