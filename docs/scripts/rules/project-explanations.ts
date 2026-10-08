import { projectExplanations0 } from "./project-explanations-0.mjs";
import { projectExplanations1 } from "./project-explanations-1.mjs";

export const projectExplanations = new Map();
for (const [id, badEn, goodEn, badJa, goodJa] of [
  ...projectExplanations0,
  ...projectExplanations1,
]) {
  if (projectExplanations.has(id)) throw new Error(`Duplicate project explanation: ${id}`);
  projectExplanations.set(id, {
    badExplanation: { en: badEn, ja: badJa },
    goodExplanation: { en: goodEn, ja: goodJa },
  });
}
