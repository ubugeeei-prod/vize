import path from "node:path";

import {
  type Entry,
  block,
  locales,
  read,
  rustFiles,
} from "./davinci-diagnostic-catalog-sources.ts";

// Producers whose messages are built per finding, so the TS-53 catalogue
// describes each code instead of templating it: croquis_cf, the S2/S3
// verifiers, the SFC type checker and the SFC compiler — plus the
// TypeScript-numbered type-checker codes, catalogued with a message and help.
// Every list is read from the producer's own source.

/** Every string literal in `file` matching `pattern`, distinct, in order. */
function literalCodes(file: string[], pattern: RegExp): string[] {
  return [...new Set([...read(...file).matchAll(pattern)].map((match) => match[1]))];
}

/** The codes of the producers that describe rather than template messages:
 * croquis_cf's `CrossFileDiagnostic::code()`, S2's and S3's
 * `ViolationCode::as_str()` arms (wherever in the crate they live), the SFC
 * type checker's and the SFC compiler's code literals. */
export function describedCodes(): Map<string, string[]> {
  return new Map([
    [
      "croquis_cf",
      literalCodes(
        ["crates", "vize_croquis_cf", "src", "diagnostics", "rules.rs"],
        /"(vize:croquis\/cf\/[a-z-]+)"/gu,
      ),
    ],
    ["s2 verifier", treeCodes(["crates", "vize_s2", "src"], /=> "(S2V\d{3})"/gu)],
    ["s3 verifier", treeCodes(["crates", "vize_impeto", "src"], /=> "(S3V\d{3})"/gu)],
    ["canon", canonCodes()],
    ["sfc", sfcCodes()],
  ]);
}

/** Every string literal in the Rust files under `dir` matching `pattern`. */
function treeCodes(dir: string[], pattern: RegExp): string[] {
  const codes = rustFiles(path.join(...dir))
    .sort()
    .flatMap((file) => [...read(file).matchAll(pattern)].map((match) => match[1]));
  return [...new Set(codes)];
}

/** `SfcTypeDiagnostic` codes: the literal `code: Some("…")` sites of the SFC
 * type checker and service, plus `SetupContextViolationKind::to_display()`. */
function canonCodes(): string[] {
  const literal = /code: Some\("([a-z-]+)"\.into\(\)\)/gu;
  const checker = treeCodes(["crates", "vize_canon", "src", "sfc_typecheck"], literal);
  const service = literalCodes(["crates", "vize_canon", "src", "typecheck_service.rs"], literal);
  const setup = read("crates", "vize_croquis", "src", "setup_context.rs");
  const kinds = [...block(setup, "pub const fn to_display").matchAll(/=> "([a-z-]+)"/gu)];
  const codes = [...checker, ...service, ...kinds.map((match) => match[1])];
  return [...new Set(codes)].map((code) => `canon/${code}`);
}

/** `SfcError.code` literals of the SFC compiler and the block parser. */
function sfcCodes(): string[] {
  const compiler = treeCodes(
    ["crates", "vize_atelier_sfc", "src"],
    /code: Some\("([A-Z][A-Z_]+)"\.(?:into|to_compact_string)\(\)\)/gu,
  );
  const parser = treeCodes(["crates", "vize_croquis", "src", "sfc"], /"([A-Z][A-Z_]{4,})"/gu);
  return [...new Set([...compiler, ...parser])].map((code) => `sfc/${code}`);
}

/** `TypeErrorCode::help_key()` arms: the TypeScript-numbered codes. */
export function typeScriptCodes(): string[] {
  const source = read("crates", "vize_canon", "src", "diagnostic.rs");
  const arms = block(source, "pub const fn help_key").matchAll(/=> "(ts\/[a-z0-9/]+)\.help"/gu);
  return [...arms].map((match) => match[1]);
}

/** Every TypeScript-numbered code missing a message or help in a locale. */
export function typeScriptProblems(codes: string[], legacy: Map<string, Partial<Entry>>): string[] {
  return codes.flatMap((code) =>
    ["message", "help"].flatMap((part) =>
      locales
        .filter((locale) => (legacy.get(`${code}.${part}`)?.[locale] ?? "").trim() === "")
        .map((locale) => `\`${code}.${part}\` is missing in ${locale}`),
    ),
  );
}

/** Every described code without a description in the catalogue. */
export function describedProblems(
  codes: Map<string, string[]>,
  entries: Array<[string, Entry]>,
): string[] {
  const table = new Map(entries);
  return [...codes.values()]
    .flat()
    .filter((code) => !table.has(`${code}.description`))
    .map((code) => `\`${code}\` has no description`);
}
