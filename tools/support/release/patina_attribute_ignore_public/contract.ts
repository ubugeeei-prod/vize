/** One finite installed-public attribute-ignore role, with immutable source custody. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync, realpathSync, statSync, writeFileSync } from "node:fs";
import { join, sep } from "node:path";
import { fileURLToPath } from "node:url";
import { asArray, asRecord, asString, parseJson, type JsonRecord } from "../../json-value.ts";
import { rejectOverrides } from "../public_install/identity.ts";
import {
  corpus,
  expected,
  presentation,
  referenceFor,
  rule,
  sourceFor,
  type AttributeCase,
} from "../../../../tests/tooling/support/patina-attribute-ignore-contract.ts";

export { corpus, expected, presentation, referenceFor, rule, sourceFor, type AttributeCase };
export const role = "literal-attribute-ignore-nine-v1";
export const requiredMerges = [
  "30153aa658b97928f73f5a12d01d676411166709",
  "a70648d367c4fb93510ab001813c7235dae8a5c0",
] as const;
export const checkout = fileURLToPath(new URL("../../../../", import.meta.url));
export const digest = (bytes: Uint8Array | string): string =>
  createHash("sha256").update(bytes).digest("hex");
export const custodySha256 = "a033d844772d6109143c4faf4ca94c1aef054e474af467358d72f70fdccec670";
export type Image = { path: string; bytes: number; sha256: string };
export type Native = {
  lintPatinaSfc: (source: string, options: Record<string, unknown>) => unknown;
  getNativeBuildToken: () => string;
};
export function image(path: string): Image {
  const bytes = readFileSync(path);
  return { path: realpathSync(path), bytes: bytes.length, sha256: digest(bytes) };
}
export function inside(root: string, path: string): string {
  const actual = realpathSync(path);
  assert(actual.startsWith(realpathSync(root) + sep), "installed ownership: " + actual);
  assert(statSync(actual).isFile());
  return actual;
}
export function write(path: string, value: unknown): void {
  writeFileSync(path, JSON.stringify(value, null, 2) + "\n", { flag: "wx" });
}
export function custody(): JsonRecord {
  const bytes = readFileSync(new URL("source-custody.json", import.meta.url));
  assert.equal(digest(bytes), custodySha256);
  return validateSourceCopies(parseJson(bytes.toString()), (path) =>
    readFileSync(join(checkout, path)),
  );
}
export function validateSourceCopies(input: unknown, read: (path: string) => Buffer): JsonRecord {
  const value = asRecord(input);
  assert.equal(value.schema, "vize.patina-attribute-ignore-public.source.v1");
  assert.equal(value.role, role);
  assert.equal(value.source, requiredMerges[1]);
  assert.deepEqual(value.requiredMerges, requiredMerges);
  assert.deepEqual(value.referenceProviders, corpus.provider);
  assert.deepEqual(value.targetProviders, {
    eslint: "10.13.0",
    "vue-eslint-parser": "10.4.1",
    oxlint: "1.78.0",
    "@oxlint/plugins": "1.78.0",
  });
  const files = asArray(value.files);
  assert.equal(files.length, 12);
  const seen = new Set<string>();
  for (const raw of files) {
    const row = asRecord(raw),
      path = asString(row.path);
    assert(!path.startsWith("/") && !path.split("/").includes("..") && !seen.has(path));
    seen.add(path);
    const current = read(path);
    assert.equal(current.length, row.bytes, path);
    assert.equal(digest(current), row.sha256, path);
  }
  assert.equal(corpus.valid.length, 9);
  assert.equal(corpus.invalid.length, 6);
  for (const row of corpus.valid) referenceFor(row);
  return value;
}
export function request(
  version: string,
  candidate: string,
  producerRun: string,
  env: NodeJS.ProcessEnv,
): void {
  assert.match(version, /^0\.[0-9]+\.0$/u);
  assert.match(candidate, /^[0-9a-f]{40}$/u);
  assert.match(producerRun, /^[1-9][0-9]*$/u);
  assert.equal(env.GITHUB_ACTIONS, "true");
  assert.equal(env.GITHUB_REPOSITORY, "ubugeeei-prod/vize");
  assert.equal(env.GITHUB_REF, "refs/heads/main");
  assert.equal(env.GITHUB_EVENT_NAME, "workflow_dispatch");
  assert.equal(
    env.GITHUB_WORKFLOW_REF,
    "ubugeeei-prod/vize/.github/workflows/patina-attribute-ignore-public.yml@refs/heads/main",
  );
  rejectOverrides(env);
  for (const name of ["NODE_PATH", "VIZE_ATTR_PUBLIC_OBSERVER", "VIZE_NATIVE_BINDING_PATH"])
    assert(!env[name], "initial override must be absent: " + name);
}
export function completedProducer(value: unknown, run: string): JsonRecord {
  const facts = asRecord(value);
  assert.equal(String(facts.id), run);
  assert.equal(asRecord(facts.repository).full_name, "ubugeeei-prod/vize");
  assert.equal(facts.head_branch, "main");
  assert.equal(facts.path, ".github/workflows/release.yml");
  assert.equal(facts.event, "workflow_dispatch");
  assert.equal(facts.status, "completed");
  assert.equal(facts.conclusion, "success");
  assert.match(asString(facts.head_sha), /^[0-9a-f]{40}$/u);
  assert(Number.isSafeInteger(facts.run_attempt) && Number(facts.run_attempt) > 0);
  return facts;
}
export function ancestry(value: unknown, base: string): void {
  const result = asRecord(value);
  assert.equal(asRecord(result.base_commit).sha, base);
  assert.equal(asRecord(result.merge_base_commit).sha, base);
  assert.equal(result.behind_by, 0);
  assert(["ahead", "identical"].includes(asString(result.status)));
}
export function expectedCli(row: AttributeCase): unknown {
  const messages = referenceFor(row).verify[0]!.messages.map((finding) => {
    const text = presentation(finding);
    return {
      ruleId: rule,
      ruleDocsPath: "docs/content/rules/vue.md",
      severity: 2,
      message: `[vize:${rule}] ${text.message}`,
      help: text.help,
      line: finding.line,
      column: finding.column,
      endLine: finding.endLine,
      endColumn: finding.endColumn,
    };
  });
  return [{ file: "src/Probe.vue", messages, errorCount: messages.length, warningCount: 0 }];
}
export const cleanCli = () => [
  { file: "src/Probe.vue", messages: [], errorCount: 0, warningCount: 0 },
];
export function expectedEslint(row: AttributeCase): unknown[] {
  const source = sourceFor(row);
  const utf16 = (offset: number) => Buffer.from(source).subarray(0, offset).toString().length;
  return expected(row).diagnostics.map((diagnostic) => {
    assert.equal(diagnostic.fix!.edits.length, 1);
    const edit = diagnostic.fix!.edits[0]!;
    return {
      ruleId: "vize/" + rule,
      severity: 1,
      message: diagnostic.message,
      line: diagnostic.location.start.line,
      column: diagnostic.location.start.column,
      endLine: diagnostic.location.end.line,
      endColumn: diagnostic.location.end.column,
      fix: { range: [utf16(edit.start), utf16(edit.end)], text: edit.newText },
    };
  });
}
/** Apply the actual returned byte edits; whole output still has an independent oracle. */
export function applyReturnedEdits(source: string, value: unknown): string {
  const raw = asRecord(value);
  const edits = asArray(raw.diagnostics)
    .flatMap((item) => {
      const fix = asRecord(asRecord(item).fix);
      return asArray(fix.edits).map((edit) => asRecord(edit));
    })
    .sort((a, b) => Number(b.start) - Number(a.start));
  let output = Buffer.from(source),
    boundary = output.length;
  for (const edit of edits) {
    assert.equal(typeof edit.start, "number");
    assert.equal(typeof edit.end, "number");
    const start = Number(edit.start),
      end = Number(edit.end);
    assert(Number.isSafeInteger(start) && Number.isSafeInteger(end));
    assert(start >= 0 && start <= end && end <= boundary);
    boundary = start;
    output = Buffer.concat([
      output.subarray(0, start),
      Buffer.from(asString(edit.newText)),
      output.subarray(end),
    ]);
  }
  return output.toString();
}
