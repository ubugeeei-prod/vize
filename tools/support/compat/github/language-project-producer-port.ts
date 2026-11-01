import { createHash } from "node:crypto";

type ReadFile = (side: "base" | "head", path: string) => Buffer | null;
const custodyPath =
  "tests/_fixtures/tooling/typescript-plugin-producer-port-10890/custody.json.txt";
const custodySha256 = "de52e7c0ca46c0e1584c699cabe9763b95b0ac0e689beaf1a64fa22b74e54edc";
const observerPath = "tests/_fixtures/tooling/typescript-plugin-metadata-observer-11260.json";
const observerSha256 = "20b748ba87cdda3fb496b4c839fbcb27e4d8f895ffacba4a18d229707603d2ed";
const digest = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
function require(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error("invalid language project producer port: " + message);
}
function object(value: unknown): Record<string, unknown> {
  require(value !== null && typeof value === "object" && !Array.isArray(value), "object required");
  return value as Record<string, unknown>;
}
function string(value: unknown): string {
  require(typeof value === "string", "string required");
  return value;
}

/** Authenticate the diagnostic-only successor before checking its entire predecessor. */
function validateMetadataObserver(
  read: ReadFile,
  test: Record<string, unknown>,
  after: Record<string, unknown>,
) {
  require(after.path ===
    ".github/workflows/typescript-plugin.yml", "unreviewed producer registration");
  const bytes = read("head", observerPath);
  require(bytes && digest(bytes) === observerSha256, "metadata observer custody changed");
  const record = object(JSON.parse(bytes.toString("utf8")) as unknown);
  require(record.schema === "vize.language-project.metadata-observer.v1" &&
    record.predecessorPortSha256 === custodySha256, "metadata observer predecessor differs");
  const workflow = object(record.workflow);
  require(workflow.path === after.path &&
    workflow.beforeSha256 === after.sha256 &&
    test.source === after.path &&
    test.sha256 === workflow.afterSha256, "unreviewed producer registration");
  const current = read("head", string(workflow.path));
  require(current &&
    digest(current) === workflow.afterSha256, "metadata observer workflow changed");
  require(Array.isArray(workflow.edits) &&
    workflow.edits.length === 4, "four exact workflow edits required");
  let prior = current.toString("utf8");
  for (const value of workflow.edits) {
    const edit = object(value),
      before = string(edit.before),
      after = string(edit.after);
    require(after.length > 0 &&
      prior.split(after).length === 2, "workflow edit missing or duplicated");
    prior = prior.replace(after, before);
  }
  require(digest(Buffer.from(prior)) ===
    workflow.beforeSha256, "whole workflow predecessor differs");
  const paths = [
    "tools/support/ci/cargo-metadata-observer.ts",
    "tests/tooling/cargo-metadata-observer.test.ts",
  ];
  require(Array.isArray(record.dependencies) &&
    record.dependencies.length === paths.length, "complete observer dependency closure required");
  for (const [index, value] of record.dependencies.entries()) {
    const dependency = object(value);
    require(dependency.path === paths[index], "observer dependency path or order differs");
    const source = read("head", paths[index]!);
    require(source && digest(source) === dependency.sha256, "observer dependency changed");
  }
}

/** Retain the original producer bytes and admit only the qualified typed port. */
export function validateLanguageProjectProducerPort(read: ReadFile, tests: readonly unknown[]) {
  const bytes = read("head", custodyPath);
  require(bytes && digest(bytes) === custodySha256, "original migration custody changed");
  const custody = object(JSON.parse(bytes.toString("utf8")) as unknown);
  require(custody.schema === "vize.language-project.producer-port.v1", "custody schema differs");
  require(Array.isArray(custody.producers) &&
    custody.producers.length === 5, "five complete producer pairs required");
  for (const raw of custody.producers) {
    const row = object(raw),
      before = object(row.before),
      after = object(row.after);
    const selector = string(row.selector);
    const current = tests.map(object).filter((test) => test.selector === selector);
    require(current.length === 1, "missing or duplicate migrated producer: " + selector);
    const test = current[0]!;
    const original = test.source === after.path && test.sha256 === after.sha256;
    if (!original) validateMetadataObserver(read, test, after);
    const snapshot = read("head", string(before.snapshot));
    require(snapshot && digest(snapshot) === before.sha256, "original producer bytes changed: " +
      selector);
    const producer = read("head", string(after.path));
    require(producer && digest(producer) === test.sha256, "qualified producer changed: " +
      selector);
  }
}
