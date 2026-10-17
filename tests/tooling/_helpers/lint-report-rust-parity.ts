import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";

export const root = path.resolve(import.meta.dirname, "../../..");
export const command = "tools/commands/fixtures/lint-divergence-report";
export const contractDirectory = "tests/_fixtures/differential/linter/reference-coverage-8849";
export const originalRef = "49545646fde08ade1000c4a33b38c9dd1ac4fb99";
export const updatedRef = "7421901d05ac2e252636dd8ee426e8783255378d";
const common = [
  [
    "tools/support/common.rs",
    "c23e55a197a410a899082875ffc34e4478573d855f3463ec8ebc6780644eb497",
    8265,
  ],
  [
    "tools/support/lint_divergence_surface.rs",
    "ccb05dcaabf1285f66f524276d3ab1744739c2c1cb2679691c51efb87f2ea30e",
    4194,
  ],
] as const;
const sources = {
  original: {
    ref: originalRef,
    files: [
      ...common,
      [`${command}.rs`, "d5ab4a8f86edcaf6934f518991d68ded8bbc50e3e1cb83217395b14faff02f5a", 68133],
    ],
  },
  updated: {
    ref: updatedRef,
    files: [
      ...common,
      [`${command}.rs`, "e46b782949303b4f91132eed7f16e9cd65d3182fcbe6e21019a54752de370622", 72562],
      [
        "tools/support/lint_reference_coverage.rs",
        "47a62d566061d742810d79a0d7cbf4fccf6fab226e6e1e50fb71df8ca5e540db",
        3268,
      ],
      [
        `${contractDirectory}/contract.json`,
        "ca146c3cf3e6e64484956d3eeef959fb04b68c0f59fd3d945f5e6c4df826e955",
        2885,
      ],
    ],
  },
} as const;
export type Oracle = keyof typeof sources;
export type Json = any;
const hash = (bytes: string | Buffer) => createHash("sha256").update(bytes).digest("hex");
export function evidenceDirectory() {
  const directory =
    process.env.VIZE_LINT_REPORT_PARITY_DIR ??
    path.join(process.env.RUNNER_TEMP ?? os.tmpdir(), "lint-report-parity");
  fs.mkdirSync(directory, { recursive: true });
  return directory;
}
function parseReport(text: string): Json {
  return JSON.parse(text, (_key: string, value: Json, context?: { source?: string }) => {
    if (
      typeof value === "number" &&
      !Number.isSafeInteger(value) &&
      typeof context?.source === "string" &&
      /^-?\d+$/u.test(context.source)
    ) {
      return BigInt(context.source);
    }
    return value;
  });
}

export function materializeFrozenReporters() {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-frozen-lint-report-"));
  const paths = {} as Record<Oracle, string>;
  try {
    for (const [name, source] of Object.entries(sources)) {
      for (const [file, expectedHash, expectedBytes] of source.files) {
        const result = spawnSync("git", ["show", `${source.ref}:${file}`], { cwd: root });
        assert.equal(
          result.status,
          0,
          `Fetch exact frozen reference ${source.ref}: ${result.stderr}`,
        );
        assert.equal(result.stdout.length, expectedBytes, file);
        assert.equal(hash(result.stdout), expectedHash, `${source.ref}:${file}`);
        if (file === `${command}.rs`) {
          assert.equal(
            result.stdout.toString().trimEnd().split("\n").length,
            name === "original" ? 1958 : 2072,
          );
        }
        const target = path.join(directory, name, file);
        fs.mkdirSync(path.dirname(target), { recursive: true });
        fs.writeFileSync(target, result.stdout);
      }
      paths[name as Oracle] = path.join(directory, name, `${command}.rs`);
    }
    fs.writeFileSync(
      path.join(evidenceDirectory(), "frozen-source-custody.json"),
      JSON.stringify(sources, null, 2) + "\n",
    );
    return { paths, clean: () => fs.rmSync(directory, { recursive: true, force: true }) };
  } catch (error) {
    fs.rmSync(directory, { recursive: true, force: true });
    throw error;
  }
}

export function fixture() {
  fs.mkdirSync(path.join(root, "target"), { recursive: true });
  const directory = fs.mkdtempSync(path.join(root, "target", "lint-report-parity-"));
  const output = path.join(directory, "output");
  const transcript = path.join(directory, "processes.jsonl");
  const response = path.join(directory, "response.json");
  const binary = path.join(directory, "fake-vize.mjs");
  const registry = path.join(directory, "registry.json");
  fs.writeFileSync(
    binary,
    [
      "#!/usr/bin/env node",
      'import fs from "node:fs";',
      `const response = JSON.parse(fs.readFileSync(${JSON.stringify(response)}, "utf8"));`,
      "const stdin = fs.readFileSync(0);",
      "const stdout = response.stdout ?? JSON.stringify(response.results);",
      'const stderr = response.stderr ?? "";',
      `fs.appendFileSync(${JSON.stringify(transcript)}, JSON.stringify({`,
      "  argv: process.argv.slice(2), cwd: process.cwd(), lang: process.env.LANG,",
      '  lcAll: process.env.LC_ALL, stdin: stdin.toString("base64"),',
      '  stdout: Buffer.from(stdout).toString("base64"), stderr: Buffer.from(stderr).toString("base64"),',
      "  status: response.status ?? 0,",
      '}) + "\\n");',
      "process.stdout.write(stdout); process.stderr.write(stderr); process.exitCode = response.status ?? 0;",
      "",
    ].join("\n"),
  );
  fs.chmodSync(binary, 0o755);
  const project = (id = "parity", extra: Json = {}) => ({
    id,
    revision: "0".repeat(40),
    fixturePath: path.relative(root, directory),
    vueGlobs: ["App.vue"],
    coverage: ["linter"],
    ...extra,
  });
  const write = (file: string, contents: string) => {
    fs.mkdirSync(path.dirname(path.join(directory, file)), { recursive: true });
    fs.writeFileSync(path.join(directory, file), contents);
  };
  write("App.vue", "<template><div /></template>\n");
  const configure = (
    projects = [project()],
    value: Json = { results: [{ file: "App.vue", messages: [] }] },
  ) => {
    fs.writeFileSync(registry, JSON.stringify({ projects }));
    fs.writeFileSync(response, JSON.stringify(value));
  };
  configure();
  return {
    directory,
    output,
    transcript,
    binary,
    project,
    write,
    configure,
    args: [
      "--registry",
      registry,
      "--output-dir",
      output,
      "--vize-bin",
      binary,
      "--timeout-ms",
      "30000",
    ],
    clean: () => fs.rmSync(directory, { recursive: true, force: true }),
  };
}
export type Fixture = ReturnType<typeof fixture>;
let measurement = 0;

export function runReporter(input: Fixture, source: string, args = input.args, rust = false) {
  fs.rmSync(input.output, { recursive: true, force: true });
  fs.rmSync(input.transcript, { force: true });
  const result = spawnSync(rust ? "rust-script" : process.execPath, [source, ...args], {
    cwd: root,
    encoding: "utf8",
    timeout: 240_000,
    maxBuffer: 8 * 1024 * 1024,
    env: { ...process.env, VIZE_REPO_ROOT: root, LANG: "C", LC_ALL: "C" },
  });
  assert.equal(result.error, undefined, result.error?.message);
  assert.equal(result.signal, null, `reporter terminated by ${result.signal}`);
  const files: Record<string, Json> = {};
  const rawFiles: Record<string, string> = {};
  if (fs.existsSync(input.output)) {
    for (const name of fs.readdirSync(input.output).sort()) {
      const text = fs.readFileSync(path.join(input.output, name), "utf8");
      rawFiles[name] = text;
      files[name] = name.endsWith(".json") ? parseReport(text) : text;
    }
  }
  const processes = fs.existsSync(input.transcript)
    ? fs
        .readFileSync(input.transcript, "utf8")
        .trim()
        .split("\n")
        .filter(Boolean)
        .map((row) => JSON.parse(row))
    : [];
  const captured = {
    status: result.status,
    stdout: result.stdout,
    stderr: result.stderr,
    processes,
    files,
  };
  fs.writeFileSync(
    path.join(
      evidenceDirectory(),
      `${String(++measurement).padStart(3, "0")}-${rust ? "rust" : "node"}.json`,
    ),
    JSON.stringify(
      { source, args, ...captured, rawFiles },
      (_key, value) => (typeof value === "bigint" ? { exactIntegerToken: String(value) } : value),
      2,
    ) + "\n",
  );
  for (const process of processes) {
    assert.equal(process.stdin, "");
    assert.equal(process.lang, "C");
    assert.equal(process.lcAll, "C");
    assert.equal(process.cwd, input.directory);
  }
  return captured;
}
export type Result = ReturnType<typeof runReporter>;

function canonical(value: Json, key = ""): string {
  if (typeof value === "bigint") return String(value);
  if (Array.isArray(value)) return `[${value.map((item) => canonical(item)).join(",")}]`;
  if (value !== null && typeof value === "object") {
    return `{${Object.keys(value)
      .sort()
      .map((name) => `${JSON.stringify(name)}:${canonical(value[name], name)}`)
      .join(",")}}`;
  }
  // serde retains the two ratio fields as f64 even when they are zero or one.
  if (["falsePositiveRatio", "falseNegativeRatio"].includes(key) && Number.isInteger(value))
    return `${value}.0`;
  return JSON.stringify(value);
}
export function assertDigest(divergence: Json) {
  const { schema, version, project, upstream, sha256, ...payload } = divergence;
  assert.equal(schema, "vize.fixtureLintDivergence");
  assert.equal(version, 1);
  assert.equal(sha256, hash(canonical(payload)), "complete independently derived digest preimage");
}

function hostSeams(result: Result, rust: boolean) {
  const rustc = rust ? spawnSync("rustc", ["--version"], { encoding: "utf8" }) : undefined;
  if (rustc) assert.equal(rustc.status, 0);
  for (const artifact of Object.values(result.files)) {
    if (typeof artifact !== "object") continue;
    assert.deepEqual(
      artifact.evidence.runtime,
      rust
        ? { name: "rust-script", version: rustc!.stdout.trim() }
        : { name: "node", version: process.version },
    );
    delete artifact.evidence.runtime;
    if (artifact.baseline) {
      assert.ok(Number.isSafeInteger(artifact.baseline.durationMs));
      assert.ok(artifact.baseline.durationMs >= 0 && artifact.baseline.durationMs <= 240_000);
      delete artifact.baseline.durationMs;
    }
  }
}

function removeOriginalAdditions(artifact: Json) {
  const require = createRequire(path.join(root, "tools/benchmarks/scripts/package.json"));
  const versions = Object.fromEntries(
    ["eslint", "eslint-plugin-vue", "vue-eslint-parser", "@typescript-eslint/parser"].map(
      (name) => [name, require(`${name}/package.json`).version],
    ),
  );
  assert.deepEqual(
    artifact.baseline.runtimeVersions,
    artifact.baseline.comparedRuleCount ? versions : {},
  );
  assert.deepEqual(Object.keys(artifact.raw).sort(), [
    "eslintResults",
    "patinaFindings",
    "patinaResults",
  ]);
  assert.ok(Array.isArray(artifact.raw.eslintResults));
  assert.ok(Array.isArray(artifact.raw.patinaResults));
  assert.deepEqual(
    artifact.raw.patinaFindings,
    artifact.raw.patinaResults.flatMap((entry: Json) =>
      entry.messages.map((finding: Json) => ({ ...finding, file: entry.file })),
    ),
  );
  assert.deepEqual(artifact.divergence.referenceCoverageExpectations, []);
  assert.equal(artifact.divergence.summary.referenceCoverageIntegrityErrorCount, 0);
  assertDigest(artifact.divergence);
  delete artifact.raw;
  delete artifact.baseline.runtimeVersions;
  delete artifact.divergence.referenceCoverageExpectations;
  delete artifact.divergence.summary.referenceCoverageIntegrityErrorCount;
  const { schema, version, project, upstream, sha256, ...payload } = artifact.divergence;
  artifact.divergence.sha256 = hash(canonical(payload));
}

export function assertParity(expected: Result, actual: Result, oracle: Oracle) {
  expected = structuredClone(expected);
  actual = structuredClone(actual);
  hostSeams(expected, true);
  hostSeams(actual, false);
  if (oracle === "original" && actual.files["lint-divergence-summary.json"]) {
    for (const row of actual.files["lint-divergence-summary.json"].projects) {
      assert.equal(row.referenceCoverageIntegrityErrorCount, 0);
      delete row.referenceCoverageIntegrityErrorCount;
    }
  }
  for (const [file, artifact] of Object.entries(actual.files)) {
    if (!file.endsWith("-lint-divergence.json")) continue;
    assertDigest(artifact.divergence);
    if (oracle === "original") {
      removeOriginalAdditions(artifact);
      assert.equal(artifact.divergence.summary.patinaOutsideBaselineSurfaceCount, 0);
      const markdown = file.replace(/\.json$/u, ".md");
      const additions = [
        `Patina findings outside reference coverage: ${artifact.divergence.summary.patinaOutsideBaselineSurfaceCount}\n`,
        "Reference coverage integrity errors: 0\n",
        "### Patina findings outside reference coverage: none\n\n",
      ];
      for (const addition of additions) {
        assert.equal(
          actual.files[markdown].split(addition).length,
          2,
          `exact Markdown addition: ${addition}`,
        );
        actual.files[markdown] = actual.files[markdown].replace(addition, "");
      }
    }
  }
  assert.deepEqual(actual, expected);
}
