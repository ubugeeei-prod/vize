/** Artifact-only observations; never accepts snapshots or qualifies a failed cohort. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { execFileSync, spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { at, parseJson } from "../../json-value.ts";

export const computedEventFilter =
  "virtual_ts::dynamic_component_names_tests::computed_component_event_names_never_become_static_listener_types";
const directory = "crates/vize_canon/src/virtual_ts/snapshots";
export const computedEventFiles = [
  "Child",
  "Child",
  "Child",
  "Child",
  "Child",
  "Child",
  "button",
  "Transition",
].map(
  (target, index) =>
    `${directory}/vize_canon__virtual_ts__dynamic_component_names_tests__computed_component_event_name_${index}_${target}.snap`,
);
const digest = (file: string) => createHash("sha256").update(fs.readFileSync(file)).digest("hex");

export function selectCanonTestBinary(raw: string, root: string): string {
  const binaries = new Set<string>();
  for (const line of raw.split("\n")) {
    let message: unknown;
    try {
      message = parseJson(line);
    } catch {
      continue;
    }
    const kinds = at(message, "target", "kind"),
      executable = at(message, "executable");
    const packageId = at(message, "package_id");
    if (
      at(message, "reason") !== "compiler-artifact" ||
      at(message, "target", "name") !== "vize_canon" ||
      !Array.isArray(kinds) ||
      !kinds.includes("lib") ||
      at(message, "profile", "test") !== true ||
      typeof packageId !== "string" ||
      !/(?:^|[/# ])vize_canon(?:[#@ ]|$)/u.test(packageId) ||
      typeof executable !== "string"
    )
      continue;
    const binary = fs.realpathSync(executable);
    // Cargo 1.99 writes target/ci/deps; Cargo 1.100 writes target/ci/build/<package>/<hash>/out.
    const relative = path.relative(fs.realpathSync(path.join(root, "target/ci")), binary);
    assert(
      relative && !relative.startsWith("..") && !path.isAbsolute(relative),
      "foreign test binary",
    );
    assert(fs.statSync(binary).isFile(), "test binary must be a file");
    assert.match(path.basename(binary), /^vize_canon-[a-zA-Z0-9]+$/u);
    binaries.add(binary);
  }
  assert.equal(binaries.size, 1, "require one compiled Canon library test binary");
  return [...binaries][0]!;
}

export function captureComputedEvents(
  root: string,
  evidence: string,
  expectedSource = process.env.GITHUB_SHA,
) {
  const git = (...args: string[]) =>
    execFileSync("git", args, { cwd: root, encoding: "utf8" }).trim();
  const source = git("rev-parse", "HEAD");
  const tree = git("rev-parse", "HEAD^{tree}");
  if (expectedSource) assert.equal(source, expectedSource, "source changed after build");
  const originalDiff = git("diff", "--name-only", "HEAD");
  assert.equal(originalDiff, "", "capture requires unchanged tracked source");
  const metadata = path.join(evidence, "cargo.jsonl");
  const binary = selectCanonTestBinary(fs.readFileSync(metadata, "utf8"), root);
  const beforeBinary = digest(binary);
  const formal = computedEventFiles.map((file) => ({
    file,
    sha256: digest(path.join(root, file)),
  }));
  const output = path.join(evidence, "computed-event-observations");
  fs.mkdirSync(output, { recursive: true });
  const list = spawnSync(binary, ["--list", "--format", "terse"], { cwd: root, encoding: "utf8" });
  assert.equal(list.status, 0, "cannot inspect the existing test binary");
  assert.equal(
    list.stdout.split("\n").filter((line) => line === `${computedEventFilter}: test`).length,
    1,
  );
  const argv = [computedEventFilter, "--exact", "--nocapture", "--test-threads=1"];
  const result = spawnSync(binary, argv, {
    cwd: root,
    env: { ...process.env, INSTA_UPDATE: "new", INSTA_FORCE_PASS: "1" },
    maxBuffer: 32 * 1024 * 1024,
  });
  fs.writeFileSync(path.join(output, "stdout.bin"), result.stdout ?? Buffer.alloc(0));
  fs.writeFileSync(path.join(output, "stderr.bin"), result.stderr ?? Buffer.alloc(0));
  const binaryUnchanged = digest(binary) === beforeBinary;
  const sourceUnchanged =
    git("diff", "--name-only", "HEAD") === "" &&
    git("rev-parse", "HEAD") === source &&
    git("rev-parse", "HEAD^{tree}") === tree;
  const formalSnapshotsUnchanged = formal.every(
    (item) => digest(path.join(root, item.file)) === item.sha256,
  );
  const executedOneTest = /test result: ok\. 1 passed; 0 failed; 0 ignored;/u.test(
    result.stdout?.toString() ?? "",
  );
  const complete =
    result.status === 0 &&
    executedOneTest &&
    binaryUnchanged &&
    sourceUnchanged &&
    formalSnapshotsUnchanged;
  const observations = formal.map((item) => {
    const pending = path.join(root, `${item.file}.new`);
    const generated = fs.existsSync(pending);
    const file = generated ? pending : path.join(root, item.file);
    const destination = `${path.basename(item.file)}${generated ? ".new" : ".matched-reference"}`;
    fs.copyFileSync(file, path.join(output, destination));
    return {
      file: item.file,
      referenceSha256: item.sha256,
      observation: destination,
      sha256: digest(file),
      bytes: fs.statSync(file).size,
      origin: generated
        ? "generated-pending-snapshot"
        : complete
          ? "reference-matched-by-complete-filter"
          : "unobserved-reference",
    };
  });
  const receipt = {
    source,
    tree,
    binary,
    binarySha256: beforeBinary,
    metadataSha256: digest(metadata),
    argv,
    mode: "artifact-only",
    qualified: false,
    env: { INSTA_UPDATE: "new", INSTA_FORCE_PASS: "1" },
    originalEnv: {
      INSTA_UPDATE: process.env.INSTA_UPDATE,
      INSTA_FORCE_PASS: process.env.INSTA_FORCE_PASS,
    },
    status: result.status,
    signal: result.signal,
    error: result.error?.message,
    originalCohortOutcome: process.env.VIZE_ORIGINAL_COHORT_OUTCOME,
    originalCohortStatusPreserved: true,
    formalSnapshotsUnchanged,
    sourceUnchanged,
    binaryUnchanged,
    executedOneTest,
    observations,
  };
  const receiptBytes = `${JSON.stringify(receipt, null, 2)}\n`;
  fs.writeFileSync(path.join(output, "receipt.json"), receiptBytes);
  // Include the small custody receipt in the existing pending snapshot artifact.
  fs.writeFileSync(path.join(root, directory, "computed-event-capture.json"), receiptBytes);
  assert(binaryUnchanged, "binary changed during observation");
  assert(sourceUnchanged, "source changed during observation");
  assert(formalSnapshotsUnchanged, "formal snapshot changed");
  assert(executedOneTest, "artifact-only filter must execute one non-ignored test");
  assert.equal(
    result.status,
    0,
    "artifact-only filter failed; original cohort remains unqualified",
  );
  return receipt;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  assert(process.argv[2], "require the existing Rust build evidence directory");
  assert.equal(
    process.env.VIZE_ORIGINAL_COHORT_OUTCOME,
    "failure",
    "capture requires the original failed cohort",
  );
  captureComputedEvents(process.cwd(), path.resolve(process.argv[2]));
}
