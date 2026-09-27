import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { parseTomlLite } from "../../support/compat/davinci/toml-lite.mjs";

export const ID = /^[A-Za-z0-9._-]+$/;
export const METHODOLOGY_KEYS = [
  "allocator",
  "flags",
  "fixture_digest",
  "guest_context",
  "libc",
  "libc_dispatch",
  "profile",
  "rustc",
  "target",
  "valgrind",
  "window_protocol",
];
export const BENCH_KEYS = ["fixture", "fixture_sha256", "instructions", "window"];
export const ALLOCATOR_PROTOCOL = "counting-mimalloc-reserve128m-preinit64m-purgeoff-v1";
export const LIBC_DISPATCH =
  "glibc.cpu.hwcaps=-AVX,-AVX2,-AVX512F,-AVX512VL,-AVX512BW,-AVX_Fast_Unaligned_Load,-ERMS,-FSRM,-Prefer_ERMS,-Prefer_FSRM,-SSSE3,-SSE4_1,-SSE4_2";
export const GUEST_ENVIRONMENT = Object.freeze({
  PATH: "/usr/bin:/bin",
  LANG: "C",
  LC_ALL: "C",
  VIZE_INSTRUCTION_COUNTS: "1",
  MIMALLOC_RESERVE_OS_MEMORY: "128MiB",
  MIMALLOC_PURGE_DELAY: "-1",
  MIMALLOC_VERBOSE: "1",
  GLIBC_TUNABLES: LIBC_DISPATCH,
});
const HASH = /^[a-f0-9]{64}$/;

function fields(value, keys, where) {
  assert.ok(
    value && typeof value === "object" && !Array.isArray(value),
    `${where}: expected object`,
  );
  assert.deepEqual(
    Object.keys(value).sort((left, right) => left.localeCompare(right)),
    [...keys].sort((left, right) => left.localeCompare(right)),
    `${where}: unexpected or missing fields`,
  );
}

function positiveInteger(value, where) {
  assert.ok(Number.isSafeInteger(value) && value > 0, `${where}: expected positive safe integer`);
}

export function validateMethodology(value) {
  fields(value, METHODOLOGY_KEYS, "methodology");
  for (const key of METHODOLOGY_KEYS) {
    assert.ok(typeof value[key] === "string" && value[key].length > 0, `methodology.${key}: empty`);
  }
  assert.equal(value.target, "x86_64-unknown-linux-gnu", "measurement target must be Linux x86_64");
  assert.equal(value.profile, "ci-opt", "measurement profile must be ci-opt");
  assert.equal(value.allocator, ALLOCATOR_PROTOCOL, "measurement allocator changed");
  assert.equal(value.fixture_digest, "input-identity-v2", "measurement input digest changed");
  assert.equal(
    value.guest_context,
    "fixed-env-fixed-argv0-v1",
    "measurement guest context changed",
  );
  assert.equal(value.libc_dispatch, LIBC_DISPATCH, "measurement libc dispatch changed");
  assert.equal(
    value.window_protocol,
    "callgrind-client-call-boundary-v1",
    "measurement windows changed",
  );
  assert.equal(
    value.flags,
    "target-cpu=x86-64;instr-atstart=no;cache-sim=no;branch-sim=no",
    "measurement flags changed",
  );
}

// Check the allocator's runtime output, not merely the requested environment.
// The harness emits its marker only after the allocator-only range is dropped
// outside every Callgrind window. Reservation failure cannot publish a baseline.
export function validateAllocatorSetup(stderr) {
  assert.match(
    stderr,
    /option 'reserve_os_memory': 131072 KiB/,
    "allocator reservation option missing",
  );
  assert.match(stderr, /option 'purge_delay': -1\s/, "allocator purge option changed");
  assert.match(stderr, /reserved 131072 KiB memory/, "allocator reservation did not succeed");
  const markers = [...stderr.matchAll(/^VIZE_INSTRUCTION_ALLOCATOR (.+)$/gm)];
  assert.equal(markers.length, 1, "expected one allocator preinitialization marker");
  assert.deepEqual(JSON.parse(markers[0][1]), { preinitialized_bytes: 67108864 });
}

export function validateBench(value, id) {
  assert.ok(ID.test(id), `invalid benchmark id: ${id}`);
  fields(value, BENCH_KEYS, id);
  positiveInteger(value.instructions, `${id}.instructions`);
  assert.ok(typeof value.fixture === "string" && value.fixture.length > 0, `${id}: empty fixture`);
  assert.ok(HASH.test(value.fixture_sha256), `${id}: invalid fixture digest`);
  assert.ok(
    ["routine-return", "stage-return"].includes(value.window),
    `${id}: invalid stage window`,
  );
}

export function loadRegistry(file) {
  const parsed = parseTomlLite(fs.readFileSync(file, "utf8"));
  assert.ok(parsed.bench && Object.keys(parsed.bench).length > 0, "allocation registry is empty");
  return new Set(Object.keys(parsed.bench));
}

export function reconcile(actual, expected, where) {
  const missing = [...expected].filter((id) => !actual.has(id));
  const extra = [...actual].filter((id) => !expected.has(id));
  assert.ok(
    missing.length === 0 && extra.length === 0,
    `${where}: missing [${missing.join(", ")}], unregistered [${extra.join(", ")}]`,
  );
}

// Callgrind's totals are exclusive costs, not summed inclusive call costs.
// The named client dump is mandatory; process-termination dumps are ignored
// only if they contain exactly zero collected instructions.
export function parseCallgrind(text) {
  assert.doesNotMatch(
    text,
    /(?:c?fn=).*__(?:memcmp|memcpy|memmove|mempcpy|memchr|memrchr|strlen|strnlen|strcmp|strncmp)_(?:avx|evex|ssse3|sse4|erms|sse2_\w*erms)/,
    "Callgrind: address-sensitive hardware-dispatched libc routine",
  );
  const one = (key) => {
    const matches = [...text.matchAll(new RegExp(`^${key}:\\s*(.*)$`, "gm"))];
    assert.equal(matches.length, 1, `Callgrind: expected exactly one ${key} header`);
    return matches[0][1].trim();
  };
  assert.equal(one("version"), "1", "Callgrind format version changed");
  assert.equal(one("events"), "Ir", "Callgrind must collect only Ir");
  const raw = one("totals");
  assert.ok(/^\d+$/.test(raw), "Callgrind: malformed instruction total");
  const instructions = Number(raw);
  assert.ok(Number.isSafeInteger(instructions), "Callgrind: unsafe instruction total");
  const trigger = one("desc: Trigger");
  if (trigger === "Program termination") {
    assert.equal(instructions, 0, "instructions leaked outside measured windows");
    return null;
  }
  const prefix = "Client Request: ";
  assert.ok(trigger.startsWith(prefix), `Callgrind: unexpected trigger ${trigger}`);
  const bench_id = trigger.slice(prefix.length);
  assert.ok(ID.test(bench_id), `Callgrind: invalid named dump ${bench_id}`);
  positiveInteger(instructions, `Callgrind ${bench_id}`);
  return { bench_id, instructions };
}

export function parseIdentities(stderr) {
  const result = new Map();
  for (const line of stderr.split("\n")) {
    if (!line.startsWith("VIZE_INSTRUCTION_BENCH ")) continue;
    const row = JSON.parse(line.slice("VIZE_INSTRUCTION_BENCH ".length));
    fields(row, ["bench_id", "fixture", "window"], "instruction identity");
    assert.ok(ID.test(row.bench_id), "invalid instruction identity");
    assert.ok(!result.has(row.bench_id), `duplicate identity ${row.bench_id}`);
    result.set(row.bench_id, row);
  }
  assert.ok(result.size > 0, "no instrumented benchmark identities");
  return result;
}

export function fixtureDigest(root, fixture, source) {
  const relative = fixture.startsWith("fixture:") ? fixture.slice("fixture:".length) : fixture;
  const file = path.resolve(root, relative);
  assert.ok(file.startsWith(`${root}${path.sep}`), "fixture escapes workspace");
  const direct = fs.existsSync(file);
  const content = fs.readFileSync(direct ? file : source);
  if (direct) return createHash("sha256").update(content).digest("hex");
  // These four level probes have exact input constants, independent of the
  // bench target, function, module, type and fixture-file names. Preserve the
  // same ids and numeric ceilings across the ordered structural rename.
  const levelInputs = {
    "synthetic:v-for-three-aliases": "VFOR_THREE_ALIASES",
    "synthetic:v-on-two-option-event-key-modifiers": "VON_TWO_PER_BUCKET",
    "synthetic:p2-11-dom-surface": "P2_11_DOM_SURFACE",
    "fixture:complexity/dashboard.vue": "COMPLEXITY_DASHBOARD",
  };
  if (Object.hasOwn(levelInputs, fixture)) {
    const name = levelInputs[fixture];
    const declaration = content
      .toString()
      .match(
        new RegExp(`const ${name}: &str =\\s*(r#"([\\s\\S]*?)"#|include_str!\\("([^"]+)"\\));`),
      );
    assert.ok(declaration, `${fixture}: exact level input constant missing`);
    const input =
      declaration[3] === undefined
        ? Buffer.from(declaration[2])
        : fs.readFileSync(path.resolve(path.dirname(source), declaration[3]));
    return createHash("sha256").update(input).digest("hex");
  }
  const normalized = content
    .toString()
    .replace(/include_str!\("[^"]+"\)/g, 'include_str!("<input-bytes>")');
  const hash = createHash("sha256").update(fixture).update("\0").update(normalized);
  if (!direct) {
    for (const [, include] of content.toString().matchAll(/include_str!\("([^"]+)"\)/g)) {
      hash.update("\0").update(fs.readFileSync(path.resolve(path.dirname(source), include)));
    }
  }
  return hash.digest("hex");
}

export function validateMeasurement(report, registry) {
  fields(
    report,
    ["schema_version", "source_commit", "recorded_run", "methodology", "runs"],
    "measurement",
  );
  assert.equal(report.schema_version, 1, "measurement schema version changed");
  assert.ok(
    /^[a-f0-9]{40}$/.test(report.source_commit),
    "measurement requires exact source commit",
  );
  assert.ok(
    /^https:\/\/github\.com\/[^/]+\/[^/]+\/actions\/runs\/\d+$/.test(report.recorded_run),
    "measurement requires GitHub Actions provenance",
  );
  validateMethodology(report.methodology);
  assert.ok(
    Array.isArray(report.runs) && report.runs.length === 3,
    "exactly three measurement runs required",
  );
  for (const [index, run] of report.runs.entries()) {
    assert.ok(run && typeof run === "object" && !Array.isArray(run), "invalid measurement run");
    reconcile(new Set(Object.keys(run)), registry, `measurement run ${index + 1}`);
    for (const [id, row] of Object.entries(run)) validateBench(row, id);
  }
  for (const id of registry) {
    for (const run of report.runs.slice(1)) {
      assert.deepEqual(run[id], report.runs[0][id], `unstable instruction measurement: ${id}`);
    }
  }
  return report;
}

export function validateBudgets(value, registry) {
  fields(
    value,
    ["schema_version", "source_commit", "recorded_run", "methodology", "instruction"],
    "instruction budgets",
  );
  const { instruction, ...metadata } = value;
  assert.ok(
    instruction && typeof instruction === "object" && Object.keys(instruction).length > 0,
    "instruction budgets must contain measured rows",
  );
  // Reuse the same strict metadata, row and provenance validators.
  validateMeasurement({ ...metadata, runs: [instruction, instruction, instruction] }, registry);
  return value;
}

export function loadBudgets(file, registry) {
  const parsed = parseTomlLite(fs.readFileSync(file, "utf8"));
  assert.ok(
    parsed.instruction && typeof parsed.instruction === "object",
    "instruction registry missing",
  );
  return validateBudgets(parsed, registry ?? new Set(Object.keys(parsed.instruction)));
}

export function checkMeasurement(report, budget) {
  assert.deepEqual(
    report.methodology,
    budget.methodology,
    "instruction methodology changed; collect a reviewed baseline",
  );
  const failures = [];
  for (const [id, row] of Object.entries(report.runs[0])) {
    const limit = budget.instruction[id];
    assert.ok(limit, `missing instruction budget ${id}`);
    for (const key of ["fixture_sha256", "window"]) {
      assert.equal(row[key], limit[key], `${id}: measurement identity changed (${key})`);
    }
    if (row.instructions > limit.instructions) {
      failures.push(`${id}: ${row.instructions} > ${limit.instructions}`);
    }
  }
  assert.equal(failures.length, 0, `instruction budget exceeded:\n${failures.join("\n")}`);
}

export function ratchetBudgets(current, base) {
  // A toolchain/environment refresh cannot silently replace a ratchet.
  assert.deepEqual(current.methodology, base.methodology, "ratchet methodology changed");
  for (const [id, row] of Object.entries(base.instruction)) {
    assert.ok(current.instruction[id], `instruction budget dropped: ${id}`);
    assert.ok(
      current.instruction[id].instructions <= row.instructions,
      `instruction budget loosened: ${id}`,
    );
  }
}

export function baselineToml(report) {
  const lines = [
    "# Measured instruction ceilings. Every row must come from three identical Actions runs.",
    "# Ratchet: existing ceilings may only decrease; rows may never disappear.",
    `schema_version = ${report.schema_version}`,
    `source_commit = ${JSON.stringify(report.source_commit)}`,
    `recorded_run = ${JSON.stringify(report.recorded_run)}`,
    "",
    "[methodology]",
  ];
  for (const key of METHODOLOGY_KEYS)
    lines.push(`${key} = ${JSON.stringify(report.methodology[key])}`);
  lines.push("", "[instruction]");
  for (const [id, row] of Object.entries(report.runs[0]).sort(([a], [b]) => a.localeCompare(b))) {
    lines.push(
      `${id} = { ${BENCH_KEYS.map((key) => `${key} = ${JSON.stringify(row[key])}`).join(", ")} }`,
    );
  }
  return `${lines.join("\n")}\n`;
}
