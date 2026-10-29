import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { expectedBuildIdentity, validateBuildReceipt } from "../differential/build-receipt.ts";
import { writeVueConfigFixture } from "./support/vue-config-fixture.ts";

const root = path.resolve(import.meta.dirname, "../..");
const directory = path.join(root, "tests/_fixtures/differential/linter/a11y-listbox-focus-10925");
const read = (name: string) => JSON.parse(fs.readFileSync(path.join(directory, name), "utf8"));
const hash = (bytes: string | Buffer) => createHash("sha256").update(bytes).digest("hex");
const rule = "a11y/interactive-supports-focus";

void test("listbox roving focus preserves whole CLI packets across Vue, HTML, JSX and TSX", () => {
  const corpus = read("cases.json");
  const original = read("original-source.json");
  const before = read("original-before.json");
  const originalBytes = fs.readFileSync(path.join(directory, original.sourceFile));
  assert.equal(hash(originalBytes), original.sourceSha256);
  assert.equal(hash(before.stdout), before.stdoutSha256);
  assert.deepEqual(JSON.parse(before.stdout), before.packet);
  assert.equal(before.packet.length, 1);
  assert.equal(before.packet[0].warningCount, 1);
  assert.equal(before.packet[0].messages[0].ruleId, rule);
  assert.equal(corpus.cases.length, 37);
  const build = expectedBuildIdentity(root);
  const receipt = JSON.parse(
    fs.readFileSync(path.join(root, `${build.binaryPath}.differential-build.json`), "utf8"),
  );
  validateBuildReceipt(receipt, build);
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "vize-listbox-10925-"));
  const artifact = path.join(root, "target/differential/lint-listbox-focus-10925.json");
  fs.mkdirSync(path.dirname(artifact), { recursive: true });
  const runs: {
    id: string;
    route: string;
    repetition: number;
    sourceSha256: string;
    argv: string[];
    expected: unknown;
    expectedStatus: number;
    status: number | null;
    signal: string | null;
    error: string | null;
    stdout: string;
    stderr: string;
    actual: unknown;
  }[] = [];
  const evidence = { schema: "vize.a11y.listbox-focus-source-cli", build, receipt, runs };
  const persist = () => fs.writeFileSync(artifact, `${JSON.stringify(evidence, null, 2)}\n`);
  const run = (
    id: string,
    route: string,
    filename: string,
    bytes: Buffer,
    argv: string[],
    expected: unknown,
    expectedStatus: number,
  ) => {
    const file = path.join(temporary, filename);
    fs.writeFileSync(file, bytes);
    for (let repetition = 0; repetition < 2; repetition++) {
      const child = spawnSync(path.join(root, build.binaryPath), argv, {
        cwd: temporary,
        encoding: "utf8",
        timeout: 30_000,
        maxBuffer: 2 * 1024 * 1024,
        env: { ...process.env, NO_COLOR: "1" },
      });
      let actual: unknown;
      try {
        actual = JSON.parse(child.stdout);
      } catch {
        actual = { invalidJson: child.stdout };
      }
      runs.push({
        id,
        route,
        repetition,
        sourceSha256: hash(bytes),
        argv,
        expected,
        expectedStatus,
        status: child.status,
        signal: child.signal,
        error: child.error?.message ?? null,
        stdout: child.stdout,
        stderr: child.stderr,
        actual,
      });
      assert.deepEqual(fs.readFileSync(file), bytes, `${id}/${route}: source must not change`);
      persist();
    }
  };
  try {
    const expectedOriginal = [{ ...before.packet[0], messages: [], warningCount: 0 }];
    run(
      "original-issue",
      "opinionated",
      "CompositeWidgets.vue",
      originalBytes,
      before.argv,
      expectedOriginal,
      0,
    );
    writeVueConfigFixture(temporary, {
      linter: { preset: "incremental", rules: { [rule]: "warn" } },
    });
    for (const entry of corpus.cases) {
      assert.deepEqual(Object.keys(entry.routes), ["vue", "html", "jsx", "tsx"]);
      for (const [name, untyped] of Object.entries(entry.routes)) {
        const route = untyped as {
          sourceFile: string;
          sourceSha256: string;
          beforeStdout: string;
          beforePacket: {
            file: string;
            messages: unknown[];
            errorCount: number;
            warningCount: number;
          }[];
          afterPacket: {
            file: string;
            messages: unknown[];
            errorCount: number;
            warningCount: number;
          }[];
          removedWholeMessages: unknown[];
          afterStatus: number;
          filename: string;
        };
        const bytes = fs.readFileSync(path.join(directory, route.sourceFile));
        assert.equal(hash(bytes), route.sourceSha256);
        assert.deepEqual(JSON.parse(route.beforeStdout), route.beforePacket);
        const previous = route.beforePacket[0];
        const removed = new Set(
          route.removedWholeMessages.map((message) => JSON.stringify(message)),
        );
        assert.deepEqual(route.afterPacket, [
          {
            ...previous,
            messages: previous.messages.filter((message) => !removed.has(JSON.stringify(message))),
            warningCount: previous.warningCount - route.removedWholeMessages.length,
          },
        ]);
        run(
          entry.id,
          name,
          route.filename,
          bytes,
          [
            "lint",
            "--config",
            "vite.config.mjs",
            route.filename,
            "--format",
            "json",
            "--locale",
            "en",
            "--quiet",
          ],
          route.afterPacket,
          route.afterStatus,
        );
      }
    }
    assert.equal(runs.length, 298);
    console.log(JSON.stringify(evidence));
    for (const observed of runs) {
      const label = `${observed.id}/${observed.route}/${observed.repetition}`;
      assert.equal(observed.error, null, label);
      assert.equal(observed.signal, null, label);
      assert.equal(observed.stderr, "", label);
      assert.equal(observed.status, observed.expectedStatus, label);
      assert.deepEqual(observed.actual, observed.expected, label);
    }
  } finally {
    persist();
    fs.rmSync(temporary, { recursive: true, force: true });
  }
});
