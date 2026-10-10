import { createHash } from "node:crypto";
import type { ArtVariant } from "./types/art.js";
import { toPascalCase } from "./utils.js";

/** Preserve ordinary bindings and disambiguate every normalized collision. */
export function variantComponentNames(
  variants: readonly Pick<ArtVariant, "name">[],
): ReadonlyMap<string, string> {
  const names = new Map<string, string>();
  const counts = new Map<string, number>();
  for (const { name } of variants) {
    if (names.has(name)) throw new Error(`Duplicate Musea variant name: ${JSON.stringify(name)}`);
    const binding = toPascalCase(name);
    names.set(name, binding);
    counts.set(binding, (counts.get(binding) ?? 0) + 1);
  }
  for (const [name, binding] of names) {
    if (counts.get(binding)! > 1) {
      const hash = createHash("sha256").update(JSON.stringify(name)).digest("hex");
      names.set(name, `__MuseaVariant_${hash}`);
    }
  }
  return names;
}
