/** Certified component-contract edits that already landed, so their packets audit exactly. */
import assert from "node:assert/strict";
import { beforeInstanceInputKeys } from "./instance-input-keys-transition.ts";
import { beforeUnownedOwnedKeys } from "./unowned-owned-keys-transition.ts";
import { propValueOrPreambleEdit } from "./prop-value-or-transition.ts";
import { dedentEdits } from "./setup-dedent.ts";
import { usageContractDeclarations } from "./usage-contract-transition.ts";

type Edit = { start: number; end: number; replacement: Buffer };

/** Whole component-helper images the audit admits, oldest first; the current image is last. */
export function certifiedHelperImages(path: string, current: string, older: string[]): string[] {
  // #12446 rewrote the TSX instance input-key set; #12467 hoisted the owned key set.
  const landed = [beforeInstanceInputKeys(path, current), beforeUnownedOwnedKeys(path, current)];
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

const navKey = String.raw`\.[^\s;,.\[\]]+|\['(?:[^'\\]|\\.)*'\]`;
const navBlock = new RegExp(
  String.raw`(  const (__vize_props_nav_\d+) = undefined as unknown as \S+ & )Record<string, unknown>;\n((?:  void \2(?:${navKey});\n)+)`,
  "gu",
);

/** #12531: every navigation statement on one ignored line, on the global `Record`. */
export function navigationBlock(prefix: string, owner: string, keys: string[]): string {
  const references = keys.map((key) => `void ${owner}${key};`).join(" ");
  return `${prefix}globalThis.Record<string, unknown>;\n  // @ts-ignore Navigation-only references; no diagnostic owner.\n  ${references}\n`;
}

/** The smallest edit turning `before` into `after`, as offsets inside them. */
function gapEdit(before: string, after: string): [number, number, string] | undefined {
  if (before === after) return undefined;
  let prefix = 0;
  while (prefix < before.length && before[prefix] === after[prefix]) prefix++;
  let suffix = 0;
  while (
    suffix < before.length - prefix &&
    suffix < after.length - prefix &&
    before[before.length - 1 - suffix] === after[after.length - 1 - suffix]
  )
    suffix++;
  return [prefix, before.length - suffix, after.slice(prefix, after.length - suffix)];
}

/** Edits between the unchanged key names, so each mapped span relocates. */
function navigationEdits(before: string, after: string): Edit[] {
  const edits: Edit[] = [];
  for (const match of before.matchAll(navBlock)) {
    const owner = match[2]!;
    const keys = [...match[3]!.matchAll(new RegExp(`void ${owner}(${navKey});`, "gu"))].map(
      (key) => key[1]!,
    );
    const replacement = navigationBlock(match[1]!, owner, keys);
    if (after.split(replacement).length !== 2) continue;
    const blockStart = match.index + match[1]!.length;
    const blockEnd = match.index + match[0].length;
    const target = after.indexOf(replacement) + match[1]!.length;
    const targetEnd = after.indexOf(replacement) + replacement.length;
    // Key name offsets in each text, in order.
    const names = (text: string, from: number) =>
      keys.map((key) => {
        const name = key.startsWith(".") ? key.slice(1) : key.slice(2, -2);
        const reference = `${owner}${key}`;
        const index = text.indexOf(reference, from) + owner.length + (key.startsWith(".") ? 1 : 2);
        from = index + name.length;
        return [index, index + name.length] as const;
      });
    const old = [blockStart, ...names(before, blockStart).flat(), blockEnd];
    const current = [target, ...names(after, target).flat(), targetEnd];
    for (let index = 0; index < old.length; index += 2) {
      const edit = gapEdit(
        before.slice(old[index], old[index + 1]),
        after.slice(current[index], current[index + 1]),
      );
      if (!edit) continue;
      const [start, end, text] = edit;
      edits.push({
        start: Buffer.byteLength(before.slice(0, old[index]! + start)),
        end: Buffer.byteLength(before.slice(0, old[index]! + end)),
        replacement: Buffer.from(text),
      });
    }
  }
  return edits;
}

const vt = String.raw`(?:import\("vize[:/]virtual-ts"\)\.)?`;
const quoted = String.raw`'((?:[^'\\]|\\.)*)'`;
const eagerValue = new RegExp(
  String.raw`(${vt}\$PropValue)<(__[\w$]+_ValueProps_\d+), ${quoted}(, ${vt}\$FallthroughValue<(__[\w$]+_Component_\d+), ${quoted}>>)`,
  "gu",
);

/** #12431: a per-prop value alias resolves its fallthrough lazily through `$PropValueOr`. */
export function lazyPropValue(
  helper: string,
  props: string,
  key: string,
  component: string,
  fallthrough: string,
): string {
  const tail = key === fallthrough ? "" : `, '${fallthrough}'`;
  return `${helper}Or<${props}, '${key}', ${component}${tail}>`;
}

/** Edits around each alias key, so its mapped span relocates instead of being replaced. */
function lazyPropValueEdits(before: string, after: string): Edit[] {
  const edits: Edit[] = [];
  const at = (start: number, end: number, replacement: string) =>
    edits.push({
      start: Buffer.byteLength(before.slice(0, start)),
      end: Buffer.byteLength(before.slice(0, end)),
      replacement: Buffer.from(replacement),
    });
  for (const match of before.matchAll(eagerValue)) {
    const [whole, helper, props, key, tail, component, fallthrough] = match as unknown as string[];
    const current = lazyPropValue(helper!, props!, key!, component!, fallthrough!);
    if (after.split(whole!).length > 1 || after.split(current).length === 1) continue;
    const helperEnd = match.index + helper!.length;
    at(helperEnd, helperEnd + 1, "Or<");
    const tailStart = match.index + whole!.length - tail!.length;
    const fallthroughKey = key === fallthrough ? "" : `, '${fallthrough}'`;
    at(tailStart, match.index + whole!.length, `, ${component}${fallthroughKey}>`);
  }
  return edits;
}

/** #12431: a non-hoisted helper block declares `$PropValueOr` right after `$PropValue`. */
function localPropValueOrEdits(before: string, after: string): Edit[] {
  const { before: line, after: lines } = propValueOrPreambleEdit();
  const local = (text: string) => `  ${text.replace(/^export /u, "")}`;
  const value = local(line);
  const added = local(lines.slice(line.length));
  const count = (text: string, needle: string) => text.split(needle).length - 1;
  const blocks = count(before, value);
  if (blocks === 0 || count(before, added) > 0 || count(after, value + added) !== blocks) return [];
  const edits: Edit[] = [];
  for (let at = before.indexOf(value); at >= 0; at = before.indexOf(value, at + 1)) {
    const end = Buffer.byteLength(before.slice(0, at + value.length));
    edits.push({ start: end, end, replacement: Buffer.from(added) });
  }
  return edits;
}

const usagePrefix = String.raw`(?:import\("vize[:/]virtual-ts"\)\.)?`;
const usageIsAny = String.raw`(?:import\("vize[:/]virtual-ts"\)\.IsAny|\$IsAny)`;
const perUsageProps = new RegExp(
  String.raw`(type (__[\w$]+)_Props_(\d+) = )typeof ([\w$.]+) extends \{ __vizeCheck: infer __F \} \? \(${usageIsAny}<__F> extends true \? (${usagePrefix})\$ComponentRawProps<typeof \4> : Record<string, unknown>\) : \5\$ComponentRawProps<typeof \4>;`,
  "gu",
);
const perUsageCheckProps = new RegExp(
  String.raw`(type (__[\w$]+)_CheckProps_(\d+) = \2_Props_\3 & )(${usagePrefix})\$EmitListeners<(\2_Component_\3)> & \(${usagePrefix}\$FallthroughProps<\5> extends infer __F \? \[__F extends unknown \? \{\} extends __F \? never : __F : never\] extends \[never\] \? ${usagePrefix}\$RequiredFallthroughProps<\5, __F, never> : ${usagePrefix}\$RequiredFallthroughProps<\5, __F> : never\);`,
  "gu",
);

/** #12431: a plain binding's props alias, as the usage contract writes it. */
export function usageProps(head: string, prefix: string, component: string): string {
  return `${head}${prefix}$UsageProps<typeof ${component}>;`;
}

/** #12431: a plain binding's check props, its props and the shared extras. */
export function usageCheckProps(head: string, prefix: string, component: string): string {
  return `${head}${prefix}$UsageCheckExtras<${component}>;`;
}

/** Edits moving each per-usage contract alias onto the usage contract, around its unchanged component reference. */
function usageContractEdits(before: string, after: string): Edit[] {
  const edits: Edit[] = [];
  const at = (start: number, end: number, replacement: string) =>
    edits.push({
      start: Buffer.byteLength(before.slice(0, start)),
      end: Buffer.byteLength(before.slice(0, end)),
      replacement: Buffer.from(replacement),
    });
  for (const match of before.matchAll(perUsageProps)) {
    const [whole, head, , , component, prefix] = match as unknown as string[];
    const current = usageProps(head!, prefix!, component!);
    if (after.split(whole!).length > 1 || after.split(current).length === 1) continue;
    // Keep the last `typeof X>;`; replace everything between the head and it.
    const kept = `typeof ${component}>;`;
    at(
      match.index + head!.length,
      match.index + whole!.length - kept.length,
      `${prefix}$UsageProps<`,
    );
  }
  for (const match of before.matchAll(perUsageCheckProps)) {
    const [whole, head, , , prefix, component] = match as unknown as string[];
    const current = usageCheckProps(head!, prefix!, component!);
    if (after.split(whole!).length > 1 || after.split(current).length === 1) continue;
    at(
      match.index + head!.length,
      match.index + whole!.length - 1,
      `${prefix}$UsageCheckExtras<${component}>`,
    );
  }
  return edits;
}

/** #12431: a non-hoisted helper block declares the usage contract right after `$PropValueOr`. */
function localUsageContractEdits(before: string, after: string): Edit[] {
  const anchor = before
    .split("\n")
    .find((line) => line.startsWith("  type $PropValueOr<P, K extends PropertyKey, C,"));
  if (anchor === undefined) return [];
  const value = `${anchor}\n`;
  const added = usageContractDeclarations()
    .map((line) => `  ${line}\n`)
    .join("");
  const count = (text: string, needle: string) => text.split(needle).length - 1;
  const blocks = count(before, value);
  if (count(before, added) > 0 || count(after, value + added) !== blocks) return [];
  const edits: Edit[] = [];
  for (let at = before.indexOf(value); at >= 0; at = before.indexOf(value, at + 1)) {
    const end = Buffer.byteLength(before.slice(0, at + value.length));
    edits.push({ start: end, end, replacement: Buffer.from(added) });
  }
  return edits;
}

/** Appends the landed edits for every line `after` replaces with its current form. */
export function withLandedEdits(before: string, after: string, edits: Edit[]): Edit[] {
  const gates: Edit[] = [
    ...navigationEdits(before, after),
    ...lazyPropValueEdits(before, after),
    ...localPropValueOrEdits(before, after),
    // #12481: authored setup lines lose the two spaces the old projection prefixed.
    ...dedentEdits(Buffer.from(before), Buffer.from(after)),
    ...usageContractEdits(before, after),
    ...localUsageContractEdits(before, after),
  ];
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
