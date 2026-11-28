import assert from "node:assert/strict";
import { readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import {
  renderStabilityPage,
  STABILITY_LOCALES,
  stabilityPagePath,
} from "../../support/release/product-channels.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
const check = process.argv.includes("--check");

for (const locale of STABILITY_LOCALES) {
  const file = stabilityPagePath(root, locale);
  const rendered = renderStabilityPage(root, locale);
  if (check)
    assert.equal(readFileSync(file, "utf8"), rendered, `${path.relative(root, file)} is stale`);
  else writeFileSync(file, rendered);
}
console.log(
  `${check ? "Verified" : "Generated"} the release channel tables of ${STABILITY_LOCALES.length} stability pages`,
);
