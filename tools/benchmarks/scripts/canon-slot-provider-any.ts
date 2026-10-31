/** Untimed production admission for declared-any and intrinsic error-any slot providers. */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { slotProviderTypes } from "../../../tests/tooling/support/canon-slot-provider-types.ts";
import { captureSlotPacket } from "./canon-slot-constructor-protocol.ts";
import { sdk, native, type Row } from "./canon-slot-constructor-audit.ts";
import { packet, isRecord } from "./canon-instance-directives-model.ts";
import { identify, hash, run, save } from "./canon-instance-directives-io.ts";

const original = path.resolve(
  "tests/_fixtures/differential/typechecker/unknown-component-slot-payload-11027",
);
const successor = path.resolve("tests/_fixtures/typechecker/slot-provider-owners-11027/imported");
export const sourceDeclarations = [
  "content-mapper-helpers.d.ts",
  "content-mapper-helpers.d.mts",
  "content-mapper-helpers.d.cts",
  "generated/content-mapper-types.d.ts",
  "virtual-ts.d.ts",
  "virtual-ts.d.mts",
  "virtual-ts.d.cts",
  "virtual-ts-globals.d.ts",
  "virtual-ts-patterns.d.ts",
];
const read = (root: string, file: string) => fs.readFileSync(path.join(root, file), "utf8");

export function sourceSdk(project: string, provider: string, binary: string) {
  sdk(project, provider, binary);
  const installed = path.join(project, "node_modules/vize"),
    source = path.resolve("npm/cli");
  const manifest: unknown = JSON.parse(read(source, "package.json"));
  assert.ok(
    isRecord(manifest) &&
      isRecord(manifest.typescript) &&
      isRecord(manifest.typescript.contentMapper),
  );
  manifest.typescript.contentMapper.exec = [binary, "content-mapper"];
  save(path.join(installed, "package.json"), manifest);
  for (const file of sourceDeclarations) {
    const destination = path.join(installed, file);
    fs.rmSync(destination, { force: true });
    fs.mkdirSync(path.dirname(destination), { recursive: true });
    fs.copyFileSync(path.join(source, file), destination);
  }
  save(path.join(project, "source-sdk-custody.json"), {
    package: identify(path.join(installed, "package.json")),
    declarations: sourceDeclarations.map((file) => ({
      file,
      source: identify(path.join(source, file)),
      installed: identify(path.join(installed, file)),
    })),
  });
}

export function expected(source: string, invalid: boolean): Row[] {
  if (!invalid) return [];
  const prefix = source.slice(0, source.indexOf("slotReport(item)")),
    lines = prefix.split("\n");
  assert.ok(source.includes("expectNumber(slotReport(item))"));
  return [
    {
      file: "src/Tiny.vue",
      line: lines.length,
      column: lines.at(-1)!.length + 1,
      code: 2345,
      message: `Argument of type '"any"' is not assignable to parameter of type '"number"'.`,
    },
  ];
}

async function main() {
  const [providerArg, binaryArg, outputArg] = process.argv.slice(2);
  assert.ok(providerArg && binaryArg && outputArg);
  const provider = path.resolve(providerArg),
    binary = path.resolve(binaryArg),
    output = path.resolve(outputArg);
  assert.ok(!fs.existsSync(output), "do not overwrite previous evidence");
  fs.mkdirSync(output, { recursive: true });
  const { default: getExecutable } = await import(
    pathToFileURL(path.join(provider, "node_modules/typescript-native/lib/getExePath.js")).href
  );
  const checker: string = getExecutable();
  const project = path.join(output, "project");
  fs.mkdirSync(path.join(project, "src"), { recursive: true });
  sourceSdk(project, provider, binary);
  const config: unknown = JSON.parse(read(original, "tsconfig.json.txt"));
  assert.ok(isRecord(config));
  save(path.join(project, "tsconfig.json"), config);
  save(path.join(project, "tsconfig.mapper.json"), {
    ...config,
    contentMappers: [{ package: "vize", extensions: [".vue"] }],
  });
  const known = read(original, "component-known.ts.txt"),
    source = read(original, "source.vue.txt");
  const stages = [
    { id: "known", component: known, source, kind: "number", invalid: false },
    {
      id: "declared-any",
      component: read(successor, "component-any.ts.txt"),
      source,
      kind: "any",
      invalid: true,
    },
    {
      id: "error-any",
      component: read(successor, "component-error.ts.txt"),
      source,
      kind: "error",
      invalid: false,
    },
    {
      id: "unknown",
      component: read(original, "component-unknown.ts.txt"),
      source,
      kind: "any",
      invalid: true,
    },
    {
      id: "unresolved-tag",
      component: known,
      source: read(original, "unresolved.vue.txt"),
      kind: "error",
      invalid: false,
    },
    { id: "repaired", component: known, source, kind: "number", invalid: false },
    {
      id: "unicode-crlf-any",
      component: read(successor, "component-any.ts.txt"),
      source: read(original, "unicode-crlf.vue.txt"),
      kind: "any",
      invalid: true,
    },
  ] as const;
  const observations: unknown[] = [];
  for (const stage of stages) {
    const out = path.join(output, stage.id);
    fs.mkdirSync(out);
    fs.writeFileSync(path.join(project, "component.ts"), stage.component);
    fs.writeFileSync(path.join(project, "src/Tiny.vue"), stage.source);
    fs.writeFileSync(path.join(out, "component.ts.txt"), stage.component);
    fs.writeFileSync(path.join(out, "source.vue.txt"), stage.source);
    const wanted = expected(stage.source, stage.invalid);
    const env = {
      ...process.env,
      TMPDIR: path.join(out, "tmp"),
      XDG_CACHE_HOME: path.join(out, "cache"),
    };
    fs.mkdirSync(env.TMPDIR);
    const transformed = captureSlotPacket(binary, project, stage.source, path.join(out, "packet"));
    assert.equal(typeof transformed.result.text, "string");
    const vts = transformed.result.text as string;
    fs.writeFileSync(path.join(out, "virtual.ts"), vts);
    const types = slotProviderTypes(path.join(project, "src/Tiny.vue.ts"), vts);
    const bindings = types.rows.filter((row) => row.role === "slot-binding");
    const reports = types.rows.filter((row) => row.role === "slot-report");
    assert.equal(bindings.length, 2, "both actual slot scopes must be queried");
    assert.equal(reports.length, 1, "the complete authored witness must be queried");
    assert.deepEqual(
      bindings.map((row) => row.intrinsicName),
      bindings.map(() => stage.kind),
    );
    assert.deepEqual(
      bindings.map((row) => row.any),
      bindings.map(() => stage.kind !== "number"),
    );
    if (stage.kind === "error")
      assert.deepEqual(
        reports.map((row) => row.intrinsicName),
        reports.map(() => "error"),
      );
    else
      assert.deepEqual(
        reports.map((row) => row.display),
        reports.map(() => JSON.stringify(stage.kind)),
      );
    save(path.join(out, "type-identity.json"), types);

    const mapper = run(
      out,
      "native-mapper",
      checker,
      [
        "--runExternalCode",
        "--checkers",
        "1",
        "--noEmit",
        "--pretty",
        "false",
        "--project",
        "tsconfig.mapper.json",
      ],
      project,
      env,
    );
    const cli = run(
      out,
      "source-cli",
      binary,
      [
        "check",
        "--no-config",
        "--format",
        "json",
        "--show-virtual-ts",
        "--tsconfig",
        "tsconfig.json",
        "--corsa-path",
        checker,
      ],
      project,
      env,
    );
    const framework = run(
      out,
      "vue-tsc",
      process.execPath,
      [
        path.join(provider, "node_modules/vue-tsc/bin/vue-tsc.js"),
        "--noEmit",
        "--pretty",
        "false",
        "--project",
        "tsconfig.json",
      ],
      project,
      env,
    );
    for (const process of [mapper, cli, framework])
      assert.equal(process.stderr.length, 0, "complete stderr must remain empty");
    const mapperRows = native(mapper.stdout, mapper.status, project);
    const cliRows = packet(cli.stdout, cli.status, project).diagnostics.map(
      ({ file, line, column, code, message }) => ({ file, line, column, code, message }),
    );
    const frameworkRows = native(framework.stdout, framework.status, project);
    assert.deepEqual(mapperRows, wanted, `${stage.id}: complete native mapper diagnostics`);
    const wantedCli = wanted.map((row) => ({
      ...row,
      message: `${row.message} (source: ${stage.source.split("\n")[row.line - 1]!.trim()})`,
    }));
    assert.deepEqual(cliRows, wantedCli, `${stage.id}: complete native CLI diagnostics`);
    assert.deepEqual(frameworkRows, wanted, `${stage.id}: independent framework diagnostics`);
    const observation = {
      stage: stage.id,
      sourceSha256: hash(stage.source),
      componentSha256: hash(stage.component),
      diagnostics: { mapper: mapperRows, cli: cliRows, framework: frameworkRows },
      typeIdentity: types,
      timingUsable: false,
    };
    observations.push(observation);
    save(path.join(out, "acceptance.json"), observation);
  }
  save(path.join(output, "acceptance.json"), {
    issue: 11027,
    sourceBinary: identify(binary),
    nativeBackend: identify(checker),
    referenceCompiler: identify(path.join(provider, "node_modules/typescript/lib/typescript.js")),
    providerLock: identify(path.join(provider, "package-lock.json")),
    stages: observations,
    invalidToRepair: true,
    timingUsable: false,
  });
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url))
  await main();
