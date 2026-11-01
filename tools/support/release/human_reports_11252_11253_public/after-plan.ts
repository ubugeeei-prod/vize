import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import type { CommandPlan, MergeDescriptor, Request, Role, SourceFile } from "./model.ts";
import { argumentsFor } from "./commands.ts";
import { recipes, requiredCommands } from "./recipes.ts";
import { array, bytes, gitBlob, object, sha256, sourcePath, text } from "./value.ts";

export const inputPlanSha256 = "08e9d98975ef3bf695e24c3aa00bf9e31c5e601bad3bf17bdbd0484720ba2f45";
// Inert fixture spelling preserves the source owner's exact immutable bytes.
const raw = readFileSync(new URL("./after-input-plan.json.txt", import.meta.url));
assert.equal(sha256(raw), inputPlanSha256);
export const inputPlanGitBlob = gitBlob(raw);
const owned = object(JSON.parse(raw.toString("utf8")));
assert.equal(owned.kind, "SOURCE_OWNED_AFTER_COMPLETE_INPUT_PLAN_PREPARATION_ONLY");
assert(Object.values(object(owned.target)).every((v) => v === null));
for (const key of [
  "captureReady",
  "accepted",
  "productQualified",
  "afterQualified",
  "issueQualified",
  "outcomeAssessed",
])
  assert.equal(owned[key], false);

export interface InputState {
  universe: string[];
  present: SourceFile[];
  absent: string[];
}
export interface PlannedCommand {
  command: CommandPlan;
  before: InputState;
  after: InputState;
}
const canon = "crates/vize_canon/src/batch/virtual_project/";
export const requiredFixPaths: Record<11252 | 11253, string[]> = {
  11252: [
    "setup_props.rs",
    "setup_props/cache.rs",
    "setup_props/runtime_props.rs",
    "setup_props/runtime_props/object.rs",
    "setup_props/runtime_props/shape.rs",
    "setup_props/syntax.rs",
  ]
    .map((path) => canon + path)
    .concat([
      "tests/_fixtures/differential/typechecker/runtime-null-prop-11252/Box.vue.txt",
      "tests/_fixtures/differential/typechecker/runtime-null-prop-11252/package.json.txt",
      "tests/_fixtures/differential/typechecker/runtime-null-prop-11252/tsconfig.json.txt",
    ]),
  11253: [
    canon + "package_shadow_runtime.rs",
    "tests/_fixtures/differential/typechecker/mixed-vue-package-imports-11253/input.json",
  ],
};
export function requireCompleteFixPaths(fix: MergeDescriptor): void {
  const expected = array(owned.fixes)
    .map(object)
    .find((f) => f.issue === fix.issue);
  assert(expected);
  assert.equal(fix.pull, expected.pull);
  assert.equal(fix.merge, expected.merge);
  assert.equal(new Set(fix.paths).size, fix.paths.length);
  for (const path of requiredFixPaths[fix.issue])
    assert(fix.paths.includes(path), `missing required fix/fixture path: ${path}`);
  fix.paths.forEach(sourcePath);
}
function image(value: unknown): SourceFile {
  const v = object(value),
    path = text(v.path),
    content = bytes(text(v.base64));
  sourcePath(path);
  assert.equal(content.length, v.bytes);
  assert.equal(sha256(content), v.sha256);
  return { path, base64: content.toString("base64"), sha256: sha256(content) };
}
function state(value: unknown, version: string, supplement: boolean): InputState {
  const v = object(value),
    universe = array(v.universe).map(text),
    absent = array(v.absent).map(text);
  assert.equal(new Set(universe).size, universe.length);
  universe.forEach(sourcePath);
  const packagePlan = object(v.parameterizedPackage),
    source = image(packagePlan.sourceImage);
  assert.equal(source.path, "package.json");
  const binding = object(packagePlan.exactVersionBinding);
  assert.equal(binding.before, '"vize": "0.627.0"');
  assert.equal(binding.occurrences, 1);
  assert.equal(binding.replacementVersionParameter, "target.version");
  assert.equal(packagePlan.resolvedBytes, null);
  assert.equal(packagePlan.resolvedSha256, null);
  const original = bytes(source.base64).toString("utf8");
  assert.equal(original.split(text(binding.before)).length, 2);
  let manifest = original.replace(text(binding.before), `"vize": "${version}"`);
  if (supplement) {
    assert.equal(object(packagePlan.supplement).tsrs, "0.20.1-ts7.1.0-beta");
    const pkg = object(JSON.parse(manifest));
    object(pkg.devDependencies).tsrs = "0.20.1-ts7.1.0-beta";
    manifest = JSON.stringify(pkg, null, 2) + "\n";
  } else assert.equal(packagePlan.supplement, null);
  const files = array(v.present).map(image);
  const packageRaw = Buffer.from(manifest);
  files.push({
    path: "package.json",
    base64: packageRaw.toString("base64"),
    sha256: sha256(packageRaw),
  });
  assert.equal(new Set(files.map((f) => f.path)).size, files.length);
  assert.equal(new Set(absent).size, absent.length);
  assert.deepEqual(new Set([...files.map((f) => f.path), ...absent]), new Set(universe));
  assert(files.every((f) => !absent.includes(f.path)));
  return { universe, present: universe.flatMap((p) => files.filter((f) => f.path === p)), absent };
}
export function afterPlan(role: Role, version: string): PlannedCommand[] {
  assert.match(version, /^\d+\.\d+\.\d+$/u);
  assert.notEqual(version, "0.627.0", "actual containing AFTER version required");
  const record = array(owned.roles)
    .map(object)
    .find((r) => r.role === role);
  assert(record);
  assert.deepEqual(record.recipe, recipes[role]);
  const commands = array(record.commands).map(object);
  assert.deepEqual(
    commands.map((c) => c.id),
    requiredCommands[role],
  );
  return commands.map((c, index) => {
    assert.equal(c.order, index + 1);
    assert.equal(c.outcomeExpectation, null);
    assert.equal(c.wholeIdenticalPublicCommandOutputOracle, "UNKNOWN");
    const id = text(c.id),
      supplement = c.consumer === "tsrs-supplement";
    assert.equal(supplement, id.includes("tsrs"));
    const before = state(c.beforeInputState, version, supplement),
      after = state(c.afterInputState, version, supplement);
    assert.equal(c.requiresNativeLoad, id !== "original-vue-tsc");
    const command: CommandPlan = {
      id,
      consumer: supplement ? "tsrs-supplement" : "original",
      executable: "",
      argv: argumentsFor({ role } as Request, id),
      sourceFiles: before.present,
      purpose: text(c.purpose) as CommandPlan["purpose"],
      adaptation: c.adaptation === null ? null : text(c.adaptation),
      checker: {
        state: "UNKNOWN",
        reason: "Authenticated installed launcher observation is required at capture time",
      },
      requiresNativeLoad: Boolean(c.requiresNativeLoad),
      oracle: null,
    };
    assert.deepEqual(command.argv, c.argv);
    return { command, before, after };
  });
}
export function inputCommand(command: CommandPlan): unknown {
  const { executable: _executable, checker: _checker, ...input } = command;
  return input;
}
export function validateAfterInputs(request: Request): void {
  assert.equal(request.mode, "UNQUALIFIED_AFTER");
  assert.equal(request.release.phase, "after");
  assert(request.sourceAdmission && request.publicBefore);
  assert.equal(request.oracleOwnerReceipt, null);
  assert(request.release.fixes.some((f) => f.issue === request.fixture.issue));
  request.release.fixes.forEach(requireCompleteFixPaths);
  const expected = afterPlan(request.role, request.release.version);
  assert.deepEqual(
    request.commands.map(inputCommand),
    expected.map((p) => inputCommand(p.command)),
  );
}
export function instantiatedInputSha256(role: Role, version: string): string {
  return sha256(
    JSON.stringify(
      afterPlan(role, version).map((p) => ({
        command: inputCommand(p.command),
        before: p.before,
        after: p.after,
      })),
    ),
  );
}
export function planAuthorities(): Record<string, unknown>[] {
  const sources = [
    ...array(owned.sourceClosure),
    object(owned.hashVueFormattingAuthority).source,
    object(owned.hashVueFormattingAuthority).lock,
  ];
  for (const role of array(owned.roles).map(object))
    for (const c of array(role.commands).map(object)) sources.push(c.inputAuthority);
  const fixtures = object(owned.originalFixtureAuthority);
  sources.push(
    ...array(fixtures.nullFixturePins),
    fixtures.mixedFixturePin,
    fixtures.originalInputs,
  );
  return [
    ...new Map(sources.map(object).map((s) => [text(s.commit) + ":" + text(s.path), s])).values(),
  ];
}
export function unqualifiedAfterVerdict(role: Role): unknown {
  return {
    schema: "vize.humanReports11252_11253.verdict.v1",
    role,
    phase: "after",
    mode: "UNQUALIFIED_AFTER",
    status: "OBSERVED_PUBLIC_AFTER_UNQUALIFIED",
    accepted: false,
    productQualified: false,
    afterQualified: false,
    issueQualified: false,
    outcomeAssessed: false,
    knownBugReproductionJudged: false,
    backend: "UNKNOWN",
    inputPlanSha256,
    inputPlanGitBlob,
    scope:
      "Authenticated input/custody capture only; independent whole-output assessment remains required",
  };
}
