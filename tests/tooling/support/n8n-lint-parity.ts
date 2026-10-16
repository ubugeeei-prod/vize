// Historical file/line/rule projection for the n8n adoption contract (#8142).
// This does not retain whole provider packets or compare complete coordinates,
// severities, suppressions or fix metadata; it is not whole51 parity authority.
// Use n8n-whole-corpus-observer.mjs for complete provenance-preserving research.
//
// Runs the pinned eslint-plugin-vue toolchain and the Vize CLI with the same
// n8n rule set and rule options over every Vue file of the licensed n8n
// fixture, then compares findings by file, line and rule. A finding that only
// Vize reports fails the check. A finding that only ESLint reports fails
// unless it is listed in `documentedDivergences` with its reason.
//
// Usage: vp node tests/tooling/support/n8n-lint-parity.ts <vize-binary> <output-dir>
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import {
  cpSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { createRequire } from "node:module";
import { dirname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

type Projection = {
  linter: { enabled: boolean; preset: string; rules: Record<string, string>; ruleOptions: unknown };
  packageVueFileCounts: Record<string, number>;
};
type Finding = { file: string; line: number; rule: string; message: string };

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../../..");
const fixture = join(root, "tests/_fixtures/_git/n8n");
const projection: Projection = JSON.parse(
  readFileSync(join(root, "tests/_fixtures/n8n-cli-adoption.json"), "utf8"),
);

/** Vize rule name -> eslint-plugin-vue rule name, where they differ. */
const eslintNames: Record<string, string> = {
  "vue/require-component-registration": "vue/no-undef-components",
  "vue/sfc-element-order": "vue/block-order",
};

/**
 * ESLint-only findings that are correct for Vize to omit. Each entry names
 * the fixture file, rule and the reason; the fixture revision is immutable.
 */
const documentedDivergences: ReadonlyArray<{ file: string; rule: string; reason: string }> = [
  {
    file: "packages/frontend/@n8n/design-system/src/components/N8nDropdownMenu/DropdownMenuItem.vue",
    rule: "vue/require-component-registration",
    reason:
      "The component renders itself recursively by its `defineOptions` name, which Vue resolves; eslint-plugin-vue does not read that name.",
  },
];

function* vueFiles(directory: string): Generator<string> {
  for (const name of readdirSync(directory).sort()) {
    if (name === "node_modules" || name === "dist" || name.startsWith(".")) continue;
    const path = join(directory, name);
    if (statSync(path).isDirectory()) yield* vueFiles(path);
    else if (name.endsWith(".vue")) yield path;
  }
}

function eslintRules(): { rules: Record<string, unknown>; back: Map<string, string> } {
  const rules: Record<string, unknown> = {};
  const back = new Map<string, string>();
  for (const vizeRule of Object.keys(projection.linter.rules)) {
    const eslintRule = eslintNames[vizeRule] ?? vizeRule.replace(/^script\//u, "vue/");
    back.set(eslintRule, vizeRule);
    rules[eslintRule] = "error";
  }
  // The options n8n's own ESLint frontend configuration passed to these rules.
  rules["vue/attribute-hyphenation"] = ["error", "always"];
  rules["vue/component-name-in-template-casing"] = [
    "error",
    "PascalCase",
    { registeredComponentsOnly: false },
  ];
  rules["vue/no-undef-components"] = [
    "error",
    {
      ignorePatterns: [
        "RouterLink",
        "RouterView",
        "Teleport",
        "Transition",
        "TransitionGroup",
        "KeepAlive",
        "Suspense",
      ],
    },
  ];
  rules["vue/block-order"] = ["error", { order: ["script", "template", "style"] }];
  rules["vue/comment-directive"] = "error";
  return { rules, back };
}

async function eslintFindings(workspace: string, packageRoot: string): Promise<Finding[]> {
  const require = createRequire(join(root, "tools/benchmarks/scripts/package.json"));
  const { ESLint } = require("eslint");
  const vue = require("eslint-plugin-vue");
  const { rules, back } = eslintRules();
  const cwd = join(workspace, packageRoot);
  const eslint = new ESLint({
    cwd,
    overrideConfigFile: true,
    overrideConfig: [
      {
        files: ["**/*.vue"],
        plugins: { vue },
        processor: vue.processors[".vue"] ?? vue.processors.vue,
        languageOptions: {
          parser: require("vue-eslint-parser"),
          parserOptions: {
            parser: require("@typescript-eslint/parser"),
            ecmaVersion: "latest",
            sourceType: "module",
          },
        },
        linterOptions: { reportUnusedDisableDirectives: "off" },
        rules,
      },
    ],
  });
  const findings: Finding[] = [];
  for (const result of await eslint.lintFiles(["**/*.vue"])) {
    for (const message of result.messages) {
      assert.ok(
        message.ruleId,
        `${result.filePath}: ESLint could not parse the file: ${message.message}`,
      );
      if (!message.ruleId.startsWith("vue/") || message.ruleId === "vue/comment-directive")
        continue;
      findings.push({
        file: join(packageRoot, relative(cwd, result.filePath)),
        line: message.line,
        rule: back.get(message.ruleId) ?? message.ruleId,
        message: message.message,
      });
    }
  }
  return findings;
}

/**
 * Mirror the installed workspace for the one dependency the selected rules
 * read: `vue-router` registers RouterLink and RouterView when it resolves.
 */
function installRouterWhenDeclared(packageRoot: string): void {
  const manifest = JSON.parse(readFileSync(join(fixture, packageRoot, "package.json"), "utf8"));
  const declared = ["dependencies", "devDependencies", "peerDependencies"].some(
    (field) => manifest[field]?.["vue-router"] !== undefined,
  );
  if (!declared) return;
  const router = join(workspace, packageRoot, "node_modules/vue-router");
  mkdirSync(router, { recursive: true });
  writeFileSync(join(router, "package.json"), '{"name":"vue-router"}\n');
}

function vizeFindings(
  binary: string,
  config: string,
  workspace: string,
  packageRoot: string,
): Finding[] {
  let stdout: string;
  try {
    stdout = execFileSync(
      binary,
      ["lint", "--help-level", "none", "--format", "json", "-c", config, "**/*.vue"],
      {
        cwd: join(workspace, packageRoot),
        encoding: "utf8",
        maxBuffer: 1 << 28,
      },
    );
  } catch (error) {
    stdout = (error as { stdout?: string }).stdout ?? "";
  }
  const files: Array<{
    file: string;
    messages: Array<{ line: number; ruleId: string; message: string }>;
  }> = JSON.parse(stdout);
  return files.flatMap((file) =>
    file.messages.map((message) => ({
      file: join(packageRoot, file.file),
      line: message.line,
      rule: message.ruleId,
      message: message.message,
    })),
  );
}

const [binaryArgument, outputArgument] = process.argv.slice(2);
assert.ok(binaryArgument && outputArgument, "usage: n8n-lint-parity.ts <vize-binary> <output-dir>");
const binary = resolve(binaryArgument);
const output = resolve(outputArgument);
const workspace = join(output, "workspace");
rmSync(output, { recursive: true, force: true });
mkdirSync(workspace, { recursive: true });
const config = join(output, "vize.config.json");
writeFileSync(config, `${JSON.stringify({ linter: projection.linter }, null, 2)}\n`);

const vizeOnly: Finding[] = [];
const eslintOnly: Finding[] = [];
let shared = 0;
for (const [packageRoot, expected] of Object.entries(projection.packageVueFileCounts)) {
  let count = 0;
  for (const file of vueFiles(join(fixture, packageRoot))) {
    const destination = join(workspace, packageRoot, relative(join(fixture, packageRoot), file));
    mkdirSync(dirname(destination), { recursive: true });
    cpSync(file, destination);
    count += 1;
  }
  assert.equal(count, expected, `${packageRoot}: Vue file count`);
  installRouterWhenDeclared(packageRoot);
  const key = (finding: Finding) => `${finding.file}:${finding.line}:${finding.rule}`;
  const pending = new Map<string, Finding[]>();
  for (const finding of await eslintFindings(workspace, packageRoot)) {
    pending.set(key(finding), [...(pending.get(key(finding)) ?? []), finding]);
  }
  for (const finding of vizeFindings(binary, config, workspace, packageRoot)) {
    const bucket = pending.get(key(finding));
    if (bucket?.length) {
      bucket.pop();
      shared += 1;
    } else {
      vizeOnly.push(finding);
    }
  }
  for (const bucket of pending.values()) eslintOnly.push(...bucket);
}

// The copied sources are inputs, not results; keep only the report.
rmSync(workspace, { recursive: true, force: true });
const documented = (finding: Finding) =>
  documentedDivergences.some((entry) => entry.file === finding.file && entry.rule === finding.rule);
const undocumented = eslintOnly.filter((finding) => !documented(finding));
writeFileSync(
  join(output, "parity.json"),
  `${JSON.stringify(
    {
      qualification:
        "Historical file/line/rule projection only; not complete packet or whole51 parity authority",
      shared,
      vizeOnly,
      eslintOnly,
      undocumented,
    },
    null,
    1,
  )}\n`,
);
console.log(
  `n8n lint parity: shared=${shared} vizeOnly=${vizeOnly.length} eslintOnly=${eslintOnly.length} (documented ${eslintOnly.length - undocumented.length})`,
);
for (const entry of documentedDivergences) {
  if (!eslintOnly.some((finding) => finding.file === entry.file && finding.rule === entry.rule)) {
    console.log(`documented divergence no longer occurs, remove it: ${entry.file} ${entry.rule}`);
  }
}
assert.deepEqual(vizeOnly, [], "Vize reports findings that eslint-plugin-vue does not");
assert.deepEqual(undocumented, [], "eslint-plugin-vue reports findings that Vize misses");
