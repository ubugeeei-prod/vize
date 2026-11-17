/** The content-mapper producer before its verbatim lookup checked the prefix first. */
import assert from "node:assert/strict";

const successor =
  "        && let Some(relative_start) = alias::verbatim_offset(generated_text, original_text)\n";
const predecessor =
  "        && let Some(relative_start) =\n            memchr::memmem::find(generated_text.as_bytes(), original_text.as_bytes())\n";

/** `producer` with the verbatim lookup restored to the substring search it replaced. */
export function beforeVerbatimOffset(producer: string): string {
  assert.equal(producer.split(successor).length, 2, "finite verbatim-offset producer successor");
  return producer.replace(successor, predecessor);
}
