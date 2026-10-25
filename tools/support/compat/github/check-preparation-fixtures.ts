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
      before: "efeddbdcf3ef827687aad6bf3ce23dd3d591ca6f7083517d6fe04b8b2c86d65d",
      after: "7b8a9d169cadbce57143ee1b4debc7c96938548f9dfe74f6163dc357fc768a01",
    },
    {
      path: "crates/vize_canon/src/batch.rs",
      before: "046a27c2f9870d3e7c8e0be955f2daad298fbf9d0c043441aaa52f8c2c379681",
      after: "8c4b1940aeba743b43e2bb835509d8cc210839d1547c82e722c34d522b8ad5d8",
    },
    {
      path: "crates/vize_canon/src/batch/type_checker.rs",
      before: "e8ddc0437605f21402f26835451c69855fed051c34b5d8eb9cfbf5dae94b1d8b",
      after: "79997491da904dcd9dd6fda06d23d22e6905139b540e337cd3681d0c1ef681c0",
    },
    {
      path: "crates/vize_canon/src/batch/type_checker/installed_sources.rs",
      before: "1c5c069a944112a794468b0897ccb0b787469d6ed8c50e151771cc500c3dab50",
      after: "9df773a5f70b2caceeb1e66a4c36a9d002f0a65cb106202198d403b2337d1f7d",
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
