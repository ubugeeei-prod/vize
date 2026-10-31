import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { mkdirSync, readFileSync, symlinkSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import type tsApi from "../../../../npm/cli/node_modules/typescript/lib/typescript.js";
import { asRecord, asString, parseJson } from "../../json-value.ts";
import { Recorder } from "../../config/windows_consumer/common.ts";
import {
  slotRequests,
  slotResponses,
} from "../../../benchmarks/scripts/canon-slot-constructor-protocol.ts";
import { hook, observed } from "../editor_vite_stub/custody.ts";
import { assertCustody, assertDeclarationCustody } from "../computed_event_custody.ts";
import { capture, completed } from "../constructor_original27_public/capture.ts";
import { assertInputs, type Stage } from "./fixtures.ts";
import { cliObservation, assertCompilerVectors } from "./oracle.ts";
import { slotProviderTypes, assertTypeIdentity } from "./types.ts";
import { nativeJournal } from "./custody.ts";
import type { install } from "./install.ts";

type Installed = Awaited<ReturnType<typeof install>>;
export function writeProject(project: string, stage: Stage, modules: string) {
  mkdirSync(project, { recursive: true });
  for (const [name, bytes] of stage.files) {
    mkdirSync(dirname(join(project, name)), { recursive: true });
    writeFileSync(join(project, name), bytes, { flag: "wx" });
  }
  const config = asRecord(parseJson(stage.files.get("tsconfig.json")!));
  assert.equal(config.contentMappers, undefined);
  const mapper = { ...config, contentMappers: [{ package: "vize", extensions: [".vue"] }] };
  writeFileSync(join(project, "tsconfig.mapper.json"), JSON.stringify(mapper, null, 2) + "\n", {
    flag: "wx",
  });
  // Only link the actual public installation. Never manufacture a Vize package or rewrite its exec.
  symlinkSync(modules, join(project, "node_modules"), "dir");
}

export function observe(
  record: Recorder,
  project: string,
  stage: Stage,
  installed: Installed,
  helperText: string,
) {
  const label = stage.population + "-" + stage.id.replaceAll("/", "-");
  const inputs = assertInputs(project, stage.files);
  record.json(label + ".inputs.json", { inputs, repairCopy: stage.repairCopy });
  const identity = { binary: installed.native };
  const packetJournal = observed(installed.root, identity, record, label + "-protocol");
  const config = parseJson(readFileSync(join(project, "tsconfig.mapper.json"), "utf8"));
  const source = stage.files.get("src/Tiny.vue")!;
  const input = slotRequests(project, source, config);
  const direct = capture(
    record,
    label + "-protocol",
    [process.execPath, "--require", hook, installed.cli, "content-mapper"],
    project,
    packetJournal.env,
    input,
  );
  completed(direct, 0);
  const packet = slotResponses(direct.stdout);
  record.json(label + ".responses.json", packet.responses);
  const directIdentity = nativeJournal(packetJournal.journal, installed, direct);
  const vts = asString(packet.result.text);
  writeFileSync(join(record.root, label + ".virtual.ts"), vts, { flag: "wx" });
  const ts: typeof tsApi = createRequire(installed.cli)(installed.typescriptModule);
  assert.equal(ts.version, "6.0.3");
  const compilerOptions = asRecord(asRecord(config).compilerOptions);
  const typeIdentity = slotProviderTypes(ts, join(project, "src/Tiny.vue.ts"), vts, {
    roots: [...stage.files.keys()]
      .filter((file) => file.endsWith(".ts"))
      .map((file) => join(project, file)),
    providerNames: stage.providerNames,
    noUnusedLocals: compilerOptions.noUnusedLocals === true,
    noUnusedParameters: compilerOptions.noUnusedParameters === true,
  });
  record.json(label + ".type-identity.json", typeIdentity);
  const mapper = capture(
    record,
    label + "-mapper",
    [
      installed.checker,
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
  );
  const journal = observed(installed.root, identity, record, label + "-cli");
  const cli = capture(
    record,
    label + "-cli",
    [
      process.execPath,
      "--require",
      hook,
      installed.cli,
      "check",
      "--no-config",
      "--format",
      "json",
      "--show-virtual-ts",
      "--quiet",
      "--tsconfig",
      "tsconfig.json",
      "--corsa-path",
      installed.checker,
    ],
    project,
    journal.env,
  );
  const framework = capture(
    record,
    label + "-framework",
    [
      process.execPath,
      installed.official,
      "--noEmit",
      "--pretty",
      "false",
      "--project",
      "tsconfig.json",
    ],
    project,
  );
  // Every full stream, protocol, input and type observation is durable before comparison.
  record.json(label + ".captured.json", {
    population: stage.population,
    id: stage.id,
    inputs,
    typeIdentity,
    direct: { ...directIdentity, requests: 4, responses: packet.responses },
    mapper: { status: mapper.status, raw: mapper.raw },
    cli: { status: cli.status, raw: cli.raw },
    framework: { status: framework.status, raw: framework.raw },
  });
  const failed = stage.expected.length > 0;
  completed(mapper, failed ? 2 : 0);
  completed(cli, failed ? 1 : 0, false);
  completed(framework, failed ? 2 : 0);
  const loaded = nativeJournal(journal.journal, installed, cli);
  const displayed = cliObservation(cli.stdout, cli.stderr, cli.status, project, stage, helperText);
  const vectors = assertCompilerVectors(stage, mapper.stdout, framework.stdout, project);
  assertTypeIdentity(stage, typeIdentity);
  assert.deepEqual(assertInputs(project, stage.files), inputs);
  assertCustody(typeIdentity.physicalInputs);
  assertCustody(installed.images);
  assertDeclarationCustody(installed.cliRoot, installed.graph.files, installed.graph.edges);
  const accepted = {
    population: stage.population,
    id: stage.id,
    repairCopy: stage.repairCopy,
    inputs,
    vectors,
    cli: displayed,
    typeIdentity,
    native: loaded,
    protocol: { ...directIdentity, completeRequests: 4, completeResponses: 4 },
    timingUsable: false,
  };
  record.json(label + ".accepted.json", accepted);
  return accepted;
}
