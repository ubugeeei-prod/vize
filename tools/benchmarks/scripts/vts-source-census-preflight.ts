/** Fail-closed source custody, instrumentation projection and profiling-gate preflight. */
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";

const sourceParent = process.argv[2],
  sourceSha = process.argv[3],
  driverSha = process.argv[4];
assert.match(sourceParent, /^[a-f0-9]{40}$/u);
assert.match(sourceSha, /^[a-f0-9]{40}$/u);
assert.match(driverSha, /^[a-f0-9]{40}$/u);
const git = (...args: string[]) => {
  const result = spawnSync("git", args, { encoding: "utf8", maxBuffer: 32 * 1024 * 1024 });
  assert.equal(result.status, 0, result.stderr);
  return result.stdout.trim();
};
assert.equal(git("rev-parse", "HEAD"), sourceSha, "checkout/source SHA drift");
assert.equal(git("rev-parse", driverSha + "^{commit}"), driverSha, "driver SHA drift");
assert.equal(
  git("rev-parse", "HEAD^"),
  sourceParent,
  "immediate parent differs: PR base advanced or the source is not a one-commit overlay",
);
assert.equal(git("status", "--porcelain", "--untracked-files=all"), "");

const production = [
  "crates/vize/Cargo.toml",
  "crates/vize/src/commands/profile_export.rs",
  "crates/vize_canon/Cargo.toml",
  "crates/vize_canon/src/lib.rs",
  "crates/vize_canon/src/profiling.rs",
  "crates/vize_canon/src/batch/import_rewriter.rs",
  "crates/vize_canon/src/batch/import_rewriter/collect.rs",
  "crates/vize_canon/src/batch/import_rewriter/project_mirror.rs",
  "crates/vize_canon/src/batch/virtual_project/dependency_scan.rs",
];
const tooling = [
  ".github/workflows/vts-source-census.yml",
  "tools/benchmarks/scripts/vts-source-census.ts",
  "tools/benchmarks/scripts/vts-source-census-fixture.ts",
  "tools/benchmarks/scripts/vts-source-census-preflight.ts",
  "tools/benchmarks/scripts/vts-source-census-profile.ts",
  "tests/tooling/vts-source-census.test.ts",
];
const expected = [...production, ...tooling].toSorted();
const changed = git("diff", "--name-only", sourceParent, sourceSha)
  .split("\n")
  .filter(Boolean)
  .toSorted();
assert.deepEqual(changed, expected, "unexpected source or workflow change");

const rustCharLiteral =
  /'(?:[^'\\\r\n\t]|\\(?:['"\\nrt0]|x[0-7][0-9a-fA-F]|u\{(?:[0-9a-fA-F]_*){1,6}\}))'/uy;
function eraseInstrumentation(source: string) {
  const marker = /vize_carton::micro_(count|span)!\s*\(/gu;
  for (let match; (match = marker.exec(source));) {
    const start = match.index,
      open = marker.lastIndex - 1;
    let depth = 0,
      quoted = false,
      escaped = false,
      comma = -1,
      end = -1;
    for (let i = open; i < source.length; i++) {
      const char = source[i];
      if (quoted) {
        if (escaped) escaped = false;
        else if (char === "\\") escaped = true;
        else if (char === '"') quoted = false;
        continue;
      }
      if (char === '"') {
        quoted = true;
        continue;
      }
      if (char === "'") {
        rustCharLiteral.lastIndex = i;
        const literal = rustCharLiteral.exec(source);
        if (literal) {
          i += literal[0].length - 1;
          continue;
        }
      }
      if ("([{".includes(char)) depth++;
      if (")]}".includes(char)) depth--;
      if (char === "," && depth === 1 && comma === -1) comma = i;
      if (depth === 0) {
        end = i;
        break;
      }
    }
    assert.ok(comma >= 0 && end > comma, "incomplete profiling macro");
    let replacement = "";
    if (match[1] === "span")
      replacement = source
        .slice(comma + 1, end)
        .trim()
        .replace(/,$/u, "")
        .trim();
    else {
      let tail = end + 1;
      while (/\s/u.test(source[tail] ?? "") && tail < source.length) tail++;
      assert.equal(source[tail], ";");
      end = tail;
    }
    source = source.slice(0, start) + replacement + source.slice(end + 1);
    marker.lastIndex = start + replacement.length;
  }
  return source;
}

const projectionProbe = [
  "vize_carton::micro_count!(\"probe\", ('(' as u32) + (')' as u32));",
  'vize_carton::micro_span!("probe", {',
  "    fn borrow<'a>(value: &'a str) -> &'a str {",
  "        let punctuation = ('(', ')', '[', ']', '{', '}');",
  "        let quote = '\"';",
  "        let apostrophe = '\\'';",
  "        let unicode = '\\u{1F600}';",
  "        let lifetime: &'a str = value;",
  "        let _ = (punctuation, quote, apostrophe, unicode);",
  "        lifetime",
  "    }",
  "})",
].join("\n");
assert.equal(
  eraseInstrumentation(projectionProbe).trim(),
  [
    "{",
    "    fn borrow<'a>(value: &'a str) -> &'a str {",
    "        let punctuation = ('(', ')', '[', ']', '{', '}');",
    "        let quote = '\"';",
    "        let apostrophe = '\\'';",
    "        let unicode = '\\u{1F600}';",
    "        let lifetime: &'a str = value;",
    "        let _ = (punctuation, quote, apostrophe, unicode);",
    "        lifetime",
    "    }",
    "}",
  ].join("\n"),
  "profiling projection must skip Rust chars without consuming lifetimes",
);

const normalize = (text: string) =>
  text
    .replace(/\s+/gu, "")
    .replace(/,\)/gu, ")")
    .replace(
      /Some\(facts\)=>\{(facts\.collect_occurrences\(&virtual_content,source_type\))\}/gu,
      "Some(facts)=>$1,",
    )
    .replace(
      /None=>\{(self\.rewriter\(\)\.collect_all_specifier_occurrences\(&virtual_content,source_type\))\}/gu,
      "None=>$1,",
    );
function productionProjection(file: string, source: string) {
  source = eraseInstrumentation(source);
  if (file.endsWith("project_mirror.rs"))
    source = source.replace(
      /\s*#\[cfg\(feature = "profiling"\)\]\s*crate::profiling::observe_rewrite_input\(source, source_type\);/u,
      "",
    );
  if (file.endsWith("dependency_scan.rs"))
    source = source.replace(
      /\s*#\[cfg\(feature = "profiling"\)\]\s*crate::profiling::observe_dependency_input\(&virtual_content, source_type\);/u,
      "",
    );
  if (file === "crates/vize/src/commands/profile_export.rs")
    source = source
      .replace(
        /\s*#\[cfg\(feature = "profiling"\)\]\s*vize_canon::profiling::ensure_vts_source_census_output_distinct\(path\)\?;/u,
        "",
      )
      .replace(/\s*let allocation = profile_support::allocation_snapshot\(\);/u, "")
      .replace(
        /allocation,\s*budget:/u,
        "allocation: profile_support::allocation_snapshot(),budget:",
      )
      .replace(
        /\s*#\[cfg\(feature = "profiling"\)\]\s*return write_profile_then_receipt\(path, export\.to_json\(\)\.as_bytes\(\)\);/u,
        "",
      )
      .replace(/\s*#\[cfg\(not\(feature = "profiling"\)\)\]\s*/u, "\n")
      .replace(/\n#\[cfg\(feature = "profiling"\)\]\nfn write_profile_then_receipt[\s\S]*$/u, "");
  if (file === "crates/vize/Cargo.toml")
    source = source.replace(
      /profiling = \[\s*"vize_l0\/micro-profiling",\s*"vize_carton\/micro-profiling",\s*"vize_canon\/profiling",?\s*\]/u,
      'profiling = ["vize_l0/micro-profiling", "vize_carton/micro-profiling"]',
    );
  if (file === "crates/vize_canon/Cargo.toml")
    source = source.replace(
      /\s*profiling = \["native", "vize_l0\/micro-profiling", "vize_carton\/micro-profiling"\]/u,
      "",
    );
  if (file === "crates/vize_canon/src/lib.rs")
    source = source.replace(
      /\s*#\[cfg\(feature = "profiling"\)\]\s*#\[doc\(hidden\)\]\s*pub mod profiling;/u,
      "",
    );
  let projected = normalize(source);
  if (file.endsWith("dependency_scan.rs")) {
    const binding = "letcontent=self.module_source(file).unwrap_or(&file.content);";
    assert.equal(projected.split(binding).length, 2);
    projected = projected.replace(binding, "");
    assert.equal(projected.split("content.clone()").length, 2);
    projected = projected.replace(
      "content.clone()",
      "self.module_source(file).unwrap_or(&file.content).clone()",
    );
  }
  return projected;
}

for (const file of production.filter((file) => !file.endsWith("/profiling.rs"))) {
  const actual = productionProjection(file, fs.readFileSync(file, "utf8")),
    expectedSource = normalize(git("show", sourceParent + ":" + file));
  assert.equal(actual, expectedSource, `${file}: disabled projection must equal production parent`);
}

const profiling = fs.readFileSync("crates/vize_canon/src/profiling.rs", "utf8"),
  macros = fs.readFileSync("davinci/vize_l0/src/profiler/micro_macros.rs", "utf8"),
  macroLaw = fs.readFileSync("davinci/vize_l0/tests/source_operations_profile.rs", "utf8"),
  rootManifest = fs.readFileSync("crates/vize/Cargo.toml", "utf8"),
  canonManifest = fs.readFileSync("crates/vize_canon/Cargo.toml", "utf8");
assert.match(rootManifest, /"vize_canon\/profiling"/u);
assert.match(canonManifest, /profiling = \["native", "vize_l0\/micro-profiling"/u);
assert.match(profiling, /static OUTPUT_PATH: OnceLock<Option<PathBuf>>/u);
assert.equal((profiling.match(/std::env::var_os/gu) ?? []).length, 1, "one-time env lookup only");
assert.match(profiling, /get_or_init\(\|\| std::env::var_os/u);
assert.match(profiling, /global_profiler\(\)\.is_enabled\(\)/u);
assert.match(profiling, /digest\.update\(source_type\)/u);
assert.match(profiling, /digest\.update\(\(source\.len\(\) as u64\)\.to_le_bytes\(\)\)/u);
assert.match(profiling, /create_new\(true\)/u);
assert.match(profiling, /Sha256/u);
assert.match(
  macros,
  /#\[cfg\(not\(feature = "micro-profiling"\)\)\][\s\S]*\(\$name:literal, \$value:expr\) => \{\{\}\}/u,
);
assert.match(macroLaw, /ordinary builds must not evaluate counters/u);
assert.equal(
  git("rev-parse", sourceParent + ":.github/workflows/cold-typechecker-scale.yml"),
  git("rev-parse", sourceSha + ":.github/workflows/cold-typechecker-scale.yml"),
  "the existing cold workflow must remain byte-identical",
);
assert.equal(
  git("rev-parse", sourceParent + ":docs/davinci/plan/profile-export.schema.json"),
  git("rev-parse", sourceSha + ":docs/davinci/plan/profile-export.schema.json"),
  "the stable profile schema must not change",
);

for (const file of changed) {
  const lines = fs.readFileSync(file, "utf8").split("\n").length;
  const budget = file.endsWith("/profiling.rs") ? 420 : file.endsWith(".rs") ? 520 : 460;
  assert.ok(lines <= budget, file + ": source budget");
  assert.ok(!/\.(?:mjs|cjs|js|jsx|py|sh)$/u.test(file), "retired tooling source");
}
const sourceHashes = Object.fromEntries(
  changed.map((file) => [file, createHash("sha256").update(fs.readFileSync(file)).digest("hex")]),
);
const sourceBlobs = Object.fromEntries(
  changed.map((file) => [file, git("rev-parse", sourceSha + ":" + file)]),
);
const workflowPath = ".github/workflows/vts-source-census.yml",
  driverWorkflowBlob = git("rev-parse", driverSha + ":" + workflowPath),
  sourceWorkflowBlob = sourceBlobs[workflowPath];
console.log(
  JSON.stringify({
    sourceParent,
    sourceSha,
    changed,
    disabledProjectionMatchesParent: true,
    coldWorkflowUnchanged: true,
    profileSchemaUnchanged: true,
    oneTimeProfilingGate: true,
    sourceHashes,
    provenance: {
      commit: sourceSha,
      parent: sourceParent,
      tree: git("rev-parse", sourceSha + "^{tree}"),
      driver: { commit: driverSha, workflowBlob: driverWorkflowBlob },
      source: {
        commit: sourceSha,
        parent: sourceParent,
        tree: git("rev-parse", sourceSha + "^{tree}"),
        workflowBlob: sourceWorkflowBlob,
        blobs: sourceBlobs,
      },
      blobs: { workflow: driverWorkflowBlob, sourceWorkflow: sourceWorkflowBlob },
    },
  }),
);
