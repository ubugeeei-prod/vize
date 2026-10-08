import type { CodeExample, RuleExample, RuleMetadata } from "./types.ts";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { manualExamples } from "./manual.ts";
import { overrides } from "./overrides.ts";

export function ruleExamples(root: string, rule: RuleMetadata): RuleExample {
  const override = overrides[rule.name];
  if (override) return override;
  if (manualExamples[rule.name]) return manualExamples[rule.name];
  const source = readFileSync(resolve(root, rule.implementationPath), "utf8");
  const docs = source
    .split("\n")
    .filter((line) => line.startsWith("//!"))
    .map((line) => line.slice(3).trimStart())
    .join("\n");
  const invalid = docs.match(/### (?:Invalid|Bad)[^\n]*\n([\s\S]*?)(?=\n### |$)/);
  const valid = docs.match(/### (?:Valid|Good)[^\n]*\n([\s\S]*?)(?=\n### |$)/);
  if (invalid && valid) {
    const bad = firstCode(invalid[1]);
    const good = firstCode(valid[1]);
    if (bad && good)
      return {
        bad: normalize(bad, rule),
        good: normalize(good, rule),
        evidence: rule.implementationPath,
      };
  }
  throw new Error(`Missing executable Bad/Good examples for ${rule.name}`);
}

function firstCode(text: string): CodeExample | null {
  const match = text.match(/```(vue|ts|js|html|css)\n([\s\S]*?)\n```/);
  return match && { language: match[1], source: match[2] };
}

function normalize(example: CodeExample, rule: RuleMetadata): CodeExample {
  let { source, language } = example;
  if (language === "html" && rule.name.startsWith("petite-vue/")) {
    source = `<!doctype html>\n<html><body>\n${source}\n<script src="https://unpkg.com/petite-vue" init></script>\n</body></html>`;
  } else if (language === "css") source = `<style scoped>\n${source}\n</style>`;
  else if (language === "ts" || language === "js") {
    const setup = !/export\s/.test(source) && rule.name !== "script/no-top-level-ref-in-script";
    const vapor = [
      "script/no-get-current-instance",
      "script/no-options-api",
      "script/no-next-tick",
    ].includes(rule.name);
    source = `<script${setup ? " setup" : ""}${vapor ? " vapor" : ""} lang="${language}">\n${source}\n</script>`;
  } else if (language !== "html" && !/<(?:template|script|style|art)(?:\s|>)/.test(source)) {
    source = `<template>\n${source}\n</template>`;
  } else if (language === "html" && !rule.name.startsWith("petite-vue/")) {
    source = `<template>\n${source}\n</template>`;
    language = "vue";
  }
  return { language: language === "html" ? "html" : "vue", source };
}
