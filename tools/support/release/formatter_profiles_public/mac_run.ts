import assert from "node:assert/strict";
import {
  existsSync,
  lstatSync,
  mkdirSync,
  readFileSync,
  realpathSync,
  statfsSync,
  writeFileSync,
} from "node:fs";
import { join, resolve } from "node:path";
import { release } from "node:os";
import { inspect } from "node:util";
import { Recorder, success, type Packet } from "../../config/windows_consumer/common.ts";
import { hook, observed, verifyJournal } from "../editor_vite_stub/custody.ts";
import { inside, inventory, type Entry } from "../editor_vite_stub/filesystem.ts";
import { assertEnvironment } from "./run.ts";
import { assertMacHost, macOriginal, macRole, validateDispatch } from "./mac_contract.ts";
import { authenticateMac } from "./mac_authority.ts";
import {
  assertMacInputs,
  installMac,
  macRootSeal,
  resetMacInput,
  type MacContext,
} from "./mac_install.ts";
import { assertFullFalse, macReference } from "./mac_reference.ts";
import { asString } from "../../json-value.ts";

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

/** The execution carrier is private; persist only the explicit journal locator. */
export function macObservation(input: {
  label: string;
  args: string[];
  packet: Packet;
  before: Entry[];
  after: Entry[];
  outputRawBase64: string | null;
  custody: { journal: string };
  writable: string[];
}) {
  return {
    label: input.label,
    args: input.args,
    packet: input.packet,
    before: input.before,
    after: input.after,
    outputRawBase64: input.outputRawBase64,
    custody: { journal: input.custody.journal },
    writable: input.writable,
  };
}

/** Capture every packet and whole output; neither version is judged here. */
export function observeMac(context: MacContext) {
  const { root, record, identity, cli } = context,
    original = macOriginal();
  assert.deepEqual(macRootSeal(root), context.rootSeal);
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
    assertMacInputs(context.rootSeal);
    const before = inventory(root);
    record.json(label + ".before.json", before);
    const custody = observed(root, identity, record, label);
    const packet = record.run(
      label,
      [process.execPath, "--require", hook, cli, ...args],
      root,
      custody.env,
    );
    assert.deepEqual(macRootSeal(root), context.rootSeal, "command replaced its owned root");
    const path = join(root, "a.vue");
    const output =
      existsSync(path) && lstatSync(path).isFile() && !lstatSync(path).isSymbolicLink()
        ? readFileSync(path).toString("base64")
        : null;
    const after = inventory(root);
    const observation = macObservation({
      label,
      args,
      packet,
      before,
      after,
      outputRawBase64: output,
      custody,
      writable,
    });
    record.json(label + ".observation.json", observation);
    return observation;
  }
  for (const command of commands) rows.push(capture(command.label, command.args, command.writable));
  const outcomes: { name: string; rawBase64: string | null }[] = [];
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
    resetMacInput(context.rootSeal, original.source);
    for (const [step, argv] of [
      ["original", args],
      ["rewrite", args],
      ["check", args.map((arg) => (arg === "--write" ? "--check" : arg))],
    ] as const)
      rows.push(capture(name + "-" + step, [...argv], step === "check" ? [] : ["a.vue"]));
    outcomes.push({ name, rawBase64: rows.at(-1)?.outputRawBase64 ?? null });
  }
  record.json("whole-observations.json", { rows, outcomes });
  return { rows, outcomes };
}

export type MacObservation = ReturnType<typeof observeMac>;

function observedFailure(error: unknown) {
  return {
    name: error instanceof Error ? error.name : null,
    message: error instanceof Error ? error.message : String(error),
    rawInspection: inspect(error, {
      showHidden: true,
      depth: null,
      getters: false,
      customInspect: false,
      maxStringLength: null,
      maxArrayLength: null,
    }),
  };
}

/** Called only after both versions' complete observations have been retained. */
export function validateMac(context: MacContext, observation: MacObservation, expected: Buffer) {
  const { identity, record } = context,
    { rows, outcomes } = observation;
  for (const row of rows) {
    success(row.packet);
    assert.equal(
      typeof row.outputRawBase64,
      "string",
      "selected output must remain a regular owned file",
    );
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
  const equals = asString(outcomes[0]?.rawBase64),
    spaced = asString(outcomes[1]?.rawBase64),
    enabled = asString(outcomes[2]?.rawBase64);
  assert.notEqual(equals, Buffer.from(macOriginal().source).toString("base64"));
  assert.notEqual(equals, enabled, "explicit true must be an effective negative control");
  const matched = equals === spaced;
  const reference = expected.toString("base64");
  record.json("boolean-comparison.json", {
    matched,
    outcomes,
    historicalFailureRequired: false,
    equalsMatchesFullReference: equals === reference,
    spacedMatchesFullReference: spaced === reference,
  });
  if (context.version === "0.628.0") {
    assert.equal(matched, true, "whole joint false outputs must agree in containing release");
    assertFullFalse(Buffer.from(equals, "base64"), expected);
    assertFullFalse(Buffer.from(spaced, "base64"), expected);
  }
  return { version: context.version, matched, outcomes, commands: rows.length };
}

/** A baseline verdict cannot prevent the containing release from being evaluated. */
export function macVerdicts(phases: readonly { version: string; evaluate: () => unknown }[]) {
  assert.equal(phases.length, 2);
  return phases.map((phase) => {
    try {
      return { version: phase.version, state: "passed", result: phase.evaluate() };
    } catch (error) {
      return {
        version: phase.version,
        state: "failed",
        error:
          error instanceof Error
            ? { name: error.name, message: error.message, stack: error.stack }
            : String(error),
      };
    }
  });
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
  const reference = macReference();
  record.json("complete-source-reference.json", reference.value);
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
  const phases: {
    context: MacContext;
    observation: MacObservation | null;
    collectionError: unknown;
  }[] = [];
  try {
    for (const [version, producer, blob] of [
      ["0.627.0", process.env.BEFORE_PRODUCER_RUN, process.env.BEFORE_AUTHORITY_BLOB],
      ["0.628.0", process.env.PRODUCER_RUN, process.env.AUTHORITY_BLOB],
    ] as const) {
      assert(producer && blob);
      const phase = new Recorder(join(output, version));
      const authority = await authenticateMac(version, producer, blob, phase);
      const context = installMac(join(phase.root, "consumer"), authority, phase);
      let observation: MacObservation | null = null,
        collectionError: unknown = null;
      try {
        observation = observeMac(context);
      } catch (error) {
        collectionError = error;
        // The complete already-recorded commands/observations remain in this
        // phase directory. Never reuse an unsafe input to finish a command list.
        phase.json("collection-failure.json", {
          state: "failed",
          error: observedFailure(error),
          packetsCompleted: phase.packets,
          expectedFormatterCommands: 11,
          scope: "partial collection; no missing command or filesystem change is accepted",
        });
      }
      phases.push({ context, observation, collectionError });
      results.push({
        version,
        state: observation ? "captured-before-verdict" : "collection-failed",
        candidate: authority.candidate,
        producerSource: authority.ownership.producerSource,
        producer,
        blob,
      });
      record.json("phase-results.json", results);
    }
    // Authority and provider trust checks precede execution. Formatting verdicts
    // follow collection of BOTH releases, including an unexpected old result.
    const verdicts = macVerdicts(
      phases.map(({ context, observation, collectionError }) => ({
        version: context.version,
        evaluate: () => {
          if (!observation) throw collectionError;
          return validateMac(context, observation, reference.output);
        },
      })),
    );
    record.json("verdicts.json", verdicts);
    assert(
      verdicts.every((verdict) => verdict.state === "passed"),
      "both completed phase verdicts are retained; inspect verdicts.json",
    );
    record.json("qualification.json", {
      state: "passed",
      role: macRole,
      results,
      verdicts,
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
