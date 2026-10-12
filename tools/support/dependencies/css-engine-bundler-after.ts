// Source-after qualification. Original whole inputs and every failed before slot stay frozen.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { isDeepStrictEqual } from "node:util";

const fixture = "tests/_fixtures/differential/css-engine-vendor-exit/bundler-before-inputs.json";
const checksum = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
const git = (...args: string[]) => {
  const result = spawnSync("git", args, { encoding: "utf8" });
  assert.equal(result.status, 0, result.stderr);
  return result.stdout;
};
const bytes = fs.readFileSync(fixture);
assert.equal(checksum(bytes), "407348d9901a5a566cc10f4e199d61c4670c018c53daed09d8633ea00ea0de1c");
const manifest = JSON.parse(bytes.toString());
assert.equal(manifest.schema, 1);
assert.equal(manifest.sourceOnly, true);
assert.deepEqual(manifest.pools, [1, 8, 32]);
assert.equal(manifest.cases.length, 21);
assert.equal(new Set(manifest.cases.map((row: { id: string }) => row.id)).size, 21);
const before = fs.readFileSync(
  "tests/_fixtures/differential/css-engine-vendor-exit/bundler-before-whole.json",
);
assert.equal(checksum(before), "d139e32de20854c5068a38bb3803884a12db81484fe032ba4c45c575774c1da1");
const beforeSeal = JSON.parse(before.toString());
assert.equal(beforeSeal.C, "e35d0d56368d2c4e9ecd10883f79900c0501cadf");
assert.equal(beforeSeal.R, 38191469890);
assert.equal(beforeSeal.workflowConclusion, "failure");
assert.deepEqual(
  beforeSeal.twoProfiles.map((row: { retainedSlots: number }) => row.retainedSlots),
  [63, 63],
);
// Shallow PR checkouts contain the actual composed H, not historical e35.
// Compare authenticated complete byte authorities rather than requiring old Git
// objects or claiming unrelated first-party main changes are the same producer.
const unchangedBeforeSources = beforeSeal.twoProfiles[0].allTenSourceCopies.filter(
  (row: { path: string }) => row.path !== "vendor/lightningcss/src/bundler.rs",
);
assert.equal(unchangedBeforeSources.length, 9);
for (const row of unchangedBeforeSources) {
  assert.equal(checksum(fs.readFileSync(row.path)), row.sha256, row.path);
}
const sourceChangeScope =
  "Authenticated library/provider, lock, manifests and full probe/input bytes; no first-party checkout equivalence claim";
const custody = spawnSync(
  process.execPath,
  ["tools/support/dependencies/css-engine-vendor-custody.ts"],
  { encoding: "utf8" },
);
assert.equal(custody.status, 0, custody.stderr);
const active = fs.readFileSync("vendor/lightningcss/src/bundler.rs");
const original = fs.readFileSync(
  "vendor/lightningcss/tests/vize_fixtures/bundler-original-tests.rs.txt",
);
assert.deepEqual(active.subarray(active.indexOf("#[cfg(test)]\nmod tests {")), original);
if (process.argv[2] === "--authenticate") {
  console.log(
    JSON.stringify({
      sourceRevision: git("rev-parse", "HEAD").trim(),
      manifestSha256: checksum(bytes),
      sourceChangeScope,
      publicationQualification: false,
    }),
  );
} else {
  const [executable, output, profile] = process.argv.slice(2);
  assert(executable && output && ["dev", "release"].includes(profile));
  fs.mkdirSync(output, { recursive: true });
  const sourceRevision = git("rev-parse", "HEAD").trim();
  const executablePath = fs.realpathSync(executable);
  // This receipt is written before the first actual child starts.
  fs.writeFileSync(
    path.join(output, "startup.json"),
    JSON.stringify(
      {
        sourceRevision,
        sourceChangeScope,
        scope: "source-after",
        beforeRun: beforeSeal.R,
        beforeSealSha256: checksum(before),
        profile,
        executablePath,
        executableSha256: checksum(fs.readFileSync(executablePath)),
        manifestSha256: checksum(bytes),
        beforeSourceC: manifest.beforeSourceC,
        beforeH: manifest.beforeH,
        sourceOnly: true,
        publicationQualification: false,
        expectationsRecaptured: false,
      },
      null,
      2,
    ),
  );
  fs.copyFileSync(fixture, path.join(output, "inputs.json"));
  const observations = [];
  let complete = true;
  for (const row of manifest.cases) {
    for (const pool of manifest.pools) {
      const directory = path.join(output, `${row.id}-pool${pool}`);
      fs.mkdirSync(directory, { recursive: true });
      const result = spawnSync(executablePath, [fixture, row.id, String(pool), directory], {
        env: { ...process.env, CSS_ENGINE_SOURCE_REVISION: sourceRevision },
        encoding: "utf8",
        timeout: 10_000,
        maxBuffer: 16 * 1024 * 1024,
      });
      fs.writeFileSync(path.join(directory, "stdout.log"), result.stdout ?? "");
      fs.writeFileSync(path.join(directory, "stderr.log"), result.stderr ?? "");
      fs.writeFileSync(
        path.join(directory, "raw-process.json"),
        JSON.stringify(
          {
            id: row.id,
            pool,
            exit: result.status,
            signal: result.signal,
            processError: result.error
              ? { name: result.error.name, message: result.error.message }
              : null,
          },
          null,
          2,
        ),
      );
      const read = (name: string) =>
        fs.existsSync(path.join(directory, name))
          ? fs.readFileSync(path.join(directory, name), "utf8")
          : null;
      const observation = read("observation.json");
      const rawEvents = read("events.jsonl");
      const parseErrors: string[] = [];
      const parse = (raw: string | null) => {
        if (raw === null) return null;
        try {
          return JSON.parse(raw);
        } catch (error) {
          parseErrors.push(String(error));
          return null;
        }
      };
      const events = rawEvents
        ?.trim()
        .split("\n")
        .filter(Boolean)
        .map((line) => parse(line))
        .filter(Boolean);
      const traceValid = Boolean(
        events?.length &&
        events.every(
          (event, index) =>
            event?.sequence === index && event?.event && typeof event.event.type === "string",
        ),
      );
      let inversionVerified = null;
      if (row.forceReadBeforeResolve && pool > 1) {
        const firstFinish =
          traceValid &&
          events?.find(
            (event) =>
              event.event.type === "resolve-finish" &&
              event.event.origin === row.entry &&
              event.event.specifier === row.forceReadBeforeResolve.specifier,
          );
        const laterRead =
          traceValid &&
          events?.find(
            (event) =>
              event.event.type === "read" && event.event.path === row.forceReadBeforeResolve.path,
          );
        inversionVerified = Boolean(
          firstFinish?.event?.gated && laterRead && laterRead.sequence < firstFinish.sequence,
        );
      }
      const packet = parse(observation);
      const startup = parse(read("startup.json"));
      const captured =
        result.status === 0 &&
        !result.error &&
        packet !== null &&
        packet.result !== undefined &&
        packet.map !== undefined &&
        startup?.case?.id === row.id &&
        isDeepStrictEqual(startup?.case, row) &&
        startup?.threads === pool &&
        startup?.sourceRevision === sourceRevision &&
        parseErrors.length === 0 &&
        traceValid &&
        inversionVerified !== false &&
        !events?.some((event) => event?.event?.type === "gate-timeout");
      complete &&= captured && (!row.wholeExpected || packet?.result?.code === row.wholeExpected);
      const receipt = {
        id: row.id,
        pool,
        captured,
        exit: result.status,
        signal: result.signal,
        processError: result.error
          ? { name: result.error.name, message: result.error.message }
          : null,
        inversionVerified,
        wholeReferenceMatches: row.wholeExpected
          ? packet?.result?.code === row.wholeExpected
          : null,
        startup: read("startup.json"),
        observation: packet,
        events,
        traceValid,
        rawObservation: observation,
        rawEvents,
        parseErrors,
      };
      fs.writeFileSync(path.join(directory, "process.json"), JSON.stringify(receipt, null, 2));
      observations.push(receipt);
    }
  }
  fs.writeFileSync(
    path.join(output, "whole-after.json"),
    JSON.stringify(
      {
        sourceRevision,
        scope: "source-after",
        beforeRun: beforeSeal.R,
        beforeSealSha256: checksum(before),
        profile,
        complete,
        expectedCount: 63,
        observations,
        sourceOnly: true,
        publicationQualification: false,
        expectationsRecaptured: false,
      },
      null,
      2,
    ),
  );
  assert.equal(observations.length, 63);
  assert(
    complete,
    "a failed child, gate or bounded timeout is retained and is not successful qualification",
  );
}
