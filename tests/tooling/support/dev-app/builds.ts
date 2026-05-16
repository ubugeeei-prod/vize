import { commandAvailable, run } from "./commands.ts";
import type { Target } from "./types.ts";

/**
 * Builds the shared native and Vite plugin artifacts required by every dev
 * fixture that exercises the Vize Vite integration.
 */
function ensureBuildCommonVite(): void {
  run("pnpm", ["-C", "npm/vize-native", "build"]);
  run("wasm-pack", [
    "build",
    "crates/vize_vitrine",
    "--target",
    "nodejs",
    "--out-dir",
    "../../npm/vite-plugin-vize/wasm",
    "--features",
    "wasm",
    "--no-default-features",
  ]);
  run("pnpm", ["-C", "npm/vize", "build"]);
  run("pnpm", ["-C", "npm/vite-plugin-vize", "build"]);
}

/**
 * Builds the Nuxt-facing stack used by the imported production fixtures.
 */
function ensureBuildNuxtStack(): void {
  ensureBuildCommonVite();
  run("pnpm", ["-C", "npm/vite-plugin-musea", "build"]);
  run("pnpm", ["-C", "npm/musea-nuxt", "build"]);
  run("pnpm", ["-C", "npm/nuxt", "build"]);
}

function ensureBuildPlayground(): void {
  ensureBuildCommonVite();
  run("pnpm", ["-C", "playground", "build:wasm"]);
}

/**
 * Builds exactly the artifacts needed before launching a dev target.
 *
 * The mapping deliberately avoids a single monolithic workspace build because
 * the real-world fixture loop should stay quick enough for day-to-day manual
 * reproduction work. Full workspace validation still belongs to CI.
 */
export function ensureTargetBuilds(currentTarget: Target, skipBuild: boolean): void {
  if (skipBuild) {
    return;
  }

  if (!commandAvailable("wasm-pack")) {
    throw new Error(
      "wasm-pack is required for dev startup. Install it or rerun with --skip-build.",
    );
  }

  switch (currentTarget) {
    case "playground":
      ensureBuildPlayground();
      break;
    case "misskey":
      ensureBuildCommonVite();
      break;
    case "npmx":
    case "elk":
    case "vuefes":
      ensureBuildNuxtStack();
      break;
  }
}
