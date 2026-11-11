import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";

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
  "d70132445182787424fddc589f261207817bd74b86a3e3ec091b0c38cfb5e8d2",
  "the reviewed project-context transformation must remain complete",
);
const plan = JSON.parse(bytes.toString("utf8")) as {
  schema: string;
  sources: SourceChange[];
  supportingSources: Record<string, string>;
};
assert.equal(plan.schema, "vize.sfc-empty-compile-project-context-source-successor.v1");

export const emptyProjectContextSupportingBlobs = plan.supportingSources;

function blob(bytes: Uint8Array): string {
  return createHash("sha1")
    .update(Buffer.from(`blob ${bytes.length}\0`))
    .update(bytes)
    .digest("hex");
}

function supportingSourceForReference(file: string, bytes: Uint8Array): Uint8Array {
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
  if (
    file !== "crates/vize_atelier_sfc/src/compile/entry/root_match.rs" ||
    blob(bytes) !== "ebff50833e6b9b5fb2998bc7adf8a3874160eaf3"
  )
    return bytes;
  let text = Buffer.from(bytes).toString("utf8");
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
      Object.keys(plan.supportingSources).map((file) => [
        file,
        readFileSync(new URL(`../../${file}`, import.meta.url)),
      ]),
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
    const current = sources[source.path];
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
