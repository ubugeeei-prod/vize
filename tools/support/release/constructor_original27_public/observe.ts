import assert from "node:assert/strict";
import { mkdirSync, readFileSync, symlinkSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { asArray, asRecord, parseJson } from "../../json-value.ts";
import { Recorder } from "../../config/windows_consumer/common.ts";
import {
  slotRequests,
  slotResponses,
} from "../../../benchmarks/scripts/canon-slot-constructor-protocol.ts";
import { hook, observed, verifyJournal } from "../editor_vite_stub/custody.ts";
import { assertCustody, assertDeclarationCustody } from "../computed_event_custody.ts";
import { assertInputs, type Inputs, type OriginalRow } from "./fixtures.ts";
import { capture, completed } from "./capture.ts";
import { assertOriginalVectors, cliPacket, diagnostics, reference } from "./oracle.ts";
import type { install } from "./install.ts";

type Installed = Awaited<ReturnType<typeof install>>;
export function writeProject(project: string, files: Map<string, string>, modules: string) {
  mkdirSync(project);
  for (const [name, bytes] of files) {
    const output = join(project, name);
    mkdirSync(join(output, ".."), { recursive: true });
    writeFileSync(output, bytes, { flag: "wx" });
  }
  symlinkSync(modules, join(project, "node_modules"), "dir");
}
function identity(installed: Installed) {
  return { binary: installed.native };
}
export function protocol(
  record: Recorder,
  label: string,
  project: string,
  row: OriginalRow,
  installed: Installed,
) {
  const config = parseJson(readFileSync(join(project, "tsconfig.mapper.json"), "utf8"));
  const input = slotRequests(project, row.source, config, row.id + "/src/Tiny.vue");
  const journal = observed(installed.root, identity(installed), record, label);
  const raw = capture(
    record,
    label,
    [process.execPath, "--require", hook, installed.cli, "content-mapper"],
    project,
    journal.env,
    input,
  );
  completed(raw, 0);
  verifyJournal(journal.journal, identity(installed), raw.status);
  const responses = slotResponses(raw.stdout);
  record.json(label + ".responses.json", responses.responses);
  return {
    label,
    raw: raw.raw,
    status: raw.status,
    responses: responses.responses,
  };
}
export function observe(
  record: Recorder,
  label: string,
  project: string,
  files: Map<string, string>,
  rows: OriginalRow[],
  installed: Installed,
  version: string,
  repaired = false,
) {
  const inputs = assertInputs(project, files);
  record.json(label + ".inputs.json", inputs);
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
  const journal = observed(installed.root, identity(installed), record, label + "-cli");
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
      "--tsconfig",
      "tsconfig.json",
      "--corsa-path",
      installed.checker,
    ],
    project,
    journal.env,
  );
  const packets = rows.map((row) =>
    protocol(record, label + "-protocol-" + row.id, project, row, installed),
  );
  const mapped = diagnostics(mapper.stdout, project);
  const displayed = cliPacket(cli.stdout, cli.status, project, referenceIds());
  record.json(label + ".complete.json", {
    inputs,
    mapper: { status: mapper.status, diagnostics: mapped },
    cli: { status: cli.status, packet: displayed },
    protocols: packets,
  });
  completed(mapper, 2); // The two generated TS2741 debts remain even in the repair control.
  completed(cli, 1);
  verifyJournal(journal.journal, identity(installed), cli.status);
  assertOriginalVectors(version, mapped, displayed.diagnostics, repaired);
  assert.deepEqual(assertInputs(project, files), inputs);
  assertCustody(installed.images);
  assertDeclarationCustody(installed.cliRoot, installed.graph.files, installed.graph.edges);
  return {
    inputs,
    mapperDiagnostics: mapped,
    cliDiagnostics: displayed.diagnostics,
    protocols: packets.length,
    generatedMapperDebt: mapped.filter((row) => row.code === 2741),
  };
}
function referenceIds() {
  return asArray(reference().caseIds).map((id) => {
    assert.equal(typeof id, "string");
    return id as string;
  });
}
export function independentOracles(record: Recorder, project: string, installed: Installed) {
  const h = capture(
    record,
    "stock-h",
    [process.execPath, installed.stock, "--pretty", "false", "--project", "tsconfig.h.json"],
    project,
  );
  const official = capture(
    record,
    "official-vue-tsc",
    [process.execPath, installed.official, "--pretty", "false", "--project", "tsconfig.json"],
    project,
  );
  const stock = {
    status: h.status,
    diagnostics: diagnostics(h.stdout, project),
  };
  const vue = {
    status: official.status,
    diagnostics: diagnostics(official.stdout, project),
  };
  record.json("independent-oracles.json", { stock, official: vue });
  completed(h, 2);
  completed(official, 2);
  assert.deepEqual(stock, asRecord(reference().stockH));
  assert.deepEqual(vue, asRecord(reference().official));
}
/** Separate copied population; the frozen original project is never edited. */
export function repairCopy(
  record: Recorder,
  output: string,
  original: Inputs,
  installed: Installed,
  version: string,
) {
  const project = join(output, "repair-copy");
  writeProject(project, original.files, installed.modules);
  const bad = "wrong-prop/src/Tiny.vue",
    before = original.files.get(bad)!;
  const repaired = before.replace(":value=\"'bad'\"", ':value="1"');
  assert.notEqual(repaired, before);
  const files = new Map(original.files),
    states: unknown[] = [];
  for (const [label, content, good] of [
    ["bad", before, false],
    ["number", repaired, true],
    ["restored", before, false],
  ] as const) {
    files.set(bad, content);
    writeFileSync(join(project, bad), content);
    const row = {
      ...original.rows.find((value) => value.id === "wrong-prop")!,
      source: content,
    };
    states.push({
      label,
      ...observe(record, "repair-" + label, project, files, [row], installed, version, good),
    });
  }
  assert.deepEqual(assertInputs(project, original.files), original.manifest);
  record.json("repair-copy.json", {
    separatePopulation: true,
    originalCases: 27,
    states,
  });
  return states;
}
