// This one CLI preparation contract owns no batch/native diagnostic rate credit.
// Later source changes cannot reuse its exact input, producer or source pairs.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";

type ReadFile = (side: "base" | "head", path: string) => Buffer | null;
export const checkPreparationRegistration = {
  id: "typechecker/preparation/parse-before-runtime-10732",
  state: "active",
  adapter: "real-check-preparation-cli-v1",
  targets: ["cli-preparation"],
  input: {
    path: "tests/_fixtures/differential/typechecker/check-parse-before-runtime-10732/cases.json",
    sha256: "c24a743240082791caed1803ec2aa9153accba6330690723ef8d9dd2ed6fb4ae",
  },
  sources: [
    {
      path: "crates/vize/src/commands/check/runner/execution.rs",
      before: "10c925338d4adde61228ce9fbb4f49fbd6026d64bc02121f7ce916aac0735573",
      after: "d98c7f41c683600deff3ca0d1719ae8d8f1030f4da1cf58631531eb48c60241d",
    },
    {
      path: "crates/vize_canon/src/batch.rs",
      before: "4514867e06fc6708b8319ae711c8baf1f87dc6ca6db437cac3e0a6109ab0cc45",
      after: "d5a45015b7d5bc44bef75366e05e9dfcab5d118be7a73e8fec3aec555b17b282",
    },
    {
      path: "crates/vize_canon/src/batch/type_checker.rs",
      before: "9dc84d01356be5e27721b1208506c7f0fc603721679ea5b17db66bd1b2fafe80",
      after: "3cbb6505e7f608b531842a8aa67930e8c30750d5f36824eea9777a93f70e1d2a",
    },
    {
      path: "crates/vize_canon/src/batch/type_checker/installed_sources.rs",
      before: "e781a14d47297b4b35b44c78c5467db0356878866f10d3751e40840309619ba5",
      after: "f1a264d08055e0c14b7b442e4cead85ebab766e8310c2e1c4c098c305369f3c5",
    },
    {
      path: "crates/vize_canon/src/batch/type_checker/preparation.rs",
      before: null,
      after: "f497a2de865cfef1da271c52517bc95bc9215bb560bc9d39d223b58ce880817d",
    },
  ],
  test: {
    path: "tests/tooling/cli-check-contract.test.ts",
    sha256: "c16f0a791dd8633322944551d03ee5e7a3c39bcc99588afaf8ec58c863b94a4f",
  },
  producer: {
    path: "tools/support/compat/github/source-cli-identity.ts",
    sha256: "c8d80351cbb43746950df7570816ce993e62c22653cb0eb8b5c72d722231db9e",
    binary: "target/ci/vize",
    receipt: "target/ci/vize.source-build.json",
    consumerReceipt: "target/ci/vize.source-use.json",
    witnessSchema: "vize.check-preparation.packets",
    packets: 9,
  },
} as const;

function pinned(
  readFile: ReadFile,
  side: "base" | "head",
  path: string,
  expected: string | null,
): void {
  const bytes = readFile(side, path);
  if (expected === null) {
    assert.equal(bytes, null, `unexpected existing preparation source: ${path}`);
    return;
  }
  assert.ok(bytes !== null, `missing preparation authority: ${side} ${path}`);
  assert.equal(
    createHash("sha256").update(bytes).digest("hex"),
    expected,
    `changed preparation authority: ${side} ${path}`,
  );
}

/** Recognize only a new, fully pinned real-CLI preparation registration. */
export function newCheckPreparationCases(
  product: string,
  previous: unknown,
  current: unknown,
  readFile: ReadFile,
): { ids: string[]; sources: Set<string> } {
  const result = { ids: [] as string[], sources: new Set<string>() };
  const required =
    product === "typechecker" &&
    checkPreparationRegistration.sources.every((pair) => {
      const before = readFile("base", pair.path);
      const after = readFile("head", pair.path);
      return (
        (pair.before === null
          ? before === null
          : before !== null && createHash("sha256").update(before).digest("hex") === pair.before) &&
        after !== null &&
        createHash("sha256").update(after).digest("hex") === pair.after
      );
    });
  if (current === undefined) {
    assert(!required, "the exact source preparation fix requires its real CLI packet contract");
    return result;
  }
  assert.ok(Array.isArray(current), "preparationCases must be an array");
  assert.ok(
    previous === undefined || Array.isArray(previous),
    "prior preparationCases must be an array",
  );
  const oldIds = new Set((previous ?? []).map((row: { id?: unknown }) => row?.id));
  const seen = new Set<unknown>();
  for (const row of current) {
    assert.ok(row !== null && typeof row === "object", "preparation case must be an object");
    assert.ok(!seen.has(row.id), "duplicate preparation case ID");
    seen.add(row.id);
    if (oldIds.has(row.id)) continue;
    assert.equal(product, "typechecker", "unregistered preparation product");
    assert.deepEqual(row, checkPreparationRegistration, "unregistered preparation contract");
    const registered = checkPreparationRegistration;
    pinned(readFile, "head", registered.input.path, registered.input.sha256);
    pinned(readFile, "head", registered.test.path, registered.test.sha256);
    pinned(readFile, "head", registered.producer.path, registered.producer.sha256);
    for (const pair of registered.sources) {
      pinned(readFile, "base", pair.path, pair.before);
      pinned(readFile, "head", pair.path, pair.after);
      result.sources.add(pair.path);
    }
    result.ids.push(registered.id);
  }
  assert(
    !required || result.ids.length === 1,
    "the exact source preparation fix requires its new real CLI packet contract",
  );
  return result;
}
