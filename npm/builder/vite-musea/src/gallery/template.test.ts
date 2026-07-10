import test from "node:test";
import assert from "node:assert/strict";

import { generateGalleryBody, generateGalleryScript } from "./template.ts";

void test("generateGalleryBody escapes the base path before writing an href attribute", () => {
  const body = generateGalleryBody('/__musea__" autofocus onfocus="alert(1)');

  assert.match(body, /href="\/__musea__&quot; autofocus onfocus=&quot;alert\(1\)"/);
  assert.doesNotMatch(body, /href="\/__musea__" autofocus onfocus="alert\(1\)"/);
});

void test("generateGalleryScript avoids inline handlers for base-path-derived preview URLs", () => {
  const script = generateGalleryScript("/__musea__' onclick='alert(1)");

  assert.doesNotMatch(script, /title="Open in new tab" onclick=/);
  assert.match(script, /data-preview-url="/);
  assert.match(script, /window\.open\(previewUrl, '_blank', 'noopener'\)/);
});
