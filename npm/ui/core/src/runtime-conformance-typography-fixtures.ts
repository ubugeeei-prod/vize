import type { RuntimeFixture } from "./runtime-conformance-fixtures.ts";
import { blockquoteRuntimeFixture } from "./families/typography/blockquote/runtime-conformance-blockquote-fixtures.ts";
import { codeRuntimeFixture } from "./families/typography/code/runtime-conformance-code-fixtures.ts";
import { headingRuntimeFixture } from "./families/typography/heading/runtime-conformance-heading-fixtures.ts";
import { kbdRuntimeFixture } from "./families/typography/kbd/runtime-conformance-kbd-fixtures.ts";
import { textRuntimeFixture } from "./families/typography/text/runtime-conformance-text-fixtures.ts";

export const typographyRuntimeFixtures: readonly RuntimeFixture[] = [
  blockquoteRuntimeFixture,
  codeRuntimeFixture,
  headingRuntimeFixture,
  kbdRuntimeFixture,
  textRuntimeFixture,
];
