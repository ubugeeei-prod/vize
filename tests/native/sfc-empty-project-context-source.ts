import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { existsSync, readFileSync } from "node:fs";

interface SourceChange {
  path: string;
  referenceBlob: string;
  currentBlob: string;
  replacements: { reference: string; current: string }[];
}

const bytes = readFileSync(
  new URL(
    "../fixtures/sfc/parsed-attributes/empty-compile-project-context-source.json",
    import.meta.url,
  ),
);
assert.equal(
  createHash("sha256").update(bytes).digest("hex"),
  "bc09805ad94f54c988ba33e3251fa6f9ab83ccfa06d137eeaf6f40a611923e0c",
  "the reviewed project-context transformation must remain complete",
);
const plan = JSON.parse(bytes.toString("utf8")) as {
  schema: string;
  sources: SourceChange[];
  supportingSources: Record<string, string>;
};
assert.equal(plan.schema, "vize.sfc-empty-compile-project-context-source-successor.v1");

export const emptyProjectContextSupportingBlobs = plan.supportingSources;

/**
 * The bytes of a source this law reviewed. A source that has since moved on
 * is authenticated through its frozen copy under `tests/_fixtures/byte-custody`.
 */
export function readReviewedSource(file: string): Buffer {
  const frozen = new URL(`../_fixtures/byte-custody/${file}`, import.meta.url);
  return readFileSync(existsSync(frozen) ? frozen : new URL(`../../${file}`, import.meta.url));
}

function blob(bytes: Uint8Array): string {
  return createHash("sha1")
    .update(Buffer.from(`blob ${bytes.length}\0`))
    .update(bytes)
    .digest("hex");
}

/** Authenticate the ownership-only dependency conversion before the older law. */
function dependencyOwnershipPredecessor(file: string, bytes: Uint8Array): Uint8Array {
  const expected =
    file === "crates/vize_vitrine/src/napi/sfc/compile.rs"
      ? ["20a0b1d7a67747d64ee22ad54d2c37211d45de26", "8ec4b58ac5b5fa47c2c1b3859368c08b8481152a"]
      : file === "crates/vize_vitrine/src/napi/sfc/types/conversions.rs"
        ? ["e6defb2f305801c24eb8979bb633278afdf68fb0", "c3a1d0e4cfffa709e5512252701cd1385b597d1b"]
        : undefined;
  if (!expected || blob(bytes) !== expected[0]) return bytes;
  let text = Buffer.from(bytes).toString("utf8");
  const replace = (current: string, reference: string) => {
    assert.equal(text.split(current).length, 2, "unique reviewed dependency ownership edit");
    text = text.replace(current, reference);
  };
  if (file.endsWith("/compile.rs")) {
    replace(
      "dependencies_to_napi(result.dependencies)",
      "dependencies_to_napi(&result.dependencies)",
    );
  } else {
    replace("dependencies: Vec<vize_l0::String>", "dependencies: &[impl AsRef<str>]");
    replace(
      "let mut paths: Vec<String> = dependencies\n            .into_iter()",
      "let mut paths: Vec<String> = dependencies\n            .iter()",
    );
    replace(
      "                // The compile result no longer needs these paths. Moving a\n" +
        "                // heap-backed CompactString into String keeps its allocation.\n" +
        "                vize::type_projects::plain_windows_path(path.as_str())\n" +
        "                    .map_or_else(|| path.into(), Into::into)",
      "                let path = path.as_ref();\n" +
        "                vize::type_projects::plain_windows_path(path)\n" +
        "                    .map_or_else(|| path.to_owned(), Into::into)",
    );
    const start = text.indexOf("\n#[cfg(test)]\nmod tests {");
    assert(start >= 0, "reviewed converter's complete trailing test module");
    text = text.slice(0, start);
  }
  const predecessor = Buffer.from(text);
  assert.equal(blob(predecessor), expected[1], "dependency packet law keeps its exact predecessor");
  return predecessor;
}

function supportingSourceForReference(file: string, bytes: Uint8Array): Uint8Array {
  bytes = dependencyOwnershipPredecessor(file, bytes);
  if (
    file === "crates/vize/src/type_projects.rs" &&
    blob(bytes) === "0adbbb012dba0be8ee90c36c234f0cb12ebba1f2"
  ) {
    // Reconstruct the complete old normalization for the existing source law.
    // Current watch-list behavior is compared to it by the Rust differential
    // test; this source inverse grants no execution or comparison credit.
    let text = Buffer.from(bytes).toString("utf8");
    for (const [current, reference] of [
      ["mod dependencies;\n", ""],
      [
        "            dependencies::include_root(&mut result.dependencies, &capture.root.to_string_lossy());\n",
        "            result\n" +
          "                .dependencies\n" +
          "                .push(capture.root.to_string_lossy().as_ref().into());\n" +
          "            result.dependencies.sort();\n" +
          "            result.dependencies.dedup();\n",
      ],
    ]) {
      assert.equal(text.split(current!).length, 2, "unique reviewed watch-list normalization edit");
      text = text.replace(current!, reference!);
    }
    bytes = Buffer.from(text);
    assert.equal(blob(bytes), "4d567c1bf5eff482c9ce78d02d30761514973cc1");
  }
  if (
    file === "crates/vize_atelier_sfc/src/compile/script_context.rs" &&
    blob(bytes) === "def91c11cd7e5c5f5c0531f0a0de67db2c300352"
  ) {
    // #13847 carries each imported type's owning binding into resolution.
    // Reconstruct the complete older source for its existing reference law;
    // this source-only inverse grants no execution or comparison credit.
    let text = Buffer.from(bytes).toString("utf8");
    const declaration = "    let mut bindings = Vec::new();\n";
    const collection = "        bindings.extend(collected.bindings);\n";
    const current = "context.merge_referenced_declarations(&bindings, &declarations, &texts);";
    assert.equal(text.split(declaration).length, 2);
    assert.equal(text.split(collection).length, 3);
    assert.equal(text.split(current).length, 2);
    text = text
      .replace(declaration, "")
      .replaceAll(collection, "")
      .replace(current, "context.merge_referenced_declarations(&declarations, &texts);");
    const reference = Buffer.from(text);
    assert.equal(blob(reference), "1be40c6336989a2c96ae8f51e4844bbecb2b06e4");
    return reference;
  }
  if (
    file === "crates/vize_vitrine/src/napi/sfc/types/conversions.rs" &&
    blob(bytes) === "c3a1d0e4cfffa709e5512252701cd1385b597d1b"
  ) {
    const current = `/// Watch inputs for the host: one file can be reached both canonically
/// (\\\\?\\C:\\… on Windows) and as authored, so list its plain spelling once.
pub(crate) fn dependencies_to_napi(dependencies: &[impl AsRef<str>]) -> Option<Vec<String>> {
    (!dependencies.is_empty()).then(|| {
        let mut paths: Vec<String> = dependencies
            .iter()
            .map(|path| {
                let path = path.as_ref();
                vize::type_projects::plain_windows_path(path)
                    .map_or_else(|| path.to_owned(), Into::into)
            })
            .collect();
        paths.sort_unstable();
        paths.dedup();
        paths
    })
}`;
    const reference = `pub(crate) fn dependencies_to_napi(dependencies: &[impl AsRef<str>]) -> Option<Vec<String>> {
    (!dependencies.is_empty()).then(|| {
        dependencies
            .iter()
            .map(|path| path.as_ref().to_owned())
            .collect()
    })
}`;
    const text = Buffer.from(bytes).toString("utf8");
    assert(current.includes("plain_windows_path"), "complete reviewed N-API host-path edit");
    const start = text.indexOf("/// Watch inputs for the host:");
    const end = text.indexOf("\n\npub(crate) fn macro_artifacts_to_napi", start);
    assert(start >= 0 && end > start, "unique reviewed N-API host-path boundary");
    const restored = Buffer.from(text.slice(0, start) + reference + text.slice(end));
    assert.equal(blob(restored), "5c313a0057306e74f031e7232f33472e86799e7a");
    return restored;
  }
  if (
    file === "crates/vize_vitrine/src/napi/sfc/types.rs" &&
    blob(bytes) === "0399155317ff085588b2cfc8e52fc261dac0efae"
  ) {
    // #13074: compile options name a registered `vue.plugins` plan by digest.
    const field =
      "    /// Digest of a `vue.plugins` plan registered with `registerPluginPlan`;\n" +
      "    /// its directive contracts apply to this compile (#13074).\n" +
      "    pub plugin_plan: Option<String>,\n";
    const text = Buffer.from(bytes).toString("utf8");
    assert.equal(text.split(field).length, 3, "reviewed plugin-plan option in both structs");
    const reference = Buffer.from(text.split(field).join(""));
    assert.equal(blob(reference), "9f76121fb52ce2eae282a3768a0f125b56a3d853");
    return reference;
  }
  if (
    file === "crates/vize/src/type_projects.rs" &&
    blob(bytes) === "4d567c1bf5eff482c9ce78d02d30761514973cc1"
  ) {
    let text = Buffer.from(bytes).toString("utf8");
    for (const [current, reference] of [
      ["mod host_path;\npub use host_path::{plain_windows_path, plain_windows_path_buf};\n\n", ""],
      [
        "    captured::owners(&host_path(path))",
        "    captured::owners(&vize_l0::typescript_project::normalize_path_lexically(path))",
      ],
      [
        "    captured::inputs(&host_path(root), revision, known_revision)\n}\n\n/// Captures index plain spellings; accept a verbatim one from older hosts.\nfn host_path(path: &Path) -> PathBuf {\n    vize_l0::typescript_project::normalize_path_lexically(&plain_windows_path_buf(\n        path.to_path_buf(),\n    ))\n}",
        "    captured::inputs(\n        &vize_l0::typescript_project::normalize_path_lexically(root),\n        revision,\n        known_revision,\n    )\n}",
      ],
    ]) {
      assert.equal(text.split(current!).length, 2, "unique reviewed host-path edit");
      text = text.replace(current!, reference!);
    }
    const reference = Buffer.from(text);
    assert.equal(blob(reference), "96073dbb55954781d825974cb3beff77b3fb9085");
    return reference;
  }
  if (file !== "crates/vize_atelier_sfc/src/compile/entry/root_match.rs") return bytes;
  let source = bytes;
  let text = Buffer.from(source).toString("utf8");
  // The jade profile forwards the pug lowering's warnings to the compile result.
  if (blob(bytes) === "a3d329411a5c3d0d4b85bd6ea8995757a82e7fdd") {
    for (const [current, reference] of [
      [
        "    let (descriptor, mut dependencies, pug_source_map, pug_warnings) =\n",
        "    let (descriptor, mut dependencies, pug_source_map) =\n",
      ],
      ["    result.warnings.extend(pug_warnings);\n", ""],
    ]) {
      assert.equal(text.split(current!).length, 2, "unique reviewed jade-profile edit");
      text = text.replace(current!, reference!);
    }
    source = Buffer.from(text);
    assert.equal(blob(source), "33e52e3d7b419f1af96562e7bbe078d9d6dfb54a");
  }
  // #13231 resolves the template lang through the block-language registry.
  if (blob(source) === "33e52e3d7b419f1af96562e7bbe078d9d6dfb54a") {
    const current =
      "    if template.src.is_some()\n        || !vize_l1::container::lang::is_html_template(template.lang.as_deref())\n    {\n";
    assert.equal(text.split(current).length, 2, "unique reviewed block-language edit");
    text = text.replace(
      current,
      '    if template.src.is_some() || template.lang.as_deref().is_some_and(|lang| lang != "html") {\n',
    );
    assert.equal(blob(Buffer.from(text)), "ebff50833e6b9b5fb2998bc7adf8a3874160eaf3");
  } else if (blob(source) !== "ebff50833e6b9b5fb2998bc7adf8a3874160eaf3") return bytes;
  for (const [current, reference] of [
    [
      "    Allocator, ParseMode, ParserOptions, PropNode, TemplateChildNode,",
      "    Allocator, ParserOptions, PropNode, TemplateChildNode,",
    ],
    ["        ParserOptions {\n            mode: ParseMode::Sfc,\n", "        ParserOptions {\n"],
  ]) {
    assert.equal(text.split(current!).length, 2, "unique reviewed SFC grammar edit");
    text = text.replace(current!, reference!);
  }
  const reference = Buffer.from(text);
  assert.equal(blob(reference), "7c678c8a0b4a29f173ad67ad298afd1cb92ede8b");
  return reference;
}

/** Reverse the exact setup-owner vocabulary change before older integrations. */
function setupOwnerPredecessor(file: string, bytes: Uint8Array): Uint8Array {
  if (
    file !== "crates/vize_atelier_sfc/src/compile.rs" ||
    blob(bytes) !== "6fe53a971db59aff6fac446803cf161e1d6054d8"
  )
    return bytes;
  const text = Buffer.from(bytes).toString("utf8");
  const edit = "setup_facts::SetupCompileAnalysis::new";
  assert.equal(text.split(edit).length, 2, "unique reviewed setup-owner rename");
  const predecessor = Buffer.from(text.replace(edit, "setup_facts::SetupAnalysis::new"));
  assert.equal(blob(predecessor), "b80c1e46486d2aaec325c748aae5b8791826daff");
  return predecessor;
}

/** Later reviewed edits to an integrated source, removed before its law is checked. */
function reviewedSourceSuccessor(file: string, bytes: Uint8Array): Uint8Array {
  bytes = dependencyOwnershipPredecessor(file, setupOwnerPredecessor(file, bytes));
  // The levels name the component render binding by concept (`_sfc_render` is
  // unchanged): `RenderFunctionName::SfcRender` became `ComponentRender`.
  if (
    file === "crates/vize_atelier_sfc/src/compile.rs" &&
    blob(bytes) === "b80c1e46486d2aaec325c748aae5b8791826daff"
  ) {
    const text = Buffer.from(bytes).toString("utf8");
    const edit = "RenderFunctionName::ComponentRender";
    assert.equal(text.split(edit).length, 2, "unique reviewed render-name edit");
    const reference = Buffer.from(text.replace(edit, "RenderFunctionName::SfcRender"));
    assert.equal(blob(reference), "54bc6c6b5137fe0df403883db93901970d8bc40a");
    return reference;
  }
  if (
    file !== "crates/vize_vitrine/src/napi/sfc/compile.rs" ||
    blob(bytes) !== "8ec4b58ac5b5fa47c2c1b3859368c08b8481152a"
  )
    return bytes;
  // #13074: a compile applies the directive table of its registered plan.
  let text = Buffer.from(bytes).toString("utf8");
  for (const edit of [
    "    let directive_contracts =\n" +
      "        crate::napi::plugin_plan::compile_directives(opts.plugin_plan.as_deref())?;\n",
    "        dom_options.directive_contracts = directive_contracts;\n",
  ]) {
    assert.equal(text.split(edit).length, 2, "unique reviewed plugin-plan edit");
    text = text.replace(edit, "");
  }
  const reference = Buffer.from(text);
  assert.equal(blob(reference), "074a7a36ebf219dbfc48f2abed51fc3a1a3a4fcd");
  return reference;
}

/** Authenticate only this reviewed integration, then retain the original law. */
export function emptyCompileSourcesForReference(
  sources: Record<string, Uint8Array>,
  referenceBlobs: Readonly<Record<string, string>>,
  supportingSources?: Record<string, Uint8Array>,
): Record<string, Uint8Array> {
  assert.deepEqual(Object.keys(sources).sort(), Object.keys(referenceBlobs).sort());
  // The original complete bundle still uses its original validation directly.
  // In particular it does not require any new project-context source files.
  if (Object.entries(referenceBlobs).every(([file, expected]) => blob(sources[file]) === expected))
    return sources;

  assert.deepEqual(
    plan.sources.map((source) => source.path).sort(),
    Object.keys(referenceBlobs).sort(),
  );
  const supporting =
    supportingSources ??
    Object.fromEntries(
      Object.keys(plan.supportingSources).map((file) => [file, readReviewedSource(file)]),
    );
  assert.deepEqual(Object.keys(supporting).sort(), Object.keys(plan.supportingSources).sort());
  for (const [file, expected] of Object.entries(plan.supportingSources))
    assert.equal(
      blob(supportingSourceForReference(file, supporting[file])),
      expected,
      `reviewed project-context dependency: ${file}`,
    );

  const restored: Record<string, Uint8Array> = {};
  for (const source of plan.sources) {
    assert.equal(source.referenceBlob, referenceBlobs[source.path]);
    const current = reviewedSourceSuccessor(source.path, sources[source.path]!);
    assert.equal(blob(current), source.currentBlob, `reviewed integration: ${source.path}`);
    let text = Buffer.from(current).toString("utf8");
    for (const replacement of [...source.replacements].reverse()) {
      assert.notEqual(replacement.current, "");
      const start = text.indexOf(replacement.current);
      assert(start >= 0, `missing reviewed integration: ${source.path}`);
      assert.equal(start, text.lastIndexOf(replacement.current), "ambiguous reviewed integration");
      text = text.replace(replacement.current, replacement.reference);
    }
    restored[source.path] = Buffer.from(text);
    assert.equal(blob(restored[source.path]), source.referenceBlob, "original law reconstruction");
  }
  return restored;
}
