/**
 * Config sources `vize init` writes.
 *
 * Every Oxlint-facing template here is derived from one settings object so the
 * `vp lint` block and the `oxlint` config can never describe different presets.
 * See `lint-target.ts` for why writing the wrong one of the two is silent.
 */

/** User-facing preset alias both Oxlint entry points run with. */
export const INIT_LINT_PRESET = "happy-path";

/** `settings.vize.helpLevel` both Oxlint entry points run with. */
export const INIT_LINT_HELP_LEVEL = "short";

/** VS Code extension id published from `editors/vscode`. */
export const VSCODE_EXTENSION_ID = "ubugeeei.vize";

export interface VizeConfigFeatures {
  readonly lint: boolean;
  readonly fmt: boolean;
  readonly typecheck: boolean;
  readonly vite: boolean;
}

/**
 * Builds `vize.config.ts` from the selected features.
 *
 * Only selected features contribute a block, so a project that asked for the
 * formatter alone does not silently get a type checker it never opted into.
 */
export function renderVizeConfig(features: VizeConfigFeatures): string {
  const blocks: string[] = [
    `  compiler: {
    templateSyntax: "standard",
  },`,
  ];
  if (features.lint) {
    blocks.push(`  linter: {
    enabled: true,
    preset: "${INIT_LINT_PRESET}",
  },`);
  }
  if (features.fmt) {
    blocks.push(`  formatter: {
    singleAttributePerLine: false,
    sortBlocks: true,
  },`);
  }
  if (features.typecheck) {
    blocks.push(`  typeChecker: {
    enabled: true,
    strict: true,
    jsxTypecheck: true,
  },`);
  }
  if (features.vite) {
    blocks.push(`  vite: {
    scanPatterns: ["src/**/*.vue"],
  },`);
  }
  return `import { defineConfig } from "vize";

export default defineConfig({
${blocks.join("\n")}
});
`;
}

/** Minimum project config written only when typechecking is selected and no config exists. */
export function renderTypecheckTsconfig(typescript: boolean): string {
  const compilerOptions: Record<string, boolean | string> = {
    strict: true,
    target: "ES2022",
    module: "ESNext",
    moduleResolution: "Bundler",
    jsx: "preserve",
  };
  if (!typescript) {
    compilerOptions.allowJs = true;
    compilerOptions.checkJs = true;
  }
  compilerOptions.noEmit = true;
  compilerOptions.skipLibCheck = true;
  return `${JSON.stringify({ compilerOptions, include: ["src/**/*"] }, null, 2)}\n`;
}

/**
 * Config for the `oxlint` binary.
 *
 * `.oxlintrc.json` cannot import `configs.happyPath`, and the bridge only runs
 * `vize/*` rules that appear in `rules`, so a JSON config would need every rule
 * id inlined and would rot on the next rule addition. `oxlint.config.ts` is
 * Oxlint's TypeScript config format and is auto-discovered (verified against
 * oxlint 1.64; `oxlint.config.mjs`, `.js`, `.cjs`, `.mts` and `.cts` are not --
 * see #3474), so it is the only form that stays correct over time.
 */
export const INIT_OXLINT_CONFIG = `import { defineConfig } from "oxlint";
import { configs } from "oxlint-plugin-vize";

export default defineConfig({
  plugins: ["vue"],
  jsPlugins: ["oxlint-plugin-vize"],
  settings: {
    vize: {
      preset: "${INIT_LINT_PRESET}",
      helpLevel: "${INIT_LINT_HELP_LEVEL}",
    },
  },
  rules: configs.happyPath,
});
`;

/** Import line the Vite+ `lint` block needs. */
export const VITE_LINT_IMPORT =
  'import { createVizeLintConfig } from "oxlint-plugin-vize";\n' as const;

/**
 * The Vite+ `lint` block, the only Oxlint configuration `vp lint` and `vp check`
 * read.
 *
 * `createVizeLintConfig()` returns a complete block, which is what makes the
 * `jsPlugins` entry impossible to omit. Hand-assembling the block is how a config
 * ends up looking wired while reporting nothing.
 */
export const VITE_LINT_BLOCK = `  lint: createVizeLintConfig({
    preset: "${INIT_LINT_PRESET}",
    settings: {
      helpLevel: "${INIT_LINT_HELP_LEVEL}",
    },
  }),
`;

/** Snippet printed when a Vite config has no `lint` block and cannot be edited safely. */
export const VITE_LINT_SNIPPET = `import { createVizeLintConfig } from "oxlint-plugin-vize";

export default defineConfig({
${VITE_LINT_BLOCK}});
`;

/**
 * Snippet printed when the Vite config already has a `lint` block.
 *
 * Spreading is the documented way to keep an existing block's other keys while
 * still taking the whole Vize block, `jsPlugins` included.
 */
export const VITE_LINT_MERGE_SNIPPET = `import { createVizeLintConfig } from "oxlint-plugin-vize";

export default defineConfig({
  lint: {
    ...createVizeLintConfig({
      preset: "${INIT_LINT_PRESET}",
      settings: {
        helpLevel: "${INIT_LINT_HELP_LEVEL}",
      },
    }),
    // keep your existing lint keys here
  },
});
`;

export const VITE_PLUGIN_IMPORT = 'import vize from "@vizejs/vite-plugin";\n' as const;

export const VITE_PLUGIN_SNIPPET = `import vize from "@vizejs/vite-plugin";

export default defineConfig({
  plugins: [vize()],
});
`;

export const NUXT_MODULE_SNIPPET = `export default defineNuxtConfig({
  modules: ["@vizejs/nuxt"],
});
`;

/**
 * `.vscode/extensions.json` written when no file exists yet.
 *
 * Recommendations are chosen over `code --install-extension` because they are
 * checked in, apply to the whole team, and change nothing on the machine that
 * runs `init`.
 */
export function renderVscodeExtensions(indent: string | number): string {
  return `${JSON.stringify({ recommendations: [VSCODE_EXTENSION_ID] }, null, indent)}\n`;
}

/** Editor integrations shipped from this repo, reported alongside the VS Code one. */
export const EDITOR_INTEGRATIONS = [
  "VS Code: ubugeeei.vize (recommended in .vscode/extensions.json)",
  "Zed: tools/zed-vize",
  "Neovim: tools/nvim-vize",
  "Vim: tools/vim-vize",
  "Helix: tools/helix-vize",
  "Emacs: tools/emacs-vize",
] as const;
