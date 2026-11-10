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
  // The blank line keeps the producer within its source length budget.
  [
    "    )?;\n    publish::publish_content_mapper_text(\n",
    "    )?;\n\n    publish::publish_content_mapper_text(\n",
  ],
] as const;

const generatorPath = "crates/vize_canon/src/virtual_ts/generator.rs";
const generatorIdentity = {
  after: {
    bytes: 37789,
    sha256: "3eb5b8a7c0e30e866a9cafe482955945f50bc35b9ac33396f52d3b97ce33acdf",
  },
  before: {
    bytes: 37699,
    sha256: "31ffef289d05da9ba517f41367ff6d21e46ef76e40b09a26a4389ceb7bdd6655",
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
