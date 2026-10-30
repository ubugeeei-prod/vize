import assert from "node:assert/strict";
import { test } from "node:test";
import {
  expectedHost,
  compilerRows,
  readGlobalsFixture,
  type Packet,
} from "./support/content-mapper-instance-globals-10617.ts";
import { assertMapperCase } from "./support/content-mapper-instance-globals-hosts-10617.ts";
import {
  assertReceiverNativeCase,
  hasReceiverNativeExpectation,
} from "./support/content-mapper-instance-globals-receiver-native-10617.ts";
import type { ReceiverProgram } from "./support/content-mapper-instance-globals-receiver-symbols-10617.ts";

const { files } = readGlobalsFixture();
const programs = JSON.parse(files.get("receiver-plan-controls.json")!)
  .programs as ReceiverProgram[];
const independent = JSON.parse(files.get("receiver-plan-independent-packets.json")!) as Array<
  Pick<Packet, "status" | "signal" | "stdout" | "stderr"> & { id: string; host: string }
>;
const compilerOptions = JSON.parse(files.get("project/tsconfig.json.txt")!).compilerOptions;
// Literal native presentation packets, distinct from the frozen TS2552 oracle.
// All three actual hosts agreed in capture 41101572572/artifact 13001175219.
const diagnostics = {
  "unresolved-value": [
    "error:2:32 [TS2551] Property '$ghost' does not exist on type 'ComponentPublicInstance<{}, {}, {}, {}, {}, {}, {}, {}, false, ComponentOptionsBase<any, any, any, any, any, any, any, any, any, {}, {}, string, {}, {}, {}, string, ComponentProvideOptions>, ... 4 more ..., any>'. Did you mean '$host'? (source: <div v-for=\"__ctx in [1]\">{{ $ghost.value }} {{ __vize_template_instance }}</div>)",
    "error:2:51 [TS2551] Property '__vize_template_instance' does not exist on the component instance. Did you mean '__vize_template_instance_'? (source: <div v-for=\"__ctx in [1]\">{{ $ghost.value }} {{ __vize_template_instance }}</div>)",
  ],
  "unresolved-type-query": [
    "error:5:46 [TS2551] Property '__vize_template_instance' does not exist on the component instance. Did you mean '__vize_template_instance_'? (source: <div>{{ ((__ctx: any) => { type T = typeof __vize_template_instance; return $ghost.value; })(0) }}</div>)",
    "error:5:79 [TS2551] Property '$ghost' does not exist on type 'ComponentPublicInstance<{}, {}, {}, {}, {}, {}, {}, {}, false, ComponentOptionsBase<any, any, any, any, any, any, any, any, any, {}, {}, string, {}, {}, {}, string, ComponentProvideOptions>, ... 4 more ..., any>'. Did you mean '$host'? (source: <div>{{ ((__ctx: any) => { type T = typeof __vize_template_instance; return $ghost.value; })(0) }}</div>)",
  ],
};
function packet(stdout: string, status: number): Packet {
  return {
    label: "independent control",
    command: "not-executed",
    args: [],
    cwd: ".",
    status,
    signal: null,
    error: null,
    stdout,
    stderr: "",
    elapsedMs: 0,
  };
}

for (const id of ["unresolved-value", "unresolved-type-query"] as const) {
  test(`10617 native receiver expectation is confined to the complete ${id} packet`, () => {
    const entry = programs.find((row) => row.id === id)!;
    const frozen = independent.find((row) => row.id === id && row.host === "beta")!;
    const context = {
      ...packet(frozen.stdout, frozen.status!),
      signal: frozen.signal,
      stderr: frozen.stderr,
    };
    const report = {
      files: [
        { file: "src/Globals.vue", diagnostics: diagnostics[id] },
        { file: "src/globals.d.ts", diagnostics: [] },
      ],
      programs: [
        {
          root: ".",
          tsconfig: "tsconfig.json",
          compilerOptions,
          files: ["src/Globals.vue", "src/globals.d.ts"],
        },
      ],
      errorCount: 2,
      warningCount: 0,
      fileCount: 2,
    };
    const cli = packet(JSON.stringify(report), 1);
    const verify = (input = entry, oracle = context, native = cli, content = input.source) =>
      assertReceiverNativeCase(content, input, oracle, native, compilerOptions, id);
    verify();
    const raw = packet(expectedHost(entry.source, entry, compilerRows(context.stdout)), 2);
    assertMapperCase(entry.source, entry, context, raw, id);
    assert.throws(() =>
      assertMapperCase(
        entry.source,
        entry,
        context,
        { ...raw, stdout: raw.stdout.replace("TS2552", "TS2551") },
        id,
      ),
    );
    const unresolvedIndex = entry.errors.findIndex((row) => row.kind === "unresolved");
    const mutateReport = (mutate: (copy: typeof report) => void) => {
      const copy = structuredClone(report);
      mutate(copy);
      assert.throws(() => verify(entry, context, { ...cli, stdout: JSON.stringify(copy) }));
    };
    mutateReport((copy) => {
      copy.files[0].diagnostics[unresolvedIndex] = copy.files[0].diagnostics[unresolvedIndex]
        .replace("[TS2551]", "[TS2552]")
        .replace(
          "Property '__vize_template_instance' does not exist on the component instance.",
          "Cannot find name '__vize_template_instance'.",
        );
    });
    mutateReport((copy) => {
      copy.files[0].diagnostics[unresolvedIndex] = copy.files[0].diagnostics[
        unresolvedIndex
      ].replace("__vize_template_instance_'?", "other'?");
    });
    mutateReport((copy) => {
      copy.files[0].diagnostics[unresolvedIndex] = copy.files[0].diagnostics[
        unresolvedIndex
      ].replace(id === "unresolved-value" ? "error:2:51" : "error:5:46", "error:1:1");
    });
    mutateReport((copy) => {
      copy.files[0].diagnostics.reverse();
    });
    mutateReport((copy) => {
      copy.files[0].diagnostics.pop();
    });
    mutateReport((copy) => {
      copy.files[0].file = "foreign.vue";
    });
    mutateReport((copy) => {
      copy.programs[0].root = "foreign";
    });
    mutateReport((copy) => {
      copy.programs[0].files.pop();
    });
    mutateReport((copy) => {
      copy.errorCount = 0;
    });
    mutateReport((copy) => {
      copy.warningCount = 1;
    });
    mutateReport((copy) => {
      copy.fileCount = 1;
    });
    assert.throws(() =>
      verify(entry, context, { ...cli, stdout: JSON.stringify({ ...report, hidden: true }) }),
    );
    for (const mutation of [
      { status: 0 },
      { signal: "SIGTERM" },
      { error: "spawn failure" },
      { stderr: "unexpected stderr" },
    ])
      assert.throws(() => verify(entry, context, { ...cli, ...mutation }));
    assert.throws(() =>
      verify(entry, { ...context, stdout: context.stdout.replace("TS2552", "TS2551") }),
    );
    assert.throws(() =>
      verify(entry, {
        ...context,
        stdout: context.stdout.replace("__vize_template_instance_'?", "other'?"),
      }),
    );
    assert.throws(() => verify(entry, context, cli, `${entry.source}\n`));
    assert.throws(() => verify({ ...entry, source: `${entry.source}\n` }));
    assert.throws(() => verify({ ...entry, id: "foreign" }));
    assert.throws(() => verify({ ...entry, receiver: "direct" }));
    assert.throws(() => verify({ ...entry, candidate: { name: "foreign", role: "value" } }));
    const wrongAnchor = structuredClone(entry);
    wrongAnchor.errors[unresolvedIndex].anchor = "$ghost.value";
    assert.throws(() => verify(wrongAnchor));
  });
}

test("10617 other receiver controls retain the ordinary whole native comparator", () => {
  assert.deepEqual(
    programs.map((entry) => [entry.id, hasReceiverNativeExpectation(entry.id)]),
    [
      ["unresolved-value", true],
      ["unresolved-type-query", true],
      ["unrelated-function-name", false],
      ["outer-script-binding", false],
    ],
  );
});
