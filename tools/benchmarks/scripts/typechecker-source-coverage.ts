/** Untimed independent all-SFC coverage; exact diagnostics retain duplicate multiplicity. */
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { parseArgs } from "node:util";
import { prepareColdScale, coldScaleSupportScriptFiles } from "./cold-typechecker-scale.ts";
import { coldScaleCommand } from "./cold-typechecker-scale-command.ts";
import { createColdScaleRuntime } from "./typechecker-cache-runtime.ts";
import { sealFixtureInputs } from "./typechecker-fixture-identity.ts";

type Diagnostic = { file: string; line: number; column: number; code: number; message: string };
const sha = (p: string) => createHash("sha256").update(fs.readFileSync(p)).digest("hex");
const { values } = parseArgs({
  options: {
    provider: { type: "string" },
    binary: { type: "string" },
    out: { type: "string" },
    sfcs: { type: "string", default: "2500" },
    source: { type: "string" },
  },
});
assert.ok(values.provider && values.binary && values.out);
const provider = fs.realpathSync(values.provider),
  binary = fs.realpathSync(values.binary);
const out = path.resolve(values.out),
  sfcs = Number(values.sfcs);
assert.ok(Number.isSafeInteger(sfcs) && sfcs >= 2500);
assert.ok(!fs.existsSync(out));
fs.mkdirSync(out, { recursive: true });
const binarySha256 = sha(binary);
let build: unknown = null;
if (values.source) {
  assert.match(values.source, /^[a-f0-9]{40}$/u);
  const receipt = JSON.parse(fs.readFileSync(binary + ".differential-build.json", "utf8"));
  assert.equal(receipt.schema, "vize.differential.build");
  assert.equal(receipt.version, 1);
  assert.equal(receipt.recipe, "cargo build --release -p vize");
  assert.equal(receipt.sourceRevision, values.source);
  assert.equal(receipt.binarySha256, binarySha256);
  build = receipt;
}
const projectRoot = path.join(fs.realpathSync(out), "project");
const corpus = prepareColdScale(projectRoot, provider, sfcs + coldScaleSupportScriptFiles, false);
assert.equal(corpus.sfcFiles, sfcs);
const fixtureIdentity = sealFixtureInputs(projectRoot);
fs.writeFileSync(path.join(out, "corpus.json"), JSON.stringify(corpus, null, 2));
const originals: { path: string; source: string }[] = [];
const expected: Diagnostic[] = [];
function point(source: string, token: string) {
  assert.equal(source.split(token).length, 2, "each probe token must be unique within its source");
  const prefix = source.slice(0, source.indexOf(token)).split("\n");
  return { line: prefix.length, column: prefix.at(-1)!.length + 1 };
}
function expect(file: string, source: string, token: string, code: number, message: string) {
  expected.push({ file, ...point(source, token), code, message });
}
const ordering = (a: Diagnostic, b: Diagnostic) =>
  a.file.localeCompare(b.file) ||
  a.line - b.line ||
  a.column - b.column ||
  a.code - b.code ||
  a.message.localeCompare(b.message);
function run(stage: string, count: number, expectedRows: Diagnostic[]) {
  const evidence = path.join(out, stage);
  fs.mkdirSync(evidence);
  const runtime = createColdScaleRuntime(evidence);
  const { command, backend } = coldScaleCommand({
    provider,
    out: evidence,
    binary,
    tool: "cli",
    app: corpus.app,
  });
  const env: NodeJS.ProcessEnv = { ...process.env, ...command.env, ...runtime.env };
  delete env.VIZE_LEXICAL_DOLLAR_CAPTURE_DIR;
  delete env.VIZE_LEXICAL_DOLLAR_CAPTURE_METADATA_ONLY;
  delete env.VIZE_CACHE_CUSTODY_CAPTURE_DIR;
  try {
    const result = spawnSync(command.executable, command.args, {
      cwd: command.cwd,
      env,
      encoding: "utf8",
      timeout: 600_000,
      maxBuffer: 128 * 1024 * 1024,
    });
    fs.writeFileSync(path.join(evidence, "stdout.json"), result.stdout ?? "");
    fs.writeFileSync(path.join(evidence, "stderr.txt"), result.stderr ?? "");
    const processReceipt = {
      executable: command.executable,
      argv: command.args,
      cwd: command.cwd,
      binarySha256,
      backendSha256: sha(backend),
      status: result.status,
      signal: result.signal,
      error: result.error?.message ?? null,
      runtime: runtime.evidence,
    };
    fs.writeFileSync(path.join(evidence, "process.json"), JSON.stringify(processReceipt, null, 2));
    let actual: Diagnostic[] = [],
      error: string | null = null;
    try {
      assert.equal(result.error, undefined);
      assert.equal(result.signal, null);
      const packet = JSON.parse(result.stdout);
      assert.equal(packet.fileCount, count);
      assert.equal(packet.warningCount, 0);
      actual = packet.files.flatMap((row: { file: string; diagnostics: string[] }) =>
        row.diagnostics.map((text) => {
          const match = /^error:(\d+):(\d+) \[TS(\d+)\] ([\s\S]+)$/.exec(text);
          assert.ok(match, "unclassified authored diagnostic");
          const absolute = path.isAbsolute(row.file)
            ? row.file
            : path.resolve(corpus.app, row.file);
          return {
            file: path.relative(projectRoot, fs.realpathSync(absolute)).replaceAll("\\", "/"),
            line: +match[1],
            column: +match[2],
            code: +match[3],
            message: match[4].replace(/ \(source: .*\)$/, ""),
          };
        }),
      );
      assert.equal(packet.errorCount, actual.length);
      assert.equal(actual.length, expectedRows.length, "no missing or duplicate diagnostics");
      assert.deepEqual(actual.toSorted(ordering), expectedRows.toSorted(ordering));
      assert.equal(result.status, expectedRows.length ? 1 : 0);
      assert.equal(sha(binary), binarySha256);
    } catch (failure) {
      error = failure instanceof Error ? (failure.stack ?? failure.message) : String(failure);
    }
    fs.writeFileSync(path.join(evidence, "actual.json"), JSON.stringify(actual, null, 2));
    fs.writeFileSync(path.join(evidence, "expected.json"), JSON.stringify(expectedRows, null, 2));
    const report = {
      stage,
      passed: error === null,
      error,
      expectedCount: expectedRows.length,
      actualCount: actual.length,
      timingEligible: false,
      scope: "per-SFC script and template reachability; not complete type accuracy",
    };
    fs.writeFileSync(path.join(evidence, "acceptance.json"), JSON.stringify(report, null, 2));
    return report;
  } finally {
    runtime.cleanup();
  }
}
const extraValue = path.join(corpus.app, "src/AuthoredExtensionValue.ts");
const extraControl = path.join(corpus.app, "src/AuthoredExtensionControl.ts");
assert.ok(!fs.existsSync(extraValue) && !fs.existsSync(extraControl));
let mutated: ReturnType<typeof run> | null = null;
try {
  for (const [index, entry] of corpus.entries
    .filter((entry) => entry.path.endsWith(".vue"))
    .entries()) {
    const file = path.join(projectRoot, entry.path),
      source = fs.readFileSync(file, "utf8");
    originals.push({ path: file, source });
    assert.equal(source.split("</script>").length, 2);
    assert.equal(source.split("</template>").length, 2);
    const id = String(index).padStart(5, "0"),
      script = "coverageScript" + id,
      missing = "coverageMissing" + id,
      anchor = "coverageAnchor" + id;
    const content = source
      .replace("</script>", `const ${script}: number = 'invalid${id}'; void ${script};\n</script>`)
      .replace("</template>", `{{ ({ ${anchor}: 1 }).${missing} }}\n</template>`);
    fs.writeFileSync(file, content);
    expect(
      entry.path.replaceAll("\\", "/"),
      content,
      script + ": number",
      2322,
      "Type 'string' is not assignable to type 'number'.",
    );
    expect(
      entry.path.replaceAll("\\", "/"),
      content,
      missing,
      2339,
      `Property '${missing}' does not exist on type '{ ${anchor}: number; }'.`,
    );
  }
  fs.writeFileSync(extraValue, "export const coverageSource = 1;\n");
  const source =
    'import { coverageSource } from "./AuthoredExtensionValue.ts";\nexport const checked = coverageSource;\n';
  fs.writeFileSync(extraControl, source);
  expect(
    path.relative(projectRoot, extraControl).replaceAll("\\", "/"),
    source,
    '"./AuthoredExtensionValue.ts"',
    5097,
    "An import path can only end with a '.ts' extension when 'allowImportingTsExtensions' is enabled.",
  );
  assert.equal(expected.length, 2 * sfcs + 1);
  fs.writeFileSync(path.join(out, "originals.json"), JSON.stringify(originals));
  mutated = run("mutated", corpus.files + 2, expected);
} finally {
  for (const row of originals) fs.writeFileSync(row.path, row.source);
  for (const file of [extraValue, extraControl]) if (fs.existsSync(file)) fs.unlinkSync(file);
}
const repaired = run("repaired", corpus.files, []);
assert.equal(
  sealFixtureInputs(projectRoot).sha256,
  fixtureIdentity.sha256,
  "repair must restore every authored input",
);
const receipt = {
  schema: "vize.source-all-sfc-admission",
  version: 1,
  source: values.source ?? "published-cli",
  binarySha256,
  build,
  fixtureIdentity,
  sfcs,
  scriptProbes: sfcs,
  templateProbes: sfcs,
  authoredExtensionProbe: 1,
  expected: 2 * sfcs + 1,
  mutated,
  repaired,
  passed: mutated?.passed === true && repaired.passed,
  scope:
    "untimed independent coverage oracle; duplicates preserved; no full-feature, no-cache or 500ms admission inferred",
};
fs.writeFileSync(path.join(out, "acceptance.json"), JSON.stringify(receipt, null, 2));
console.log(JSON.stringify(receipt));
assert.equal(
  receipt.passed,
  true,
  "all original SFC probes and repaired clean result are mandatory",
);
