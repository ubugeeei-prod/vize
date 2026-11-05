import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  assertOutput,
  expectedOutput,
  identity,
  roles,
  run,
  writeJson,
} from "./direct-child-props-runtime-11809.ts";

/** Exercise the same authored two-SFC input against both installed native hosts. */
export function verifyHelperAbiFallbacks(
  install: string,
  output: string,
  hosts: Record<(typeof roles)[number], string>,
  environment: NodeJS.ProcessEnv,
) {
  const sdk = path.join(install, "node_modules/vize");
  const manifest = JSON.parse(fs.readFileSync(path.join(sdk, "package.json"), "utf8")) as {
    files: string[];
  };
  const files = [
    "package.json",
    ...manifest.files.filter((file) => /\.d\.(?:ts|mts|cts)$/u.test(file)),
  ];
  const originals = new Map(files.map((file) => [file, fs.readFileSync(path.join(sdk, file))]));
  const before = files.map((file) => identity(path.join(sdk, file)));
  type Mutation =
    | { kind: "replace"; files: string[]; before: string; after: string }
    | { kind: "remove"; file: string }
    | { kind: "remove-helper-export" };
  const planFile = fileURLToPath(
    new URL(
      "../../../../tests/_fixtures/differential/typechecker/content-mapper-helper-abi-11809/cases.json",
      import.meta.url,
    ),
  );
  const planIdentity = identity(planFile);
  const plan = JSON.parse(fs.readFileSync(planFile, "utf8")) as {
    version: number;
    cases: { name: string; mutations: Mutation[] }[];
  };
  assert.equal(plan.version, 1);
  const cases = plan.cases;
  assert.equal(cases.length, 8);
  assert.equal(new Set(cases.map((row) => row.name)).size, 8);
  const write = (file: string, text: string) => {
    assert.ok(originals.has(file), "only snapshotted SDK inputs may change");
    fs.writeFileSync(path.join(sdk, file), text);
  };
  const mutate = (mutations: Mutation[]) => {
    for (const mutation of mutations) {
      switch (mutation.kind) {
        case "replace":
          for (const file of mutation.files) {
            assert.ok(originals.has(file));
            const original = originals.get(file)!.toString();
            assert.ok(mutation.before);
            assert.equal(original.split(mutation.before).length, 2, file);
            write(file, original.replace(mutation.before, mutation.after));
          }
          break;
        case "remove":
          assert.ok(originals.has(mutation.file));
          fs.unlinkSync(path.join(sdk, mutation.file));
          break;
        case "remove-helper-export": {
          const value = JSON.parse(originals.get("package.json")!.toString()) as {
            exports: Record<string, unknown>;
          };
          delete value.exports["./content-mapper-helpers"];
          write("package.json", JSON.stringify(value, null, 2) + "\n");
          break;
        }
        default:
          assert.fail("unknown ABI mutation");
      }
    }
  };
  const capture = path.join(output, "helper-abi");
  fs.mkdirSync(capture, { recursive: true });
  const completed: string[] = [];
  let qualified = false;
  let sdkRestored = true;
  try {
    for (const { name, mutations } of cases) {
      try {
        sdkRestored = false;
        mutate(mutations);
        const directory = path.join(capture, name);
        fs.mkdirSync(directory);
        writeJson(path.join(directory, "sdk-state.json"), {
          before,
          after: files.map((file) =>
            fs.existsSync(path.join(sdk, file))
              ? identity(path.join(sdk, file))
              : { path: path.join(sdk, file), missing: true },
          ),
        });
        for (const role of roles) {
          const project = path.join(output, "cases", `${role}-bench-count-false-wrong`);
          const inputs = ["PropsChild.vue", "PropsParent.vue", "tsconfig.json"].map((file) =>
            identity(path.join(project, file)),
          );
          const result = run(
            hosts[role],
            ["--runExternalCode", "--noEmit", "--pretty", "false", "-p", "tsconfig.json"],
            project,
            path.join(directory, role),
            environment,
          );
          assertOutput(result.status, result.stdout, result.stderr, expectedOutput("wrong"));
          assert.deepEqual(
            inputs.map((file) => identity(file.path)),
            inputs,
          );
          completed.push(`${name}:${role}`);
        }
      } finally {
        for (const [file, bytes] of originals) fs.writeFileSync(path.join(sdk, file), bytes);
        assert.deepEqual(
          files.map((file) => identity(path.join(sdk, file))),
          before,
          "SDK restored after every contrast",
        );
        sdkRestored = true;
      }
    }
    assert.deepEqual(
      completed,
      cases.flatMap(({ name }) => roles.map((role) => name + ":" + role)),
    );
    assert.deepEqual(identity(planFile), planIdentity);
    qualified = true;
  } finally {
    writeJson(path.join(capture, "inventory.json"), {
      inputPlan: planIdentity,
      completed,
      qualified,
      authoredInputContrastsVerified: completed.length,
      sdkRestored,
    });
  }
}
