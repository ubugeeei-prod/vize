import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { publicNative } from "../public_acceptance/installed.ts";
import {
  assertFiles,
  authenticate,
  digest,
  rejectSubstitution,
} from "../musea_static_acceptance/custody.ts";

import { assertSuiteCensus } from "./fixtures.ts";
import { assertObservations } from "./observations.ts";

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
  schema: "vize-public-musea-literal-names-v1",
  source: preparation.source,
  version: preparation.version,
  observer: preparation.observer,
  platform: process.platform,
  arch: process.arch,
  preparation,
  success: false,
  scope: `Installed public native/compiler/plugin (${process.platform}/${process.arch}) with original five literal/ordinary static previews and Controls2/Keys3 real dev RunVRT New-to-Pass zero-difference repeats. No aggregate, hosted companion/HTTPS, general VRT update/ownership/tombstone, LSP or broader n8n credit.`,
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
      [
        "--test",
        "--test-reporter=tap",
        "--test-concurrency=1",
        "musea_literal_acceptance/suite.ts",
      ],
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
  const observations = assertObservations(
    path.join(root, "authority/artifacts/musea-native-literal-variants"),
  );
  const executed = read("executed-provider.json");
  assert.equal(executed.complete, true);
  assert.deepEqual(executed.loaded, custody.loaded);
  assert.deepEqual(publicNative(preparation.version).loaded, custody.loaded);
  for (const record of custody.packages) assertFiles(root, record.files);
  assertFiles(root, files);
  assert.deepEqual(fs.readFileSync(path.join(root, "package-lock.json")), lockBytes);
  receipt.observations = observations;
  receipt.executed = executed;
  receipt.denominator = {
    originalBrowserLaws: 2,
    staticPreviews: 5,
    galleryPreviews: 5,
    devControlVariants: 2,
    devLiteralVariants: 3,
    devObservationPhases: 2,
    physicalPngObservations: 10,
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
  fs.cpSync(path.join(root, "authority/tests"), path.join(output, "original-fixtures"), {
    recursive: true,
  });
}
