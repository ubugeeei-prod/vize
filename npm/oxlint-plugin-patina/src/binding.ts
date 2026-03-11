import { createRequire } from "node:module";

import type {
  PatinaBinding,
  PatinaLintResult,
  PatinaRuleMeta,
  PatinaSettings,
} from "./model.js";

const require = createRequire(import.meta.url);

let bindingCache: PatinaBinding | null = null;

function loadBinding(): PatinaBinding {
  if (bindingCache) {
    return bindingCache;
  }

  const binding = require("@vizejs/native") as Partial<PatinaBinding>;
  if (typeof binding.lintPatinaSfc !== "function" || typeof binding.getPatinaRules !== "function") {
    throw new Error(
      "@vizejs/native does not expose the Patina Oxlint bridge. Rebuild or upgrade @vizejs/native.",
    );
  }

  bindingCache = binding as PatinaBinding;
  return bindingCache;
}

export function lintPatina(
  source: string,
  filename: string,
  settings: PatinaSettings,
  enabledRules?: readonly string[],
): PatinaLintResult {
  return loadBinding().lintPatinaSfc(source, {
    filename,
    locale: settings.locale,
    enabled_rules: enabledRules ? [...enabledRules] : undefined,
  });
}

export function getPatinaRules(): PatinaRuleMeta[] {
  return loadBinding().getPatinaRules();
}
