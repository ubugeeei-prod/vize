// Canon and S3 producer readers for the TS-53 catalog check. Split from
// davinci-diagnostic-catalog-sources.ts so both files stay under the
// source-length budget.

import {
  type Entry,
  block,
  locales,
  read,
  stringArms,
} from "./davinci-diagnostic-catalog-sources.ts";

export interface CanonCodes {
  variants: string[];
  /** Variant name to its `help_key()` string. */
  help: Map<string, string>;
  /** Variant name to the enum discriminant. */
  numbers: Map<string, number>;
}

/** `vize_canon::TypeErrorCode`: variants, discriminants and `help_key()` arms. */
export function parseCanonCodes(): CanonCodes {
  const source = read("crates", "vize_canon", "src", "diagnostic.rs");
  const variants: string[] = [];
  const numbers = new Map<string, number>();
  for (const match of block(source, "pub enum TypeErrorCode").matchAll(/^\s*(\w+) = (\d+),$/gmu)) {
    variants.push(match[1]);
    numbers.set(match[1], Number(match[2]));
  }
  return {
    variants,
    numbers,
    help: stringArms(block(source, "pub const fn help_key"), "Self"),
  };
}

/** Canon codes whose help or message text is missing in any locale. */
export function canonProblems(
  canon: CanonCodes,
  legacy: Map<string, Partial<Entry>>,
): string[] {
  const problems: string[] = [];
  if (canon.help.size !== canon.variants.length) {
    problems.push("TypeErrorCode::help_key does not cover every variant");
  }
  for (const variant of canon.variants) {
    const number = canon.numbers.get(variant);
    const help = canon.help.get(variant);
    const expected =
      number !== undefined && number >= 9000 ? `ts/vue/${number}.help` : `ts/${number}.help`;
    if (typeof help !== "string" || help !== expected) {
      problems.push(`TypeErrorCode::${variant} help key is not \`${expected}\``);
      continue;
    }
    for (const key of [help, help.replace(/\.help$/u, ".message")]) {
      const entry = legacy.get(key);
      for (const locale of locales) {
        const text = entry?.[locale];
        if (text === undefined || text.trim() === "") {
          problems.push(`\`${key}\` has no ${locale} catalog text`);
        }
      }
    }
  }
  return problems;
}

/** Every `vize:croquis/cf/…` code literal in `CrossFileDiagnostic::code`. */
export function parseCroquisCodes(): string[] {
  const body = block(
    read("crates", "vize_croquis_cf", "src", "diagnostics", "rules.rs"),
    "pub fn code",
  );
  return [...new Set([...body.matchAll(/"(vize:croquis\/cf\/[^"]+)"/gu)].map((match) => match[1]))].sort();
}

/** Croquis codes missing a message or help entry in any locale. */
export function croquisProblems(codes: string[], entries: Array<[string, Entry]>): string[] {
  const table = new Map(entries);
  const problems: string[] = [];
  for (const code of codes) {
    for (const part of ["message", "help"]) {
      const key = `${code}.${part}`;
      const entry = table.get(key);
      if (!entry) {
        problems.push(`\`${key}\` is not catalogued`);
        continue;
      }
      for (const locale of locales) {
        if (entry[locale].trim() === "") problems.push(`\`${key}\` is empty in ${locale}`);
      }
    }
  }
  return problems;
}

/** `ViolationCode::as_str` arms: variant name to `S3V00N`. */
export function parseS3Codes(): Map<string, string> {
  const source = read("crates", "vize_impeto", "src", "verify", "violation.rs");
  return stringArms(block(source, "pub const fn as_str"), "Self");
}

/** S3 verifier codes missing a description or help entry in any locale. */
export function s3Problems(codes: Map<string, string>, entries: Array<[string, Entry]>): string[] {
  const table = new Map(entries);
  const problems: string[] = [];
  const seen = [...codes.values()].sort();
  if (new Set(seen).size !== seen.length) problems.push("ViolationCode::as_str repeats a code");
  for (const code of seen) {
    for (const part of ["message", "help"]) {
      const key = `s3/${code}.${part}`;
      const entry = table.get(key);
      if (!entry) {
        problems.push(`\`${key}\` is not catalogued`);
        continue;
      }
      for (const locale of locales) {
        if (entry[locale].trim() === "") problems.push(`\`${key}\` is empty in ${locale}`);
      }
    }
  }
  return problems;
}
