/** Certify the finite content-mapper component export naming transition. */
import assert from "node:assert/strict";
import { bufferOccurrences, type Edit } from "./projection-byte-relocation.ts";

const previousPrefix = Buffer.from("declare const __vize_component__: ");
const currentPrefix = Buffer.from("type ConstructorComponent = ");
const previousSuffix = Buffer.from(
  "\ndeclare const Constructor: typeof __vize_component__;\nexport default Constructor;",
);
const currentSuffix = Buffer.from(
  "\ndeclare const __vize_component__: ConstructorComponent;\n" +
    "declare const ConstructorVueComponent: typeof __vize_component__;\n" +
    "export default ConstructorVueComponent;",
);
const typedSuffix = Buffer.concat([
  Buffer.from("\ndeclare const __vize_component__: ConstructorComponent;"),
  previousSuffix,
]);

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
  const style = (bytes: Buffer) => {
    const inventory = {
      previousPrefix: bufferOccurrences(bytes, previousPrefix),
      previousSuffix: bufferOccurrences(bytes, previousSuffix),
      currentPrefix: bufferOccurrences(bytes, currentPrefix),
      typedSuffix: bufferOccurrences(bytes, typedSuffix),
      currentSuffix: bufferOccurrences(bytes, currentSuffix),
    };
    if (Object.values(inventory).every((count) => count === 0)) return "absent" as const;
    for (const [name, expected] of [
      ["reference", [1, 0, 0, 0, 0]],
      ["previous", [1, 1, 0, 0, 0]],
      ["typed", [1, 1, 1, 1, 0]],
      ["renamed", [1, 0, 1, 0, 1]],
    ] as const)
      if (expected.every((count, index) => count === Object.values(inventory)[index])) return name;
    assert.fail(`unknown component-export inventory: ${JSON.stringify(inventory)}`);
  };
  const beforeStyle = style(before);
  const afterStyle = style(after);
  if (beforeStyle === afterStyle) return;
  assert.equal(beforeStyle, "previous", "component export must start from the reviewed form");
  assert.ok(
    afterStyle === "typed" || afterStyle === "renamed",
    "component export must follow a reviewed naming transition",
  );
  const start = unique(before, previousPrefix);
  const suffix = unique(before, previousSuffix);
  assert.ok(start + previousPrefix.length < suffix, "component export body must be nonempty");
  const body = before.subarray(start + previousPrefix.length, suffix);
  const selectedSuffix = afterStyle === "renamed" ? currentSuffix : typedSuffix;
  const replacement = Buffer.concat([currentPrefix, body, selectedSuffix]);
  unique(after, replacement);
  const certificate: {
    edits: (Edit & { label: string })[];
    addedMapping?: [number, number, number, number, number, number];
  } = {
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
        replacement: selectedSuffix,
        label: "component export value name",
      },
    ],
  };
  const exportName = Buffer.from(
    afterStyle === "renamed" ? "ConstructorVueComponent" : "Constructor",
  );
  const exportMarker = Buffer.concat([Buffer.from("declare const "), exportName, Buffer.from(":")]);
  const exportStart = unique(after, exportMarker) + Buffer.byteLength("declare const ");
  certificate.addedMapping = [exportStart, exportName.length, 0, 0, 1, 248];
  return certificate;
}
