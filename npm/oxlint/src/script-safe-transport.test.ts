import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import { prepareWorkaroundSource } from "./workaround.ts";

const packageDir = fileURLToPath(new URL("..", import.meta.url));
const repository = path.resolve(packageDir, "../..");
const corpus = path.join(
  repository,
  "tests/_fixtures/differential/lint/oxlint-script-safe-carrier-7903",
);
const engine =
  process.env.VIZE_OXLINT_TEST_ENTRYPOINT ??
  path.join(repository, "node_modules/oxlint/bin/oxlint");
const plugin = path.join(packageDir, "dist/index.mjs");
const cli = path.join(packageDir, "dist/cli.mjs");
const manifest = JSON.parse(fs.readFileSync(path.join(corpus, "source.json"), "utf8")) as {
  cases: { file: string; originalFilename: string; sha256: string; expectedSpan: unknown }[];
};
const frozen = JSON.parse(fs.readFileSync(path.join(corpus, "rules.json"), "utf8"));
type Packet = {
  code?: string;
  filename: string;
  severity: string;
  message: string;
  labels: unknown[];
};
type Report = {
  diagnostics: Packet[];
  number_of_files: number;
  number_of_rules: number;
  threads_count: number;
  start_time: number;
};
const capture: unknown[] = [];
const capturePath =
  process.env.VIZE_OXLINT_SCRIPT_SAFE_CAPTURE ??
  path.join(repository, "target/oxlint-script-safe-carrier-7903.json");
function save(row: unknown): void {
  capture.push(row);
  fs.mkdirSync(path.dirname(capturePath), { recursive: true });
  fs.writeFileSync(capturePath, `${JSON.stringify(capture, null, 2)}\n`);
}

void test("safe carriers preserve complete original packets and activate malformed/scriptless originals", (t) => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-safe-originals-"));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  fs.mkdirSync(path.join(root, "node_modules/oxlint/bin"), { recursive: true });
  fs.symlinkSync(engine, path.join(root, "node_modules/oxlint/bin/oxlint"));
  fs.mkdirSync(path.join(root, "src"));
  fs.mkdirSync(path.join(root, ".git"));
  fs.writeFileSync(path.join(root, ".gitignore"), "src/Ignored.vue\n");
  const originals = manifest.cases.map((entry) => ({
    ...entry,
    source: fs.readFileSync(path.join(corpus, entry.file), "utf8"),
  }));
  for (const entry of originals) {
    assert.equal(createHash("sha256").update(entry.source).digest("hex"), entry.sha256);
    fs.writeFileSync(path.join(root, "src", entry.originalFilename), entry.source);
  }
  fs.writeFileSync(
    path.join(root, "src/OriginalSelf.ts"),
    "import './OriginalSelf.ts';\ndebugger;\nexport const answer = 1;\n",
  );
  fs.writeFileSync(
    path.join(root, "src/Ignored.vue"),
    '<template><div v-html="ignored"/></template>\n',
  );
  const owner = path.join(root, "owner.mjs");
  fs.writeFileSync(
    owner,
    `export default {meta:{name:'owner'},rules:{source:{meta:{type:'problem',schema:[{type:'object',properties:{token:{type:'string'}},required:['token'],additionalProperties:false}]},create(context){return{Program(program){context.report({node:program,message:'Owner '+context.physicalFilename+' option '+context.options[0].token});}};}}}};`,
  );
  const config = {
    ...frozen,
    plugins: ["vue", "import"],
    jsPlugins: [plugin, owner],
    rules: {
      ...frozen.rules,
      "no-debugger": "warn",
      "import/no-self-import": "error",
      "owner/source": ["warn", { token: "configured-options" }],
    },
  };
  const configPath = path.join(root, ".oxlintrc.json");
  const run = (entry: string, args = ["-f", "json", "--threads", "1", "src"]): Report => {
    const result = spawnSync(process.execPath, [entry, ...args], {
      cwd: root,
      encoding: "utf8",
      timeout: 30_000,
      env: { ...process.env, VIZE_OXLINT_NATIVE_CONTROL: args.at(-1) },
    });
    save({ entry, args, configBytes: fs.readFileSync(configPath, "utf8"), ...result });
    assert.equal(result.status, 1, result.stderr);
    assert.equal(result.stderr, "");
    return JSON.parse(result.stdout) as Report;
  };
  const foreign = (report: Report) =>
    report.diagnostics.filter((packet) => !packet.code?.startsWith("vize("));
  for (const imports of [true, false]) {
    const { "import/no-self-import": importRule, ...rules } = config.rules;
    const configBytes = JSON.stringify({
      ...config,
      plugins: imports ? config.plugins : ["vue"],
      rules: imports ? { ...rules, "import/no-self-import": importRule } : rules,
    });
    fs.writeFileSync(configPath, configBytes);
    const stock = run(engine),
      bridged = run(cli);
    assert.deepEqual(
      foreign(bridged),
      foreign(stock),
      `complete ${imports ? "import" : "ordinary"} packets, multiplicity and traversal order`,
    );
    assert.equal(foreign(bridged).length, imports ? 13 : 12);
    const scriptErrors = foreign(bridged).filter((packet) => packet.code === undefined);
    assert.equal(scriptErrors.length, 5);
    assert.equal(new Set(scriptErrors.map((packet) => packet.filename)).size, 5);
    for (const entry of originals) {
      const packets = bridged.diagnostics.filter(
        (packet) =>
          packet.filename === `src/${entry.originalFilename}` &&
          packet.code === "vize(vue/no-v-html)",
      );
      assert.equal(packets.length, 1, entry.originalFilename);
      assert.equal(packets[0].severity, "error");
      assert.deepEqual(packets[0].labels, [{ span: entry.expectedSpan }]);
    }
    for (const key of ["number_of_files", "number_of_rules", "threads_count"] as const)
      assert.equal(bridged[key], stock[key]);
    assert.equal(fs.readFileSync(configPath, "utf8"), configBytes);
    assert.equal(
      bridged.diagnostics.some((packet) => packet.filename.includes("Ignored")),
      false,
    );
    const selectedConfig = JSON.parse(configBytes);
    selectedConfig.settings.vize.rules = frozen.rules;
    fs.writeFileSync(configPath, JSON.stringify(selectedConfig));
    const batched = run(cli);
    assert.deepEqual(
      batched.diagnostics,
      bridged.diagnostics,
      "all selected-rule carrier packets remain exact with the 51-rule batching hint",
    );
    for (const key of ["number_of_files", "number_of_rules", "threads_count"] as const)
      assert.equal(batched[key], bridged[key]);
    fs.writeFileSync(configPath, configBytes);
  }
  for (const entry of originals)
    assert.equal(
      fs.readFileSync(path.join(root, "src", entry.originalFilename), "utf8"),
      entry.source,
    );

  const source = originals[0].source,
    original = path.join(root, "src", originals[0].originalFilename);
  const safe = prepareWorkaroundSource(source, original);
  const guards = [
    safe.source.replace(
      /(<!--vize-original-source:)([a-f0-9])/u,
      (_, prefix: string, value: string) => prefix + (value === "0" ? "1" : "0"),
    ),
    safe.source.replace(/:([A-Za-z0-9_-]*)-->$/u, ":%-->"),
    safe.source.replace(
      Buffer.from(original).toString("base64url"),
      Buffer.from(path.join(root, "Other.vue")).toString("base64url"),
    ),
    safe.source.slice(0, safe.locations.scriptStart) +
      "\t" +
      safe.source.slice(safe.locations.scriptStart + 1),
    prepareWorkaroundSource(source.replaceAll("html", "evil"), original).source,
  ];
  fs.writeFileSync(configPath, JSON.stringify({ ...frozen, jsPlugins: [plugin] }));
  for (const [index, bytes] of guards.entries()) {
    const file = `Guard-${index}.vue`;
    fs.writeFileSync(path.join(root, file), bytes);
    const report = run(engine, ["-f", "json", "--threads", "1", file]);
    assert.equal(report.diagnostics.length, 1);
    assert.equal(report.diagnostics[0].code, undefined);
    assert.equal(report.diagnostics[0].severity, "error");
    assert.match(report.diagnostics[0].message, /Invalid Vize|carrier bytes|original file bytes/u);
  }
  assert.equal(fs.readFileSync(original, "utf8"), source);
  fs.unlinkSync(configPath);
  fs.writeFileSync(
    path.join(root, ".oxlintrc.jsonc"),
    `// authored JSONC\n${JSON.stringify(config)}`,
  );
  for (const configArgs of [[], ["-c", ".oxlintrc.jsonc"]]) {
    // Remove JSONC for the no-config control so the actual stock engine owns
    // precisely the same original parser/core reports in both invocations.
    const jsonc = path.join(root, ".oxlintrc.jsonc");
    const bytes = fs.readFileSync(jsonc, "utf8");
    if (configArgs.length === 0) fs.unlinkSync(jsonc);
    const args = [...configArgs, "-f", "json", "--threads", "1", "src"];
    const invoke = (entry: string) => {
      const result = spawnSync(process.execPath, [entry, ...args], {
        cwd: root,
        encoding: "utf8",
        timeout: 30_000,
      });
      save({ qualification: "unsupported-config refusal", entry, args, ...result });
      return result;
    };
    const stock = invoke(engine),
      refused = invoke(cli);
    assert.equal(refused.status, 1);
    assert.match(refused.stderr, /Vize transport is unqualified/u);
    assert.deepEqual(JSON.parse(refused.stdout).diagnostics, JSON.parse(stock.stdout).diagnostics);
    for (const key of ["number_of_files", "number_of_rules", "threads_count"])
      assert.equal(JSON.parse(refused.stdout)[key], JSON.parse(stock.stdout)[key]);
    if (configArgs.length === 0) fs.writeFileSync(jsonc, bytes);
  }
  fs.writeFileSync(configPath, "{ malformed");
  const invalid = (entry: string) =>
    spawnSync(process.execPath, [entry, "-f", "json", "src"], {
      cwd: root,
      encoding: "utf8",
      timeout: 30_000,
    });
  for (const conflict of [true, false]) {
    if (!conflict) fs.unlinkSync(path.join(root, ".oxlintrc.jsonc"));
    const invalidStock = invalid(engine),
      invalidWrapper = invalid(cli);
    save({
      qualification: "original invalid-config failure",
      conflict,
      invalidStock,
      invalidWrapper,
    });
    assert.equal(invalidWrapper.status, invalidStock.status);
    assert.equal(invalidWrapper.stdout, invalidStock.stdout);
    assert.equal(invalidWrapper.stderr, invalidStock.stderr);
  }
  assert.equal(
    fs.readdirSync(root).some((name) => name.startsWith("oxlint-vize")),
    false,
  );
  save({
    terminal: "whole original/malformed/scriptless/escape/tamper controls passed",
    qualification:
      "wrapper route only; full n8n/direct51/source-native identity requires separate authenticated proof",
  });
});
