import { parse as parseToml } from "@iarna/toml";
import {
  coreRoot,
  hostRoot,
  declarations,
  compiler,
  coreTests,
  hostTests,
  testMarker,
  wrappers,
  hostScope,
  changes,
  calls,
  edges,
} from "./i18n-host-contract.ts";

type Reader = (path: string) => string;
type Exists = (path: string) => boolean;
export const compact = (source: string) => source.replace(/\s+/gu, "");
function requireState(condition: boolean, message: string) {
  if (!condition) throw new Error(message);
}
function expectMigration(
  source: string,
  before: string,
  after: string,
  old: boolean,
  label: string,
) {
  const valid = old
    ? source.includes(before) && (!after || !source.includes(after))
    : source.includes(after) && (after.includes(before) || !source.includes(before));
  requireState(valid, `unexpected i18n integration state: ${label}`);
}
export function lockPackage(lock: string, name: string) {
  const start = lock.indexOf(`name = "${name}"\n`);
  const end = lock.indexOf("\n[[package]]", start);
  requireState(start >= 0 && end > start, `missing lock package: ${name}`);
  return lock.slice(start, end);
}

type Table = Record<string, unknown>;
const isTable = (value: unknown): value is Table =>
  !!value && typeof value === "object" && !Array.isArray(value);
export function validateManifestEdges(read: Reader, old: boolean) {
  for (const [name, additions] of edges) {
    let manifest: Table;
    try {
      manifest = parseToml(read(`crates/${name}/Cargo.toml`));
    } catch {
      throw new Error("malformed host manifest: " + name);
    }
    for (const dependency of additions) {
      const found: { key: string; path: string; value: unknown }[] = [];
      function visit(table: Table, path: string) {
        for (const [key, value] of Object.entries(table)) {
          if (!isTable(value)) continue;
          if (["dependencies", "dev-dependencies", "build-dependencies"].includes(key)) {
            for (const [alias, declaration] of Object.entries(value))
              if (
                alias === dependency ||
                (isTable(declaration) && declaration.package === dependency)
              )
                found.push({ key: alias, path: path + key, value: declaration });
          } else visit(value, path + key + ".");
        }
      }
      visit(manifest, "");
      const entry = found[0];
      requireState(
        old
          ? found.length === 0
          : found.length === 1 &&
              entry.key === dependency &&
              entry.path === "dependencies" &&
              isTable(entry.value) &&
              Object.keys(entry.value).length === 1 &&
              entry.value.workspace === true,
        "unexpected host manifest ownership: " + name + " -> " + dependency,
      );
    }
  }
}

/** Validate every accepted state before performing a single replay write. */
export function preflight(read: Reader, exists: Exists): string {
  const core = read(coreRoot + "lib.rs"),
    host = read(hostRoot + "lib.rs");
  const hasModules = (source: string) => /\bmod i18n(?:_|;)/u.test(source);
  const old =
    core.includes(declarations) && !hasModules(core.replace(declarations, "")) && !hasModules(host);
  const next =
    !hasModules(core) && host.includes(declarations) && !hasModules(host.replace(declarations, ""));
  requireState(old || next, "unexpected i18n module ownership");
  validateManifestEdges(read, old);
  requireState(
    old
      ? host.includes("pub mod config;\n") && !host.includes(hostScope)
      : host.includes(hostScope),
    "unexpected host lint scope",
  );
  expectMigration(
    host,
    "Canonical implementations live in `vize_l0`.",
    "Portable storage identities are re-exported from `vize_l0`.",
    old,
    "host storage documentation",
  );
  const code = read(compiler);
  requireState(
    old
      ? code.includes(wrappers) && code.includes("use crate::i18n::{Locale, translator};\n")
      : !/crate::i18n|locale: Locale|translator\(/u.test(code),
    "unexpected compiler locale wrappers",
  );
  for (const [path, before, after] of changes)
    expectMigration(read(path), before, after, old, path);
  for (const [path, before, after] of calls) {
    const source = compact(read(path));
    const count = (call: string) => {
      const escaped = compact(call).replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
      return [...source.matchAll(new RegExp(`(?<![\\w.])${escaped}`, "gu"))].length;
    };
    requireState(
      old ? count(before) === 1 && count(after) === 0 : count(before) === 0 && count(after) === 1,
      "unexpected i18n provider call: " + path + ": " + before,
    );
  }
  for (const file of ["i18n.rs", "i18n/catalog.rs", "i18n/tests.rs"]) {
    const text = read(hostRoot + file);
    if (file !== "i18n.rs")
      expectMigration(text, "use alloc::borrow::Cow;", "use std::borrow::Cow;", old, file);
    expectMigration(
      text,
      "use crate::diag::MessageLookup;",
      "use vize_l0::diag::MessageLookup;",
      old,
      file,
    );
  }
  expectMigration(
    read(hostRoot + "i18n.rs"),
    "use vize_l0::i18n::{Locale, Translator};",
    "use vize_carton::i18n::{Locale, Translator};",
    old,
    "catalog example",
  );
  requireState(
    old
      ? !read(hostRoot + "i18n.rs").includes("mod compiler_error_tests;")
      : read(hostRoot + "i18n.rs").includes("mod compiler_error_tests;"),
    "unexpected host test declaration",
  );
  expectMigration(
    read(coreRoot + "compiler_error/codes.rs"),
    "The catalog (`vize_l0::i18n`)",
    "The host translation catalog",
    old,
    "compiler documentation",
  );
  const tests = read(coreTests);
  requireState(
    old
      ? tests.includes(testMarker) && !exists(hostTests)
      : !tests.includes(testMarker) && exists(hostTests),
    "unexpected compiler locale test ownership",
  );
  if (old)
    for (const kind of ["message", "help"])
      requireState(
        tests.includes(
          `            assert_eq!(code.localized_${kind}(locale).as_str(), ${kind});\n`,
        ),
        "unexpected extracted locale test",
      );
  else
    requireState(
      !/localized_(?:message|help)\(locale\)/u.test(read(hostTests)),
      "retired host test API",
    );
  const catalog = read("crates/vize/src/commands/explain/catalog.rs");
  requireState(catalog.includes("pub(crate) const fn locale("), "missing explain insertion anchor");
  requireState(
    old
      ? !catalog.includes("pub(crate) fn messages(")
      : compact(catalog).includes("self.translator.for_locale(self.locale)"),
    "unexpected explain provider binding",
  );
  const wasm = read("crates/vize_vitrine/src/wasm/lint.rs");
  for (const [before, after] of [
    ["L0Locale", "TranslationLocale"],
    ["l0_locale", "translation_locale"],
    ["Convert to L0 locale for i18n.", "Bind the host translation locale."],
  ])
    expectMigration(wasm, before, after, old, "wasm locale");
  const reader = read("tests/tooling/davinci-diagnostic-catalog-sources.ts");
  for (const suffix of ["file)", '"i18n",'])
    expectMigration(
      reader,
      `read("davinci", "vize_l0", "src", ${suffix}`,
      `read("crates", "vize_carton", "src", ${suffix}`,
      old,
      "catalog source reader",
    );
  let lock = read("Cargo.lock");
  for (const dependency of ["once_cell", "rustc-hash"])
    requireState(
      lockPackage(lock, "vize_l0").includes(` "${dependency}",\n`),
      "retained core dependency: " + dependency,
    );
  for (const [name, additions] of edges) {
    const entry = lockPackage(lock, name),
      match = entry.match(/dependencies = \[\n([\s\S]*?)\n\]/u);
    requireState(!!match, "unexpected lock dependencies: " + name);
    const lines = match![1].split("\n");
    requireState(
      lines.every((line) => /^ "[^"\n]+",$/u.test(line)),
      "malformed lock dependency: " + name,
    );
    const dependencies = new Set(lines);
    for (const dependency of additions) {
      const edge = ` "${dependency}",`;
      requireState(
        old ? !dependencies.has(edge) : dependencies.has(edge),
        "unexpected lock ownership: " + name + " -> " + dependency,
      );
      dependencies.add(edge);
    }
    lock = lock.replace(
      entry,
      entry.replace(match![0], "dependencies = [\n" + [...dependencies].sort().join("\n") + "\n]"),
    );
  }
  return lock;
}
