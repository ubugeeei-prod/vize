/** Untimed production admission for exact registry-key slot provider ownership. */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { slotProviderTypes } from "../../../tests/tooling/support/canon-slot-provider-types.ts";
import { captureSlotPacket } from "./canon-slot-constructor-protocol.ts";
import { native } from "./canon-slot-constructor-audit.ts";
import { slotCliPacket } from "./canon-slot-provider-cli.ts";
import { identify, hash, run, save } from "./canon-instance-directives-io.ts";
import { sourceSdk, expected } from "./canon-slot-provider-any.ts";

const fixture = path.resolve("tests/_fixtures/typechecker/slot-provider-owners-11027/registered");
const kinds = {
  known: "number",
  "declared-any": "any",
  "error-any": "error",
  "missing-key": "error",
  "fallback-any": "any",
  "pascal-any-raw-typed": "any",
  "pascal-typed-raw-any": "number",
  "camel-any-only": "any",
  "camel-typed-only": "number",
  "camel-any-raw-typed": "any",
  "camel-typed-raw-any": "number",
  "pascal-any-camel-typed": "any",
  "pascal-typed-camel-any": "number",
  repaired: "number",
  "unicode-any": "any",
} as const;
type Case = {
  id: keyof typeof kinds;
  inputs: Array<{ file: string; source: string }>;
  diagnostics: Array<{
    file: string;
    line: number;
    column: number;
    code: number;
    message: string;
    severity: number;
  }>;
};

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
  const cases = JSON.parse(fs.readFileSync(path.join(fixture, "controls.json"), "utf8"))
    .cases as Case[];
  assert.deepEqual(
    cases.map((row) => row.id),
    Object.keys(kinds),
    "all original controls must run in order",
  );
  const project = path.join(output, "project");
  fs.mkdirSync(path.join(project, "src"), { recursive: true });
  sourceSdk(project, provider, binary);
  const observations: unknown[] = [];
  for (const stage of cases) {
    const kind = kinds[stage.id],
      out = path.join(output, stage.id);
    fs.mkdirSync(out);
    for (const input of stage.inputs) {
      const bytes = fs.readFileSync(path.join(fixture, input.source));
      const destination = path.join(project, input.file);
      fs.mkdirSync(path.dirname(destination), { recursive: true });
      fs.writeFileSync(destination, bytes);
      fs.copyFileSync(path.join(fixture, input.source), path.join(out, input.source));
    }
    const source = fs.readFileSync(path.join(project, "src/Tiny.vue"), "utf8");
    const globals = fs.readFileSync(path.join(project, "globals.d.ts"), "utf8");
    const config: unknown = JSON.parse(
      fs.readFileSync(path.join(project, "tsconfig.json"), "utf8"),
    );
    assert.ok(config && typeof config === "object" && !Array.isArray(config));
    save(path.join(project, "tsconfig.mapper.json"), {
      ...config,
      contentMappers: [{ package: "vize", extensions: [".vue"] }],
    });
    const wanted = expected(source, kind === "any");
    assert.deepEqual(
      stage.diagnostics.map(({ severity, ...row }) => {
        assert.equal(severity, 1);
        return row;
      }),
      wanted,
      `${stage.id}: complete independent framework contract`,
    );
    const env = {
      ...process.env,
      TMPDIR: path.join(out, "tmp"),
      XDG_CACHE_HOME: path.join(out, "cache"),
    };
    fs.mkdirSync(env.TMPDIR);
    const transformed = captureSlotPacket(binary, project, source, path.join(out, "packet"));
    assert.equal(typeof transformed.result.text, "string");
    const vts = transformed.result.text as string;
    fs.writeFileSync(path.join(out, "virtual.ts"), vts);
    const types = slotProviderTypes(path.join(project, "src/Tiny.vue.ts"), vts, {
      roots: [path.join(project, "globals.d.ts"), path.join(project, "component.ts")],
      providerNames: ["RegistryChild", "registry_child"],
    });
    const bindings = types.rows.filter((row) => row.role === "slot-binding"),
      reports = types.rows.filter((row) => row.role === "slot-report"),
      providers = types.rows.filter((row) => row.role === "actual-provider");
    assert.equal(providers.length, 1, "one actual generated registry provider must remain");
    assert.equal(providers[0]!.any, kind !== "number", `${stage.id}: actual selected provider`);
    const authored = stage.inputs.find((input) => input.file === "src/Tiny.vue");
    assert.ok(authored);
    assert.ok(
      ["source-known.txt", "source-fallback-any.txt", "source-unicode-any.txt"].includes(
        authored.source,
      ),
    );
    // Under the retained checkUnknownComponents:false config, the existing
    // PascalCase policy emits a props scope for RegistryChild. The authored
    // registry-child controls have one template scope and no props scope.
    // This describes emission roles, never the provider's any/error identity.
    const expectedScopes =
      authored.source === "source-fallback-any.txt" ? ["template"] : ["template", "props"];
    const actualScopes = bindings.map((row) => {
      const owner = /^_slot_(props_)?default_([0-9]+)$/u.exec(row.ownerFunction ?? "");
      assert.ok(owner, `${stage.id}: each item binding must belong to an actual slot function`);
      return { role: owner[1] ? "props" : "template", scope: owner[2] };
    });
    assert.deepEqual(
      actualScopes.map((row) => row.role),
      expectedScopes,
      `${stage.id}: complete expected slot emission roles`,
    );
    assert.equal(
      new Set(actualScopes.map((row) => row.scope)).size,
      1,
      `${stage.id}: both roles must retain the same authored slot`,
    );
    assert.equal(reports.length, 1, "the authored witness must remain complete");
    assert.deepEqual(
      bindings.map(({ flags, any, intrinsicName }) => ({ flags, any, intrinsicName })),
      expectedScopes.map(() => ({
        flags: kind === "number" ? 64 : 1,
        any: kind !== "number",
        intrinsicName: kind,
      })),
    );
    assert.deepEqual(
      reports.map(({ flags, any, display, intrinsicName }) => ({
        flags,
        any,
        display,
        intrinsicName,
      })),
      [
        kind === "error"
          ? { flags: 1, any: true, display: "any", intrinsicName: "error" }
          : { flags: 1024, any: false, display: JSON.stringify(kind), intrinsicName: null },
      ],
    );
    const local = types.diagnostics.filter((row) => row.file === types.file);
    assert.deepEqual(
      local.map(({ code, message }) => ({ code, message })),
      wanted.map(({ code, message }) => ({ code, message })),
      `${stage.id}: exact generated TypeScript diagnostics`,
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
        "--quiet",
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
    for (const [name, result] of [
      ["native-mapper", mapper],
      ["vue-tsc", framework],
    ] as const)
      assert.equal(
        result.stderr.length,
        0,
        `${stage.id}/${name}: complete stderr must remain empty`,
      );
    const mapperRows = native(mapper.stdout, mapper.status, project),
      cliRows = slotCliPacket(cli, project).diagnostics.map(
        ({ file, line, column, code, message }) => ({ file, line, column, code, message }),
      ),
      frameworkRows = native(framework.stdout, framework.status, project);
    assert.deepEqual(
      mapperRows,
      wanted,
      `${stage.id}: complete official native mapper diagnostics`,
    );
    assert.deepEqual(
      cliRows,
      wanted.map((row) => ({
        ...row,
        message: `${row.message} (source: ${source.split("\n")[row.line - 1]!.trim()})`,
      })),
      `${stage.id}: complete native CLI diagnostics`,
    );
    assert.deepEqual(frameworkRows, wanted, `${stage.id}: independent framework diagnostics`);
    const observation = {
      stage: stage.id,
      sourceSha256: hash(source),
      globalsSha256: hash(globals),
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
    simultaneousKeyRepairs: true,
    timingUsable: false,
  });
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url))
  await main();
