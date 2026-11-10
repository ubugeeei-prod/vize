/** Producer lines introduced by the diagnostics-only projection, beside their predecessors. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";

export const CHECK_MODE_PRODUCER_SUCCESSORS = [
  [
    "            template_use_navigation: !transform_options.diagnostics_only(),\n",
    "            template_use_navigation: true,\n",
  ],
  [
    "            preserve_event_navigation: !transform_options.diagnostics_only(),\n",
    "            preserve_event_navigation: true,\n",
  ],
  ["            omit_navigation_references: transform_options.diagnostics_only(),\n", ""],
] as const;

const generatorPath = "crates/vize_canon/src/virtual_ts/generator.rs";
const generatorIdentity = {
  after: {
    bytes: 37608,
    sha256: "72cf3f5f3bb32701820466a39ed5ef4e46516c6872109d806a2aa9e23a8a41aa",
  },
  before: {
    bytes: 37518,
    sha256: "3c66ad8205f3239d33e02db8afb792a1d6c194f9956d8b6cb92812787c08b0d6",
  },
};

/**
 * Generator lines the diagnostics-only projection threads into scope options,
 * beside their predecessors. The blank line keeps the file within its length budget.
 */
export const CHECK_MODE_GENERATOR_SUCCESSORS = [
  [
    "                props_const_assertions.splice_output_line(&mut output_line, line_start);\n                // Strip `export` from non-import lines inside setup scope\n",
    "                props_const_assertions.splice_output_line(&mut output_line, line_start);\n\n                // Strip `export` from non-import lines inside setup scope\n",
  ],
  [
    "                preserve_event_navigation: generation_options.preserve_event_navigation,\n                omit_navigation_references: generation_options.omit_navigation_references,\n",
    "                preserve_event_navigation: generation_options.preserve_event_navigation,\n",
  ],
] as const;

const identity = (text: string) => ({
  bytes: Buffer.byteLength(text),
  sha256: createHash("sha256").update(text).digest("hex"),
});

/** The exact generator bytes before check-mode threading; other images pass through. */
export function beforeCheckModeGenerator(path: string, current: string): string {
  if (path !== generatorPath || identity(current).sha256 !== generatorIdentity.after.sha256)
    return current;
  assert.deepEqual(identity(current), generatorIdentity.after, "check-mode generator successor");
  for (const [successor, predecessor] of CHECK_MODE_GENERATOR_SUCCESSORS) {
    assert.equal(current.split(successor).length, 2, "finite check-mode generator successor");
    current = current.replace(successor, () => predecessor);
  }
  assert.deepEqual(identity(current), generatorIdentity.before, "check-mode generator predecessor");
  return current;
}
