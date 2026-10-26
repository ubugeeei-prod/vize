import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const directory = (root) => path.join(root, "npm/native/.artifacts/native/formatter-history");

/** Observe the actual default Cargo target, without relying on task environment forwarding. */
export function captureAbsentNativeTarget(repository, source, runId) {
  const root = fs.realpathSync(repository);
  const target = path.join(root, "target");
  assert.equal(
    fs.lstatSync(target, { throwIfNoEntry: false }),
    undefined,
    "Cargo target already exists",
  );
  assert.match(source, /^[a-f0-9]{40}$/);
  assert.match(runId, /^[1-9][0-9]*$/);
  return {
    schema: "vize.parcel.native-target-before.v1",
    repository: root,
    target,
    source,
    runId,
    absent: true,
  };
}

/** Bind the preflight to the compiler's original emitted-artifact and environment packet. */
export function validateNativeTargetCustody(before, build) {
  assert.equal(before.schema, "vize.parcel.native-target-before.v1");
  assert.equal(before.absent, true);
  assert.equal(before.target, path.join(before.repository, "target"));
  assert.equal(before.source, build.source.head);
  assert.equal(before.source, build.sourceBefore.head);
  assert.equal(before.runId, build.request.runId);
  for (const environment of [build.environment.before, build.environment.after])
    assert.equal(environment.CARGO_TARGET_DIR, null, "unexpected Cargo target override");
  const emitted = build.emitted.filename;
  const relative = path.relative(before.target, emitted);
  assert.ok(
    relative &&
      relative !== ".." &&
      !relative.startsWith(`..${path.sep}`) &&
      !path.isAbsolute(relative),
    "compiler used a different Cargo target",
  );
  assert.equal(build.emitted.cargo.fresh, false, "native artifact was reused");
  assert.match(build.generated.sha256, /^[a-f0-9]{64}$/);
  assert.equal(build.generated.sha256, build.frozen.sha256);
  return {
    schema: "vize.parcel.native-target-custody.v1",
    preflight: before,
    source: before.source,
    imageHash: build.generated.sha256,
    emitted: { filename: emitted, fresh: build.emitted.cargo.fresh },
  };
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  assert.equal(process.env.GITHUB_ACTIONS, "true");
  assert.equal(process.platform, "linux");
  const root = fs.realpathSync(process.argv[3] ?? process.cwd());
  const output = directory(root);
  const initial = path.join(output, "parcel-target-before.json");
  if (process.argv[2] === "before") {
    const source = execFileSync("git", ["rev-parse", "HEAD"], {
      cwd: root,
      encoding: "utf8",
    }).trim();
    const before = captureAbsentNativeTarget(root, source, process.env.GITHUB_RUN_ID);
    fs.mkdirSync(output, { recursive: true });
    fs.writeFileSync(initial, JSON.stringify(before) + "\n", { flag: "wx" });
  } else {
    assert.equal(process.argv[2], "after");
    const before = JSON.parse(fs.readFileSync(initial, "utf8"));
    assert.equal(before.runId, process.env.GITHUB_RUN_ID);
    const build = JSON.parse(fs.readFileSync(path.join(output, "build-receipt.json"), "utf8"));
    assert.equal(fs.realpathSync(before.target), before.target, "Cargo target became a symlink");
    assert.equal(fs.realpathSync(build.emitted.filename), build.emitted.filename);
    const custody = validateNativeTargetCustody(before, build);
    fs.writeFileSync(
      path.join(output, "parcel-target-custody.json"),
      JSON.stringify(custody) + "\n",
      { flag: "wx" },
    );
  }
}
