/** Exact shared-declaration import for one source or published SDK. */
import assert from "node:assert/strict";

export const CHECKER_SDK_ENTRY = "vize/virtual-ts-checker";
export const PUBLISHED_SDK_ENTRY = "vize/virtual-ts";
const CHECKER_DECLARATION = "virtual-ts-checker.d.ts";
const PUBLISHED_DECLARATION = "virtual-ts.d.ts";
const CHECKER_EXPORT = "./virtual-ts-checker";
const PUBLISHED_EXPORT = "./virtual-ts";
const SHIPPED_MODULE = "exercise the actual shipped shared declaration module";
const EMBEDDED_HELPER = "embedded helpers would hide the published regression";
const EMPTY_TYPE_IMPORT = /import type \{\} from "([^"]+)";/gu;

export type SdkDeclarationContract = {
  mode: "source" | "published";
  exports: Readonly<Record<string, unknown>> | undefined;
  files: readonly string[];
};

function declaresFile(condition: unknown, file: string): boolean {
  if (typeof condition === "string") return condition === `./${file}`;
  if (!condition || typeof condition !== "object") return false;
  return Object.values(condition).some((value) => declaresFile(value, file));
}

function ships(contract: SdkDeclarationContract, file: string, exportName: string): boolean {
  return contract.files.includes(file) && declaresFile(contract.exports?.[exportName], file);
}

/** Checker entry for source, and for a published package that ships it. */
export function sharedDeclarationEntry(contract: SdkDeclarationContract): string {
  const checker = ships(contract, CHECKER_DECLARATION, CHECKER_EXPORT);
  if (contract.mode === "source" || contract.files.includes(CHECKER_DECLARATION)) {
    assert.ok(checker, SHIPPED_MODULE);
    return CHECKER_SDK_ENTRY;
  }
  assert.ok(ships(contract, PUBLISHED_DECLARATION, PUBLISHED_EXPORT), SHIPPED_MODULE);
  return PUBLISHED_SDK_ENTRY;
}

/** One exact entry. A missing, extra, or embedded helper does not qualify. */
export function assertShippedDeclarationModule(text: string, entry: string): void {
  assert.ok(entry === CHECKER_SDK_ENTRY || entry === PUBLISHED_SDK_ENTRY, SHIPPED_MODULE);
  const expected = `import type {} from "${entry}";`;
  assert.equal(text.split(expected).length, 2, SHIPPED_MODULE);
  assert.deepEqual(
    [...text.matchAll(EMPTY_TYPE_IMPORT)].map((match) => match[1]),
    [entry],
    SHIPPED_MODULE,
  );
  assert.ok(!text.includes("vize:virtual-ts"), SHIPPED_MODULE);
  assert.ok(!text.includes("type $PropValue<"), EMBEDDED_HELPER);
}
