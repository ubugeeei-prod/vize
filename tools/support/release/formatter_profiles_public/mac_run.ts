import assert from "node:assert/strict";
import { mkdirSync, readFileSync, realpathSync, statfsSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { release } from "node:os";
import { Recorder, success } from "../../config/windows_consumer/common.ts";
import { hook, observed, verifyJournal } from "../editor_vite_stub/custody.ts";
import { inside, inventory, type Entry } from "../editor_vite_stub/filesystem.ts";
import { assertEnvironment } from "./run.ts";
import { assertMacHost, macOriginal, macRole, validateDispatch } from "./mac_contract.ts";
import { authenticateMac } from "./mac_authority.ts";
import { installMac, type MacContext } from "./mac_install.ts";

export function assertMacFiles(before: Entry[], after: Entry[], writable: string[]): void {
  const keep = (rows: Entry[]) => rows.filter((row) => !writable.includes(row.path));
  assert.deepEqual(
    keep(after),
    keep(before),
    "whole project/unselected files/modules/links must remain exact",
  );
  for (const path of writable) {
    assert.equal(before.find((row) => row.path === path)?.kind, "file");
    assert.equal(after.find((row) => row.path === path)?.kind, "file");
    assert.equal(
      after.find((row) => row.path === path)?.mode,
      before.find((row) => row.path === path)?.mode,
    );
  }
}

export function macJournalExit(label: string): 0 | null {
  // Both clap DisplayHelp and DisplayVersion exit inside Rust, bypassing Node's exit event.
  return label === "original-version" || label === "original-help" ? null : 0;
}

/** Capture every packet and whole output before evaluating any formatter result. */
export function observeMac(context: MacContext) {
  const { root, record, identity, cli } = context,
    original = macOriginal();
  writeFileSync(join(root, "a.vue"), original.source, { flag: "wx" });
  writeFileSync(join(root, "Unselected.vue"), original.source, { flag: "wx" });
  writeFileSync(join(root, "vite.config.mjs"), "export default { unrelated: 'unchanged' };\n", {
    flag: "wx",
  });
  const commands: { label: string; args: string[]; writable: string[] }[] = [
    { label: "original-version", args: ["--version"], writable: [] },
    { label: "original-help", args: ["fmt", "--help"], writable: [] },
  ];
  const rows: ReturnType<typeof capture>[] = [];
  function capture(label: string, args: string[], writable: string[]) {
    const before = inventory(root);
    record.json(label + ".before.json", before);
    const custody = observed(root, identity, record, label);
    const packet = record.run(
      label,
      [process.execPath, "--require", hook, cli, ...args],
      root,
      custody.env,
    );
    const output = readFileSync(join(root, "a.vue"));
    const after = inventory(root);
    const observation = {
      label,
      args,
      packet,
      before,
      after,
      outputRawBase64: output.toString("base64"),
      custody,
      writable,
    };
    record.json(label + ".observation.json", observation);
    return observation;
  }
  for (const command of commands) rows.push(capture(command.label, command.args, command.writable));
  const outcomes: { name: string; rawBase64: string }[] = [];
  for (const [name, args] of [
    ["equals", original.equals],
    ["spaced", original.spaced],
    [
      "true-control",
      [
        "fmt",
        "a.vue",
        "--no-config",
        "--write",
        "--semi=true",
        "--bracket-spacing=true",
        "--sort-attributes=true",
      ],
    ],
  ] as const) {
    writeFileSync(join(root, "a.vue"), original.source);
    for (const [step, argv] of [
      ["original", args],
      ["rewrite", args],
      ["check", args.map((arg) => (arg === "--write" ? "--check" : arg))],
    ] as const)
      rows.push(capture(name + "-" + step, [...argv], step === "check" ? [] : ["a.vue"]));
    outcomes.push({ name, rawBase64: readFileSync(join(root, "a.vue")).toString("base64") });
  }
  record.json("whole-observations.json", { rows, outcomes });
  // Every command (including both original forms) has now completed and been retained.
  for (const row of rows) {
    success(row.packet);
    assertMacFiles(row.before, row.after, row.writable);
    verifyJournal(row.custody.journal, identity, macJournalExit(row.label));
  }
  assert.equal(rows[0]?.packet.stdout, "vize " + context.version + "\n");
  assert.equal(rows[0]?.packet.stderr, "");
  const help = rows[1]?.packet.stdout ?? "";
  for (const flag of ["--semi", "--bracket-spacing", "--sort-attributes"])
    assert(help.includes(flag));
  assert(help.includes("true") && help.includes("false"));
  for (const name of ["equals", "spaced", "true-control"]) {
    const own = rows.filter((row) => row.label.startsWith(name + "-"));
    assert.equal(own.length, 3);
    assert.equal(
      own[1]?.outputRawBase64,
      own[0]?.outputRawBase64,
      "whole rewrite fixed point: " + name,
    );
    assert.equal(
      own[2]?.outputRawBase64,
      own[0]?.outputRawBase64,
      "whole check fixed point: " + name,
    );
  }
  const equals = outcomes[0]!,
    spaced = outcomes[1]!,
    enabled = outcomes[2]!;
  assert.notEqual(equals.rawBase64, Buffer.from(original.source).toString("base64"));
  assert.notEqual(
    equals.rawBase64,
    enabled.rawBase64,
    "explicit true must be an effective negative control",
  );
  const matched = equals.rawBase64 === spaced.rawBase64;
  record.json("boolean-comparison.json", { matched, outcomes, historicalFailureRequired: false });
  if (context.version === "0.628.0")
    assert.equal(matched, true, "whole joint false outputs must agree in containing release");
  return { version: context.version, matched, outcomes, commands: rows.length };
}

export async function runMac(): Promise<void> {
  // Selection/required tuple validation precedes authority access, and cannot pass by skipping both jobs.
  validateDispatch(
    process.env.GITHUB_EVENT_NAME ?? "",
    process.env.GITHUB_REF ?? "",
    process.env.PUBLIC_ROLE ?? "",
    {
      version: process.env.PUBLIC_VERSION,
      producer: process.env.PRODUCER_RUN,
      authority: process.env.AUTHORITY_BLOB,
      beforeProducer: process.env.BEFORE_PRODUCER_RUN,
      beforeAuthority: process.env.BEFORE_AUTHORITY_BLOB,
    },
  );
  assert.equal(process.env.PUBLIC_ROLE, macRole);
  assertEnvironment(process.env);
  assertMacHost(process.platform, process.arch, process.version);
  const arg = process.argv[2];
  assert(arg);
  const output = resolve(arg),
    cwd = realpathSync(process.cwd());
  assert(!inside(cwd, output) && output !== cwd);
  mkdirSync(output, { recursive: false });
  const disk = statfsSync(output);
  assert(
    disk.bavail * disk.bsize >= 5 * 1024 ** 3,
    "one GiB floor plus four GiB two-install forecast",
  );
  const record = new Recorder(realpathSync(output));
  record.json("controller.json", {
    controllerSource: process.env.GITHUB_SHA,
    run: process.env.GITHUB_RUN_ID,
    attempt: process.env.GITHUB_RUN_ATTEMPT,
    role: macRole,
    original: macOriginal().value,
    node: process.version,
    platform: process.platform,
    arch: process.arch,
    actualDarwinRelease: release(),
    reporterOS: "macOS 26.6.2",
    exactReporterOSRecreated: false,
    adaptations: [
      "instrumented direct installed CLI invocation preserves original argv; original lifecycle policy enabled",
      "controlled unselected/config fixtures added for strict filesystem guards",
    ],
  });
  const results: unknown[] = [];
  try {
    for (const [version, producer, blob] of [
      ["0.627.0", process.env.BEFORE_PRODUCER_RUN, process.env.BEFORE_AUTHORITY_BLOB],
      ["0.628.0", process.env.PRODUCER_RUN, process.env.AUTHORITY_BLOB],
    ] as const) {
      assert(producer && blob);
      const phase = new Recorder(join(output, version));
      const authority = await authenticateMac(version, producer, blob, phase);
      const context = installMac(join(phase.root, "consumer"), authority, phase);
      const result = observeMac(context);
      results.push({
        ...result,
        candidate: authority.candidate,
        producerSource: authority.ownership.producerSource,
        producer,
        blob,
      });
      record.json("phase-results.json", results);
    }
    record.json("qualification.json", {
      state: "passed",
      role: macRole,
      results,
      scope:
        "one original11264 SFC, joint three equals/spaced false forms and true/fixed-point controls; no broad formatter/native-adapter/6098 closure",
      limitation:
        "genuine BEFORE may already agree; record observation without inventing a historical failure; reporter exact OS not reproduced",
    });
  } catch (error) {
    record.json("qualification.json", {
      state: "failed",
      role: macRole,
      results,
      error:
        error instanceof Error
          ? { name: error.name, message: error.message, stack: error.stack }
          : String(error),
      scope: "retained partial observations are not public acceptance",
    });
    throw error;
  }
}
if (import.meta.main) await runMac();
