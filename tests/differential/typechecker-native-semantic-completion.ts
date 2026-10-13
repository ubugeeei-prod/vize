import assert from "node:assert/strict";
import path from "node:path";
import { readPinnedArtifact } from "./harness.mjs";

const pack = "native-semantic-completion";
const originalSource = "d55b8238f89bab4418a1a961bd8db1c84911cb42";
const repair = "7a782477742608f5fa98646448c53321974f6d44";
const composedSource = "15e51ec040f83cd03639b2f593c5cbf48055e890";
const frozenPackSha256 = "a84264c89bbdd611033fb372bd6d43819da4adf5527c837cf89c14cfc86e9c18";
const files = [
  "Comp0.vue",
  "Comp1.vue",
  "Comp2.vue",
  "Comp3.vue",
  "node_modules/leaf-types/index.d.ts",
  "node_modules/leaf-types/package.json",
  "shared.ts",
  "tsconfig.json",
];
const reportedFiles = ["Comp0.vue", "Comp1.vue", "Comp2.vue", "Comp3.vue", "shared.ts"];
const diagnostics = [
  {
    file: "Comp0.vue",
    line: 0,
    column: 58,
    code: 1260,
    severity: 1,
    message: "Keywords cannot contain escape characters.",
    blockType: "ScriptSetup",
  },
  {
    file: "shared.ts",
    line: 0,
    column: 20,
    code: 2304,
    severity: 1,
    message: "Cannot find name 'LeafGlobal'.",
    blockType: null,
  },
];
const originalOutputs = {
  "original.stdout.json": {
    bytes: 1010,
    sha256: "ae6ce301b3fc356ff80e5489c7add52d5a8515a87693872dc66bc9c78f9fa6ba",
  },
  "original.stderr.bin": {
    bytes: 194,
    sha256: "b0d0fc96d6f715fec696b677c0733bec36e3a91d2f1c5b990d8d56c31834d121",
  },
  "original-command.json": {
    bytes: 933,
    sha256: "785ca61dd1a31c8ad616a8e89cdf00f2fff09a2d6d07c8e0c50b4405f441dd60",
  },
  "original-provenance.json": {
    bytes: 4189,
    sha256: "35ddc74d34b01b758537b927fde009d8d700b8c97e6066ed54805fbac9f3d1c2",
  },
};

type Input = { file: string; path: string; sha256: string };
export type NativeSemanticCompletionCase = {
  id: string;
  state: "active";
  targets: ["source-cli", "configured-lsp-owner"];
  contract: "source-canon-semantic-completion-v1";
  pack: string;
  case: string;
  test: string;
  regressionCommit: string;
  sourceRevision: string;
  packReference: { path: string; sha256: string };
  inputs: Input[];
};

// Supplemental source/runtime custody is separate from the original legacy
// diagnostic packs. Loading this contract grants no execution or native credit.
export function loadNativeSemanticCompletionCases(root: string, value: unknown) {
  assert(Array.isArray(value), "supplemental cases are required");
  assert.equal(value.length, 1, "exactly one registered supplemental case is required");
  const fixture = value[0] as NativeSemanticCompletionCase;
  assert.deepEqual(Object.keys(fixture).sort(), [
    "case",
    "contract",
    "id",
    "inputs",
    "pack",
    "packReference",
    "regressionCommit",
    "sourceRevision",
    "state",
    "targets",
    "test",
  ]);
  assert.equal(fixture.id, `typechecker/${pack}/complete-original-escaped-import-keyword`);
  assert.equal(fixture.pack, pack, "unregistered supplemental pack");
  assert.equal(fixture.case, "complete-original-escaped-import-keyword");
  assert.equal(fixture.state, "active");
  assert.deepEqual(fixture.targets, ["source-cli", "configured-lsp-owner"]);
  assert.equal(fixture.contract, "source-canon-semantic-completion-v1");
  assert.equal(
    fixture.test,
    "native_semantic_completion_keeps_frozen_full_vector_and_actual_terminal_owner",
    "unregistered supplemental test binary body",
  );
  assert.equal(fixture.regressionCommit, repair);
  assert.equal(fixture.sourceRevision, originalSource);
  assert.deepEqual(
    fixture.packReference,
    {
      path: `${pack}/case.json`,
      sha256: frozenPackSha256,
    },
    "the entire frozen supplemental oracle is immutable",
  );
  const original = JSON.parse(readPinnedArtifact(root, fixture.packReference).toString("utf8"));
  assert.equal(original.schemaVersion, 1);
  assert.equal(original.id, pack);
  assert.equal(original.sourceRepairRevision, repair);
  assert.equal(
    original.sourceRecord,
    `actual original record271 leaf-parity-escaped-import-keyword base ${originalSource}; composed artifact ${composedSource}`,
  );
  assert.deepEqual(
    original.inputFiles.map((input: { file: string }) => input.file),
    files,
  );
  assert.deepEqual(
    fixture.inputs,
    original.inputFiles.map((input: { file: string; fixture: string; sha256: string }) => ({
      file: input.file,
      path: `${pack}/${input.fixture}`,
      sha256: input.sha256,
    })),
  );
  for (const [index, input] of fixture.inputs.entries()) {
    assert.equal(input.path, `${pack}/inputs/${input.file}.txt`);
    assert.equal(readPinnedArtifact(root, input).length, original.inputFiles[index].bytes);
  }
  assert.deepEqual(original.reportedFiles, reportedFiles);
  assert.deepEqual(original.expectedDiagnostics, diagnostics);
  assert.deepEqual(original.cliArgv, [
    "check",
    "--no-config",
    "--quiet",
    "--format",
    "json",
    "--tsconfig",
    "tsconfig.json",
    "--corsa-path",
    "<authenticated-native>",
    "--servers",
    "1",
  ]);
  assert.equal(original.expectedExit, 1);
  assert.deepEqual(original.expectedStderr, []);
  assert.deepEqual(original.originalOutputs, originalOutputs);
  const outputs = new Map<string, Buffer>();
  for (const [file, pin] of Object.entries(originalOutputs)) {
    const bytes = readPinnedArtifact(root, { path: `${pack}/${file}`, sha256: pin.sha256 });
    assert.equal(bytes.length, pin.bytes);
    outputs.set(file, bytes);
  }
  const stdout = JSON.parse(outputs.get("original.stdout.json")!.toString("utf8"));
  assert.deepEqual(stdout, {
    files: [
      {
        file: "Comp0.vue",
        diagnostics: ["error:1:59 [TS1260] Keywords cannot contain escape characters."],
      },
      { file: "Comp1.vue", diagnostics: [] },
      { file: "Comp2.vue", diagnostics: [] },
      { file: "Comp3.vue", diagnostics: [] },
      { file: "shared.ts", diagnostics: ["error:1:21 [TS2304] Cannot find name 'LeafGlobal'."] },
    ],
    programs: [
      {
        root: ".",
        tsconfig: "tsconfig.json",
        compilerOptions: {
          lib: ["ESNext", "DOM"],
          module: "ESNext",
          moduleResolution: "Bundler",
          skipLibCheck: true,
          strict: true,
          target: "ESNext",
        },
        files: reportedFiles,
      },
    ],
    errorCount: 2,
    warningCount: 0,
    fileCount: 5,
  });
  const command = JSON.parse(outputs.get("original-command.json")!.toString("utf8"));
  const provenance = JSON.parse(outputs.get("original-provenance.json")!.toString("utf8"));
  assert.equal(command.id, "271-leaf-parity-escaped-import-keyword-1-server-parity-base");
  assert.equal(command.status, 1);
  assert.equal(command.signal, null);
  assert.equal(command.side, "base");
  assert.equal(command.corpus, "leaf-parity-escaped-import-keyword");
  assert.deepEqual(command.environment, {
    RAYON_NUM_THREADS: "1",
    GOMAXPROCS: null,
    VIZE_BENCH: "1",
  });
  assert.equal(command.command[0], provenance.binaries.base.measuredPath);
  assert.deepEqual(
    command.command.slice(1),
    original.cliArgv.map((arg: string) =>
      arg === "<authenticated-native>" ? provenance.binaries.typescript.measuredPath : arg,
    ),
  );
  assert.equal(provenance.baseSha, originalSource);
  assert.equal(provenance.headSha, composedSource);
  assert.equal(provenance.dependencies.runtimePackageVersion, "7.0.2");
  assert.equal(
    provenance.binaries.base.sha256,
    "ec8abd8cd054fc110c15111b4e9ae45cc7a6abc874be69de272734ea91d61eee",
  );
  assert.equal(
    provenance.binaries.head.sha256,
    "970d278498856c07a7b259f003a93c86578453e3ae334cb513300f3148a2e062",
  );
  assert.equal(
    provenance.binaries.typescript.sha256,
    "4f2de678286401759b3fb4475bafe35b8f32b4b3a07d92642bbf37eadc9b34a4",
  );
  assert.equal(path.basename(command.cwd), "leaf-parity-escaped-import-keyword");
  return [fixture];
}
