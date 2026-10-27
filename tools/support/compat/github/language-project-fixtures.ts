import { createHash } from "node:crypto";

type ReadFile = (side: "base" | "head", path: string) => Buffer | null;
export const LANGUAGE_PROJECT_CASE_ID =
  "typechecker/language-project/inherited-configuration-10740";
const root = "tests/_fixtures/differential/typechecker/";
const directory = "typescript-project-configuration-10740/";
const witnessHash = "462925989cfc5c1ec84b9ddcdabf175579059df648013acc25b00a5fcc316892";
const inputs = [
  "case.json",
  "src/Inherited.vue.txt",
  "src/control.ts.txt",
  "inherited-before.json.txt",
  "inherited-after.json.txt",
  "tsconfig-6.0.json.txt",
  "tsconfig-5.7.json.txt",
  "expected.json",
  "before-witness.json",
  "before-inputs.json",
];
export const LANGUAGE_PROJECT_PRODUCERS = [
  ["generate_vue_language_code", "crates/vize_vitrine/src/napi/language_code.rs"],
  ["init", "npm/typescript-plugin/index.cjs"],
  ["createProjectConfiguration", "npm/typescript-plugin/virtual-modules.cjs"],
  ["createMappedLanguageService", "npm/typescript-plugin/language-service.cjs"],
  ["readProjectFixture", "tests/typescript-plugin/project-fixture.cjs"],
  ["checkInheritedConfig", "tests/typescript-plugin/inherited-config.ts"],
  ["createOrdinaryHostFrontier", "tests/typescript-plugin/ordinary-host-frontier.ts"],
  [
    "one project fingerprints its native closure once per epoch and disposes exact watches",
    "tests/typescript-plugin/project-configuration.test.cjs",
  ],
  ["packed-consumer", "tests/typescript-plugin/packed-consumer.mjs"],
  ["startTsserver", "tests/typescript-plugin/tsserver-client.mjs"],
  ["checkInheritedConfiguration", "editors/vscode/test/suite/typescript-provider-host.cjs"],
  ["runPackagedExtensionHost", "editors/vscode/test/run-typescript-provider-host.mjs"],
  ["TypeScript 5.7.3 and 6.0.3", ".github/workflows/typescript-plugin.yml"],
  ["Linux and Windows physical VSIX", ".github/workflows/typescript-provider-vsix.yml"],
] as const;

// These are whole-file identities of the reviewed transition, not a path or
// directory exemption. Another edit to any of these files follows the ordinary law.
const transition: ReadonlyArray<readonly [string, string | null, string]> = [
  [
    "crates/vize_canon/src/batch/virtual_project/content_mapper_config.rs",
    "d1a9ef036cd8942fcb6e16a9396045928e82b4bb7b841dccf53fcabd49636404",
    "a39846a38f2ba05696093242380a50438f6d299c5df19d5b55806345d45c8476",
  ],
  [
    "crates/vize_canon/src/batch/virtual_project/content_mapper_config_tests.rs",
    null,
    "228e6f47714892e892d53af57a97e3c3fcd7ef1004e8bd543bd10c1f1b0fdfff",
  ],
  [
    "crates/vize_canon/src/batch/virtual_project.rs",
    "98fdb4def0cff8c851eedff2aabe2e0d77782fbb944b864af599a1383bdaaa71",
    "e073f2bc72e117cde99d1a87c9dab7c6cb33df5835135419b27a89c71a2d4ac4",
  ],
  [
    "crates/vize_canon/src/batch/virtual_project/tsconfig_gen/vue_compiler_options.rs",
    "66200f1e40bef06c53a252134659eccb0bd876a34ae87868de388bbfc2824a81",
    "7c7d7c8d71b82e288c81b2fcb462d1fec29dbe4c60e8687fb1a7366d21bbb612",
  ],
  [
    "crates/vize_canon/src/batch/virtual_project/tsconfig_gen/vue_compiler_options/extends.rs",
    null,
    "6f819ecd03963fddfaa3be40bd69efcf484c5a83a002be4464a2bb217c45380f",
  ],
  [
    "crates/vize_canon/src/batch/virtual_project/tsconfig_gen/vue_compiler_options/tests.rs",
    "9a18acbeb1f013efc35f8e2f8da266d46ef5e373e45fdba790b67db4688e3255",
    "993f7e02c951234951be843db702dfff2d3875c298d6089d228c9d3a21aed46e",
  ],
];
const digest = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
function require(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(`invalid language project case: ${message}`);
}
function object(value: unknown, keys: string[]): Record<string, unknown> {
  require(value && typeof value === "object" && !Array.isArray(value), "object required");
  require(JSON.stringify(Object.keys(value).sort()) ===
    JSON.stringify([...keys].sort()), "closed schema fields differ");
  return value as Record<string, unknown>;
}
function pinned(readFile: ReadFile, path: string, hash: unknown) {
  require(typeof hash === "string" && /^[a-f0-9]{64}$/.test(hash), `invalid hash: ${path}`);
  const bytes = readFile("head", path);
  require(bytes && digest(bytes) === hash, `missing or stale input: ${path}`);
  return bytes;
}

export function newLanguageProjectCases(
  previous: unknown,
  current: unknown,
  readFile: ReadFile,
): { ids: string[]; sources: Set<string> } {
  const applicable = transition.every(([source, before, head]) => {
    const a = readFile("base", source),
      b = readFile("head", source);
    return (
      (before === null ? a === null : a !== null && digest(a) === before) &&
      b !== null &&
      digest(b) === head
    );
  });
  require(current === undefined || Array.isArray(current), "languageProjectCases must be an array");
  require(previous === undefined ||
    Array.isArray(previous), "prior languageProjectCases must be an array");
  const oldIds = new Set((previous ?? []).map((row: { id?: unknown }) => row?.id));
  const ids = new Set<string>();
  const added = (current ?? []).filter((value: unknown) => {
    require(value &&
      typeof value === "object" &&
      "id" in value &&
      typeof value.id === "string", "missing ID");
    require(!ids.has(value.id), "duplicate ID");
    ids.add(value.id);
    return !oldIds.has(value.id);
  });
  if (!applicable) {
    require(added.length === 0, "unregistered source transition");
    return { ids: [], sources: new Set() };
  }
  require(added.length ===
    1, "the exact reviewed transition requires one new protocol case; batch filler cannot replace it");
  const row = object(added[0], ["id", "state", "adapter", "targets", "inputs", "tests", "witness"]);
  require(row.id === LANGUAGE_PROJECT_CASE_ID &&
    row.state === "active", "unregistered or inactive ID");
  require(row.adapter === "typescript-project-configuration-v1", "unregistered adapter");
  require(JSON.stringify(row.targets) ===
    '["typescript-project","vscode-typescript"]', "unregistered targets");
  require(Array.isArray(row.inputs) &&
    row.inputs.length === inputs.length, "exact whole input pack required");
  const files = new Map<string, Buffer>();
  for (const [index, value] of row.inputs.entries()) {
    const input = object(value, ["path", "sha256"]);
    const path = directory + inputs[index];
    require(input.path === path, "input path or order differs");
    files.set(inputs[index]!, pinned(readFile, root + path, input.sha256));
  }
  const witness = object(row.witness, ["path", "sha256", "sourceRevision", "run", "artifact"]);
  require(witness.path === directory + "before-witness.json" &&
    witness.sha256 === witnessHash, "original failing witness changed");
  require(witness.sourceRevision === "085b4753c45997744d477cd2be1a882c43704a79" &&
    witness.run === 40226962544 &&
    witness.artifact === 12610814625, "before provenance changed");
  require(digest(files.get("before-witness.json")!) ===
    witnessHash, "whole original failing packet changed");
  const original = JSON.parse(files.get("before-witness.json")!.toString("utf8"));
  const expected = JSON.parse(files.get("expected.json")!.toString("utf8"));
  require(JSON.stringify(original.after.body) === "[]", "original warm failure absent");
  require(JSON.stringify(original.fresh.body) ===
    JSON.stringify(expected.strict), "independent full diagnostic oracle changed");
  require(JSON.stringify(original.controlAfter.body) ===
    JSON.stringify(expected.controlAfter), "independent ordinary TS oracle changed");
  require(files.get("src/Inherited.vue.txt")!.toString("utf8") ===
    original.source, "original authored input changed");
  require(Array.isArray(row.tests) &&
    row.tests.length === LANGUAGE_PROJECT_PRODUCERS.length, "all actual producers required");
  for (const [index, value] of row.tests.entries()) {
    const test = object(value, ["selector", "source", "sha256"]);
    const [selector, source] = LANGUAGE_PROJECT_PRODUCERS[index]!;
    require(test.selector === selector && test.source === source, "unregistered producer");
    pinned(readFile, source, test.sha256);
  }
  const beforeInputs = files.get("before-inputs.json")!;
  require(digest(beforeInputs) ===
    "8fedd5da81bb3f78ae01dbe525bfad594702933f5ad3ff05b270a719d70f0e43", "original input witness changed");
  const originalInputs = JSON.parse(beforeInputs.toString("utf8"));
  require(files.get("src/control.ts.txt")!.toString("utf8") ===
    originalInputs.controlSource, "original ordinary TS control changed");
  require(files.get("tsconfig-6.0.json.txt")!.toString("utf8") ===
    originalInputs.root, "original explicit configuration changed");
  require(JSON.stringify(original.before.body) === JSON.stringify(expected.before) &&
    JSON.stringify(original.controlBefore.body) ===
      JSON.stringify(expected.controlBefore), "original before diagnostic oracles changed");
  const pack = object(JSON.parse(files.get("case.json")!.toString("utf8")), [
    "schema",
    "issue",
    "contract",
    "inputs",
    "beforeWitness",
  ]);
  require(pack.contract ===
    "whole-tsserver-diagnostics-and-project-metadata-v1", "unregistered whole-packet contract");
  require(pack.schema === "vize.typescriptProjectConfigurationFixture.v1" &&
    pack.issue === 10740, "invalid fixture pack");
  require(Array.isArray(pack.inputs) &&
    pack.inputs.length === inputs.length - 2, "incomplete fixture pack inputs");
  for (const raw of [...pack.inputs, pack.beforeWitness]) {
    const value = object(
      raw,
      raw === pack.beforeWitness
        ? ["path", "sha256", "sourceRevision", "run", "artifact", "nativeImage"]
        : ["path", "sha256"],
    );
    require(typeof value.path === "string" &&
      files.has(value.path), "unregistered fixture pack member");
    require(digest(files.get(value.path)!) === value.sha256, "fixture pack member changed");
  }
  return {
    ids: [LANGUAGE_PROJECT_CASE_ID],
    sources: new Set(transition.map(([source]) => source)),
  };
}
