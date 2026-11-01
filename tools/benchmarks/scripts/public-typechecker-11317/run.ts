import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdirSync, realpathSync, writeFileSync } from "node:fs";
import { dirname, isAbsolute, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { assertCustody, fileImage } from "../../../support/release/computed_event_custody.ts";
import { rejectOverrides } from "../../../support/release/public_install/identity.ts";
import { authenticate } from "./authority.ts";
import { fixturePath, input, ownerSource, profiles, type Role } from "./contract.ts";
import { assess, execute, recorder } from "./execute.ts";
import { install, managerImages } from "./install.ts";

export function argumentsFor(argv: string[]) {
  assert.equal(argv.length, 6);
  const values = new Map<string, string>();
  for (let index = 0; index < argv.length; index += 2) {
    const key = argv[index]!,
      value = argv[index + 1]!;
    assert.ok(["--out", "--npm", "--npx"].includes(key));
    assert.ok(!values.has(key));
    assert.ok(isAbsolute(value));
    values.set(key, value);
  }
  assert.equal(values.size, 3);
  return { out: values.get("--out")!, npm: values.get("--npm")!, npx: values.get("--npx")! };
}
export async function main(argv: string[]) {
  const options = argumentsFor(argv);
  assert.equal(realpathSync(dirname(options.out)), dirname(options.out));
  mkdirSync(options.out); // Fresh owner only; never reuse or truncate another run.
  const record = recorder(options.out);
  try {
    record.json("phase.json", { stage: "controller-inputs", accepted: false });
    rejectOverrides(process.env);
    assert.equal(process.env.GITHUB_EVENT_NAME, "workflow_dispatch");
    assert.equal(process.env.GITHUB_REPOSITORY, "ubugeeei-prod/vize");
    const head = execFileSync("git", ["rev-parse", "HEAD"], { encoding: "utf8" }).trim();
    assert.equal(head, process.env.GITHUB_SHA);
    const original = input();
    record.json("controller-source.json", {
      head,
      run: process.env.GITHUB_RUN_ID,
      attempt: process.env.GITHUB_RUN_ATTEMPT,
      workflow: ".github/workflows/public-typechecker-consumer.yml",
      inputOwner: ownerSource,
      fixturePath,
      original,
      npm: fileImage(options.npm),
      npx: fileImage(options.npx),
      node: fileImage(process.execPath),
      limits: {
        wsl2Acceptance: false,
        selectedOriginalCliPid: null,
        selectedOriginalBackendPid: null,
        identityCompanionsDoNotAttestOriginalPid: true,
      },
    });
    const captured = [];
    for (const role of ["before", "after"] as Role[]) {
      const out = join(options.out, role);
      mkdirSync(out);
      const owned = recorder(out),
        root = join(dirname(options.out), "11317-" + role);
      record.json("phase.json", { stage: "public-role", role, accepted: false });
      const authority = await authenticate(role, owned);
      const manager = managerImages(options.npm, options.npx);
      const installed = await install(root, role, options.npm, original, authority, owned);
      owned.json("manager-inputs.json", manager);
      const outcomes = execute(root, role, options.npx, original, owned, installed.verifyRoute);
      captured.push({ role, owned, installed, manager, outcomes });
    }
    record.json(
      "all-twenty-observations.json",
      captured.map((row) => ({ role: row.role, outcomes: row.outcomes })),
    );
    const results = captured.map((row) => {
      const outcomes = assess(row.role, row.outcomes, row.owned);
      assertCustody(row.installed.images);
      assertCustody(row.manager);
      row.installed.verifyRoute();
      row.owned.json("custody-after.json", {
        unchanged: true,
        images: row.installed.images,
        manager: row.manager,
      });
      return {
        role: row.role,
        profile: profiles[row.role],
        outcomes,
        publicAcceptance: row.role === "after",
      };
    });
    record.json("result.json", {
      status: "complete",
      results,
      beforeIsWitnessOnly: true,
      originalTextCommands: 2,
      wholeJsonCommands: 18,
      publicAcceptance: true,
      wsl2Acceptance: false,
      selectedOriginalBackendPid: null,
    });
    record.json("phase.json", { stage: "complete", accepted: true });
  } catch (error) {
    record.json("failure.json", {
      accepted: false,
      name: error instanceof Error ? error.name : null,
      message: String(error),
      stack: error instanceof Error ? error.stack : null,
    });
    throw error;
  }
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main(process.argv.slice(2)).catch((error) => {
    writeFileSync(2, String(error) + "\n");
    process.exitCode = 1;
  });
}
