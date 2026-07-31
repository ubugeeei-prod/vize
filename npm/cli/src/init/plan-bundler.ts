import fs from "node:fs";
import path from "node:path";

import type { ProjectDetection } from "./detect.js";
import { injectNuxtModule, injectVitePlugin } from "./edit-config.js";
import { skipped, type PlanDraft } from "./plan-types.js";

/**
 * Plans the bundler integration: the Vite plugin, or the Nuxt module.
 *
 * Nuxt outranks Vite because a Nuxt project owns its own Vite instance --
 * adding `@vizejs/vite-plugin` to a Nuxt-managed Vite config would fight the
 * module rather than complement it.
 *
 * @returns the possibly-edited Vite config source, or the input unchanged.
 */
export function planBundler(
  detection: ProjectDetection,
  viteDraft: string | null,
  draft: PlanDraft,
): string | null {
  if (detection.framework === "nuxt") {
    planNuxtModule(detection, draft);
    return viteDraft;
  }
  if (detection.framework !== "vite" || detection.viteConfigs.length !== 1) {
    draft.features.push(skipped("bundler", "no single vite.config or nuxt.config to configure"));
    return viteDraft;
  }
  draft.dependencies.add("@vizejs/vite-plugin");
  const filename = detection.viteConfigs[0]!;
  if (detection.hasVizeVitePlugin) {
    draft.features.push({
      id: "bundler",
      outcome: "unchanged",
      detail: `${filename} already uses @vizejs/vite-plugin`,
      snippet: null,
    });
    return viteDraft;
  }
  const injected = viteDraft === null ? null : injectVitePlugin(viteDraft);
  if (injected === null) {
    draft.features.push({
      id: "bundler",
      outcome: "blocked",
      detail: `${filename} has no plugins array this tool can extend safely`,
      snippet: "plugins: [vize()]",
    });
    return viteDraft;
  }
  draft.features.push({
    id: "bundler",
    outcome: "configured",
    detail: `adds vize() to ${filename}`,
    snippet: null,
  });
  return injected;
}

function planNuxtModule(detection: ProjectDetection, draft: PlanDraft): void {
  draft.dependencies.add("@vizejs/nuxt");
  if (detection.nuxtConfig === null) {
    draft.features.push(skipped("bundler", "no nuxt.config file to add @vizejs/nuxt to"));
    return;
  }
  if (detection.hasVizeNuxtModule) {
    draft.features.push({
      id: "bundler",
      outcome: "unchanged",
      detail: `${detection.nuxtConfig} already lists @vizejs/nuxt`,
      snippet: null,
    });
    return;
  }
  const source = fs.readFileSync(path.join(detection.root, detection.nuxtConfig), "utf8");
  const injected = injectNuxtModule(source);
  if (injected === null) {
    draft.features.push({
      id: "bundler",
      outcome: "blocked",
      detail: `${detection.nuxtConfig} is not a single plain defineNuxtConfig({ ... }) call`,
      snippet: 'modules: ["@vizejs/nuxt"]',
    });
    return;
  }
  draft.files.push({
    filename: path.join(detection.root, detection.nuxtConfig),
    source: injected,
  });
  draft.updatedFiles.push(detection.nuxtConfig);
  draft.features.push({
    id: "bundler",
    outcome: "configured",
    detail: `adds @vizejs/nuxt to ${detection.nuxtConfig}`,
    snippet: null,
  });
}
