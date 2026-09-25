import { existsSync, readFileSync, readdirSync } from "node:fs";
import path from "node:path";

import { isRelativeSpecifier, packageNameOfSpecifier, scanModuleSpecifiers } from "./imports.ts";
import type { LibRegistryExample } from "./types.ts";

/** Directory, next to an item's entry file, that holds its usage examples. */
export const EXAMPLES_DIRECTORY = "examples";

/**
 * Family that owns an example file: examples are named `<family>-<slug>.vue`
 * (or `<family>.vue`), and when several families share one directory the
 * longest matching family name wins (`icon-button-basic.vue` belongs to
 * `icon-button`, not `icon`).
 */
export function exampleOwner(fileName: string, families: readonly string[]): string | null {
  const stem = fileName.replace(/\.vue$/, "");
  const owners = families.filter((family) => stem === family || stem.startsWith(`${family}-`));
  return owners.sort((left, right) => right.length - left.length)[0] ?? null;
}

/** `switch-controlled-state.vue` (family `switch`) -> `Controlled state`. */
export function exampleTitle(fileName: string, family = ""): string {
  let stem = fileName.replace(/\.vue$/, "");
  if (family !== "" && stem.startsWith(`${family}-`)) stem = stem.slice(family.length + 1);
  else if (stem === family) stem = "basic";
  const sentence = stem.split("-").filter(Boolean).join(" ");
  return sentence.charAt(0).toUpperCase() + sentence.slice(1);
}

/** The leading `<!-- … -->` comment of an example, collapsed to one line. */
export function exampleDescription(source: string): string {
  const match = /^\s*<!--([\s\S]*?)-->/.exec(source);
  return match?.[1]?.replace(/\s+/g, " ").trim() ?? "";
}

/** Result of scanning one item's examples. */
export interface CollectedExamples {
  readonly examples: readonly LibRegistryExample[];
  readonly bytes: ReadonlyMap<string, Uint8Array>;
}

/**
 * Collect the `<entry dir>/examples/<family>-*.vue` files of one item.
 *
 * Examples are unstyled usage demos: every relative import must resolve to
 * a file of the item's registry closure (so a pulled example runs against the
 * pulled sources) and every bare import must be an allowed npm package.
 */
export function collectExamples(
  sourceRoot: string,
  family: string,
  siblings: readonly string[],
  entry: string,
  closureFiles: ReadonlySet<string>,
  allowedPackages: ReadonlySet<string>,
  sha256Hex: (data: Uint8Array) => string,
): CollectedExamples {
  const directory = path.posix.join(path.posix.dirname(entry), EXAMPLES_DIRECTORY);
  const absolute = path.join(sourceRoot, ...directory.split("/"));
  if (!existsSync(absolute)) return { examples: [], bytes: new Map() };
  const names = readdirSync(absolute)
    .filter((name) => name.endsWith(".vue") && exampleOwner(name, siblings) === family)
    .sort((left, right) => (left < right ? -1 : left > right ? 1 : 0));
  const bytes = new Map<string, Uint8Array>();
  const examples = names.map((name): LibRegistryExample => {
    const file = `${directory}/${name}`;
    const content = readFileSync(path.join(absolute, name));
    const source = content.toString("utf8");
    for (const specifier of scanModuleSpecifiers(file, source)) {
      if (isRelativeSpecifier(specifier)) {
        const target = path.posix.normalize(path.posix.join(directory, specifier));
        if (!closureFiles.has(target)) {
          throw new Error(
            `${file}: imports ${specifier}, which is outside the item's registry closure`,
          );
        }
      } else if (!allowedPackages.has(packageNameOfSpecifier(specifier))) {
        throw new Error(`${file}: imports undeclared package "${specifier}"`);
      }
    }
    bytes.set(file, content);
    return {
      path: file,
      title: exampleTitle(name, family),
      description: exampleDescription(source),
      sha256: sha256Hex(content),
      size: content.byteLength,
    };
  });
  return { examples, bytes };
}
