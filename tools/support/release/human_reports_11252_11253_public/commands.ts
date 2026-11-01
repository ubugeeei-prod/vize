import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import type { CommandPlan, InstallReceipt, Request } from "./model.ts";
import { array, object, sha256, text, wholeJson } from "./value.ts";
import { validateFiles } from "./fixtures.ts";
export function argumentsFor(request: Request, id: string): string[] {
  if (request.role === "runtime-prop-null-11252") {
    if (id === "original-pnpm-check" || /^(?:error|repair|restore-error)-ts7$/u.test(id))
      return ["check"];
    if (id === "original-cli") return ["exec", "vize", "check", "src"];
    if (id === "supplement-tsrs" || /^(?:error|repair|restore-error)-tsrs$/u.test(id))
      return ["exec", "tsrs", "--runExternalCode", "--noEmit", "-p", "tsconfig.json"];
    if (/^(?:error|repair|restore-error)-cli-json$/u.test(id))
      return [
        "exec",
        "vize",
        "check",
        "--quiet",
        "--format",
        "json",
        "--tsconfig",
        "tsconfig.json",
        "src",
      ];
  } else {
    if (id === "original-cli") return ["exec", "vize", "check"];
    if (id === "original-vue-tsc") return ["exec", "vue-tsc", "--noEmit"];
    if (id === "original-cli-json") return ["exec", "vize", "check", "-f", "json"];
    if (
      [
        "hash-only",
        "relative",
        "vue-package",
        "hash-vue",
        "main-only",
        "repair",
        "restore-main-only",
        "restore-original",
      ].includes(id)
    )
      return ["exec", "vize", "check", "--format", "json", "--quiet"];
  }
  throw new Error(`Unregistered issue-bound command: ${id}`);
}
export function validateOwnedCommands(request: Request, installs: InstallReceipt[]): void {
  assert(request.oracleOwnerReceipt);
  const raw = readFileSync(request.oracleOwnerReceipt.path);
  assert.equal(raw.length, request.oracleOwnerReceipt.bytes);
  assert.equal(sha256(raw), request.oracleOwnerReceipt.sha256);
  const owner = object(wholeJson(raw));
  assert.equal(owner.schema, "vize.humanReportPublic.sourceOracles.v1");
  assert.equal(owner.issue, request.fixture.issue);
  assert.equal(owner.fixtureReceiptSha256, request.fixture.receiptSha256);
  const commands = array(owner.commands).map(object);
  assert.deepEqual(
    commands.map((c) => c.id),
    request.commands.map((c) => c.id),
  );
  for (const command of request.commands) {
    assert.equal(command.requiresNativeLoad, command.id !== "original-vue-tsc");
    const installation = installs.find((v) => v.kind === command.consumer);
    assert(installation);
    assert.equal(command.executable, installation.pnpm.path);
    assert.deepEqual(command.argv, argumentsFor(request, command.id));
    validateFiles(command.sourceFiles);
    const supplied = commands.find((c) => c.id === command.id);
    assert(supplied);
    const owned = { ...supplied };
    delete owned.executable;
    assert.deepEqual(
      owned,
      {
        id: command.id,
        consumer: command.consumer,
        argv: argumentsFor(request, command.id),
        sourceFiles: command.sourceFiles,
        purpose: command.purpose,
        adaptation: command.adaptation,
        requiresNativeLoad: command.requiresNativeLoad,
        oracle: command.oracle,
      },
      `source-owned complete oracle/inputs differ: ${command.id}`,
    );
    assert.equal(command.consumer === "tsrs-supplement", command.id.includes("tsrs"));
    if (command.consumer === "tsrs-supplement") assert.equal(request.fixture.issue, 11252);
    assert(command.oracle);
    assert.equal(command.oracle.sourceOwnedBy.commit, text(owner.commit));
  }
}
export function commandSettings(command: CommandPlan): unknown {
  return {
    id: command.id,
    executable: command.executable,
    argv: command.argv,
    consumer: command.consumer,
    purpose: command.purpose,
    adaptation: command.adaptation,
    sourceFiles: command.sourceFiles,
    checker: command.checker,
  };
}
