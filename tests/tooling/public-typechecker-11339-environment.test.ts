import assert from "node:assert/strict";
import { mkdtempSync, readFileSync, realpathSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import { overrides } from "../../tools/support/release/public_install/identity.ts";
import {
  defaultProviderKeys,
  requireDefaultProvider,
} from "../../tools/benchmarks/scripts/public-typechecker-11339/contract.ts";
import {
  guardedCapture,
  recorder,
} from "../../tools/benchmarks/scripts/public-typechecker-11339/execute.ts";
import { requireMainDispatch } from "../../tools/benchmarks/scripts/public-typechecker-11339/run.ts";

test("default provider requires true absence, including empty aliases, without value egress or execution", () => {
  assert.deepEqual(defaultProviderKeys, [
    ...overrides,
    "VIZE_PUBLIC_NATIVE_CUSTODY",
    "VIZE_OXLINT_NATIVE_CUSTODY",
  ]);
  assert(requireDefaultProvider({}).every((row) => !row.present && !row.empty));
  const root = realpathSync(mkdtempSync(join(tmpdir(), "public-11339-environment-")));
  try {
    const record = recorder(root),
      secretCanary = "PRIVATE_11339_ENV_CANARY_MUST_NOT_EGRESS";
    for (const name of defaultProviderKeys)
      for (const value of ["", secretCanary]) {
        let executed = 0;
        const packet = guardedCapture(
          record,
          "environment-refusal",
          ["inert-vize", "check"],
          root,
          () =>
            requireDefaultProvider({ [name]: value }, (rows) =>
              record.json("environment.json", rows),
            ),
          () => {
            executed++;
            throw Error("must not execute");
          },
        );
        assert.equal(executed, 0);
        assert.equal(packet.status, null);
        assert(packet.error);
        const evidence = readFileSync(join(root, "environment.json"), "utf8");
        assert(!evidence.includes(secretCanary));
        assert(!JSON.stringify(packet).includes(secretCanary));
        assert(
          !readFileSync(join(root, "environment-refusal.process.json"), "utf8").includes(
            secretCanary,
          ),
        );
        assert.deepEqual(
          JSON.parse(evidence).filter((row: { present: boolean }) => row.present),
          [{ name, present: true, empty: value === "" }],
        );
      }
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
test("a foreign workflow ref refuses before the public callback", () => {
  const environment = {
    GITHUB_EVENT_NAME: "workflow_dispatch",
    GITHUB_REPOSITORY: "ubugeeei-prod/vize",
    GITHUB_REF: "refs/heads/main",
  };
  requireMainDispatch(environment);
  const root = realpathSync(mkdtempSync(join(tmpdir(), "public-11339-ref-")));
  try {
    let executed = 0;
    const privateRef = "PRIVATE_11339_FOREIGN_REF_MUST_NOT_EGRESS";
    const packet = guardedCapture(
      recorder(root),
      "ref-refusal",
      ["inert-vize", "check"],
      root,
      () => requireMainDispatch({ ...environment, GITHUB_REF: "refs/heads/" + privateRef }),
      () => {
        executed++;
        throw Error("must not execute");
      },
    );
    assert.equal(executed, 0);
    assert.equal(packet.status, null);
    assert(packet.error);
    assert(!JSON.stringify(packet).includes(privateRef));
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
