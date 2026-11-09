/** Certified component-contract edits that already landed, so their packets audit exactly. */
import assert from "node:assert/strict";
import { beforeInstanceInputKeys } from "./instance-input-keys-transition.ts";

type Edit = { start: number; end: number; replacement: Buffer };

/** Whole component-helper images the audit admits, oldest first; the current image is last. */
export function certifiedHelperImages(path: string, current: string, older: string[]): string[] {
  // #12446 rewrote the TSX instance input-key set.
  const landed = [beforeInstanceInputKeys(path, current)];
  const known = new Set([...older, current]);
  return [...older, ...new Set(landed.filter((image) => !known.has(image))), current];
}

const owner = String.raw`__[A-Za-z_$][\w$]*_Component_\d+`;
const helper = String.raw`(?:import\("vize\/virtual-ts"\)\.)?`;
const ungated = new RegExp(
  String.raw` & (${helper})\$RequiredFallthroughProps<(${owner})>;\n`,
  "gu",
);

/** #12457: a contract gates its required fallthrough keys on every member admitting `{}`. */
export function requiredFallthroughGate(prefix: string, component: string): string {
  const fallthrough = `${prefix}$FallthroughProps<${component}>`;
  const required = `${prefix}$RequiredFallthroughProps<${component}, __F`;
  return ` & (${fallthrough} extends infer __F ? [__F extends unknown ? {} extends __F ? never : __F : never] extends [never] ? ${required}, never> : ${required}> : never);\n`;
}

/** Appends the gate edits for every ungated contract line `after` replaces with its gated form. */
export function withLandedEdits(before: string, after: string, edits: Edit[]): Edit[] {
  const gates: Edit[] = [];
  for (const match of before.matchAll(ungated)) {
    const replacement = requiredFallthroughGate(match[1]!, match[2]!);
    if (after.split(match[0]).length > 1 || after.split(replacement).length === 1) continue;
    const start = Buffer.byteLength(before.slice(0, match.index));
    const end = start + Buffer.byteLength(match[0]);
    assert.ok(
      !edits.some((edit) => edit.start < end && start < edit.end),
      "a gated contract line is disjoint from other certified edits",
    );
    gates.push({ start, end, replacement: Buffer.from(replacement) });
  }
  return gates.length > 0 ? [...edits, ...gates] : edits;
}
