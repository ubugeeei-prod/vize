import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdirSync, realpathSync, writeFileSync } from "node:fs";
import { dirname, isAbsolute, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { assertCustody, fileImage } from "../../../support/release/computed_event_custody.ts";
import { authenticate } from "./authority.ts";
import {
  input,
  profileFor,
  vueVersions,
  limitations,
  reporter,
  requireDefaultProvider,
  type Role,
} from "./contract.ts";
import { execute, assess, recorder } from "./execute.ts";
import { install } from "./install.ts";

export function argumentsFor(argv: string[]) {
  assert.equal(argv.length, 6);
  const values = new Map<string, string>();
  for (let i = 0; i < argv.length; i += 2) {
    const key = argv[i]!,
      value = argv[i + 1]!;
    assert(["--out", "--npm", "--npx"].includes(key));
    assert(!values.has(key));
    assert(isAbsolute(value));
    values.set(key, value);
  }
  assert.equal(values.size, 3);
  return { out: values.get("--out")!, npm: values.get("--npm")!, npx: values.get("--npx")! };
}
export function requireMainDispatch(environment: NodeJS.ProcessEnv) {
  assert(environment.GITHUB_EVENT_NAME === "workflow_dispatch", "manual consumer event required");
  assert(environment.GITHUB_REPOSITORY === "ubugeeei-prod/vize", "consumer repository required");
  assert(environment.GITHUB_REF === "refs/heads/main", "consumer requires the main source ref");
}
export function verifyAllBeforeAssessment(rows: { verify: () => void }[]) {
  const errors: unknown[] = [];
  for (const row of rows) {
    try {
      row.verify();
      errors.push(null);
    } catch (error) {
      errors.push(
        error instanceof Error ? { name: error.name, message: error.message } : String(error),
      );
    }
  }
  return errors;
}
export async function main(argv: string[]) {
  const options = argumentsFor(argv);
  // UNKNOWN containing release cannot trigger any public package operation.
  profileFor("before");
  profileFor("after");
  assert.equal(realpathSync(dirname(options.out)), dirname(options.out));
  mkdirSync(options.out);
  const record = recorder(options.out),
    captured = [],
    cache = new Map<string, Buffer>();
  const own = [
    "original.ts",
    "contract.ts",
    "authority.ts",
    "install.ts",
    "execute.ts",
    "run.ts",
  ].map((name) => fileImage(fileURLToPath(new URL(name, import.meta.url))));
  const checkSource = () => assertCustody(own);
  try {
    requireDefaultProvider(process.env, (rows) =>
      record.json("default-provider-environment.json", rows),
    );
    requireMainDispatch(process.env);
    const head = execFileSync("git", ["rev-parse", "HEAD"], { encoding: "utf8" }).trim();
    assert.equal(head, process.env.GITHUB_SHA);
    record.json("controller.json", {
      head,
      run: process.env.GITHUB_RUN_ID,
      attempt: process.env.GITHUB_RUN_ATTEMPT,
      own,
      reporter,
      limitations,
      sourceRole: "ADAPTED_DEFAULT_PROVIDER_CONTROL_NOT_EXACT_REPORTED_CORSA_PATH_ENVIRONMENT",
    });
    for (const role of ["before", "after"] as Role[]) {
      const authorityOut = join(options.out, role + "-authority");
      mkdirSync(authorityOut);
      const authority = await authenticate(role, recorder(authorityOut));
      for (const vue of vueVersions) {
        const out = join(options.out, role + "-vue-" + vue);
        mkdirSync(out);
        const owned = recorder(out);
        const original = input(vue),
          root = join(dirname(options.out), "11339-" + role + "-" + vue);
        const installed = await install(
          root,
          role,
          original,
          options.npm,
          options.npx,
          authority,
          owned,
          cache,
        );
        const verify = () => {
          checkSource();
          installed.verify();
        };
        const results = execute(root, role, options.npx, original, owned, verify);
        captured.push({ role, vue, owned, installed, results, verify });
      }
    }
    record.json(
      "all-complete-observations.json",
      captured.map((row) => ({
        role: row.role,
        vue: row.vue,
        results: row.results,
        publicAcceptance: false,
      })),
    );
    // Every role/control final custody is checked BEFORE any semantic aggregate.
    const custody = verifyAllBeforeAssessment(captured);
    record.json("all-final-custody.json", { custody, publicAcceptance: false });
    assert(
      custody.every((error) => error === null),
      "final custody failure; do not assess semantics",
    );
    const outcomes: unknown[] = [],
      errors: unknown[] = [];
    for (const row of captured) {
      try {
        outcomes.push({
          role: row.role,
          vue: row.vue,
          result: assess(row.role, row.results, row.owned),
        });
        errors.push(null);
      } catch (error) {
        outcomes.push(null);
        errors.push(
          error instanceof Error ? { name: error.name, message: error.message } : String(error),
        );
      }
    }
    record.json("result.json", {
      status: "CAPTURED_UNQUALIFIED",
      outcomes,
      errors,
      limitations,
      publicAcceptance: false,
      accepted: false,
      productQualified: false,
      afterQualified: false,
      issueQualified: false,
      wholePublicOracleOwned: false,
      originalBackend: "UNKNOWN",
    });
    assert(
      errors.every((error) => error === null),
      "partial facts differ; whole raw remains retained/unqualified",
    );
  } catch (error) {
    record.json("failure.json", {
      accepted: false,
      publicAcceptance: false,
      error: String(error),
      stack: error instanceof Error ? error.stack : null,
    });
    throw error;
  }
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url))
  main(process.argv.slice(2)).catch((error) => {
    writeFileSync(2, String(error) + "\n");
    process.exitCode = 1;
  });
