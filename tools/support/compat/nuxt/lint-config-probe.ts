// Execute original full-source controls with the real Nuxt module and Oxlint CLI.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  asArray,
  asRecord,
  asString,
  asStringArray,
  at,
  parseJson,
  readJsonRecord,
} from "../../json-value.ts";
import { importChecked } from "./host-types.ts";
import { expectedCliDiagnostic } from "./lint-config-observer.ts";

interface Nuxt3 {
  options: { rootDir: unknown } & Record<string, unknown>;
  close(): Promise<void>;
}
interface LintProbeHost {
  loadNuxt: (options: Record<string, unknown>) => Promise<Nuxt3>;
  api: {
    renderNuxtOxlintConfig(items: unknown, plugin: string, paths: Record<string, string>): string;
    setupNuxtLintConfigGeneration(options: Record<string, unknown>, nuxt: Nuxt3): Promise<unknown>;
    setupNuxtLintChecker(
      enabled: boolean,
      nuxt: { options: Record<string, unknown> },
      generation: unknown,
      builder: { addVitePlugin(): void },
    ): Promise<unknown>;
  };
  lintPluginEntry: string;
}
interface RunOptions {
  invalid?: boolean;
  files?: number;
}

const [project, artifacts, inputDir, version] = process.argv.slice(2);
assert.ok(project && artifacts && inputDir && version, "usage: PROJECT ARTIFACTS INPUTS VERSION");
const require = createRequire(path.join(project, "package.json"));
const corpus = readJsonRecord(path.join(inputDir, "corpus.json"));
const items = asArray(corpus.items, "corpus.items");
const corpusFiles = asRecord(corpus.files, "corpus.files");
const expectedReportedPaths = asStringArray(corpus.expectedReportedPaths, "expectedReportedPaths");
const originalAddon = { name: "issue-7983", getConfigs: () => structuredClone(items) };
const source = fs.readFileSync(path.join(inputDir, "Input.vue.txt"), "utf8");
const hash = (bytes: Uint8Array) => createHash("sha256").update(bytes).digest("hex");
const originals = asStringArray(corpus.originalPaths, "corpus.originalPaths");
const controls = [
  "dist/Nested.vue",
  "app/pages/generated/Generated.vue",
  "app/sibling-pages/Near.vue",
];
for (const name of [...originals, ...controls]) {
  const file = path.join(project, name);
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, source, { flag: "wx" });
  assert.equal(hash(fs.readFileSync(file)), corpusFiles["Input.vue.txt"]);
}
const externalFiles = [
  path.join(artifacts, "outside-layer", "Original.vue"),
  path.join(artifacts, "neighbor", "app/pages/About.vue"),
];
for (const file of externalFiles) {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, source, { flag: "wx" });
}
const externalNames = externalFiles.map((file) =>
  path.relative(project, file).replaceAll(path.sep, "/"),
);
const authoredPaths = [...originals, ...controls, ...externalNames];
assert.equal(at(require("nuxt/package.json"), "version"), version);
const apiHost = path.join(project, "vize-lint-probe-api.mjs");
const apiHostSource =
  'export { loadNuxt } from "nuxt";\nexport * as api from "@vizejs/nuxt/lint";\nexport const lintPluginEntry = import.meta.resolve("oxlint-plugin-vize");\n';
fs.writeFileSync(apiHost, apiHostSource, { flag: "wx" });
fs.writeFileSync(path.join(artifacts, "probe-api.mjs"), apiHostSource);
const { loadNuxt, api, lintPluginEntry } = await importChecked<LintProbeHost>(apiHost, [
  "loadNuxt",
]);
for (const name of [
  "renderNuxtOxlintConfig",
  "setupNuxtLintConfigGeneration",
  "setupNuxtLintChecker",
])
  assert.equal(typeof at(api, name), "function", `@vizejs/nuxt/lint exports ${name}`);
assert.equal(typeof lintPluginEntry, "string");
const cli = path.join(project, "node_modules/oxlint-plugin-vize/bin/oxlint-vize");
const rows: unknown[] = [];
const writeConfig = (name: string, content: string): string => {
  const file = path.join(project, name);
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, content);
  fs.writeFileSync(path.join(artifacts, name.replaceAll("/", "_")), content);
  return file;
};
function run(
  id: string,
  file: string,
  targets: readonly string[],
  expectedPaths: readonly string[],
  { invalid = false, files = targets.length }: RunOptions = {},
): void {
  const args = [cli, "--config", file, "--format", "json", ...targets];
  const started = performance.now();
  const processResult = spawnSync(process.execPath, args, {
    cwd: project,
    encoding: "utf8",
    timeout: 30_000,
    env: { ...process.env, NO_COLOR: "1" },
  });
  fs.writeFileSync(
    path.join(artifacts, `${id}-process.json`),
    JSON.stringify(
      {
        args,
        cwd: project,
        status: processResult.status,
        signal: processResult.signal,
        error: processResult.error?.message ?? null,
        elapsedMs: performance.now() - started,
        stdout: processResult.stdout,
        stderr: processResult.stderr,
      },
      null,
      2,
    ) + "\n",
  );
  assert.equal(processResult.error, undefined);
  assert.equal(processResult.signal, null);
  if (invalid) {
    const status = processResult.status;
    assert.ok(status !== null && Number.isInteger(status) && status > 0);
    assert.ok(
      (processResult.stdout + processResult.stderr).includes("Invalid pattern `../**/dist`"),
    );
    rows.push({ id, invalid: true, status: processResult.status });
    return;
  }
  assert.equal(processResult.status, 0, processResult.stderr || processResult.stdout);
  assert.equal(processResult.stderr, "");
  const report = asRecord(parseJson(processResult.stdout), "oxlint report");
  assert.deepEqual(
    asArray(report.diagnostics, "report.diagnostics")
      .map((value) => {
        const { code, filename, message, severity, labels } = asRecord(value, "diagnostic");
        return {
          code,
          filename: asString(filename, "diagnostic.filename"),
          message,
          severity,
          labels,
        };
      })
      .sort((a, b) => a.filename.localeCompare(b.filename)),
    expectedPaths
      .map((filename) => expectedCliDiagnostic(corpus, project, filename))
      .sort((a, b) => a.filename.localeCompare(b.filename)),
  );
  assert.equal(report.number_of_files, files);
  rows.push({ id, targets, expectedPaths, report });
}

let nuxt: Nuxt3 | undefined;
try {
  nuxt = await loadNuxt({
    cwd: project,
    overrides: {
      dev: false,
      srcDir: path.join(project, "app"),
      vize: { compiler: false, lint: { autoInit: false }, musea: false },
      hooks: { "vize:lint:config:addons": (addons: unknown[]) => addons.push(originalAddon) },
    },
  });
  assert.equal(nuxt.options.rootDir, project);
  const generated = path.join(project, ".oxlint.vize.json");
  const defaultBytes = fs.readFileSync(generated, "utf8");
  const artifact = parseJson(defaultBytes);
  assert.equal(at(artifact, "settings", "vize", "generatedBy"), "@vizejs/nuxt");
  assert.deepEqual(asArray(at(artifact, "overrides"), "overrides").slice(-2), [
    { files: ["**/*.vue"], rules: { "vize/vue/no-inline-style": "warn" } },
    { files: ["app/pages/**/*.vue"], rules: { "vize/vue/no-inline-style": "off" } },
  ]);
  fs.writeFileSync(path.join(artifacts, "default-module-config.json"), defaultBytes);
  run("default-module", generated, [...originals, "dist/Nested.vue"], expectedReportedPaths, {
    files: 2,
  });

  // The complete original bad artifact is executed separately and remains a failure.
  const brokenBytes = fs.readFileSync(path.join(inputDir, "original-broken-config.json"), "utf8");
  run(
    "original-parent-ignore",
    writeConfig(".nuxt/oxlint.config.json", brokenBytes),
    originals,
    [],
    { invalid: true },
  );
  const withoutIgnores = asRecord(parseJson(brokenBytes), "original broken config");
  delete withoutIgnores.ignorePatterns;
  run(
    "original-parent-overrides",
    writeConfig(".nuxt/no-ignores.json", JSON.stringify(withoutIgnores, null, 2) + "\n"),
    originals,
    [],
  );
  assert.throws(
    () =>
      api.renderNuxtOxlintConfig(items, "oxlint-plugin-vize", {
        rootDir: project,
        configDir: path.join(project, ".nuxt"),
      }),
    {
      message: "Generated oxlint config must be in the lint root or an ancestor directory",
    },
  );
  const originalCurrent = api.renderNuxtOxlintConfig(items, "oxlint-plugin-vize", {
    rootDir: project,
    configDir: project,
  });
  assert.deepEqual(parseJson(originalCurrent), {
    plugins: ["vue"],
    jsPlugins: [{ name: "vize", specifier: "oxlint-plugin-vize" }],
    settings: { vize: { preset: "incremental" } },
    ignorePatterns: ["**/dist"],
    overrides: [
      { files: ["**/*.vue"], rules: { "vize/vue/no-inline-style": "warn" } },
      { files: ["app/pages/**/*.vue"], rules: { "vize/vue/no-inline-style": "off" } },
    ],
  });
  run(
    "original-root-renderer",
    writeConfig("root-renderer.json", originalCurrent),
    [...originals, "dist/Nested.vue"],
    expectedReportedPaths,
    { files: 2 },
  );
  const exclusions = [
    items[0],
    items[1],
    {
      name: "pages-with-exclusion",
      files: ["app/pages/**/*.vue"],
      ignores: ["app/pages/generated/**"],
      rules: { "vue/no-inline-style": "off" },
    },
  ];
  run(
    "exact-page-exclusion",
    writeConfig(
      "exclusions.json",
      api.renderNuxtOxlintConfig(exclusions, "oxlint-plugin-vize", {
        rootDir: project,
        configDir: project,
      }),
    ),
    [...originals, ...controls],
    [
      "app/components/InfoCard.vue",
      "app/pages/generated/Generated.vue",
      "app/sibling-pages/Near.vue",
    ],
    { files: 4 },
  );
  const ancestor = path.join(artifacts, "ancestor-config.json");
  const ancestorBytes = api.renderNuxtOxlintConfig(items, fileURLToPath(lintPluginEntry), {
    rootDir: project,
    configDir: artifacts,
  });
  fs.writeFileSync(ancestor, ancestorBytes, { flag: "wx" });
  run(
    "ancestor-owned-namespace",
    ancestor,
    [...originals, "dist/Nested.vue"],
    expectedReportedPaths,
    { files: 2 },
  );
  const layerItems = [
    items[0],
    items[1],
    items[2],
    {
      name: "exact-outside-layer",
      files: [externalNames[0]],
      rules: { "vue/no-inline-style": "off" },
    },
  ];
  run(
    "outside-layer-and-neighbor",
    writeConfig(
      "outside-layer.json",
      api.renderNuxtOxlintConfig(layerItems, "oxlint-plugin-vize", {
        rootDir: project,
        configDir: project,
      }),
    ),
    [...originals, ...externalFiles],
    ["app/components/InfoCard.vue", externalNames[1] ?? ""],
  );
  const generation = await api.setupNuxtLintConfigGeneration({ autoInit: false }, nuxt);
  const checker = await api.setupNuxtLintChecker(
    true,
    { options: { ...nuxt.options, dev: true } },
    generation,
    { addVitePlugin: () => {} },
  );
  assert.equal(at(checker, "configFile"), generated);
  assert.equal(at(checker, "builder"), "vite");

  await nuxt.close();
  nuxt = undefined;
  const custodyFile = process.env.VIZE_NUXT_LINT_NATIVE_CUSTODY;
  assert.ok(custodyFile, "VIZE_NUXT_LINT_NATIVE_CUSTODY names the custody file");
  const custody = readJsonRecord(custodyFile);
  const events = fs
    .readFileSync(asString(custody.calls, "custody.calls"), "utf8")
    .trim()
    .split("\n")
    .map((line) => asRecord(parseJson(line), "native event"));
  const loaded = new Set<number>();
  const linted = new Set<string>();
  let lintCalls = 0;
  for (const event of events) {
    assert.equal(event.sourceHead, process.env.GITHUB_SHA);
    assert.equal(
      event.binary,
      fs.realpathSync(asString(at(custody, "binary", "path"), "custody.binary.path")),
    );
    assert.equal(event.sha256, at(custody, "binary", "sha256"));
    const pid = event.pid;
    assert.ok(typeof pid === "number" && Number.isSafeInteger(pid) && pid > 0);
    if (event.kind === "load") {
      loaded.add(pid);
      continue;
    }
    assert.equal(event.kind, "call");
    assert.ok(loaded.has(pid));
    assert.equal(event.outcome, "return");
    if (event.entrypoint === "getPatinaRules") {
      assert.ok(Array.isArray(event.result));
      continue;
    }
    assert.equal(event.entrypoint, "lintPatinaSfc");
    const args = asArray(event.args, "event.args");
    assert.equal(args[0], source);
    const physical = asString(at(args, 1, "filename"), "lint filename");
    const filename = path.relative(project, physical).replaceAll(path.sep, "/");
    assert.ok(authoredPaths.includes(filename));
    assert.equal(at(event.result, "errorCount"), 0);
    const enabledRules = at(args, 1, "enabledRules");
    if (Array.isArray(enabledRules) && enabledRules.includes("vue/no-inline-style")) {
      assert.deepEqual(event.result, {
        filename: physical,
        errorCount: 0,
        warningCount: 1,
        diagnostics: [
          {
            rule: "vue/no-inline-style",
            severity: "warning",
            message: "Avoid using inline style attributes",
            help: "Use CSS classes or scoped styles instead",
            location: {
              start: { line: 6, column: 7, offset: 76 },
              end: { line: 6, column: 25, offset: 94 },
            },
          },
        ],
      });
      linted.add(filename);
    }
    lintCalls++;
  }
  assert.ok(lintCalls > 0);
  assert.deepEqual(
    [...linted].sort((a, b) => a.localeCompare(b)),
    [
      "app/components/InfoCard.vue",
      "app/pages/generated/Generated.vue",
      "app/sibling-pages/Near.vue",
      externalNames[1],
    ].sort((a, b) => a.localeCompare(b)),
  );
  for (const name of authoredPaths)
    assert.equal(hash(fs.readFileSync(path.join(project, name))), corpusFiles["Input.vue.txt"]);
  fs.writeFileSync(
    path.join(artifacts, "proof.json"),
    JSON.stringify(
      {
        executionHead: process.env.GITHUB_SHA,
        nuxt: version,
        oxlint: at(require("oxlint/package.json"), "version"),
        originalHashes: corpusFiles,
        sourceBinding: custody,
        loadedProcesses: [...loaded],
        lintCalls,
        linted: [...linted],
        rows,
      },
      null,
      2,
    ) + "\n",
  );
} finally {
  await nuxt?.close();
}
