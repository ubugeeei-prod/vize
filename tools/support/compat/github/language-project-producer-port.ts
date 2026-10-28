import { createHash } from "node:crypto";

type ReadFile = (side: "base" | "head", path: string) => Buffer | null;
const custodyPath =
  "tests/_fixtures/tooling/typescript-plugin-producer-port-10890/custody.json.txt";
const custodySha256 = "de52e7c0ca46c0e1584c699cabe9763b95b0ac0e689beaf1a64fa22b74e54edc";
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
    require(test.source === after.path &&
      test.sha256 === after.sha256, "unreviewed producer registration: " + selector);
    const snapshot = read("head", string(before.snapshot));
    require(snapshot && digest(snapshot) === before.sha256, "original producer bytes changed: " +
      selector);
    const producer = read("head", string(after.path));
    require(producer && digest(producer) === after.sha256, "qualified producer changed: " +
      selector);
  }
}
