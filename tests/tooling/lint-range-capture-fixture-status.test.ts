import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { verifyFixtureStatus } from "../../tools/support/ci/lint-range-capture/preflight.ts";
import { projects } from "../../tools/support/ci/lint-range-capture/projects.ts";
import { decodeFrames } from "../../tools/support/ci/lint-range-capture/codec.ts";

test("fixture status retains the exact seventeen bytes, phase and identity before strict refusal", () => {
  const root = mkdtempSync(join(tmpdir(), "vize-lint-fixture-status-"));
  const invoke = (cwd: string, ...args: string[]) =>
    execFileSync("git", ["-C", cwd, ...args], { stdio: ["ignore", "pipe", "pipe"] });
  const commit = (cwd: string) => {
    invoke(cwd, "add", ".");
    invoke(
      cwd,
      "-c",
      "user.name=Capture Law",
      "-c",
      "user.email=capture@example.invalid",
      "commit",
      "-qm",
      "original fixture",
    );
  };
  try {
    const project = projects[3],
      cwd = join(root, project.fixturePath);
    mkdirSync(cwd, { recursive: true });
    invoke(cwd, "init", "-q");
    writeFileSync(join(cwd, "App.vue"), "<template>original</template>\n");
    commit(cwd);
    const fixtureHead = invoke(cwd, "rev-parse", "HEAD").toString().trim();
    invoke(root, "init", "-q");
    writeFileSync(join(root, "control.txt"), "original control\n");
    commit(root);
    const head = invoke(root, "rev-parse", "HEAD").toString().trim();
    verifyFixtureStatus(root, project, head, "pre-reporter-runtime");
    writeFileSync(join(cwd, "untracked.txt"), "actual observed output\n");
    const expected = invoke(cwd, "status", "--porcelain", "--untracked-files=normal");
    assert.equal(expected.length, 17); // Synthetic exact length, not a guessed hosted filename.
    assert.throws(
      () => verifyFixtureStatus(root, project, head, "post-reporter-reports"),
      /Fixture source has edits/u,
    );
    const out = join(root, "eslint-invalid-range-capture"),
      path = join(out, "fixture-runtime-drift.json");
    const original = readFileSync(path),
      record = JSON.parse(original.toString());
    assert.equal(record.phase, "post-reporter-reports");
    assert.equal(record.expectedSource, head);
    assert.equal(record.actualSource, head);
    assert.equal(record.fixture.id, project.id);
    assert.equal(record.fixture.path, project.fixturePath);
    assert.equal(record.fixture.expectedRevision, project.revision);
    assert.equal(record.fixture.actualRevision, fixtureHead);
    assert.equal(record.originalStatusBytes, 17);
    assert.deepEqual(Buffer.from(record.originalStatus), expected);
    assert.equal(record.completeStatus, expected.toString());
    assert.equal(record.trackedDiff, "");
    assert.equal(record.untrackedPaths, "untracked.txt\0");
    assert.equal(record.acceptance, false);
    for (const row of record.gitOutputs)
      for (const file of [row.stdout, row.stderr]) {
        const bytes = readFileSync(join(out, file.path));
        assert.equal(bytes.length, file.bytes);
        assert.equal(createHash("sha256").update(bytes).digest("hex"), file.sha256);
      }
    const frames = execFileSync(process.execPath, [
      fileURLToPath(
        new URL("../../tools/support/ci/lint-range-capture/frames.ts", import.meta.url),
      ),
      root,
      head,
    ]).toString();
    const decoded = decodeFrames(frames, head);
    assert.deepEqual(decoded.files.get("fixture-runtime-drift.json"), original);
    for (const row of record.gitOutputs)
      for (const file of [row.stdout, row.stderr])
        assert.deepEqual(decoded.files.get(file.path), readFileSync(join(out, file.path)));
    writeFileSync(join(cwd, "later.txt"), "later output\n");
    assert.throws(
      () => verifyFixtureStatus(root, project, head, "later-refusal"),
      /Fixture source has edits/u,
    );
    assert.deepEqual(readFileSync(path), original);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
