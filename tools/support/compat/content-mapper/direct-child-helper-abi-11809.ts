import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
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
  const frontends = ["ts", "mts", "cts"].map(
    (extension) => `content-mapper-helpers.d.${extension}`,
  );
  const variants = ["ts", "mts", "cts"].map((extension) => `virtual-ts.d.${extension}`);
  const write = (file: string, text: string) => fs.writeFileSync(path.join(sdk, file), text);
  const replace = (file: string, from: string, to: string) => {
    const original = originals.get(file)!.toString();
    assert.equal(original.split(from).length, 2, `${file} unique mutation boundary`);
    write(file, original.replace(from, to));
  };
  const legacyFrontends = () =>
    frontends.forEach((file) =>
      replace(
        file,
        'export type * from "./generated/content-mapper-types.js" with { "resolution-mode": "import" };\n',
        "",
      ),
    );
  const cases: [string, () => void][] = [
    [
      "missing-public-virtual-alias",
      () =>
        variants.forEach((file) =>
          replace(file, "type $ComponentRawProps<", "type $MissingComponentRawProps<"),
        ),
    ],
    [
      "missing-private-binding",
      () =>
        replace(
          "virtual-ts-emits.d.ts",
          "type $LooseEmitPropsFactory<",
          "type $MissingLooseEmitPropsFactory<",
        ),
    ],
    ["missing-private-dependency", () => fs.unlinkSync(path.join(sdk, "virtual-ts-emits.d.ts"))],
    [
      "missing-core-export",
      () => replace("generated/content-mapper-types.d.ts", "  $ComponentInput,\n", ""),
    ],
    [
      "legacy-no-generated-types",
      () => {
        legacyFrontends();
        fs.unlinkSync(path.join(sdk, "generated/content-mapper-types.d.ts"));
      },
    ],
    ["legacy-unexported-types", legacyFrontends],
    [
      "legacy-unexported-package",
      () => {
        const value = JSON.parse(originals.get("package.json")!.toString()) as {
          exports: Record<string, unknown>;
        };
        delete value.exports["./content-mapper-helpers"];
        write("package.json", JSON.stringify(value, null, 2) + "\n");
      },
    ],
    ["restored-sdk", () => {}],
  ];
  const capture = path.join(output, "helper-abi");
  fs.mkdirSync(capture, { recursive: true });
  const completed: string[] = [];
  let qualified = false;
  let sdkRestored = true;
  try {
    for (const [name, mutate] of cases) {
      try {
        sdkRestored = false;
        mutate();
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
    assert.equal(completed.length, 16);
    qualified = true;
  } finally {
    writeJson(path.join(capture, "inventory.json"), {
      completed,
      qualified,
      authoredInputContrastsVerified: completed.length,
      sdkRestored,
    });
  }
}
