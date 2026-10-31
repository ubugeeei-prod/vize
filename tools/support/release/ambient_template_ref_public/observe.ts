import assert from "node:assert/strict";
import {
  copyFileSync,
  existsSync,
  mkdirSync,
  readFileSync,
  symlinkSync,
  unlinkSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { asArray, asNumber, asRecord, parseJson } from "../../json-value.ts";
import { Recorder, sha256 } from "../../config/windows_consumer/common.ts";
import { assertCustody, assertDeclarationCustody } from "../computed_event_custody.ts";
import { hook, observed, verifyJournal } from "../editor_vite_stub/custody.ts";
import { capture, completed } from "../constructor_original27_public/capture.ts";
import type { install } from "./install.ts";

const fixtureRoot = fileURLToPath(
  new URL("../../../../tests/fixtures/release/ambient-template-ref-public-10980/", import.meta.url),
);
export const phases = ["original", "typed-repair", "restored", "missing-global"] as const;
export type Phase = (typeof phases)[number];
export function loadFixture() {
  const read = (name: string) => readFileSync(join(fixtureRoot, name), "utf8");
  const body = read("issue-body.txt"),
    pins = asRecord(parseJson(read("primary-capture.json")));
  const block = (language: string) => {
    const matches = [
      ...body.matchAll(new RegExp("```" + language + "\\n([\\s\\S]*?)\\n```", "gu")),
    ];
    assert.equal(matches.length, 1);
    return matches[0][1] + "\n";
  };
  const probe = read("Probe.vue.txt"),
    global = read("global.d.ts.txt"),
    config = read("tsconfig.json.txt");
  assert.equal(pins.issue, 10980);
  assert.equal(sha256(body), pins.bodySha256);
  assert.equal(probe, block("vue"));
  assert.equal(global, block("ts"));
  assert.equal(sha256(probe), pins.probeSha256);
  assert.equal(sha256(global), pins.globalSha256);
  assert.deepEqual(JSON.parse(config), {
    compilerOptions: {
      noEmit: true,
      strict: true,
      skipLibCheck: true,
      module: "ESNext",
      moduleResolution: "Bundler",
      lib: ["ES2024", "DOM"],
      types: [],
    },
    include: ["*.vue", "*.ts"],
    vueCompilerOptions: { composables: { useTemplateRef: [] } },
  });
  assert.equal(probe.split("useTemplateRef('target')").length, 2);
  return {
    probe,
    global,
    config,
    pins,
    repaired: probe.replace(
      "useTemplateRef('target')",
      "useTemplateRef<HTMLButtonElement>('target')",
    ),
  };
}
export function expectedReport(phase: Phase) {
  const missing = phase === "missing-global",
    clean = phase === "typed-repair";
  const names = missing ? ["Probe.vue"] : ["Probe.vue", "global.d.ts"];
  const diagnostics = clean
    ? []
    : [
        missing
          ? "error:2:16 [TS2304] Cannot find name 'useTemplateRef'."
          : "error:3:32 [TS2339] Property 'focus' does not exist on type '{}'.",
      ];
  return {
    files: names.map((file) => ({ file, diagnostics: file === "Probe.vue" ? diagnostics : [] })),
    programs: [
      {
        root: ".",
        tsconfig: "tsconfig.json",
        compilerOptions: {
          lib: ["ES2024", "DOM"],
          module: "ESNext",
          moduleResolution: "Bundler",
          noEmit: true,
          skipLibCheck: true,
          strict: true,
          types: [],
        },
        files: names,
      },
    ],
    errorCount: clean ? 0 : 1,
    warningCount: 0,
    fileCount: names.length,
  };
}
export function assertCliOutput(
  phase: Phase,
  stdout: Buffer,
  stderr: Buffer,
  status: number | null,
  project: string,
) {
  const expected = expectedReport(phase),
    json = Buffer.from(JSON.stringify(expected, null, 2) + "\n");
  assert.equal(status, phase === "typed-repair" ? 0 : 1);
  assert.deepEqual(
    stdout,
    json,
    "whole ordered JSON bytes, including every diagnostic and option, must match",
  );
  assert.deepEqual(
    stderr,
    Buffer.from(
      `Building Corsa virtual project for ${expected.fileCount} files under ${project}...\nRunning Corsa diagnostics for ${expected.fileCount} files...\n`,
    ),
  );
  return expected;
}
type Child = Pick<ReturnType<typeof capture>, "argv" | "pid" | "status" | "signal" | "error">;
export function assertJournal(
  raw: Buffer,
  identity: {
    root: string;
    cli: string;
    binary: { path: string; sha256: string };
    checker: string;
  },
  child: Child,
) {
  assert.equal(child.error, null);
  assert.equal(child.signal, null);
  assert.ok(Number.isSafeInteger(child.pid) && child.pid > 0);
  assert.ok(child.status === 0 || child.status === 1);
  assert.ok(raw.length > 0 && raw.at(-1) === 10);
  const events = raw
    .toString("utf8")
    .slice(0, -1)
    .split("\n")
    .map((line) => asRecord(parseJson(line)));
  assert.deepEqual(
    events.map((row) => row.event),
    child.status === 0
      ? ["initialized", "attempt", "returned", "exit"]
      : ["initialized", "attempt", "returned"],
  );
  for (const event of events) {
    assert.equal(event.schema, "vize-public-native-custody-event-v1");
    assert.equal(asNumber(event.pid), child.pid);
    const keys =
      event.event === "initialized"
        ? ["schema", "pid", "event", "installRoot", "nativePath", "sha256", "node", "argv"]
        : event.event === "exit"
          ? ["schema", "pid", "event", "code"]
          : ["schema", "pid", "event", "actualPath", "sha256", "expectedNative", "corsaPath"];
    assert.deepEqual(
      Object.keys(event),
      keys,
      "complete native journal field inventory must match",
    );
  }
  assert.equal(events[0].installRoot, identity.root);
  assert.equal(events[0].nativePath, identity.binary.path);
  assert.equal(events[0].sha256, identity.binary.sha256);
  assert.equal(events[0].node, child.argv[0]);
  assert.deepEqual(asArray(events[0].argv), [child.argv[0], ...child.argv.slice(3)]);
  assert.deepEqual(child.argv.slice(1, 4), ["--require", hook, identity.cli]);
  assert.deepEqual(child.argv.slice(4), [
    "check",
    "--no-config",
    "--tsconfig",
    "tsconfig.json",
    "--format",
    "json",
    "--corsa-path",
    identity.checker,
  ]);
  for (const event of events.slice(1, 3)) {
    assert.equal(event.actualPath, identity.binary.path);
    assert.equal(event.sha256, identity.binary.sha256);
    assert.equal(event.expectedNative, true);
    assert.equal(event.corsaPath, "");
  }
  if (child.status === 0) assert.equal(events[3].code, 0);
}
export function observe(
  installed: Awaited<ReturnType<typeof install>>,
  out: string,
  record: Recorder,
) {
  const fixture = loadFixture(),
    project = join(out, "project");
  mkdirSync(project);
  symlinkSync(installed.modules, join(project, "node_modules"), "dir");
  writeFileSync(join(project, "tsconfig.json"), fixture.config, { flag: "wx" });
  writeFileSync(join(project, "global.d.ts"), fixture.global, { flag: "wx" });
  const observations = [];
  for (const phase of phases) {
    const directory = join(out, phase);
    mkdirSync(directory);
    const source = phase === "typed-repair" ? fixture.repaired : fixture.probe;
    writeFileSync(join(project, "Probe.vue"), source);
    if (phase === "missing-global") unlinkSync(join(project, "global.d.ts"));
    const names =
      phase === "missing-global"
        ? ["Probe.vue", "tsconfig.json"]
        : ["Probe.vue", "global.d.ts", "tsconfig.json"];
    const inputs = names.map((name) => {
      const bytes = readFileSync(join(project, name));
      copyFileSync(join(project, name), join(directory, name));
      return {
        file: name,
        bytes: bytes.length,
        sha256: sha256(bytes),
        wholeUtf8: bytes.toString("utf8"),
      };
    });
    record.json(phase + ".inputs.json", {
      files: inputs,
      absent: phase === "missing-global" ? ["global.d.ts"] : [],
    });
    const journal = observed(installed.root, { binary: installed.native }, record, phase);
    const child = capture(
      record,
      phase,
      [
        process.execPath,
        "--require",
        hook,
        installed.cli,
        "check",
        "--no-config",
        "--tsconfig",
        "tsconfig.json",
        "--format",
        "json",
        "--corsa-path",
        installed.checker,
      ],
      project,
      journal.env,
    );
    completed(child, phase === "typed-repair" ? 0 : 1, false);
    const result = assertCliOutput(phase, child.stdout, child.stderr, child.status, project);
    verifyJournal(journal.journal, { binary: installed.native }, child.status === 0 ? 0 : null);
    assertJournal(
      readFileSync(journal.journal),
      {
        root: installed.root,
        cli: installed.cli,
        binary: installed.native,
        checker: installed.checker,
      },
      child,
    );
    for (const input of inputs)
      assert.equal(sha256(readFileSync(join(project, input.file))), input.sha256);
    assert.equal(existsSync(join(project, "global.d.ts")), phase !== "missing-global");
    assertCustody(installed.images);
    assertDeclarationCustody(installed.cliRoot, installed.graph.files, installed.graph.edges);
    observations.push({
      phase,
      result,
      pid: child.pid,
      status: child.status,
      raw: child.raw,
      journal: journal.journal,
    });
    record.json("four-phase-observations.json", observations);
  }
  assert.deepEqual(
    observations.map((row) => row.phase),
    [...phases],
  );
  assert.deepEqual(observations[0].result, observations[2].result);
  return observations;
}
