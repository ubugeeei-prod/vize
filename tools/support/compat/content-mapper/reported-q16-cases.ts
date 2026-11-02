/** Closed report case selection and post-integration-compile executable provenance. */
import assert from "node:assert/strict";
import path from "node:path";
export const q16Cases = [
  [
    "model",
    "content_mapper_tsgo_cli::reported_model_props_11359::resolved_model_props_preserve_authored_contract_and_assignment_errors",
  ],
  [
    "generic",
    "content_mapper_tsgo_cli::reported_generic_components::reported_generic_slots_and_global_refs_keep_authored_members",
  ],
  [
    "dynamic",
    "content_mapper_tsgo_cli::reported_dynamic_targets::originals_and_error_repair_restore_keep_component_target_types",
  ],
  [
    "nested",
    "content_mapper_tsgo_cli::nested_callback_11361::nested_callback_keeps_original_acceptance_and_checked_body_errors",
  ],
  [
    "declaration-unused",
    "check_reported_declarations_11364_11365_cli::repeated_component_declarations_preserve_no_unused_and_authored_errors",
  ],
  [
    "declaration-fallthrough",
    "check_reported_declarations_11364_11365_cli::emitted_component_declarations_accept_fallthrough_and_reject_owned_prop_errors",
  ],
] as const;
export function selectQ16Cases(value: unknown) {
  const ids: Record<string, readonly string[]> = {
    all: q16Cases.map((r) => r[0]),
    A: ["model", "declaration-unused"],
    "A-with-declaration": ["model", "declaration-unused", "declaration-fallthrough"],
    B: ["generic"],
    C: ["dynamic"],
    D: ["nested"],
  };
  assert(
    typeof value === "string" && Object.hasOwn(ids, value),
    "closed reported-q16-cases is required",
  );
  const selected = q16Cases.filter((r) => ids[value].includes(r[0]));
  assert.equal(selected.length, ids[value].length);
  return selected;
}
export function verifyQ16List(
  text: string,
  selected: readonly (readonly [string, string])[] = q16Cases,
) {
  const lines = text.split(/\r?\n/u);
  for (const [, name] of selected)
    assert.equal(
      lines.filter((line) => line === `${name}: test`).length,
      1,
      `one compiled test required: ${name}`,
    );
}
export function verifyQ16Execution(text: string, status: number | null) {
  assert(/(?:^|\n)running 1 test\r?\n/u.test(text), "test filter must execute exactly one test");
  assert(/test result: (?:ok|FAILED)\. [01] passed; [01] failed; 0 ignored;/u.test(text));
  return status === 0 && /test result: ok\. 1 passed; 0 failed; 0 ignored;/u.test(text);
}

type CargoArtifact = {
  reason: string;
  package_id: string;
  target: { name: string; kind: string[] };
  profile: { test: boolean };
  executable: string;
};
export function compiledQ16Artifacts(messages: CargoArtifact[], packageId: string) {
  const rows = messages.filter(
    (r) => r.reason === "compiler-artifact" && r.package_id === packageId && r.executable,
  );
  const integration = rows.filter(
    (r) => r.target.name === "integration" && r.target.kind.includes("test") && r.profile.test,
  );
  const cli = rows.filter(
    (r) => r.target.name === "vize" && r.target.kind.includes("bin") && r.profile.test === false,
  );
  assert.equal(integration.length, 1);
  assert.equal(cli.length, 1);
  return { integration: integration[0], cli: cli[0] };
}
type Image = { path: string; bytes: number; sha256: string };
export function attestQ16CompiledCli(
  initialBuild: Image,
  executed: Image,
  artifact: CargoArtifact,
  packageId: string,
) {
  assert.equal(artifact.reason, "compiler-artifact");
  assert.equal(artifact.package_id, packageId);
  assert.equal(artifact.target.name, "vize");
  assert(artifact.target.kind.includes("bin"));
  assert.equal(artifact.profile.test, false);
  assert(path.isAbsolute(artifact.executable));
  assert.equal(executed.path, path.resolve(artifact.executable));
  assert.equal(executed.path, initialBuild.path);
  for (const image of [initialBuild, executed]) {
    assert.match(image.sha256, /^[a-f0-9]{64}$/u);
    assert(Number.isSafeInteger(image.bytes) && image.bytes > 0);
  }
  return {
    initialBuild,
    executed,
    artifact,
    changedAfterIntegrationCompile: initialBuild.sha256 !== executed.sha256,
    scope:
      "compiled Cargo normal-bin artifact consumed by CARGO_BIN_EXE_vize; actual post-compile image guarded, initial build retained as separate provenance",
  };
}
