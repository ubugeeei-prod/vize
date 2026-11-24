/**
 * Fails when a Linux GNU release image asks the dynamic loader for more than
 * the documented glibc floor (`tools/support/github/glibc-floor.ts`).
 *
 * Usage: node tools/ci/commands/github/verify-glibc-symbols.ts [--max 2.17] <elf>...
 *
 * Every image is read in process (no binutils needed). It fails on any
 * `GLIBC_x.y` symbol version above the floor (naming the symbols that need
 * it), on packed relative relocations (`DT_RELR`, which only glibc 2.36+
 * loads), on private or unknown glibc versions, on `libgcc_s` versions newer
 * than the floor's GCC 4.8 runtime, and on shared libraries outside the base
 * glibc runtime.
 */
import { readElfFileDynamicRequirements } from "../../../support/github/elf-dynamic-requirements.ts";
import type { ElfDynamicRequirements } from "../../../support/github/elf-dynamic-requirements.ts";
import { linuxGnuGlibcFloor } from "../../../support/github/glibc-floor.ts";
import { isEntrypoint, runEntrypoint } from "../../../support/command-runtime.ts";

/** libgcc_s shipped with GCC 4.8, the compiler of the glibc 2.17 distributions (RHEL/CentOS 7). */
const libgccCeiling = "4.8.0";

const baseRuntimeLibraries = new Set([
  "libc.so.6",
  "libm.so.6",
  "libgcc_s.so.1",
  "libpthread.so.0",
  "libdl.so.2",
  "librt.so.1",
  "libutil.so.1",
  "ld-linux-x86-64.so.2",
  "ld-linux-aarch64.so.1",
]);

export interface GlibcVersion {
  readonly major: number;
  readonly minor: number;
  readonly patch: number;
  readonly text: string;
}

export function compareGlibcVersions(left: GlibcVersion, right: GlibcVersion): number {
  return left.major - right.major || left.minor - right.minor || left.patch - right.patch;
}

export function parseGlibcVersion(value: string): GlibcVersion {
  const match = /^(\d+)\.(\d+)(?:\.(\d+))?$/u.exec(value);
  if (match === null) {
    throw new Error(`glibc version must look like MAJOR.MINOR[.PATCH], got ${value}`);
  }
  const major = Number(match[1]);
  const minor = Number(match[2]);
  const patch = match[3] === undefined ? 0 : Number(match[3]);
  return { major, minor, patch, text: `${major}.${minor}${patch === 0 ? "" : `.${patch}`}` };
}

export interface GlibcFloorReport {
  /** Highest `GLIBC_x.y` the image requires, if any. */
  readonly highest: GlibcVersion | undefined;
  readonly violations: readonly string[];
}

function symbolList(symbols: readonly string[]): string {
  if (symbols.length === 0) return "";
  const shown = symbols.slice(0, 8).join(", ");
  return ` (${shown}${symbols.length > 8 ? `, +${symbols.length - 8} more` : ""})`;
}

export function evaluateGlibcFloor(
  image: ElfDynamicRequirements,
  floor: GlibcVersion,
): GlibcFloorReport {
  const violations: string[] = [];
  const libgcc = parseGlibcVersion(libgccCeiling);
  let highest: GlibcVersion | undefined;
  for (const library of image.needed) {
    if (!baseRuntimeLibraries.has(library)) {
      violations.push(`links ${library}, which is not part of the base glibc runtime`);
    }
  }
  for (const { library, version, symbols } of image.requirements) {
    const glibc = /^GLIBC_(\d+\.\d+(?:\.\d+)?)$/u.exec(version);
    const gcc = /^GCC_(\d+\.\d+(?:\.\d+)?)$/u.exec(version);
    if (glibc !== null) {
      const parsed = parseGlibcVersion(glibc[1]!);
      if (highest === undefined || compareGlibcVersions(parsed, highest) > 0) highest = parsed;
      if (compareGlibcVersions(parsed, floor) > 0) {
        violations.push(
          `requires ${version} from ${library}${symbolList(symbols)}, above the glibc ${floor.text} floor`,
        );
      }
    } else if (gcc !== null && library === "libgcc_s.so.1") {
      if (compareGlibcVersions(parseGlibcVersion(gcc[1]!), libgcc) > 0) {
        violations.push(
          `requires ${version} from ${library}${symbolList(symbols)}, newer than the GCC ${libgccCeiling} runtime of glibc ${floor.text} systems`,
        );
      }
    } else if (version === "GLIBC_ABI_DT_RELR") {
      violations.push(
        "requires GLIBC_ABI_DT_RELR (packed relative relocations), which only glibc 2.36+ provides",
      );
    } else {
      violations.push(`requires ${version} from ${library}${symbolList(symbols)}`);
    }
  }
  if (image.packedRelativeRelocations && !violations.some((line) => line.includes("DT_RELR"))) {
    violations.push("uses packed relative relocations (DT_RELR), which only glibc 2.36+ loads");
  }
  if (highest === undefined) violations.push("has no GLIBC_* version requirements");
  return { highest, violations };
}

function parseArguments(arguments_: readonly string[]): {
  files: string[];
  floor: GlibcVersion;
} {
  let floor = linuxGnuGlibcFloor;
  const files: string[] = [];
  for (let index = 0; index < arguments_.length; index += 1) {
    const argument = arguments_[index]!;
    if (argument === "--max") {
      floor = arguments_[index + 1] ?? "";
      if (floor === "") throw new Error("--max requires a value");
      index += 1;
    } else {
      files.push(argument);
    }
  }
  return { files, floor: parseGlibcVersion(floor) };
}

export function verifyGlibcSymbols(arguments_: readonly string[]): number {
  const { files, floor } = parseArguments(arguments_);
  if (files.length === 0) {
    throw new Error(
      `Usage: node tools/ci/commands/github/verify-glibc-symbols.ts [--max ${linuxGnuGlibcFloor}] <elf>...`,
    );
  }
  let failed = false;
  for (const file of files) {
    let image: ElfDynamicRequirements;
    try {
      image = readElfFileDynamicRequirements(file);
    } catch (error) {
      console.error(`::error::${error instanceof Error ? error.message : String(error)}`);
      failed = true;
      continue;
    }
    const { highest, violations } = evaluateGlibcFloor(image, floor);
    if (violations.length > 0) {
      for (const violation of violations) console.error(`::error::${file} ${violation}`);
      failed = true;
    } else {
      console.log(
        `${file}: ${image.machine}, GLIBC_${highest!.text} <= glibc ${floor.text} floor; needs ${image.needed.join(", ")}`,
      );
    }
  }
  if (failed) throw new Error(`glibc ${floor.text} floor verification failed`);
  return 0;
}

if (isEntrypoint(import.meta.url)) {
  runEntrypoint(() => verifyGlibcSymbols(process.argv.slice(2)));
}
