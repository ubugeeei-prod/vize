/** Preserve original Callgrind dumps and their measured images for changed rows. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import {
  copyFileSync,
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  writeFileSync,
} from "node:fs";
import { join, relative } from "node:path";
import { ID, parseCallgrind } from "./instruction-counts-lib.mjs";
import type { Side } from "./release-measure.ts";

type Label = "base" | "head" | "base-twin" | "head-twin";
type Measurement = {
  source_commit: string;
  runs: Array<Record<string, { instructions: number; fixture_sha256: string; window: string }>>;
};
type Built = { pkg: string; bench: string; sha256: string };
type RecordRow = {
  side: Label;
  source: string;
  suite: string;
  run: number;
  id: string;
  instructions: number;
  fixtureSha256: string;
  window: string;
  originalDump: string;
  dump: string;
  dumpSha256: string;
  image: string;
  imageSha256: string;
};
const digest = (bytes: Uint8Array) => createHash("sha256").update(bytes).digest("hex");
const labels: readonly Label[] = ["base", "head", "base-twin", "head-twin"];

/** Diagnostic evidence only: no measurement, threshold or A/A assertion changes. */
export function retainMovedAttribution(
  out: string,
  changed: readonly string[],
  sides: Partial<Record<Label, Side>>,
) {
  const pair = JSON.parse(readFileSync(join(out, "pair.json"), "utf8")) as {
    baseSha: string;
    headSha: string;
    sourceDateEpoch: string;
  };
  assert.match(pair.baseSha, /^[a-f0-9]{40}$/);
  assert.match(pair.headSha, /^[a-f0-9]{40}$/);
  assert.ok(sides.base && sides.head, "both primary sides are required");
  for (const id of changed) {
    assert.match(id, ID);
    assert.ok(
      Object.hasOwn(sides.base.rows, id) || Object.hasOwn(sides.head.rows, id),
      `unmeasured attribution row ${id}`,
    );
  }
  assert.equal(new Set(changed).size, changed.length, "duplicate attribution row");
  const destination = join(out, "attribution");
  mkdirSync(destination); // Refuse to overwrite an earlier attribution packet.
  const records: RecordRow[] = [];
  let complete = false;
  const failures: unknown[] = [];
  try {
    for (const label of labels) {
      const side = sides[label];
      if (!side) continue;
      const source = label.startsWith("base") ? pair.baseSha : pair.headSha;
      const expected = new Map<string, number>();
      const observed = new Set<string>();
      const images = new Map<string, string>();
      for (const suite of ["stages", "formatter", "sfc"]) {
        const root = join(out, label, suite, "measurement");
        if (!existsSync(join(root, "measurement.json"))) continue;
        const measured = JSON.parse(
          readFileSync(join(root, "measurement.json"), "utf8"),
        ) as Measurement;
        assert.equal(measured.source_commit, source, `${label}/${suite}: foreign source`);
        const selected = changed.filter((id) => Object.hasOwn(measured.runs[0], id));
        if (!selected.length) continue;
        const built = JSON.parse(readFileSync(join(root, "binaries.json"), "utf8")) as Built[];
        for (const id of selected) {
          assert.ok(!expected.has(id), `duplicate measured row ${label}/${id}`);
          expected.set(id, measured.runs.length);
        }
        for (let run = 1; run <= measured.runs.length; run++) {
          const directory = join(root, `run-${run}`);
          for (const file of readdirSync(directory).sort()) {
            if (!/\.callgrind(?:\.\d+)*$/u.test(file)) continue;
            const bytes = readFileSync(join(directory, file));
            const dump = parseCallgrind(bytes.toString("utf8")) as {
              bench_id: string;
              instructions: number;
            } | null;
            if (!dump || !selected.includes(dump.bench_id)) continue;
            const id = dump.bench_id;
            const observation = measured.runs[run - 1][id];
            assert.equal(
              dump.instructions,
              observation.instructions,
              `${label}/${id}: raw count differs`,
            );
            assert.equal(
              dump.instructions,
              side.rows[id].instructions,
              `${label}/${id}: side count differs`,
            );
            assert.equal(observation.fixture_sha256, side.rows[id].fixture_sha256);
            assert.equal(observation.window, side.rows[id].window);
            const key = `${run}/${id}`;
            assert.ok(!observed.has(key), `duplicate raw dump ${label}/${key}`);
            observed.add(key);
            const owners = built
              .map((entry, index) => ({ ...entry, index }))
              .filter((entry) => {
                const stem = `${entry.pkg}-${entry.bench}.callgrind`;
                return file === stem || file.startsWith(`${stem}.`);
              });
            assert.equal(owners.length, 1, `${label}/${id}: ambiguous measured executable`);
            const owner = owners[0];
            assert.equal(owner.sha256, side.binaries?.[`${owner.pkg}/${owner.bench}`]);
            const probe = `probe-${String(owner.index).padStart(2, "0")}`;
            const imageBytes = readFileSync(join(root, "probes", probe));
            assert.equal(
              digest(imageBytes),
              owner.sha256,
              `${label}/${id}: executable bytes differ`,
            );
            const scope = join(destination, label, suite);
            mkdirSync(scope, { recursive: true });
            const imagePath = join(scope, probe);
            if (!images.has(imagePath)) {
              copyFileSync(join(root, "probes", probe), imagePath);
              images.set(imagePath, owner.sha256);
            }
            const target = join(scope, `run-${run}`, `${id}.callgrind`);
            mkdirSync(join(scope, `run-${run}`), { recursive: true });
            copyFileSync(join(directory, file), target);
            records.push({
              side: label,
              source,
              suite,
              run,
              id,
              instructions: dump.instructions,
              fixtureSha256: observation.fixture_sha256,
              window: observation.window,
              originalDump: relative(out, join(directory, file)),
              dump: relative(out, target),
              dumpSha256: digest(bytes),
              image: relative(out, imagePath),
              imageSha256: owner.sha256,
            });
          }
        }
      }
      for (const id of changed.filter((id) => Object.hasOwn(side.rows, id))) {
        assert.ok(expected.has(id), `${label}/${id}: no original capture`);
        for (let run = 1; run <= expected.get(id)!; run++)
          assert.ok(
            observed.has(`${run}/${id}`),
            `${label}/${id}: missing raw dump for run ${run}`,
          );
      }
    }
    complete = true;
  } catch (error) {
    failures.push(error);
  }
  try {
    writeFileSync(
      join(destination, "manifest.json"),
      JSON.stringify(
        {
          schema: "vize.release-measure.attribution.v1",
          scope: "diagnostic evidence; no performance acceptance credit",
          state: complete ? "retained" : "partial",
          producer: {
            source: process.env.GITHUB_SHA ?? null,
            repository: process.env.GITHUB_REPOSITORY ?? null,
            runId: process.env.GITHUB_RUN_ID ?? null,
            attempt: process.env.GITHUB_RUN_ATTEMPT ?? null,
          },
          pair,
          changed,
          sides: labels.filter((label) => sides[label]),
          records,
        },
        null,
        2,
      ) + "\n",
      { flag: "wx" },
    );
  } catch (error) {
    failures.push(error);
  }
  if (failures.length === 1) throw failures[0];
  if (failures.length > 1)
    throw new AggregateError(failures, "attribution or evidence retention failed");
  return records;
}

/** Publish only when both primary sides exist; failed builds retain their earlier evidence. */
export function publishMovedAttribution(out: string, moved: (base: Side, head: Side) => string[]) {
  const sides: Partial<Record<Label, Side>> = {};
  for (const label of labels) {
    const file = join(out, label, "side.json");
    if (existsSync(file)) sides[label] = JSON.parse(readFileSync(file, "utf8")) as Side;
  }
  if (sides.base && sides.head)
    return retainMovedAttribution(out, moved(sides.base, sides.head), sides);
}

/** Keep the exact optimized source CLI available for later consumer qualification. */
export function retainSourceCli(file: string, side: string, source: string) {
  assert.match(source, /^[a-f0-9]{40}$/);
  const target = join(side, "cli");
  mkdirSync(target);
  const bytes = readFileSync(file);
  copyFileSync(file, join(target, "vize"));
  const identity = {
    schema: "vize.release-measure.cli.v1",
    source,
    profile: "release",
    target: "x86_64-unknown-linux-gnu.2.36",
    sizeBytes: bytes.length,
    sha256: digest(bytes),
  };
  writeFileSync(join(target, "identity.json"), JSON.stringify(identity, null, 2) + "\n", {
    flag: "wx",
  });
  return identity;
}
