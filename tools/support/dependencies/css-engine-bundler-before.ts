// Additive source-only capture. All raw child packets survive errors and timeouts.
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
assert.equal(manifest.beforeSourceC, "b2205cf5a53ac57c4ad3c0a5d92cb649087fe402");
assert.equal(manifest.beforeH, "da002e6f1068181afbf6e8472530bc628730ee75");
assert.equal(
  checksum(fs.readFileSync("vendor/lightningcss/src/bundler.rs")),
  "532b321252c008f1c581a1613fda4cc6e57369bf2bf712b95618f3ca60a0b5ed",
);
assert.equal(
  manifest.bundlerSha256,
  "532b321252c008f1c581a1613fda4cc6e57369bf2bf712b95618f3ca60a0b5ed",
);
assert.deepEqual(manifest.pools, [1, 8, 32]);
assert.equal(manifest.cases.length, 21);
assert.equal(new Set(manifest.cases.map((row: { id: string }) => row.id)).size, 21);
const productionDelta = git(
  "diff",
  "--name-only",
  manifest.beforeSourceC,
  "--",
  "Cargo.toml",
  "Cargo.lock",
  "crates",
  "davinci",
  "vendor/lightningcss/src",
  "vendor/parcel_sourcemap",
);
assert.equal(productionDelta, "", "before capture retains the exact checked producer");
const active = fs.readFileSync("vendor/lightningcss/src/bundler.rs");
const original = fs.readFileSync(
  "vendor/lightningcss/tests/vize_fixtures/bundler-original-tests.rs.txt",
);
assert.equal(
  checksum(original),
  "e1d3d9d8ba8a988177e8bd82707ab8d51e5b68f6667c4c7b1e236d89c943b098",
);
assert.deepEqual(active.subarray(active.indexOf("#[cfg(test)]")), original);
const registration =
  '\n[[example]]\nname = "vize_bundler_before_observer"\npath = "examples/vize_bundler_before_observer.rs"\nrequired-features = ["bundler", "sourcemap", "serde"]\n';
assert.equal(
  fs.readFileSync("vendor/lightningcss/Cargo.toml", "utf8").replace(registration, ""),
  git("show", `${manifest.beforeSourceC}:vendor/lightningcss/Cargo.toml`),
  "only the named additive observer registration differs",
);
if (process.argv[2] === "--authenticate") {
  console.log(
    JSON.stringify({
      sourceRevision: git("rev-parse", "HEAD").trim(),
      manifestSha256: checksum(bytes),
      productionDelta,
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
      complete &&= captured;
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
    path.join(output, "whole-before.json"),
    JSON.stringify(
      {
        sourceRevision,
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
