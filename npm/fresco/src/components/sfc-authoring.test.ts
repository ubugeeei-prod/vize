import assert from "node:assert/strict";
import { existsSync, readFileSync, readdirSync } from "node:fs";
import path from "node:path";
import { test } from "vitest";

const canonicalSfcComponents = [
  { name: "Alert", file: "FrescoAlert" },
  { name: "Avatar", file: "FrescoAvatar" },
  { name: "Badge", file: "FrescoBadge" },
  { name: "Breadcrumb", file: "FrescoBreadcrumb" },
  { name: "Card", file: "FrescoCard" },
  { name: "FrameBuffer", file: "FrescoFrameBuffer" },
  { name: "Header", file: "FrescoHeader" },
  { name: "Link", file: "FrescoLink" },
  { name: "Markdown", file: "FrescoMarkdown" },
  { name: "Newline", file: "FrescoNewline" },
  { name: "ScrollBar", file: "ScrollBar" },
  { name: "ScrollBox", file: "ScrollBox" },
  { name: "Spacer", file: "FrescoSpacer" },
  { name: "Spinner", file: "FrescoSpinner" },
  { name: "Static", file: "FrescoStatic" },
  { name: "Transform", file: "FrescoTransform" },
  { name: "Tooltip", file: "FrescoTooltip" },
];
const packageRoot = existsSync(path.join(process.cwd(), "src", "components"))
  ? process.cwd()
  : path.join(process.cwd(), "npm", "fresco");

for (const component of canonicalSfcComponents) {
  test(`${component.name} keeps its canonical implementation in an authored SFC`, () => {
    const sfc = path.join(packageRoot, "src", "components", `${component.file}.vue`);
    const source = readFileSync(sfc, "utf8");

    assert.match(source, /<script setup(?=[^>]*\blang="ts")[^>]*>/u);
    assert.match(source, /<template>/u);
    assert.doesNotMatch(source, /\b(?:h|createVNode|createElementVNode)\s*\(/u);
    const siblingNames = readdirSync(path.join(packageRoot, "src", "components"));
    assert.equal(siblingNames.includes(`${component.name}.ts`), false);
  });
}
