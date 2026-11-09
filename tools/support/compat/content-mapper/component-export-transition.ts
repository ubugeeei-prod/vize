/** Certify the finite content-mapper component export naming transition. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import type { Edit } from "./projection-byte-relocation.ts";

const componentExportSource = readFileSync(
  new URL(
    "../../../../crates/vize_canon/src/virtual_ts/generator/component_export.rs",
    import.meta.url,
  ),
);
assert.equal(componentExportSource.length, 16143, "certified component export producer size");
assert.equal(
  createHash("sha256").update(componentExportSource).digest("hex"),
  "06e5c08eb73702e39e2629fda3253c51b405d5e520cd9d2b8f1d36af01fccebf",
  "certified component export producer source",
);

const identifier = "[A-Za-z_$][\\w$]*";
const previousPattern = new RegExp(
  `declare const __vize_component__: (?<body>[\\s\\S]+?)\\ndeclare const (?<name>${identifier}): typeof __vize_component__;\\nexport default \\k<name>;`,
  "gu",
);
const currentPattern = new RegExp(
  `type (?<type>${identifier})Component = (?<body>[\\s\\S]+?)\\ndeclare const __vize_component__: \\k<type>Component;\\ndeclare const (?<name>${identifier}): typeof __vize_component__;\\nexport default \\k<name>;`,
  "gu",
);

type Match = RegExpMatchArray & {
  groups: { body: string; name: string; type?: string };
  index: number;
};

function one(text: string, pattern: RegExp, label: string): Match | undefined {
  const matches = [...text.matchAll(pattern)] as Match[];
  assert.ok(matches.length <= 1, `${label} component export must be unique`);
  return matches[0];
}

function unique(bytes: Buffer, needle: Buffer) {
  const at = bytes.indexOf(needle);
  assert.notEqual(at, -1, `missing component-export boundary: ${needle.toString()}`);
  assert.equal(bytes.indexOf(needle, at + 1), -1, "component-export boundary must be unique");
  return at;
}

/**
 * Keep the complete generated component body byte-exact while naming the type
 * and selecting a collision-free exported value (#11666).
 */
export function certifyComponentExportTransition(
  before: Buffer,
  after: Buffer,
):
  | {
      edits: (Edit & { label: string })[];
      addedMapping?: [number, number, number, number, number, number];
    }
  | undefined {
  const old = before.toString("utf8");
  const next = after.toString("utf8");
  const oldPrevious = one(old, previousPattern, "previous");
  const oldCurrent = one(old, currentPattern, "current");
  const nextPrevious = one(next, previousPattern, "previous");
  const nextCurrent = one(next, currentPattern, "current");
  if (oldCurrent && nextCurrent) return;
  if (!oldCurrent && !nextCurrent) {
    assert.equal(Boolean(oldPrevious), Boolean(nextPrevious), "component export style stays exact");
    return;
  }
  assert.ok(oldPrevious && nextCurrent, "component export follows one reviewed naming transition");
  assert.equal(oldCurrent, undefined, "component export must start from the reviewed form");
  assert.ok(nextPrevious, "named component export keeps its public value declaration");
  assert.equal(nextPrevious.groups.body, `${nextCurrent.groups.type}Component;`);
  assert.equal(nextPrevious.groups.name, nextCurrent.groups.name);
  assert.equal(nextCurrent.groups.type, oldPrevious.groups.name, "component type keeps its name");
  assert.ok(
    nextCurrent.groups.name === oldPrevious.groups.name ||
      nextCurrent.groups.name === `${oldPrevious.groups.name}VueComponent`,
    "component value uses the reviewed collision-free name",
  );
  assert.equal(nextCurrent.groups.body, oldPrevious.groups.body, "component body stays exact");

  const previousPrefix = Buffer.from("declare const __vize_component__: ");
  const currentPrefix = Buffer.from(`type ${oldPrevious.groups.name}Component = `);
  const previousSuffix = Buffer.from(
    `\ndeclare const ${oldPrevious.groups.name}: typeof __vize_component__;\nexport default ${oldPrevious.groups.name};`,
  );
  const currentSuffix = Buffer.from(
    `\ndeclare const __vize_component__: ${oldPrevious.groups.name}Component;\n` +
      `declare const ${nextCurrent.groups.name}: typeof __vize_component__;\n` +
      `export default ${nextCurrent.groups.name};`,
  );
  const start = unique(before, previousPrefix);
  const suffix = unique(before, previousSuffix);
  assert.ok(start + previousPrefix.length < suffix, "component export body must be nonempty");
  const body = before.subarray(start + previousPrefix.length, suffix);
  unique(after, Buffer.concat([currentPrefix, body, currentSuffix]));
  const exportName = Buffer.from(nextCurrent.groups.name);
  const exportMarker = Buffer.concat([Buffer.from("declare const "), exportName, Buffer.from(":")]);
  const exportStart = unique(after, exportMarker) + Buffer.byteLength("declare const ");
  return {
    edits: [
      {
        start,
        end: start + previousPrefix.length,
        replacement: currentPrefix,
        label: "component export type name",
      },
      {
        start: suffix,
        end: suffix + previousSuffix.length,
        replacement: currentSuffix,
        label: "component export value name",
      },
    ],
    addedMapping: [exportStart, exportName.length, 0, 0, 1, 248],
  };
}
