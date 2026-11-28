import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import {
  existsSync,
  mkdtempSync,
  mkdirSync,
  readFileSync,
  rmSync,
  unlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import {
  retainMeasuredProbes,
  retainMovedAttribution,
  retainSourceCli,
} from "./release-measure-attribution.ts";
import type { Side } from "./release-measure.ts";

const labels = ["base", "head", "base-twin", "head-twin"] as const;
const sha = (bytes: Uint8Array | string) => createHash("sha256").update(bytes).digest("hex");
const dump = (id: string, instructions: number) =>
  `version: 1\nevents: Ir\ndesc: Trigger: Client Request: ${id}\ntotals: ${instructions}\n`;

void test("capture rejects missing or foreign images before the next target is built", () => {
  for (const fault of ["missing", "foreign"] as const) {
    const root = mkdtempSync(join(tmpdir(), "release-probe-invalid-"));
    try {
      const measurement = join(root, "measurement");
      const probes = join(root, "instruction-probes");
      mkdirSync(measurement);
      mkdirSync(probes);
      writeFileSync(
        join(measurement, "binaries.json"),
        JSON.stringify([{ pkg: "p", bench: "b", sha256: sha("expected") }]),
      );
      if (fault === "foreign") writeFileSync(join(probes, "probe-00"), "foreign");
      assert.throws(
        () => retainMeasuredProbes(measurement, probes, join(root, "retained")),
        fault === "missing" ? /ENOENT/ : /image digest differs/,
      );
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  }
});
function capture() {
  const out = mkdtempSync(join(tmpdir(), "release-attribution-"));
  const pair = { baseSha: "a".repeat(40), headSha: "b".repeat(40), sourceDateEpoch: "1700000000" };
  writeFileSync(join(out, "pair.json"), JSON.stringify(pair));
  const sides = {} as Record<(typeof labels)[number], Side>;
  for (const label of labels) {
    const base = label.startsWith("base");
    const instructions = base ? 10 : 13;
    const image = Buffer.from(`measured-image-${base ? "base" : "head"}`);
    const root = join(out, label, "stages", "measurement");
    mkdirSync(join(root, "probes"), { recursive: true });
    writeFileSync(join(root, "probes", "probe-00"), image);
    writeFileSync(
      join(root, "binaries.json"),
      JSON.stringify([{ pkg: "p", bench: "b", sha256: sha(image) }]),
    );
    const row = { instructions, fixture_sha256: "f".repeat(64), window: "routine-return" };
    const unchanged = { ...row, instructions: 7 };
    sides[label] = {
      rows: { changed: { ...row, methodology: {} }, unchanged: { ...unchanged, methodology: {} } },
      sizes: {},
      binaries: { "p/b": sha(image) },
    };
    writeFileSync(
      join(root, "measurement.json"),
      JSON.stringify({
        source_commit: base ? pair.baseSha : pair.headSha,
        runs: Array.from({ length: 3 }, () => ({ changed: row, unchanged })),
      }),
    );
    for (let run = 1; run <= 3; run++) {
      const dir = join(root, `run-${run}`);
      mkdirSync(dir);
      writeFileSync(join(dir, "p-b.callgrind.1"), dump("changed", instructions));
      writeFileSync(join(dir, "p-b.callgrind.2"), dump("unchanged", 7));
    }
  }
  return { out, sides, pair };
}

void test("changed-row attribution retains original dumps, measured images, all runs and both twins", () => {
  const { out, sides, pair } = capture();
  try {
    const records = retainMovedAttribution(out, ["changed"], sides);
    assert.deepEqual(
      records.map(({ side, run, id, instructions, source }) => [
        side,
        run,
        id,
        instructions,
        source,
      ]),
      labels.flatMap((label) =>
        [1, 2, 3].map((run) => [
          label,
          run,
          "changed",
          label.startsWith("base") ? 10 : 13,
          label.startsWith("base") ? pair.baseSha : pair.headSha,
        ]),
      ),
    );
    for (const row of records) {
      assert.deepEqual(
        readFileSync(join(out, row.dump)),
        readFileSync(join(out, row.originalDump)),
      );
      assert.equal(sha(readFileSync(join(out, row.dump))), row.dumpSha256);
      assert.equal(sha(readFileSync(join(out, row.image))), row.imageSha256);
      assert.equal(row.imageSha256, sides[row.side].binaries?.["p/b"]);
    }
    const manifest = JSON.parse(readFileSync(join(out, "attribution", "manifest.json"), "utf8"));
    assert.equal(manifest.state, "retained");
    assert.deepEqual(manifest.records, records);
    assert.throws(() => retainMovedAttribution(out, ["changed"], sides), /EEXIST/);
  } finally {
    rmSync(out, { recursive: true, force: true });
  }
});

void test("foreign source, changed image and changed counter refuse diagnostic authority", () => {
  for (const fault of ["source", "image", "counter"] as const) {
    const { out, sides } = capture();
    try {
      const root = join(out, "base", "stages", "measurement");
      if (fault === "source") {
        const file = join(root, "measurement.json");
        const value = JSON.parse(readFileSync(file, "utf8"));
        value.source_commit = "c".repeat(40);
        writeFileSync(file, JSON.stringify(value));
      } else if (fault === "image")
        writeFileSync(join(root, "probes", "probe-00"), "foreign image");
      else writeFileSync(join(root, "run-1", "p-b.callgrind.1"), dump("changed", 11));
      assert.throws(
        () => retainMovedAttribution(out, ["changed"], sides),
        /foreign source|executable bytes differ|raw count differs/,
      );
      assert.deepEqual(
        JSON.parse(readFileSync(join(out, "attribution", "manifest.json"), "utf8")).records,
        [],
      );
      assert.equal(
        JSON.parse(readFileSync(join(out, "attribution", "manifest.json"), "utf8")).state,
        "partial",
      );
    } finally {
      rmSync(out, { recursive: true, force: true });
    }
  }
});

void test("a missing raw execution preserves the available packet without claiming completeness", () => {
  const { out, sides } = capture();
  try {
    unlinkSync(join(out, "head", "stages", "measurement", "run-2", "p-b.callgrind.1"));
    assert.throws(
      () => retainMovedAttribution(out, ["changed"], sides),
      /missing raw dump for run 2/,
    );
    const manifest = JSON.parse(readFileSync(join(out, "attribution", "manifest.json"), "utf8"));
    assert.equal(manifest.state, "partial");
    assert.deepEqual(
      manifest.records.map((row: { side: string; run: number }) => [row.side, row.run]),
      [
        ["base", 1],
        ["base", 2],
        ["base", 3],
        ["head", 1],
        ["head", 3],
      ],
    );
  } finally {
    rmSync(out, { recursive: true, force: true });
  }
});

void test("unknown rows cannot produce an empty successful attribution packet", () => {
  const { out, sides } = capture();
  try {
    assert.throws(
      () => retainMovedAttribution(out, ["unknown"], sides),
      /unmeasured attribution row/,
    );
  } finally {
    rmSync(out, { recursive: true, force: true });
  }
});

void test("source CLI retention binds complete bytes and refuses overwriting an earlier image", () => {
  for (const buildTarget of ["x86_64-unknown-linux-gnu.2.17", "x86_64-unknown-linux-gnu.2.36"]) {
    const root = mkdtempSync(join(tmpdir(), "release-cli-retention-"));
    try {
      const file = join(root, "built-cli");
      const bytes = Buffer.from([0x7f, 0x45, 0x4c, 0x46, 255, 0, 13, 10]);
      writeFileSync(file, bytes);
      const identity = retainSourceCli(file, root, "a".repeat(40), buildTarget);
      unlinkSync(file);
      assert.deepEqual(readFileSync(join(root, "cli", "vize")), bytes);
      assert.deepEqual(identity, {
        schema: "vize.release-measure.cli.v1",
        source: "a".repeat(40),
        profile: "release",
        target: buildTarget,
        sizeBytes: 8,
        sha256: sha(bytes),
      });
      assert.deepEqual(
        JSON.parse(readFileSync(join(root, "cli", "identity.json"), "utf8")),
        identity,
      );
      assert.throws(() => retainSourceCli(file, root, "a".repeat(40), buildTarget), /EEXIST/);
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  }
});

void test("source CLI identity rejects an unspecified or foreign target before copying", () => {
  const root = mkdtempSync(join(tmpdir(), "release-cli-target-"));
  try {
    for (const target of [
      "x86_64-unknown-linux-gnu",
      "aarch64-unknown-linux-gnu.2.17",
      "x86_64-unknown-linux-gnu.2.17\n",
    ])
      assert.throws(() => retainSourceCli(join(root, "absent"), root, "a".repeat(40), target));
    assert.equal(existsSync(join(root, "cli")), false);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
