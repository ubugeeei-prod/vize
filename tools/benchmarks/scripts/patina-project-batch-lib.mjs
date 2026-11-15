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

function canonicalShardPlan(frozen) {
  const plan = [];
  for (let cohort = 0; cohort < frozen.authority.cohorts; cohort++) {
    const id = `cohort-${String(cohort).padStart(2, "0")}`;
    const directory = `projects/${id}`;
    const target = `src/${id}`;
    const members = [...frozen.seed].map(([name]) => `${target}/${name.slice(4)}`);
    const rows = frozen.rows.filter((row) => row.file.startsWith(`${target}/`));
    const stdout = Buffer.from(JSON.stringify(rows, null, 2));
    plan.push({
      id,
      directory,
      target,
      members,
      packet: {
        fileCount: rows.length,
        diagnosticCount: rows.reduce((sum, row) => sum + row.messages.length, 0),
        byteLength: stdout.length,
        stdoutSha256: sha256(stdout),
      },
    });
  }
  return plan;
}

/** The one deterministic partition of the immutable 1,210-file authority. */
export function makeShardPlan(frozen) {
  const plan = canonicalShardPlan(frozen);
  assertShardPlan(frozen, plan);
  return plan;
}

/** Refuse missing, duplicate, foreign, reordered or packet-drifted shards. */
export function assertShardPlan(frozen, plan) {
  const expected = canonicalShardPlan(frozen);
  assert.equal(plan.length, frozen.authority.cohorts, "every frozen cohort needs one shard");
  const members = plan.flatMap((shard) => shard.members);
  assert.equal(new Set(members).size, members.length, "shard members must be unique");
  assert.deepEqual(
    members,
    expected.flatMap((shard) => shard.members),
    "shards must contain every source exactly once in frozen order",
  );
  assert.deepEqual(plan, expected, "shard IDs, order and complete packet authority changed");
}

/** The exact expected stdout bytes for one validated shard. */
export function shardPacket(frozen, shard) {
  assertShardPlan(frozen, makeShardPlan(frozen));
  const rows = frozen.rows.filter((row) => row.file.startsWith(`${shard.target}/`));
  const stdout = Buffer.from(JSON.stringify(rows, null, 2));
  assert.deepEqual(
    {
      fileCount: rows.length,
      diagnosticCount: rows.reduce((sum, row) => sum + row.messages.length, 0),
      byteLength: stdout.length,
      stdoutSha256: sha256(stdout),
    },
    shard.packet,
  );
  return { rows, stdout };
}

/**
 * Give each shard an independent physical project without changing any frozen
 * authored source. Dependencies remain installed once at the parent root.
 */
export function writeShardProjects(consumer, frozen, plan) {
  const rootConfig = JSON.parse(fs.readFileSync(path.join(consumer, "tsconfig.json"), "utf8"));
  for (const shard of plan) {
    const directory = path.join(consumer, shard.directory);
    const source = path.join(consumer, shard.target);
    const isolatedSource = path.join(directory, shard.target);
    fs.mkdirSync(path.dirname(isolatedSource), { recursive: true });
    fs.renameSync(source, isolatedSource);
    const include = [`${shard.target}/**/*`];
    const bytes = `${JSON.stringify({ ...rootConfig, include }, null, 2)}\n`;
    fs.writeFileSync(path.join(directory, "tsconfig.json"), bytes);
  }
  fs.rmdirSync(path.join(consumer, "src"));
  return assertShardProjects(consumer, frozen, plan);
}

/** Prove physical roots contain exactly their manifest, config and no local dependencies. */
export function assertShardProjects(consumer, frozen, plan) {
  assert.equal(
    fs.existsSync(path.join(consumer, "src")),
    false,
    "unsharded sources must be absent",
  );
  const rootConfig = JSON.parse(fs.readFileSync(path.join(consumer, "tsconfig.json"), "utf8"));
  const projects = [];
  for (const shard of plan) {
    const directory = path.join(consumer, shard.directory);
    const include = [`${shard.target}/**/*`];
    const configBytes = `${JSON.stringify({ ...rootConfig, include }, null, 2)}\n`;
    const expectedSources = Object.fromEntries(
      shard.members
        .map((member) => {
          const seed = `src/${member.slice(`${shard.target}/`.length)}`;
          const text = frozen.seed.get(seed);
          assert.equal(typeof text, "string", `${member} must resolve to one frozen seed`);
          return [member, { bytes: Buffer.byteLength(text), sha256: sha256(text) }];
        })
        .sort(([left], [right]) => compare(left, right)),
    );
    const expectedInventory = {
      ...expectedSources,
      "tsconfig.json": {
        bytes: Buffer.byteLength(configBytes),
        sha256: sha256(configBytes),
      },
    };
    const actualInventory = inventory(directory, ["."]);
    assert.deepEqual(
      actualInventory,
      expectedInventory,
      `${shard.id} must contain only its byte-identical sources and exact config`,
    );
    projects.push({
      id: shard.id,
      directory: shard.directory,
      include,
      sourceCount: Object.keys(expectedSources).length,
      sourceInventorySha256: sha256(JSON.stringify(expectedSources)),
      configByteLength: Buffer.byteLength(configBytes),
      configSha256: sha256(configBytes),
    });
  }
  assert.deepEqual(
    projects.map((project) => project.id),
    plan.map((shard) => shard.id),
    "physical shard roots must retain the manifest order",
  );
  return projects;
}

/** Reassemble shard packets and require the immutable full public packet. */
export function assertAggregatePackets(frozen, plan, packets) {
  const authority = plan.map((shard) => ({
    id: shard.id,
    rows: shardPacket(frozen, shard).rows,
    ...shard.packet,
  }));
  const aggregate = assertPacketAuthority(plan, authority, packets);
  assertBytesEqual(
    aggregate.stdout,
    frozen.stdout,
    "shards did not reconstruct the historical public packet",
  );
  return aggregate;
}

/** Reassemble packets against an exact ordered authority sealed for this run. */
export function assertPacketAuthority(plan, authority, packets) {
  assert.equal(authority.length, plan.length, "authority must contain every shard packet");
  assert.equal(packets.length, plan.length, "every shard must produce one packet");
  const rows = [];
  const expectedRows = [];
  for (let index = 0; index < plan.length; index++) {
    const shard = plan[index];
    const expected = authority[index];
    const packet = packets[index];
    assert.equal(expected.id, shard.id, "authority shard order changed");
    assert.equal(packet.id, shard.id, "shard packet order changed");
    assert.equal(packet.stdoutSha256, expected.stdoutSha256, "shard packet bytes changed");
    assert.equal(packet.byteLength, expected.byteLength, "shard packet byte length changed");
    assert.equal(packet.fileCount, expected.fileCount, "shard file count changed");
    assert.equal(
      packet.diagnosticCount,
      expected.diagnosticCount,
      "shard diagnostic count changed",
    );
    rows.push(...packet.rows);
    expectedRows.push(...expected.rows);
  }
  const stdout = Buffer.from(JSON.stringify(rows, null, 2));
  const expectedStdout = Buffer.from(JSON.stringify(expectedRows, null, 2));
  assertBytesEqual(stdout, expectedStdout, "shards did not reconstruct the sealed base packet");
  return {
    rows,
    stdout,
    fileCount: rows.length,
    diagnosticCount: rows.reduce((sum, row) => sum + row.messages.length, 0),
    byteLength: stdout.length,
    stdoutSha256: sha256(stdout),
  };
}

const PACKET_CONTEXT_BEFORE = 32;
const PACKET_CONTEXT_AFTER = 96;

/** Compare complete packets without asking AssertionError to render a multi-megabyte diff. */
function assertBytesEqual(actual, expected, label) {
  if (actual.equals(expected)) return;
  const common = Math.min(actual.length, expected.length);
  let mismatch = 0;
  while (mismatch < common && actual[mismatch] === expected[mismatch]) mismatch++;
  const start = Math.max(0, mismatch - PACKET_CONTEXT_BEFORE);
  const end = mismatch + PACKET_CONTEXT_AFTER;
  const receipt = {
    actual: {
      byteLength: actual.length,
      sha256: sha256(actual),
      contextBase64: actual.subarray(start, end).toString("base64"),
    },
    expected: {
      byteLength: expected.length,
      sha256: sha256(expected),
      contextBase64: expected.subarray(start, end).toString("base64"),
    },
    firstMismatch: mismatch,
    contextStart: start,
  };
  assert.fail(`${label}: ${JSON.stringify(receipt)}`);
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
  assertBytesEqual(
    stdout,
    expectedStdout,
    "every ordered file/diagnostic field and raw output byte must match the frozen public authority",
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
      assert.ok(
        typeof span.attribution.pass === "string" && span.attribution.pass.length > 0,
        "every request must retain its explicit pass attribution",
      );
      assert.ok(Number.isSafeInteger(span.count) && span.count > 0);
      return { key: span.key, attribution: span.attribution, count: span.count };
    })
    .sort((a, b) =>
      compare(JSON.stringify([a.key, a.attribution]), JSON.stringify([b.key, b.attribution])),
    );
}

/** Require one complete, non-duplicated checker request-count packet. */
export function assertProfilePacket(profile) {
  const counts = profileCounts(profile);
  assert.ok(counts.length > 0, "checker request telemetry must not be empty");
  const identities = counts.map((row) => JSON.stringify([row.key, row.attribution]));
  assert.equal(
    new Set(identities).size,
    identities.length,
    "checker request telemetry identities must be unique",
  );
  return counts;
}

export function assertProfile(profile, expected) {
  assert.deepEqual(
    assertProfilePacket(profile),
    expected,
    "complete checker phase-count vector changed",
  );
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

/** Parse the GNU-time packet retained beside every timed shard. */
export function parseResourceUsage(text) {
  const value = (label, pattern) => {
    const matches = [...text.matchAll(pattern)];
    assert.equal(matches.length, 1, `GNU time must report one ${label}`);
    const number = Number(matches[0][1]);
    assert.ok(Number.isFinite(number) && number >= 0, `${label} must be non-negative`);
    return number;
  };
  const userSeconds = value("user CPU time", /^\s*User time \(seconds\): ([0-9.]+)\s*$/gmu);
  const systemSeconds = value("system CPU time", /^\s*System time \(seconds\): ([0-9.]+)\s*$/gmu);
  return {
    userCpuMs: userSeconds * 1000,
    systemCpuMs: systemSeconds * 1000,
    cpuMs: (userSeconds + systemSeconds) * 1000,
    maximumRssKiB: parseRss(text),
  };
}

const PER_PROJECT_PROFILE_PASSES = new Set([
  "describeCapabilities",
  "getDefaultProjectForFile",
  "initialize",
  "release",
  "updateSnapshot",
]);

/** Current optimized whole-project counts projected onto one identical cohort. */
export function optimizedShardProfile(authority) {
  const shards = authority.cohorts;
  return authority.requestPhaseCounts.head.map((row) => {
    if (PER_PROJECT_PROFILE_PASSES.has(row.attribution.pass)) return structuredClone(row);
    assert.equal(row.count % shards, 0, `${row.attribution.pass} must divide across cohorts`);
    return { ...structuredClone(row), count: row.count / shards };
  });
}

/** Sum complete sorted profile vectors without losing attribution identity. */
export function aggregateProfileCounts(vectors) {
  const counts = new Map();
  for (const vector of vectors) {
    for (const row of vector) {
      const key = JSON.stringify([row.key, row.attribution]);
      const current = counts.get(key);
      counts.set(
        key,
        current == null ? structuredClone(row) : { ...current, count: current.count + row.count },
      );
    }
  }
  return [...counts.values()].sort((a, b) =>
    compare(JSON.stringify([a.key, a.attribution]), JSON.stringify([b.key, b.attribution])),
  );
}

export function median(values) {
  assert.ok(values.length > 0 && values.every((value) => Number.isFinite(value) && value > 0));
  const sorted = [...values].sort((a, b) => a - b);
  const middle = Math.floor(sorted.length / 2);
  return sorted.length % 2 ? sorted[middle] : (sorted[middle - 1] + sorted[middle]) / 2;
}
