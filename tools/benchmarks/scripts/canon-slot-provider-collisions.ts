/** Actual source CLI/official mapper/framework checks for distinct registry owners. */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { slotProviderTypes } from "../../../tests/tooling/support/canon-slot-provider-types.ts";
import { captureSlotPacket } from "./canon-slot-constructor-protocol.ts";
import { native, type Row } from "./canon-slot-constructor-audit.ts";
import { slotCliPacket } from "./canon-slot-provider-cli.ts";
import { identify, hash, run, save } from "./canon-instance-directives-io.ts";
import { sourceSdk } from "./canon-slot-provider-any.ts";

const fixture = path.resolve("tests/_fixtures/typechecker/slot-provider-owners-11027/collisions");
type Control = {
  id: string;
  names: string[];
  anyKey: string;
  typeOnly: boolean;
  sourceSha256: string;
  globalsSha256: string;
  diagnostics: Row[];
};
async function main() {
  const [providerArg, binaryArg, outputArg] = process.argv.slice(2);
  assert.ok(providerArg && binaryArg && outputArg);
  const provider = path.resolve(providerArg),
    binary = path.resolve(binaryArg),
    output = path.resolve(outputArg);
  assert.ok(!fs.existsSync(output), "fresh evidence directory required");
  fs.mkdirSync(output, { recursive: true });
  const { default: getExecutable } = await import(
    pathToFileURL(path.join(provider, "node_modules/typescript-native/lib/getExePath.js")).href
  );
  const checker: string = getExecutable();
  const controls = JSON.parse(fs.readFileSync(path.join(fixture, "controls.json"), "utf8"))
    .cases as Control[];
  assert.equal(
    controls.length,
    8,
    "all independent original both-order/both-owner controls remain",
  );
  const project = path.join(output, "project");
  fs.mkdirSync(path.join(project, "src"), { recursive: true });
  sourceSdk(project, provider, binary);
  for (const file of ["component.ts", "types.ts", "tsconfig.json"])
    fs.copyFileSync(path.join(fixture, `${file}.txt`), path.join(project, file));
  const config = JSON.parse(fs.readFileSync(path.join(project, "tsconfig.json"), "utf8")) as Record<
    string,
    unknown
  >;
  save(path.join(project, "tsconfig.mapper.json"), {
    ...config,
    contentMappers: [{ package: "vize", extensions: [".vue"] }],
  });
  const observations: unknown[] = [];
  for (const control of controls) {
    const source = fs.readFileSync(path.join(fixture, control.id, "source.vue.txt"), "utf8");
    const originalGlobals = fs.readFileSync(
      path.join(fixture, control.id, "globals.d.ts.txt"),
      "utf8",
    );
    assert.equal(hash(source), control.sourceSha256);
    assert.equal(hash(originalGlobals), control.globalsSha256);
    assert.equal(originalGlobals.split(": any;").length - 1, 1);
    const repairedGlobals = originalGlobals.replace(
      ": any;",
      ": typeof import('./component').default;",
    );
    for (const [state, globals, wanted] of [
      ["invalid", originalGlobals, control.diagnostics],
      ["repair", repairedGlobals, []],
    ] as const) {
      const out = path.join(output, control.id, state);
      fs.mkdirSync(out, { recursive: true });
      fs.writeFileSync(path.join(project, "src/Tiny.vue"), source);
      fs.writeFileSync(path.join(project, "globals.d.ts"), globals);
      fs.writeFileSync(path.join(out, "source.vue.txt"), source);
      fs.writeFileSync(path.join(out, "globals.d.ts.txt"), globals);
      const env = {
        ...process.env,
        TMPDIR: path.join(out, "tmp"),
        XDG_CACHE_HOME: path.join(out, "cache"),
      };
      fs.mkdirSync(env.TMPDIR);
      const transformed = captureSlotPacket(binary, project, source, path.join(out, "packet"));
      assert.equal(typeof transformed.result.text, "string");
      const text = transformed.result.text as string;
      fs.writeFileSync(path.join(out, "virtual.ts"), text);
      const types = slotProviderTypes(path.join(project, "src/Tiny.vue.ts"), text, {
        roots: [
          path.join(project, "globals.d.ts"),
          path.join(project, "component.ts"),
          path.join(project, "types.ts"),
        ],
        providerNames: ["A_b", "$Component_A_x2d_b", "$Component_A_x5f_b"],
      });
      const providers = types.rows.filter((row) => row.role === "actual-provider");
      assert.equal(
        providers.length,
        2,
        "two actual distinct declarations, never one sanitized alias",
      );
      assert.equal(new Set(providers.map((row) => row.symbol)).size, 2);
      assert.deepEqual(
        providers.map((row) => row.any).sort((left, right) => Number(left) - Number(right)),
        state === "invalid" ? [false, true] : [false, false],
      );
      const bindings = types.rows.filter((row) => row.role === "slot-binding");
      assert.equal(bindings.length, 4, "both emitted slot scopes for each distinct provider");
      assert.deepEqual(
        bindings
          .map((row) => row.intrinsicName)
          .sort((left, right) => String(left).localeCompare(String(right))),
        state === "invalid"
          ? ["any", "any", "number", "number"]
          : ["number", "number", "number", "number"],
      );
      // Full authored diagnostics below prove which tag owns each any; these
      // retained intrinsic queries additionally reject fabricated error-any.
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
          `${control.id}/${state}/${name}: complete stderr must remain empty`,
        );
      const mapperRows = native(mapper.stdout, mapper.status, project);
      const frameworkRows = native(framework.stdout, framework.status, project);
      const cliRows = slotCliPacket(cli, project).diagnostics.map(
        ({ file, line, column, code, message }) => ({ file, line, column, code, message }),
      );
      assert.deepEqual(mapperRows, wanted, `${control.id}/${state}: full official mapper vector`);
      assert.deepEqual(
        frameworkRows,
        wanted,
        `${control.id}/${state}: independent framework vector`,
      );
      assert.deepEqual(
        cliRows,
        wanted.map((row) => ({
          ...row,
          message: `${row.message} (source: ${source.split("\n")[row.line - 1]!.trim()})`,
        })),
        `${control.id}/${state}: full source CLI vector`,
      );
      const observation = {
        id: control.id,
        state,
        sourceSha256: hash(source),
        globalsSha256: hash(globals),
        diagnostics: { mapper: mapperRows, cli: cliRows, framework: frameworkRows },
        typeIdentity: types,
        timingUsable: false,
      };
      observations.push(observation);
      save(path.join(out, "acceptance.json"), observation);
    }
  }
  save(path.join(output, "acceptance.json"), {
    issue: 11027,
    sourceBinary: identify(binary),
    nativeBackend: identify(checker),
    providerLock: identify(path.join(provider, "package-lock.json")),
    stages: observations,
    bothSourceOrders: true,
    bothAnyOwners: true,
    typeOnlyAndOrdinary: true,
    invalidToRepair: true,
    timingUsable: false,
  });
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url))
  await main();
