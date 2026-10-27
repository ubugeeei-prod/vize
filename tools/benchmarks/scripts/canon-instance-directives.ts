/** Actual source CLI/native directive receipts; no performance claim. */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { cases, referenceSource } from "./canon-instance-directives-cases.ts";
import {
  assertErrors,
  isExecutableFactory,
  jsonValue,
  manifest,
  packet,
  request,
  type PacketObservation,
} from "./canon-instance-directives-model.ts";
import {
  hash,
  identify,
  run,
  save,
  wrapperSource,
  type Identity,
} from "./canon-instance-directives-io.ts";
export { packet } from "./canon-instance-directives-model.ts";
export { cases } from "./canon-instance-directives-cases.ts";

type Role = "base" | "head";
type Provider = { version: string; manifest: Identity };
type Observation = {
  id: string;
  sourceSha256: string;
  base: PacketObservation["diagnostics"];
  head: PacketObservation["diagnostics"];
  virtualTsUnchanged: boolean;
};
const augmentation =
  'import "vue";\ndeclare module "vue" {\n  interface ComponentCustomProperties {\n    x: number;\n    pluginMethod(value: string): string;\n  }\n}\nexport {};\n';

export async function main(argv: string[]): Promise<void> {
  assert.equal(argv.length, 5, "sourceRoot providerRoot baseCli headCli captureRoot");
  const [sourceRoot, providerRoot, baseCli, headCli, output] = argv.map((file) =>
    path.resolve(file),
  );
  assert.ok(
    sourceRoot !== undefined &&
      providerRoot !== undefined &&
      baseCli !== undefined &&
      headCli !== undefined &&
      output !== undefined,
  );
  fs.mkdirSync(output, { recursive: true });
  const fixture = path.join(
    sourceRoot,
    "tests/_fixtures/vue-language-tools/upstream/test-workspace",
  );
  const original = fs.readFileSync(path.join(fixture, "tsc/#3488/main.vue"), "utf8");
  const baseConfig = fs.readFileSync(path.join(fixture, "tsconfig.base.json")),
    config = fs.readFileSync(path.join(fixture, "tsc/#3488/tsconfig.json"));
  const providers: Record<string, Provider> = {};
  const versions: Array<[string, string]> = [
    ["vue", "3.6.0-rc.6"],
    ["typescript", "6.0.3"],
    ["native-typescript", "7.1.0-beta"],
  ];
  for (const [name, version] of versions) {
    const file = path.join(providerRoot, "node_modules", name, "package.json"),
      actual = manifest(jsonValue(fs.readFileSync(file, "utf8")));
    assert.equal(actual.version, version);
    providers[name] = { version, manifest: identify(file) };
    save(path.join(output, "providers", name + ".manifest.json"), actual);
  }
  const loader: unknown = await import(
    pathToFileURL(path.join(providerRoot, "node_modules/native-typescript/lib/getExePath.js")).href
  );
  assert.ok(isExecutableFactory(loader), "invalid native executable factory");
  const candidate = loader.default();
  assert.ok(typeof candidate === "string", "native executable path must be a string");
  const native = fs.realpathSync(candidate);
  assert.ok(native.startsWith(fs.realpathSync(providerRoot) + path.sep));
  const platformManifest = path.resolve(path.dirname(native), "../package.json");
  assert.equal(
    manifest(jsonValue(fs.readFileSync(platformManifest, "utf8"))).version,
    "7.1.0-beta",
  );
  const jsCompiler = path.join(providerRoot, "node_modules/typescript/bin/tsc");
  const version = run(output, "providers/native-version", native, ["--version"], providerRoot);
  assert.equal(version.status, 0);
  assert.equal(version.stdout.toString(), "Version 7.1.0-beta\n");
  const jsVersion = run(
    output,
    "providers/js-version",
    process.execPath,
    [jsCompiler, "--version"],
    providerRoot,
  );
  assert.equal(jsVersion.status, 0);
  assert.equal(jsVersion.stdout.toString(), "Version 6.0.3\n");
  for (const name of ["package.json", "package-lock.json"])
    fs.copyFileSync(path.join(providerRoot, name), path.join(output, "providers", name));
  const binaries = {
    base: identify(baseCli),
    head: identify(headCli),
    native: identify(native),
    node: identify(process.execPath),
    jsCompiler: identify(jsCompiler),
  };
  assert.notEqual(binaries.base.sha256, binaries.head.sha256, "different source products required");
  save(path.join(output, "providers.json"), {
    providers,
    binaries,
    platform: identify(platformManifest),
  });
  const wrapper = path.join(output, "native-observer.mjs");
  fs.writeFileSync(wrapper, wrapperSource, { mode: 0o755 });
  const observations: Observation[] = [];
  for (const row of cases(original)) {
    const directory = path.join(output, "projects", row.id, "test-workspace/tsc/#3488"),
      workspace = path.resolve(directory, "../..");
    fs.mkdirSync(directory, { recursive: true });
    fs.symlinkSync(
      path.join(providerRoot, "node_modules"),
      path.join(workspace, "node_modules"),
      "dir",
    );
    fs.writeFileSync(path.join(workspace, "tsconfig.base.json"), baseConfig);
    fs.writeFileSync(path.join(directory, "tsconfig.json"), config);
    fs.writeFileSync(path.join(directory, "main.vue"), row.source);
    if (row.augmented) fs.writeFileSync(path.join(directory, "augmentation.d.ts"), augmentation);
    const inputPaths = [
      path.join(directory, "main.vue"),
      path.join(directory, "tsconfig.json"),
      path.join(workspace, "tsconfig.base.json"),
      ...(row.augmented ? [path.join(directory, "augmentation.d.ts")] : []),
    ];
    const inputs = inputPaths.map(identify);
    save(path.join(output, "cases", row.id, "input.json"), { ...row, inputs });
    const roles: Partial<Record<Role, PacketObservation>> = {};
    const products: Array<[Role, string]> = [
      ["base", baseCli],
      ["head", headCli],
    ];
    for (const [role, binary] of products) {
      const env: NodeJS.ProcessEnv = {
        ...process.env,
        NO_COLOR: "1",
        CORSA_PATH: wrapper,
        VIZE_DIRECTIVE_NATIVE: native,
        VIZE_DIRECTIVE_NATIVE_CAPTURE: path.join(output, "cases", row.id, role, "native"),
      };
      const result = run(
        output,
        "cases/" + row.id + "/" + role + "/cli",
        binary,
        [
          "check",
          ".",
          "--no-config",
          "--format",
          "json",
          "--show-virtual-ts",
          "--quiet",
          "--corsa-path",
          wrapper,
        ],
        directory,
        env,
      );
      const parsed = packet(result.stdout, result.status, directory);
      roles[role] = parsed;
      assert.ok(
        parsed.result.files.some(
          (file) => path.basename(file.file) === "main.vue" && typeof file.virtualTs === "string",
        ),
      );
      save(path.join(output, "cases", row.id, role, "packet.json"), parsed);
      assert.deepEqual(
        inputPaths.map(identify),
        inputs,
        "source/config must remain byte-identical",
      );
      const nativeCapture = env.VIZE_DIRECTIVE_NATIVE_CAPTURE;
      assert.ok(typeof nativeCapture === "string");
      const nativeCalls = fs.readdirSync(nativeCapture).filter((name) => name.startsWith("call-"));
      assert.ok(
        nativeCalls.some((name) =>
          request(
            jsonValue(fs.readFileSync(path.join(nativeCapture, name, "request.json"), "utf8")),
          ).argv.includes("--project"),
        ),
        "real native checking must occur",
      );
    }
    const base = roles.base,
      head = roles.head;
    assert.ok(base && head, "both source roles must return complete packets");
    const virtual = (value: PacketObservation) =>
      value.result.files
        .map(({ file, virtualTs }) => ({ file, virtualTs }))
        .sort((left, right) => left.file.localeCompare(right.file));
    assert.deepEqual(virtual(base), virtual(head), row.id + ": complete production VTS unchanged");
    assertErrors(row, head.diagnostics);
    for (const diagnostic of head.diagnostics)
      assert.ok(
        base.diagnostics.some((before) => JSON.stringify(before) === JSON.stringify(diagnostic)),
        row.id + ": retained complete diagnostic changed",
      );
    if (row.original)
      assert.deepEqual(
        base.diagnostics
          .map((diagnostic) => diagnostic.code)
          .sort((left, right) => (left ?? 0) - (right ?? 0)),
        [2339, 2578],
      );
    const reference = path.join(output, "references", row.id);
    fs.mkdirSync(reference, { recursive: true });
    fs.symlinkSync(
      path.join(providerRoot, "node_modules"),
      path.join(reference, "node_modules"),
      "dir",
    );
    fs.writeFileSync(path.join(reference, "reference.ts"), referenceSource(row));
    fs.writeFileSync(path.join(reference, "tsconfig.base.json"), baseConfig);
    if (row.augmented) fs.writeFileSync(path.join(reference, "augmentation.d.ts"), augmentation);
    save(path.join(reference, "tsconfig.json"), {
      extends: "./tsconfig.base.json",
      compilerOptions: {
        module: "ESNext",
        moduleResolution: "Bundler",
        target: "ESNext",
        types: [],
      },
      files: ["reference.ts", ...(row.augmented ? ["augmentation.d.ts"] : [])],
      include: [],
    });
    for (const kind of ["js", "native"]) {
      const args = [
        "-p",
        "tsconfig.json",
        "--pretty",
        "false",
        ...(kind === "native" ? ["--checkers", "1"] : []),
      ];
      const result = run(
        output,
        "cases/" + row.id + "/reference-" + kind,
        kind === "js" ? process.execPath : native,
        kind === "js" ? [jsCompiler, ...args] : args,
        reference,
      );
      const codes = [...result.stdout.toString().matchAll(/^.+\(\d+,\d+\): error TS(\d+):/gmu)]
        .map((match) => {
          const code = match[1];
          assert.ok(code !== undefined);
          return +code;
        })
        .sort((left, right) => left - right);
      assert.deepEqual(
        codes,
        row.errors.map((diagnostic) => diagnostic.code).sort((left, right) => left - right),
        row.id + ": independent " + kind + " reference",
      );
      assert.equal(result.status === 0, codes.length === 0);
    }
    observations.push({
      id: row.id,
      sourceSha256: hash(row.source),
      base: base.diagnostics,
      head: head.diagnostics,
      virtualTsUnchanged: true,
    });
  }
  save(path.join(output, "acceptance.json"), {
    schema: "vize.canonInstanceDirectives.v1",
    cases: observations,
    binaries,
    original: { file: "tsc/#3488/main.vue", sha256: hash(original) },
    scope:
      "actual source CLI+native correctness; complete VTS equality; independent TS6/native7 references, no speed claim",
    knownSeparateScope:
      "triple-slash and last-line block generator retention gaps are not qualified by this matrix",
  });
}
if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href)
  await main(process.argv.slice(2));
