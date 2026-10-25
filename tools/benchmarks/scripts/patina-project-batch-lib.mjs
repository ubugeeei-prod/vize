import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

export const CLI_ARGS = [
  "lint",
  "--no-config",
  "--preset",
  "all",
  "--type-aware",
  "--strict-reactivity",
  "--format",
  "json",
  "src",
];

export function sha256(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

export function readAuthority(directory) {
  const authority = JSON.parse(fs.readFileSync(path.join(directory, "authority.json"), "utf8"));
  assert.equal(authority.schema, "vize.patina-project-batch-authority");
  assert.equal(authority.version, 1);
  assert.deepEqual(authority.origin.argv, CLI_ARGS);
  for (const [name, hash] of Object.entries(authority.artifacts)) {
    assert.equal(
      sha256(fs.readFileSync(path.join(directory, name))),
      hash,
      `frozen artifact changed: ${name}`,
    );
  }
  const component = fs.readFileSync(path.join(directory, "component.vue.txt"), "utf8");
  const module = fs.readFileSync(path.join(directory, "module.ts.txt"), "utf8");
  const composable = fs.readFileSync(path.join(directory, "use-counter.ts.txt"), "utf8");
  const seed = new Map([["src/use-counter.ts", composable]]);
  for (let index = 0; index < 40; index++)
    seed.set(`src/Comp${index}.vue`, component.replaceAll("__INDEX__", String(index)));
  for (let index = 0; index < 80; index++)
    seed.set(`src/mod-${index}.ts`, module.replaceAll("__INDEX__", String(index)));
  assert.deepEqual(
    Object.fromEntries(
      [...seed].map(([name, text]) => [name, sha256(text)]).sort(([a], [b]) => compare(a, b)),
    ),
    authority.seedInputs,
    "generated seed must preserve every captured source byte",
  );
  const rows = JSON.parse(fs.readFileSync(path.join(directory, "cohort.stdout"), "utf8"));
  assert.equal(rows.length, authority.filesPerCohort);
  assert.equal(
    rows.reduce((sum, row) => sum + row.messages.length, 0),
    authority.diagnosticsPerCohort,
  );
  const expanded = [];
  for (let cohort = 0; cohort < authority.cohorts; cohort++) {
    for (const row of rows) {
      assert.ok(row.file.startsWith("src/"));
      expanded.push({
        ...row,
        file: `src/cohort-${String(cohort).padStart(2, "0")}/${row.file.slice(4)}`,
      });
    }
  }
  // The public CLI's complete JSON output deliberately has no trailing newline.
  const stdout = Buffer.from(JSON.stringify(expanded, null, 2));
  assert.equal(
    sha256(stdout),
    authority.expandedStdoutSha256,
    "expanded complete output authority changed",
  );
  return { authority, seed, stdout, rows: expanded };
}

export function makeCorpus(fixture, consumer) {
  const frozen = readAuthority(fixture);
  fs.mkdirSync(consumer, { recursive: true });
  for (const name of ["package.json", "package-lock.json", "tsconfig.json"])
    fs.copyFileSync(
      path.join(fixture, name === "package-lock.json" ? name : `${name}.txt`),
      path.join(consumer, name),
    );
  for (let cohort = 0; cohort < frozen.authority.cohorts; cohort++) {
    for (const [name, text] of frozen.seed) {
      const file = path.join(
        consumer,
        `src/cohort-${String(cohort).padStart(2, "0")}`,
        name.slice(4),
      );
      fs.mkdirSync(path.dirname(file), { recursive: true });
      fs.writeFileSync(file, text);
    }
  }
  return frozen;
}

export function inventory(
  directory,
  prefixes = ["src", "node_modules", "package.json", "package-lock.json", "tsconfig.json"],
) {
  const files = {};
  function visit(relative) {
    const file = path.join(directory, relative);
    const stat = fs.lstatSync(file);
    if (stat.isSymbolicLink()) {
      const resolved = fs.realpathSync(file);
      assert.ok(
        resolved.startsWith(`${fs.realpathSync(directory)}${path.sep}`),
        `dependency link escapes consumer: ${relative}`,
      );
      files[relative] = { symlink: fs.readlinkSync(file) };
    } else if (stat.isDirectory()) {
      for (const name of fs.readdirSync(file).sort(compare)) visit(path.join(relative, name));
    } else {
      assert.ok(stat.isFile(), `unexpected input kind: ${relative}`);
      files[relative] = { bytes: stat.size, sha256: sha256(fs.readFileSync(file)) };
    }
  }
  for (const prefix of prefixes) visit(prefix);
  return files;
}

export function assertProcess(result, expectedStdout) {
  assert.equal(result.error, undefined, result.error?.message);
  assert.equal(result.signal, null, "sample must exit normally");
  assert.equal(
    result.status,
    1,
    `lint must report the frozen findings: ${result.stderr?.toString()}`,
  );
  const stdout = Buffer.isBuffer(result.stdout) ? result.stdout : Buffer.from(result.stdout);
  assert.deepEqual(
    stdout,
    expectedStdout,
    "every ordered file/diagnostic field and raw output byte must match the independent frozen public authority",
  );
  return sha256(stdout);
}

function compare(a, b) {
  return a < b ? -1 : a > b ? 1 : 0;
}

export function profileCounts(profile) {
  assert.deepEqual(
    profile.truncation,
    { dropped_spans: 0, dropped_counters: 0 },
    "truncated telemetry cannot prove the snapshot count",
  );
  assert.ok(Array.isArray(profile.spans));
  return profile.spans
    .filter((span) => span.key.startsWith("corsa.request."))
    .map((span) => {
      assert.equal(span.attribution?.stage, "patina");
      assert.notEqual(
        span.attribution.pass,
        "other",
        "unknown API methods cannot be hidden in evidence",
      );
      assert.ok(Number.isSafeInteger(span.count) && span.count > 0);
      return { key: span.key, attribution: span.attribution, count: span.count };
    })
    .sort((a, b) =>
      compare(JSON.stringify([a.key, a.attribution]), JSON.stringify([b.key, b.attribution])),
    );
}

export function assertProfile(profile, expected) {
  assert.deepEqual(profileCounts(profile), expected, "complete checker phase-count vector changed");
}

export function cleanEnvironment(base, corsa) {
  const env = Object.fromEntries(
    Object.entries(base).filter(([key]) => !/^(VIZE_|CORSA_|TSGO_|NAPI_RS_)/u.test(key)),
  );
  delete env.RAYON_NUM_THREADS;
  env.CORSA_PATH = corsa;
  env.LANG = "C.UTF-8";
  env.LC_ALL = "C.UTF-8";
  return env;
}

export function parseRss(text) {
  const matches = [...text.matchAll(/^\s*Maximum resident set size \(kbytes\): (\d+)\s*$/gmu)];
  assert.equal(matches.length, 1, "GNU time must report one peak-RSS observation");
  const kib = Number(matches[0][1]);
  assert.ok(Number.isSafeInteger(kib) && kib > 0);
  return kib;
}

export function median(values) {
  assert.ok(values.length > 0 && values.every((value) => Number.isFinite(value) && value > 0));
  const sorted = [...values].sort((a, b) => a - b);
  const middle = Math.floor(sorted.length / 2);
  return sorted.length % 2 ? sorted[middle] : (sorted[middle - 1] + sorted[middle]) / 2;
}
