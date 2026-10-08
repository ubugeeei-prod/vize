import { accessibilityExplanations } from "./explanations-accessibility.mjs";
import { cssHtmlExplanations } from "./explanations-css-html.mjs";
import { ecosystemMuseaVaporExplanations } from "./explanations-ecosystem-musea-vapor.mjs";
import { scriptType0Explanations } from "./explanations-script-type-0.mjs";
import { scriptType1Explanations } from "./explanations-script-type-1.mjs";
import { scriptType2Explanations } from "./explanations-script-type-2.mjs";
import { scriptType3Explanations } from "./explanations-script-type-3.mjs";
import { vue0Explanations } from "./explanations-vue-0.mjs";
import { vue1Explanations } from "./explanations-vue-1.mjs";
import { vue2Explanations } from "./explanations-vue-2.mjs";

const records = [
  ...accessibilityExplanations,
  ...cssHtmlExplanations,
  ...ecosystemMuseaVaporExplanations,
  ...scriptType0Explanations,
  ...scriptType1Explanations,
  ...scriptType2Explanations,
  ...scriptType3Explanations,
  ...vue0Explanations,
  ...vue1Explanations,
  ...vue2Explanations,
];
export const exampleExplanations = new Map();
for (const [name, badEn, goodEn, badJa, goodJa] of records) {
  if (exampleExplanations.has(name)) throw new Error(`Duplicate example explanation: ${name}`);
  for (const text of [badEn, goodEn, badJa, goodJa]) {
    if (typeof text !== "string" || !text.trim())
      throw new Error(`Incomplete bilingual example explanation: ${name}`);
  }
  exampleExplanations.set(name, {
    bad: { en: badEn, ja: badJa },
    good: { en: goodEn, ja: goodJa },
  });
}

export function validateExampleExplanations(rules) {
  const registered = new Set(rules.map((rule) => rule.name));
  for (const name of registered) {
    if (!exampleExplanations.has(name)) throw new Error(`Missing example explanation: ${name}`);
  }
  for (const name of exampleExplanations.keys()) {
    if (!registered.has(name)) throw new Error(`Unregistered example explanation: ${name}`);
  }
}
