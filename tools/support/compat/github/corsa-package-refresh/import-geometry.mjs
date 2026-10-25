// Execute the actual source libtest and preserve its full output before asserting.
import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

assert.equal(process.env.GITHUB_ACTIONS, "true");
const capture = process.env.VIZE_IMPORT_GEOMETRY_CAPTURE;
assert(capture);
fs.mkdirSync(capture, { recursive: true });
const hash = (file) => createHash("sha256").update(fs.readFileSync(file)).digest("hex");
const git = (...args) => execFileSync("git", args, { encoding: "utf8" }).trim();
const save = (name, data) =>
  fs.writeFileSync(path.join(capture, name), JSON.stringify(data, null, 2) + "\n");
const state = (result) => ({
  status: result.status,
  signal: result.signal,
  error: result.error?.message ?? null,
  stdout: result.stdout ?? "",
  stderr: result.stderr ?? "",
});
const sources = [
  "crates/vize_canon/src/batch/import_rewriter.rs",
  "crates/vize_canon/src/batch/import_rewriter/source_map.rs",
  "crates/vize_canon/src/batch/import_rewriter/literal_geometry_tests.rs",
];
assert.equal(git("diff", "--name-only"), "");
const receipt = {
  schema: "vize.import-literal-geometry-source",
  version: 1,
  source: git("rev-parse", "HEAD"),
  tree: git("rev-parse", "HEAD^{tree}"),
  run: process.env.GITHUB_RUN_ID,
  attempt: process.env.GITHUB_RUN_ATTEMPT,
  qualification: false,
  sources: sources.map((file) => {
    const target = path.join(capture, "source", file);
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.copyFileSync(file, target);
    return { file, blob: git("rev-parse", `HEAD:${file}`), sha256: hash(file) };
  }),
};
save("receipt.json", receipt);
const args = [
  "test",
  "--lib",
  "--no-run",
  "--message-format=json",
  "--locked",
  "--profile",
  "ci",
  "-p",
  "vize_canon",
];
const build = spawnSync("cargo", args, {
  encoding: "utf8",
  timeout: 900_000,
  maxBuffer: 64_000_000,
});
save("build.json", { command: "cargo", args, ...state(build) });
assert.equal(build.error, undefined);
assert.equal(build.status, 0, build.stderr);
const artifacts = build.stdout
  .split("\n")
  .filter(Boolean)
  .map((line) => JSON.parse(line))
  .filter(
    (item) =>
      item.reason === "compiler-artifact" &&
      item.target.name === "vize_canon" &&
      item.profile.test &&
      item.executable,
  );
assert.equal(artifacts.length, 1);
const binary = fs.realpathSync(artifacts[0].executable);
receipt.binary = { path: binary, sha256: hash(binary), cargoArtifact: artifacts[0] };
save("receipt.json", receipt);
const required = [
  "literal_geometry_tests::changed_literal_boundaries_and_interiors_are_atomic_in_both_directions",
  "literal_geometry_tests::repeated_imports_keep_each_literal_and_later_symbol_coordinates",
  "literal_geometry_tests::unchanged_literal_contents_keep_every_byte_coordinate_and_callback_occurrence",
  "literal_geometry_tests::legacy_adjustment_constructor_keeps_its_existing_contract",
  "literal_geometry_tests::overflowing_queries_cannot_wrap_onto_an_unrelated_document_byte",
  "literal_geometry_tests::zero_length_and_multiple_replacement_records_keep_exclusive_end_shifts",
  "literal_geometry_tests::changed_module_map_composes_with_the_existing_star_replacement_carrier",
  "literal_geometry_tests::source_kind_variants_use_the_same_geometry_without_an_extra_parse",
  "literal_geometry_tests::shared_syntax_still_parses_once_and_resolves_every_emitted_occurrence",
  "literal_geometry_tests::tsx_module_geometry_does_not_rewrite_attribute_or_text_literals",
  "literal_geometry_tests::lsp_utf16_positions_cross_the_actual_rewritten_byte_map",
  "source_map::storage_tests::normal_rewrites_use_one_presized_metadata_buffer_without_reallocation",
].map((name) => `batch::import_rewriter::${name}`);
const testArgs = ["batch::import_rewriter", "--nocapture", "--test-threads=1"];
const result = spawnSync(binary, testArgs, {
  encoding: "utf8",
  timeout: 180_000,
  maxBuffer: 32_000_000,
});
save("execution.json", { command: binary, args: testArgs, ...state(result) });
assert.equal(result.error, undefined);
assert.equal(result.status, 0, result.stdout + result.stderr);
for (const name of required) assert(result.stdout.includes(`test ${name} ... ok`), name);
assert.match(result.stdout, /test result: ok\. \d+ passed; 0 failed; 0 ignored;/);
assert.match(result.stderr, /literal_mapping_syntax .*parses: 1.*resolver_calls=4/);
assert.match(
  result.stderr,
  /literal_mapping_storage rows=256 owned_buffers=1 capacity=256 bytes=4096/,
);
assert.equal(git("diff", "--name-only"), "");
receipt.qualification = true;
receipt.requiredExecuted = required;
receipt.executionSha256 = hash(path.join(capture, "execution.json"));
save("receipt.json", receipt);
