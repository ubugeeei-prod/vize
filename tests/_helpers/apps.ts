import type { Page } from "@playwright/test";
import { execFileSync, execSync } from "node:child_process";
import * as fs from "node:fs";
import * as path from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const TESTS_DIR = path.resolve(__dirname, "..");
const GIT_DIR = path.join(TESTS_DIR, "_fixtures", "_git");
const PROJECTS_DIR = path.join(TESTS_DIR, "_fixtures", "_projects");
const MUTABLE_GIT_PROJECTS_DIR = path.join(PROJECTS_DIR, "_git-worktrees");
const MUTABLE_GIT_WORKTREE_INSTANCE = process.env.VIZE_TEST_WORKTREE_ID ?? `pid-${process.pid}`;
const NPM_DIR = path.resolve(__dirname, "../../npm");
const REPO_ROOT = path.resolve(__dirname, "../..");

export interface AppConfig {
  name: string;
  cwd: string;
  command: string;
  args: string[];
  port: number;
  url: string;
  mountSelector: string;
  readyPattern: RegExp;
  allowNon200?: boolean;
  waitUntil?: "load" | "domcontentloaded" | "networkidle" | "commit";
  readyDelay?: number;
  startupTimeout: number;
  env?: Record<string, string>;
  setup?: () => void;
  setupPage?: (page: Page) => Promise<void>;
  build?: { command: string; args: string[]; timeout: number };
  preview?: {
    command: string;
    args: string[];
    port: number;
    url: string;
    readyPattern: RegExp;
  };
  check?: {
    cwd: string;
    patterns: string[];
  };
  lint?: {
    cwd: string;
    patterns: string[];
  };
}

// --- Setup helpers ---

const VIZE_SYMLINK_TARGETS: Record<string, string> = {
  native: path.join(NPM_DIR, "vize-native"),
  "vite-plugin": path.join(NPM_DIR, "vite-plugin-vize"),
  nuxt: path.join(NPM_DIR, "nuxt"),
  "vite-plugin-musea": path.join(NPM_DIR, "vite-plugin-musea"),
  "musea-nuxt": path.join(NPM_DIR, "musea-nuxt"),
};
const VIZE_LOCAL_BUILD_TARGETS = [
  {
    name: "vize",
    filter: "vize",
    dir: path.join(NPM_DIR, "vize"),
    outputs: ["dist/index.mjs", "dist/config.mjs"],
  },
  {
    name: "@vizejs/vite-plugin",
    filter: "@vizejs/vite-plugin",
    dir: path.join(NPM_DIR, "vite-plugin-vize"),
    outputs: ["dist/index.mjs"],
  },
  {
    name: "@vizejs/nuxt",
    filter: "@vizejs/nuxt",
    dir: path.join(NPM_DIR, "nuxt"),
    outputs: ["dist/index.mjs"],
  },
  {
    name: "@vizejs/vite-plugin-musea",
    filter: "@vizejs/vite-plugin-musea",
    dir: path.join(NPM_DIR, "vite-plugin-musea"),
    outputs: ["dist/index.mjs", "dist/cli/index.mjs"],
  },
  {
    name: "@vizejs/musea-nuxt",
    filter: "@vizejs/musea-nuxt",
    dir: path.join(NPM_DIR, "musea-nuxt"),
    outputs: ["dist/index.mjs"],
  },
] as const;
const MISSKEY_FLUENT_EMOJI_RE = /\/fluent-emoji(?:s)?\/([0-9a-z-]+\.png)\b/g;
const NPMX_E2E_ENV = {
  NUXT_SESSION_PASSWORD: "e2e-test-dummy-session-password-32chars!",
  VIZE_E2E_DISABLE_LUNARIA: "1",
} as const;
const VUEFES_E2E_ENV = {
  AUTH_SECRET: "e2e-test-dummy-auth-secret-32chars!",
  NEXTAUTH_SECRET: "e2e-test-dummy-auth-secret-32chars!",
} as const;
const FRONTEND_PHPCON_E2E_ENV = {
  NUXT_PUBLIC_API_BASE: "/__vize_e2e/api",
  NUXT_TELEMETRY_DISABLED: "1",
} as const;
const VUE_BETA_OVERRIDES = {
  "@vue/compiler-core": "3.6.0-beta.10",
  "@vue/compiler-dom": "3.6.0-beta.10",
  "@vue/compiler-sfc": "3.6.0-beta.10",
  "@vue/compiler-ssr": "3.6.0-beta.10",
  "@vue/reactivity": "3.6.0-beta.10",
  "@vue/runtime-core": "3.6.0-beta.10",
  "@vue/runtime-dom": "3.6.0-beta.10",
  "@vue/server-renderer": "3.6.0-beta.10",
  "@vue/shared": "3.6.0-beta.10",
  vue: "3.6.0-beta.10",
} as const;
const TRANSPARENT_PNG = Buffer.from(
  "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVQIW2P8z/C/HwAFgwJ/lE6nWQAAAABJRU5ErkJggg==",
  "base64",
);
const BUILT_VIZE_PACKAGES = new Set<string>();

function hasBuildOutputs(dir: string, outputs: readonly string[]): boolean {
  return outputs.every((output) => fs.existsSync(path.join(dir, output)));
}

function ensureLocalVizePackagesBuilt(): void {
  for (const target of VIZE_LOCAL_BUILD_TARGETS) {
    if (BUILT_VIZE_PACKAGES.has(target.name) && hasBuildOutputs(target.dir, target.outputs)) {
      continue;
    }

    if (!hasBuildOutputs(target.dir, target.outputs)) {
      console.log(`[vize:setup] building ${target.name}...`);
      execFileSync("npx", ["-y", "pnpm@10", "--filter", target.filter, "build"], {
        cwd: REPO_ROOT,
        stdio: "inherit",
        timeout: 300_000,
      });
    }

    BUILT_VIZE_PACKAGES.add(target.name);
  }
}

function installPnpmDependencies(cwd: string): void {
  console.log(`[vize:setup] pnpm install in ${cwd}...`);
  execSync("npx -y pnpm@10 install --no-frozen-lockfile --prefer-offline", {
    cwd,
    stdio: "inherit",
    timeout: 600_000,
  });
}

function ensureSymlink(link: string, target: string): void {
  try {
    const stat = fs.lstatSync(link);
    if (stat.isSymbolicLink()) {
      try {
        fs.statSync(link);
        return; // valid symlink
      } catch {
        fs.unlinkSync(link); // broken symlink — recreate
      }
    } else {
      return; // real dir/file
    }
  } catch {
    // does not exist
  }
  fs.symlinkSync(target, link, "dir");
}

function createVizeSymlinks(nodeModulesDir: string): void {
  const vizejsDir = path.join(nodeModulesDir, "@vizejs");
  fs.mkdirSync(vizejsDir, { recursive: true });
  for (const [name, target] of Object.entries(VIZE_SYMLINK_TARGETS)) {
    ensureSymlink(path.join(vizejsDir, name), target);
  }
}

function patchNuxtConfig(
  configPath: string,
  opts?: { enableVize?: boolean; removeModules?: string[] },
): void {
  let config = fs.readFileSync(configPath, "utf-8");
  let changed = false;
  const enableVize = opts?.enableVize ?? true;

  if (enableVize && !config.includes("@vizejs/nuxt")) {
    config = config.replace("modules: [", "modules: [\n    '@vizejs/nuxt',");
    config = config.replace(
      "compatibilityDate:",
      "vize: {\n    musea: false,\n  },\n\n  compatibilityDate:",
    );
    changed = true;
  }

  // Remove modules that cause issues in the e2e environment
  if (opts?.removeModules) {
    for (const mod of opts.removeModules) {
      const escapedMod = mod.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
      const re = new RegExp(`\\s*["']${escapedMod}["'],?\\n?`);
      if (re.test(config)) {
        config = config.replace(re, "\n");
        changed = true;
      }
    }
  }

  if (changed) {
    fs.writeFileSync(configPath, config);
  }
}

function patchNpmxLunariaModule(modulePath: string): void {
  const source = fs.readFileSync(modulePath, "utf-8");
  if (source.includes("VIZE_E2E_DISABLE_LUNARIA")) {
    return;
  }

  const nextSource = source.replace(
    "    if (nuxt.options.dev || nuxt.options._prepare || nuxt.options.test || isTest) {\n",
    "    if (process.env.VIZE_E2E_DISABLE_LUNARIA === '1' || nuxt.options.dev || nuxt.options._prepare || nuxt.options.test || isTest) {\n",
  );
  if (nextSource !== source) {
    fs.writeFileSync(modulePath, nextSource);
  }
}

function patchNpmxPackageJson(packageJsonPath: string): void {
  const pkg = JSON.parse(fs.readFileSync(packageJsonPath, "utf-8"));
  if (pkg.dependencies?.["@lunariajs/core"] === "0.1.1") {
    return;
  }

  pkg.dependencies ??= {};
  pkg.dependencies["@lunariajs/core"] = "0.1.1";
  fs.writeFileSync(packageJsonPath, JSON.stringify(pkg, null, "\t") + "\n");
}

function patchNpmxPrerenderRoutes(configPath: string): void {
  const source = fs.readFileSync(configPath, "utf-8");
  const nextSource = source.replace(/prerender: true/g, "prerender: false");
  if (nextSource !== source) {
    fs.writeFileSync(configPath, nextSource);
  }
}

function patchVitepressConfig(configPath: string): void {
  let config = fs.readFileSync(configPath, "utf-8");
  let changed = false;

  if (!config.includes("from '@vizejs/vite-plugin'")) {
    config = config.replace(
      "import llmstxt, { copyOrDownloadAsMarkdownButtons } from 'vitepress-plugin-llms'",
      "import llmstxt, { copyOrDownloadAsMarkdownButtons } from 'vitepress-plugin-llms'\nimport vize from '@vizejs/vite-plugin'",
    );
    changed = true;
  }

  if (!config.includes("vize()")) {
    config = config.replace(
      "plugins: [llmstxt({",
      "plugins: [vize({ handleNodeModulesVue: false }), llmstxt({",
    );
    changed = true;
  }

  if (changed) {
    fs.writeFileSync(configPath, config);
  }
}

function patchNuxtUiLinkComponent(linkPath: string): void {
  let source = fs.readFileSync(linkPath, "utf-8");
  const nextSource = source
    .replace(
      "          rel: (rest as NuxtLinkDefaultSlotProps).rel,",
      "          rel: (rest as Partial<NuxtLinkDefaultSlotProps> | undefined)?.rel,",
    )
    .replace(
      "          target: (rest as NuxtLinkDefaultSlotProps).target,",
      "          target: (rest as Partial<NuxtLinkDefaultSlotProps> | undefined)?.target,",
    )
    .replace(
      "          isExternal: (rest as NuxtLinkDefaultSlotProps).isExternal,",
      "          isExternal: (rest as Partial<NuxtLinkDefaultSlotProps> | undefined)?.isExternal,",
    )
    .replace(
      "        rel: (rest as NuxtLinkDefaultSlotProps).rel,",
      "        rel: (rest as Partial<NuxtLinkDefaultSlotProps> | undefined)?.rel,",
    )
    .replace(
      "        target: (rest as NuxtLinkDefaultSlotProps).target,",
      "        target: (rest as Partial<NuxtLinkDefaultSlotProps> | undefined)?.target,",
    )
    .replace(
      "        isExternal: (rest as NuxtLinkDefaultSlotProps).isExternal",
      "        isExternal: (rest as Partial<NuxtLinkDefaultSlotProps> | undefined)?.isExternal",
    );

  if (nextSource !== source) {
    fs.writeFileSync(linkPath, nextSource);
  }
}

function patchNuxtUiFormComponent(formPath: string): void {
  const source = fs.readFileSync(formPath, "utf-8");
  const nextSource = source.replace(
    "  validateOn() {\n    return ['input', 'blur', 'change'] as FormInputEvents[]\n  },",
    "  validateOn: () => {\n    return ['input', 'blur', 'change'] as FormInputEvents[]\n  },",
  );

  if (nextSource !== source) {
    fs.writeFileSync(formPath, nextSource);
  }
}

function hoistPnpmPackage(nodeModulesDir: string, packageName: string): void {
  const link = path.join(nodeModulesDir, packageName);
  // Check if already a valid symlink or real dir
  try {
    const stat = fs.lstatSync(link);
    if (stat.isSymbolicLink()) {
      try {
        fs.statSync(link);
        return; // valid
      } catch {
        fs.unlinkSync(link); // broken — remove
      }
    } else {
      return; // real dir
    }
  } catch {
    // does not exist
  }
  const pnpmDir = path.join(nodeModulesDir, ".pnpm");
  if (!fs.existsSync(pnpmDir)) return;
  const candidates = fs.readdirSync(pnpmDir).filter((d) => d.startsWith(`${packageName}@`));
  for (const candidate of candidates) {
    const target = path.join(pnpmDir, candidate, "node_modules", packageName);
    if (fs.existsSync(target)) {
      fs.symlinkSync(target, link, "dir");
      return;
    }
  }
}

function addPnpmOverrides(packageJsonPath: string, overrides: Record<string, string>): void {
  const pkg = JSON.parse(fs.readFileSync(packageJsonPath, "utf-8"));
  if (!pkg.pnpm) pkg.pnpm = {};
  if (!pkg.pnpm.overrides) pkg.pnpm.overrides = {};
  let changed = false;
  for (const [key, value] of Object.entries(overrides)) {
    if (pkg.pnpm.overrides[key] !== value) {
      pkg.pnpm.overrides[key] = value;
      changed = true;
    }
  }
  if (changed) {
    fs.writeFileSync(packageJsonPath, JSON.stringify(pkg, null, "\t") + "\n");
  }
}

function patchVuefesVisualFixture(vuefesDir: string): void {
  const configPath = path.join(vuefesDir, "nuxt.config.ts");
  const configSource = fs.readFileSync(configPath, "utf-8");
  const nextConfigSource = configSource.replace(
    '  i18n: {\n    langDir: ".",',
    '  i18n: {\n    bundle: {\n      optimizeTranslationDirective: false,\n    },\n    langDir: ".",',
  );
  if (nextConfigSource !== configSource) {
    fs.writeFileSync(configPath, nextConfigSource);
  }

  const staffSectionPath = path.join(vuefesDir, "app", "pages", "_components", "SectionStaff.vue");
  const source = fs.readFileSync(staffSectionPath, "utf-8");
  const nextSource = source
    .replace(
      "onMounted(() => {\n  if (!staffList.value) return;\n  staffList.value.leaders = shuffleNonPinned(staffList.value.leaders);\n  staffList.value.cores = shuffleNonPinned(staffList.value.cores);\n});",
      "onMounted(() => {\n  if (!staffList.value) return;\n  staffList.value.leaders = keepPinnedOrder(staffList.value.leaders);\n  staffList.value.cores = keepPinnedOrder(staffList.value.cores);\n});",
    )
    .replace(
      "function shuffleNonPinned(staffArray: Staff[]): Staff[] {\n  const pinned = staffArray.filter(staff => staff.pinned);\n  const nonPinned = staffArray.filter(staff => !staff.pinned);\n  const shuffledNonPinned = shuffleArray(nonPinned);\n  return [...pinned, ...shuffledNonPinned];\n}",
      "function keepPinnedOrder(staffArray: Staff[]): Staff[] {\n  const pinned = staffArray.filter(staff => staff.pinned);\n  const nonPinned = staffArray.filter(staff => !staff.pinned);\n  return [...pinned, ...nonPinned];\n}",
    );

  if (nextSource !== source) {
    fs.writeFileSync(staffSectionPath, nextSource);
  }

  const staffApiPath = path.join(vuefesDir, "server", "api", "staffs", "index.get.ts");
  const apiSource = fs.readFileSync(staffApiPath, "utf-8");
  const nextApiSource = apiSource
    .replace(
      "function shuffleNonPinned(staffArray: Staff[]): Staff[] {\n  const pinned = staffArray.filter(staff => staff.pinned);\n  const nonPinned = staffArray.filter(staff => !staff.pinned);\n  const shuffledNonPinned = shuffleArray(nonPinned);\n  return [...pinned, ...shuffledNonPinned];\n}",
      "function keepPinnedOrder(staffArray: Staff[]): Staff[] {\n  const pinned = staffArray.filter(staff => staff.pinned);\n  const nonPinned = staffArray.filter(staff => !staff.pinned);\n  return [...pinned, ...nonPinned];\n}",
    )
    .replace(
      "leaders: shuffleNonPinned(staffs.leaders),",
      "leaders: keepPinnedOrder(staffs.leaders),",
    )
    .replace("cores: shuffleNonPinned(staffs.cores),", "cores: keepPinnedOrder(staffs.cores),");

  if (nextApiSource !== apiSource) {
    fs.writeFileSync(staffApiPath, nextApiSource);
  }
}

function ensureMisskeyFluentEmojiAssets(misskeyDir: string): void {
  const sourceRoot = path.join(misskeyDir, "packages", "frontend", "src");
  const distDir = path.join(misskeyDir, "fluent-emojis", "dist");
  const assetNames = new Set<string>();

  function visit(dir: string): void {
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
      const entryPath = path.join(dir, entry.name);
      if (entry.isDirectory()) {
        visit(entryPath);
        continue;
      }

      if (!/\.(vue|ts|tsx|js|jsx)$/.test(entry.name)) {
        continue;
      }

      const source = fs.readFileSync(entryPath, "utf-8");
      for (const match of source.matchAll(MISSKEY_FLUENT_EMOJI_RE)) {
        const assetName = match[1];
        if (assetName) {
          assetNames.add(assetName);
        }
      }
    }
  }

  if (fs.existsSync(sourceRoot)) {
    visit(sourceRoot);
  }

  fs.mkdirSync(distDir, { recursive: true });
  for (const assetName of assetNames) {
    const assetPath = path.join(distDir, assetName);
    if (!fs.existsSync(assetPath)) {
      fs.writeFileSync(assetPath, TRANSPARENT_PNG);
    }
  }
}

function ensureMisskeyOptionalDependencyStubs(misskeyDir: string): void {
  for (const vCodeDiffDir of [
    path.join(misskeyDir, "node_modules", "v-code-diff"),
    path.join(misskeyDir, "packages", "frontend", "node_modules", "v-code-diff"),
  ]) {
    writeVCodeDiffStub(vCodeDiffDir);
  }
}

function writeVCodeDiffStub(vCodeDiffDir: string): void {
  ensureFileContent(
    path.join(vCodeDiffDir, "package.json"),
    `${JSON.stringify(
      {
        name: "v-code-diff",
        version: "0.0.0-vize-fixture",
        type: "module",
        exports: "./index.js",
        main: "./index.js",
      },
      null,
      2,
    )}\n`,
  );
  ensureFileContent(
    path.join(vCodeDiffDir, "index.js"),
    `import { h } from "vue";

export const CodeDiff = {
  name: "CodeDiff",
  props: {
    context: null,
    hideHeader: null,
    language: null,
    maxHeight: null,
    newString: null,
    oldString: null,
  },
  setup(props) {
    return () =>
      h(
        "pre",
        {
          style: {
            maxHeight: props.maxHeight ?? undefined,
            overflow: "auto",
            whiteSpace: "pre-wrap",
          },
        },
        String(props.oldString ?? "") + "\\n---\\n" + String(props.newString ?? ""),
      );
  },
};

export default CodeDiff;
`,
  );
}

function removeManualChunksObject(viteConfigPath: string): void {
  let viteConfig = fs.readFileSync(viteConfigPath, "utf-8");
  const nextConfig = viteConfig.replace(
    /\n\s*manualChunks:\s*\{[\s\S]*?\n\s*\},\n(?=\s*entryFileNames:)/,
    "\n",
  );
  if (nextConfig !== viteConfig) {
    fs.writeFileSync(viteConfigPath, nextConfig);
  }
}

function mirrorLoaderAssetsForViteBase(publicDir: string, baseDirName: string): void {
  const sourceDir = path.join(publicDir, "loader");
  if (!fs.existsSync(sourceDir)) {
    return;
  }

  const targetDir = path.join(publicDir, baseDirName, "loader");
  fs.mkdirSync(targetDir, { recursive: true });

  for (const fileName of ["boot.js", "style.css"]) {
    const sourcePath = path.join(sourceDir, fileName);
    if (!fs.existsSync(sourcePath)) {
      continue;
    }

    fs.copyFileSync(sourcePath, path.join(targetDir, fileName));
  }
}

function ensureFileContent(filePath: string, content: string): void {
  const current = fs.existsSync(filePath) ? fs.readFileSync(filePath, "utf-8") : null;
  if (current === content) {
    return;
  }

  fs.mkdirSync(path.dirname(filePath), { recursive: true });
  fs.writeFileSync(filePath, content);
}

const PRESERVED_WORKTREE_ENTRIES = ["node_modules"] as const;
const MUTABLE_WORKTREE_CACHE_PATHS = [
  ".nuxt",
  ".output",
  ".vite",
  "node_modules/.cache",
  "node_modules/.vite",
] as const;

type PreservedWorktreeSnapshot = {
  entries: Array<{
    name: (typeof PRESERVED_WORKTREE_ENTRIES)[number];
    tempPath: string;
  }>;
  root: string | null;
};

function getGitFixtureSourceDir(name: string): string {
  return path.join(GIT_DIR, name);
}

function getMutableGitFixtureDir(name: string, variant?: string): string {
  if (variant) {
    return path.join(MUTABLE_GIT_PROJECTS_DIR, MUTABLE_GIT_WORKTREE_INSTANCE, variant, name);
  }
  return path.join(MUTABLE_GIT_PROJECTS_DIR, MUTABLE_GIT_WORKTREE_INSTANCE, name);
}

function readGitHeadRevision(repoDir: string): string {
  return execFileSync("git", ["rev-parse", "HEAD"], {
    cwd: repoDir,
    encoding: "utf-8",
    env: {
      ...process.env,
      LANG: "C",
      LC_ALL: "C",
    },
  }).trim();
}

function exportGitHeadToDir(repoDir: string, targetDir: string): void {
  const env = {
    ...process.env,
    LANG: "C",
    LC_ALL: "C",
  };
  const archive = execFileSync("git", ["archive", "--format=tar", "HEAD"], {
    cwd: repoDir,
    encoding: "buffer",
    maxBuffer: 200 * 1024 * 1024,
    env,
  });
  fs.mkdirSync(targetDir, { recursive: true });
  execFileSync("tar", ["-xf", "-"], {
    cwd: targetDir,
    input: archive,
    maxBuffer: 200 * 1024 * 1024,
    env,
  });
}

function preserveMutableWorktreeEntries(workDir: string): PreservedWorktreeSnapshot {
  if (!fs.existsSync(workDir)) {
    return { root: null, entries: [] };
  }

  let root: string | null = null;
  const entries: PreservedWorktreeSnapshot["entries"] = [];

  for (const name of PRESERVED_WORKTREE_ENTRIES) {
    const sourcePath = path.join(workDir, name);
    if (!fs.existsSync(sourcePath)) {
      continue;
    }

    if (root == null) {
      fs.mkdirSync(MUTABLE_GIT_PROJECTS_DIR, { recursive: true });
      root = fs.mkdtempSync(path.join(MUTABLE_GIT_PROJECTS_DIR, ".preserve-"));
    }

    const tempPath = path.join(root, name);
    fs.mkdirSync(path.dirname(tempPath), { recursive: true });
    fs.renameSync(sourcePath, tempPath);
    entries.push({ name, tempPath });
  }

  return { root, entries };
}

function restorePreservedWorktreeEntries(
  workDir: string,
  snapshot: PreservedWorktreeSnapshot,
): void {
  try {
    for (const entry of snapshot.entries) {
      const targetPath = path.join(workDir, entry.name);
      fs.rmSync(targetPath, { recursive: true, force: true });
      fs.mkdirSync(path.dirname(targetPath), { recursive: true });
      fs.renameSync(entry.tempPath, targetPath);
    }
  } finally {
    if (snapshot.root != null) {
      fs.rmSync(snapshot.root, { recursive: true, force: true });
    }
  }
}

function cleanMutableWorktreeCaches(workDir: string): void {
  for (const relativePath of MUTABLE_WORKTREE_CACHE_PATHS) {
    fs.rmSync(path.join(workDir, relativePath), { recursive: true, force: true });
  }
}

function syncGitFixtureWorktree(name: string, variant?: string): string {
  const sourceDir = getGitFixtureSourceDir(name);
  const workDir = getMutableGitFixtureDir(name, variant);
  const parentDir = path.dirname(workDir);

  fs.mkdirSync(parentDir, { recursive: true });

  const stagingDir = fs.mkdtempSync(path.join(parentDir, `${name}-staging-`));
  exportGitHeadToDir(sourceDir, stagingDir);

  const preserved = preserveMutableWorktreeEntries(workDir);

  try {
    fs.rmSync(workDir, { recursive: true, force: true });
    fs.renameSync(stagingDir, workDir);
  } catch (error) {
    if (!fs.existsSync(workDir)) {
      fs.mkdirSync(workDir, { recursive: true });
    }
    restorePreservedWorktreeEntries(workDir, preserved);
    throw error;
  } finally {
    fs.rmSync(stagingDir, { recursive: true, force: true });
  }

  restorePreservedWorktreeEntries(workDir, preserved);
  cleanMutableWorktreeCaches(workDir);
  ensureFileContent(
    path.join(workDir, ".vize-fixture-source.json"),
    `${JSON.stringify(
      {
        revision: readGitHeadRevision(sourceDir),
        sourceDir,
      },
      null,
      2,
    )}\n`,
  );

  return workDir;
}

const ELK_WORK_DIR = getMutableGitFixtureDir("elk");
export const MISSKEY_WORK_DIR = getMutableGitFixtureDir("misskey");
const NPMX_WORK_DIR = getMutableGitFixtureDir("npmx.dev");
const FRONTEND_PHPCON_WORK_DIR = getMutableGitFixtureDir("frontend-phpcon-do-website");
const NUXT_UI_WORK_DIR = getMutableGitFixtureDir("nuxt-ui", "playground");
const REKA_UI_DOCS_WORK_DIR = getMutableGitFixtureDir("reka-ui", "docs");
const VUEFES_WORK_DIR = getMutableGitFixtureDir("vuefes-2025");

// --- App configurations ---

const ELK_E2E_ENV = {
  NUXT_STORAGE_DRIVER: "fs",
  VIZE_E2E_BUILD_TIME: "1767225600000",
} as const;

function patchElkBuildEnvTime(buildEnvPath: string): void {
  const source = fs.readFileSync(buildEnvPath, "utf-8");
  const nextSource = source.replace(
    "      time: +Date.now(),",
    "      time: Number(process.env.VIZE_E2E_BUILD_TIME ?? Date.now()),",
  );
  if (nextSource !== source) {
    fs.writeFileSync(buildEnvPath, nextSource);
  }
}

function setupElkWorktree(opts?: { enableVize?: boolean; variant?: string }): string {
  const enableVize = opts?.enableVize ?? true;
  const elkDir = syncGitFixtureWorktree("elk", opts?.variant);

  if (enableVize) {
    ensureLocalVizePackagesBuilt();
  }

  addPnpmOverrides(path.join(elkDir, "package.json"), {
    vite: "^8.0.0",
  });
  patchElkBuildEnvTime(path.join(elkDir, "modules", "build-env.ts"));

  console.log(`[elk:${enableVize ? "candidate" : "reference"}:setup] pnpm install...`);
  execSync("npx -y pnpm@10 install --no-frozen-lockfile", {
    cwd: elkDir,
    stdio: "inherit",
    timeout: 300_000,
  });

  if (enableVize) {
    createVizeSymlinks(path.join(elkDir, "node_modules"));
  }
  patchNuxtConfig(path.join(elkDir, "nuxt.config.ts"), { enableVize });

  return elkDir;
}

export const elkApp: AppConfig = {
  name: "elk",
  cwd: ELK_WORK_DIR,
  command: "npx",
  args: ["-y", "pnpm@10", "exec", "nuxt", "dev", "--port", "5314", "--host", "0.0.0.0"],
  port: 5314,
  url: "http://127.0.0.1:5314",
  mountSelector: "#__nuxt",
  readyPattern: /Local:\s+http:\/\/(localhost|127\.0\.0\.1|0\.0\.0\.0):5314/,
  allowNon200: true,
  waitUntil: "load",
  readyDelay: 15_000,
  startupTimeout: 120_000,
  env: ELK_E2E_ENV,
  setup() {
    setupElkWorktree();
  },
  build: {
    command: "npx",
    args: ["-y", "pnpm@10", "build"],
    timeout: 300_000,
  },
  preview: {
    command: "npx",
    args: ["-y", "pnpm@10", "start"],
    port: 5315,
    url: "http://localhost:5315",
    readyPattern: /Listening on/,
  },
  check: {
    cwd: path.join(GIT_DIR, "elk"),
    patterns: ["app/**/*.vue"],
  },
  lint: {
    cwd: path.join(GIT_DIR, "elk"),
    patterns: ["app/**/*.vue"],
  },
};

function createElkVisualParityApp(kind: "candidate" | "reference", port: number): AppConfig {
  const variant = `vrt-${kind}`;
  return {
    name: `elk:${kind}`,
    cwd: getMutableGitFixtureDir("elk", variant),
    command: "npx",
    args: ["-y", "pnpm@10", "exec", "nuxt", "dev", "--port", String(port), "--host", "0.0.0.0"],
    port,
    url: `http://127.0.0.1:${port}`,
    mountSelector: "#__nuxt",
    readyPattern: new RegExp(
      `Local:\\s+http:\\/\\/(localhost|127\\.0\\.0\\.1|0\\.0\\.0\\.0):${port}`,
    ),
    allowNon200: true,
    waitUntil: "load",
    readyDelay: 15_000,
    startupTimeout: 120_000,
    env: ELK_E2E_ENV,
    setup() {
      setupElkWorktree({ enableVize: kind === "candidate", variant });
    },
  };
}

export function createElkVisualParityApps(): { candidate: AppConfig; reference: AppConfig } {
  return {
    reference: createElkVisualParityApp("reference", 5324),
    candidate: createElkVisualParityApp("candidate", 5325),
  };
}

function setupMisskeyWorktree(opts?: {
  base?: string;
  enableVize?: boolean;
  port?: number;
  variant?: string;
}): string {
  const base = opts?.base ?? "/vite/";
  const enableVize = opts?.enableVize ?? true;
  const port = opts?.port ?? 5173;
  const misskeyDir = syncGitFixtureWorktree("misskey", opts?.variant);
  const frontendDir = path.join(misskeyDir, "packages", "frontend");

  if (enableVize) {
    ensureLocalVizePackagesBuilt();
  }

  // Create .config/default.yml
  const configDir = path.join(misskeyDir, ".config");
  const configFile = path.join(configDir, "default.yml");
  if (!fs.existsSync(configFile)) {
    fs.mkdirSync(configDir, { recursive: true });
    fs.writeFileSync(configFile, "url: http://localhost:3000\nport: 3000\n");
  }

  // Generate index.html
  const indexHtml = path.join(frontendDir, "index.html");
  fs.writeFileSync(
    indexHtml,
    `<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<meta property="instance_url" content="http://localhost:3000">
<meta property="og:site_name" content="Misskey">
</head>
<body>
<div id="misskey_app"></div>
<script type="module" src="/src/_boot_.ts"></script>
</body>
</html>
`,
  );

  addPnpmOverrides(path.join(misskeyDir, "package.json"), {
    vite: "^8.0.0",
  });

  console.log(`[misskey:${enableVize ? "candidate" : "reference"}:setup] pnpm install...`);
  execSync("npx -y pnpm@10 install --no-frozen-lockfile --ignore-scripts", {
    cwd: misskeyDir,
    env: {
      ...process.env,
      CYPRESS_INSTALL_BINARY: "0",
      PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD: "1",
      PUPPETEER_SKIP_DOWNLOAD: "1",
    },
    stdio: "inherit",
    timeout: 300_000,
  });

  ensureMisskeyFluentEmojiAssets(misskeyDir);
  ensureMisskeyOptionalDependencyStubs(misskeyDir);

  // Build workspace packages needed by frontend
  for (const pkg of [
    "i18n",
    "icons-subsetter",
    "misskey-js",
    "misskey-bubble-game",
    "misskey-reversi",
    "frontend-shared",
  ]) {
    console.log(
      `[misskey:${enableVize ? "candidate" : "reference"}:setup] building ${pkg} package...`,
    );
    execSync(`npx -y pnpm@10 --filter ${pkg} build`, {
      cwd: misskeyDir,
      stdio: "inherit",
      timeout: 120_000,
    });
  }

  if (enableVize) {
    createVizeSymlinks(path.join(misskeyDir, "node_modules"));
  }

  // Patch vite.config.ts
  const viteConfigPath = path.join(frontendDir, "vite.config.ts");
  let viteConfig = fs.readFileSync(viteConfigPath, "utf-8");
  if (enableVize && !viteConfig.includes("@vizejs/vite-plugin")) {
    viteConfig = viteConfig.replace(
      "import pluginVue from '@vitejs/plugin-vue';",
      "import { vize as pluginVue } from '@vizejs/vite-plugin';",
    );
  }
  viteConfig = viteConfig
    .replace(/base:\s*['"]\/vite\/['"],/g, `base: ${JSON.stringify(base)},`)
    .replace(/port:\s*5173,/g, `port: ${port},`)
    .replace(/clientPort:\s*5173,/g, `clientPort: ${port},`);
  fs.writeFileSync(viteConfigPath, viteConfig);

  removeManualChunksObject(viteConfigPath);
  removeManualChunksObject(path.join(misskeyDir, "packages", "frontend-embed", "vite.config.ts"));
  mirrorLoaderAssetsForViteBase(path.join(frontendDir, "public"), "vite");
  mirrorLoaderAssetsForViteBase(
    path.join(misskeyDir, "packages", "frontend-embed", "public"),
    "embed_vite",
  );

  const clientServerServicePath = path.join(
    misskeyDir,
    "packages",
    "backend",
    "src",
    "server",
    "web",
    "ClientServerService.ts",
  );
  let clientServerService = fs.readFileSync(clientServerServicePath, "utf-8");
  let clientServerServiceChanged = false;
  if (clientServerService.includes("rewritePrefix: '/vite',")) {
    clientServerService = clientServerService.replace(
      "rewritePrefix: '/vite',",
      "rewritePrefix: '',",
    );
    clientServerServiceChanged = true;
  }
  if (clientServerService.includes("rewritePrefix: '/embed_vite',")) {
    clientServerService = clientServerService.replace(
      "rewritePrefix: '/embed_vite',",
      "rewritePrefix: '',",
    );
    clientServerServiceChanged = true;
  }
  if (clientServerServiceChanged) {
    fs.writeFileSync(clientServerServicePath, clientServerService);
  }

  const misskeyDevScriptPath = path.join(misskeyDir, "scripts", "dev.mjs");
  let misskeyDevScript = fs.readFileSync(misskeyDevScriptPath, "utf-8");
  if (!misskeyDevScript.includes("['--filter', 'frontend', 'build']")) {
    misskeyDevScript = misskeyDevScript.replace(
      `\texeca('pnpm', ['--filter', 'backend...', 'build'], {\n\t\tcwd: _dirname + '/../',\n\t\tstdout: process.stdout,\n\t\tstderr: process.stderr,\n\t}),`,
      `\texeca('pnpm', ['--filter', 'backend...', 'build'], {\n\t\tcwd: _dirname + '/../',\n\t\tstdout: process.stdout,\n\t\tstderr: process.stderr,\n\t}),\n\texeca('pnpm', ['--filter', 'frontend', 'build'], {\n\t\tcwd: _dirname + '/../',\n\t\tstdout: process.stdout,\n\t\tstderr: process.stderr,\n\t}),\n\texeca('pnpm', ['--filter', 'frontend-embed', 'build'], {\n\t\tcwd: _dirname + '/../',\n\t\tstdout: process.stdout,\n\t\tstderr: process.stderr,\n\t}),`,
    );
  }
  if (!misskeyDevScript.includes("await execa('pnpm', ['--filter', 'icons-subsetter', 'build']")) {
    misskeyDevScript = misskeyDevScript.replace(
      "await Promise.all([",
      `await execa('pnpm', ['--filter', 'icons-subsetter', 'build'], {\n\tcwd: _dirname + '/../',\n\tstdout: process.stdout,\n\tstderr: process.stderr,\n});\n\nawait Promise.all([`,
    );
    misskeyDevScript = misskeyDevScript.replace(
      `\t// icons-subsetterは開発段階では使用されないが、型エラーを抑制するためにはじめの一度だけビルドする\n\texeca('pnpm', ['--filter', 'icons-subsetter', 'build'], {\n\t\tcwd: _dirname + '/../',\n\t\tstdout: process.stdout,\n\t\tstderr: process.stderr,\n\t}),\n`,
      "",
    );
  }
  if (!misskeyDevScript.includes("['--filter', 'misskey-bubble-game', 'build']")) {
    misskeyDevScript = misskeyDevScript.replace(
      `\texeca('pnpm', ['--filter', 'misskey-js', 'build'], {\n\t\tcwd: _dirname + '/../',\n\t\tstdout: process.stdout,\n\t\tstderr: process.stderr,\n\t}),`,
      `\texeca('pnpm', ['--filter', 'misskey-js', 'build'], {\n\t\tcwd: _dirname + '/../',\n\t\tstdout: process.stdout,\n\t\tstderr: process.stderr,\n\t}),\n\texeca('pnpm', ['--filter', 'misskey-bubble-game', 'build'], {\n\t\tcwd: _dirname + '/../',\n\t\tstdout: process.stdout,\n\t\tstderr: process.stderr,\n\t}),`,
    );
  }
  fs.writeFileSync(misskeyDevScriptPath, misskeyDevScript);

  return misskeyDir;
}

export const misskeyApp: AppConfig = {
  name: "misskey",
  cwd: path.join(MISSKEY_WORK_DIR, "packages", "frontend"),
  command: "npx",
  args: ["-y", "pnpm@10", "exec", "vite"],
  port: 5173,
  url: "http://127.0.0.1:5173/vite/",
  mountSelector: "#misskey_app",
  readyPattern: /Local:\s+http:\/\//,
  allowNon200: true,
  waitUntil: "domcontentloaded",
  startupTimeout: 180_000,
  setup() {
    setupMisskeyWorktree({ enableVize: true, port: 5173 });
  },
  async setupPage(page) {
    await page.addInitScript(() => {
      const _origFetch = window.fetch;
      window.fetch = function (input, init) {
        const url =
          typeof input === "string" ? input : input instanceof URL ? input.toString() : input.url;
        if (url.includes("/api/")) {
          let body = "{}";
          if (url.includes("/api/meta")) {
            body = JSON.stringify({
              name: "Misskey",
              uri: "http://localhost:3000",
              version: "2024.11.0",
              description: "A Misskey instance",
              disableRegistration: false,
              federation: "all",
              iconUrl: null,
              backgroundImageUrl: null,
              defaultDarkTheme: null,
              defaultLightTheme: null,
              clientOptions: {},
              policies: { ltlAvailable: true, gtlAvailable: true },
              maxNoteTextLength: 3000,
              features: {
                registration: true,
                localTimeline: true,
                globalTimeline: true,
                miauth: true,
              },
            });
          } else if (url.includes("/api/emojis")) {
            body = JSON.stringify({ emojis: [] });
          }
          return Promise.resolve(
            new Response(body, {
              status: 200,
              headers: { "Content-Type": "application/json" },
            }),
          );
        }
        if (url.includes("/assets/locales/")) {
          return Promise.resolve(
            new Response(
              JSON.stringify({
                _lang_: "English",
                headlineMisskey: "A network connected by notes",
                introMisskey:
                  "Welcome! Misskey is an open source, decentralized microblogging platform.",
                monthAndDay: "{month}/{day}",
                search: "Search",
                notifications: "Notifications",
                username: "Username",
                password: "Password",
                forgotPassword: "Forgot password",
                fetchingAsAp498: "Fetching...",
                login: "Sign In",
                loggingIn: "Signing In",
                signup: "Sign Up",
                uploading: "Uploading...",
                save: "Save",
                users: "Users",
                notes: "Notes",
                following: "Following",
                followers: "Followers",
                ok: "OK",
                gotIt: "Got it!",
                cancel: "Cancel",
                enterUsername: "Enter username",
                renotedBy: "Boosted by {user}",
                noNotes: "No notes",
                noNotifications: "No notifications",
                instance: "Instance",
                settings: "Settings",
                basicSettings: "General",
                otherSettings: "Other Settings",
                openInWindow: "Open in window",
                profile: "Profile",
                timeline: "Timeline",
                noAccountDescription: "No description",
                loginFailed: "Sign in failed",
                showMore: "Show More",
                youGotNewFollower: "followed you",
                explore: "Explore",
                favorited: "Favorited",
                unfavorite: "Unfavorite",
                pinnedNote: "Pinned note",
                somethingHappened: "Something went wrong",
                retry: "Retry",
                pageLoadError: "An error occurred while loading the page.",
                pageLoadErrorDescription:
                  "This is usually caused by a network error or the browser's cache.",
                serverIsDead: "Server is not responding. Please wait a moment and try again.",
                youShouldUpgradeClient: "Please refresh the page to use the updated client.",
                enterListName: "Enter list name",
                privacy: "Privacy",
                makeFollowManuallyApprove: "Follow requests require approval",
                defaultNavigationBehaviour: "Default navigation behavior",
                editProfile: "Edit profile",
                noteOfThisUser: "Notes by this user",
                joinThisServer: "Sign up at this instance",
                exploreOtherServers: "Look for another instance",
                letsLookAtTimeline: "Have a look at the timeline",
                invitationRequiredToRegister: "This instance is invite-only.",
              }),
              {
                status: 200,
                headers: { "Content-Type": "application/json" },
              },
            ),
          );
        }
        return _origFetch.call(window, input, init);
      } as typeof window.fetch;
    });
  },
  build: {
    command: "npx",
    args: ["-y", "pnpm@10", "exec", "vite", "build"],
    timeout: 180_000,
  },
  preview: {
    command: "npx",
    args: ["-y", "pnpm@10", "exec", "vite", "preview", "--port", "5174"],
    port: 5174,
    url: "http://localhost:5174/vite/",
    readyPattern: /Local:\s+http:\/\//,
  },
  check: {
    cwd: path.join(MISSKEY_WORK_DIR, "packages", "frontend"),
    patterns: ["src/**/*.vue"],
  },
  lint: {
    cwd: path.join(MISSKEY_WORK_DIR, "packages", "frontend"),
    patterns: ["src/**/*.vue"],
  },
};

function createMisskeyVisualParityApp(kind: "candidate" | "reference", port: number): AppConfig {
  const variant = `vrt-${kind}`;
  return {
    name: `misskey:${kind}`,
    cwd: path.join(getMutableGitFixtureDir("misskey", variant), "packages", "frontend"),
    command: "npx",
    args: ["-y", "pnpm@10", "exec", "vite"],
    port,
    url: `http://127.0.0.1:${port}/`,
    mountSelector: "#misskey_app",
    readyPattern: /Local:\s+http:\/\//,
    allowNon200: true,
    waitUntil: "domcontentloaded",
    startupTimeout: 180_000,
    setup() {
      setupMisskeyWorktree({
        base: "/",
        enableVize: kind === "candidate",
        port,
        variant,
      });
    },
  };
}

export function createMisskeyVisualParityApps(): {
  candidate: AppConfig;
  reference: AppConfig;
} {
  return {
    reference: createMisskeyVisualParityApp("reference", 5322),
    candidate: createMisskeyVisualParityApp("candidate", 5323),
  };
}

export const npmxApp: AppConfig = {
  name: "npmx.dev",
  cwd: NPMX_WORK_DIR,
  command: "npx",
  args: ["-y", "pnpm@10", "exec", "nuxt", "dev", "--port", "3001", "--host", "0.0.0.0"],
  port: 3001,
  url: "http://127.0.0.1:3001",
  mountSelector: "#__nuxt",
  readyPattern: /Local:\s+http:\/\/(localhost|127\.0\.0\.1|0\.0\.0\.0):3001/,
  allowNon200: true,
  waitUntil: "load",
  readyDelay: 30_000,
  env: NPMX_E2E_ENV,
  startupTimeout: 120_000,
  setup() {
    setupNpmxWorktree();
  },
  build: {
    command: "npx",
    args: ["-y", "pnpm@10", "build"],
    timeout: 300_000,
  },
  preview: {
    command: "npx",
    args: ["-y", "pnpm@10", "exec", "nuxt", "preview", "--port", "3002"],
    port: 3002,
    url: "http://127.0.0.1:3002",
    readyPattern: /Listening on/,
  },
  check: {
    cwd: path.join(GIT_DIR, "npmx.dev"),
    patterns: ["app/**/*.vue"],
  },
  lint: {
    cwd: path.join(GIT_DIR, "npmx.dev"),
    patterns: ["app/**/*.vue"],
  },
};

function setupNpmxWorktree(opts?: { enableVize?: boolean; variant?: string }): string {
  const enableVize = opts?.enableVize ?? true;
  const npmxDir = syncGitFixtureWorktree("npmx.dev", opts?.variant);
  const nmDir = path.join(npmxDir, "node_modules");

  if (enableVize) {
    ensureLocalVizePackagesBuilt();
  }

  const packageJsonPath = path.join(npmxDir, "package.json");
  patchNpmxPackageJson(packageJsonPath);
  addPnpmOverrides(packageJsonPath, {
    vite: "^8.0.0",
  });

  console.log(`[npmx.dev:${enableVize ? "candidate" : "reference"}:setup] pnpm install...`);
  execSync("npx -y pnpm@10 install --no-frozen-lockfile --ignore-scripts", {
    cwd: npmxDir,
    stdio: "inherit",
    timeout: 300_000,
    env: {
      ...process.env,
      ...NPMX_E2E_ENV,
    },
  });
  for (const script of ["generate:lexicons", "generate:sprite"]) {
    console.log(`[npmx.dev:${enableVize ? "candidate" : "reference"}:setup] pnpm ${script}...`);
    execSync(`npx -y pnpm@10 ${script}`, {
      cwd: npmxDir,
      stdio: "inherit",
      timeout: 120_000,
      env: {
        ...process.env,
        ...NPMX_E2E_ENV,
      },
    });
  }

  if (enableVize) {
    createVizeSymlinks(nmDir);
  }

  patchNuxtConfig(path.join(npmxDir, "nuxt.config.ts"), {
    enableVize,
    removeModules: ["@nuxtjs/html-validator"],
  });
  patchNpmxPrerenderRoutes(path.join(npmxDir, "nuxt.config.ts"));
  patchNpmxLunariaModule(path.join(npmxDir, "modules", "lunaria.ts"));

  const npmxAppPath = path.join(npmxDir, "app", "app.vue");
  const npmxAppSource = fs.readFileSync(npmxAppPath, "utf-8");
  const nextNpmxAppSource = npmxAppSource.replace(/\n\s*<NuxtPwaAssets\s*\/>\s*/g, "\n");
  if (nextNpmxAppSource !== npmxAppSource) {
    fs.writeFileSync(npmxAppPath, nextNpmxAppSource);
  }
  hoistPnpmPackage(nmDir, "vue-i18n");

  // Ensure .nuxt/tsconfig.server.json exists (vite 8 needs it at startup)
  console.log(`[npmx.dev:${enableVize ? "candidate" : "reference"}:setup] nuxt prepare...`);
  execSync("npx -y pnpm@10 exec nuxt prepare", {
    cwd: npmxDir,
    stdio: "inherit",
    timeout: 180_000,
    env: {
      ...process.env,
      ...NPMX_E2E_ENV,
    },
  });

  return npmxDir;
}

type VisualParityMode = "dev" | "preview";

function ensureNpmxPreviewVueRuntime(npmxDir: string): void {
  const outputNodeModulesDir = path.join(npmxDir, ".output", "server", "node_modules");
  const vueRuntimeLink = path.join(outputNodeModulesDir, "vue");
  const vueRuntimeTarget = path.join(npmxDir, "node_modules", "vue");
  const vuePackageJsonPath = path.join(vueRuntimeTarget, "package.json");
  const existingRendererEntry = path.join(vueRuntimeLink, "server-renderer", "index.mjs");

  if (fs.existsSync(existingRendererEntry) || !fs.existsSync(vueRuntimeTarget)) {
    return;
  }

  let target = vueRuntimeTarget;
  if (fs.existsSync(vuePackageJsonPath)) {
    const version = JSON.parse(fs.readFileSync(vuePackageJsonPath, "utf-8")).version;
    const bundledTarget = path.join(outputNodeModulesDir, ".nitro", `vue@${version}`);
    if (fs.existsSync(path.join(bundledTarget, "server-renderer", "index.mjs"))) {
      target = bundledTarget;
    }
  }

  fs.mkdirSync(outputNodeModulesDir, { recursive: true });
  try {
    const stat = fs.lstatSync(vueRuntimeLink);
    if (stat.isSymbolicLink()) {
      fs.unlinkSync(vueRuntimeLink);
      fs.symlinkSync(target, vueRuntimeLink, "dir");
      return;
    }
  } catch {
    // does not exist
  }

  ensureSymlink(path.join(vueRuntimeLink, "server-renderer"), path.join(target, "server-renderer"));
}

function createNpmxVisualParityApp(
  kind: "candidate" | "reference",
  port: number,
  mode: VisualParityMode,
): AppConfig {
  const variant = `vrt-${mode}-${kind}`;
  const command =
    mode === "preview"
      ? ["exec", "nuxt", "preview", "--port", String(port)]
      : ["exec", "nuxt", "dev", "--port", String(port), "--host", "0.0.0.0"];

  return {
    name: `npmx.dev:${mode}:${kind}`,
    cwd: getMutableGitFixtureDir("npmx.dev", variant),
    command: "npx",
    args: ["-y", "pnpm@10", ...command],
    port,
    url: `http://127.0.0.1:${port}`,
    mountSelector: "#__nuxt",
    readyPattern:
      mode === "preview"
        ? /Listening on/
        : new RegExp(`Local:\\s+http:\\/\\/(localhost|127\\.0\\.0\\.1|0\\.0\\.0\\.0):${port}`),
    allowNon200: true,
    waitUntil: "load",
    readyDelay: 30_000,
    env: NPMX_E2E_ENV,
    startupTimeout: 120_000,
    setup() {
      const npmxDir = setupNpmxWorktree({ enableVize: kind === "candidate", variant });
      if (mode === "preview") {
        execSync("npx -y pnpm@10 build", {
          cwd: npmxDir,
          env: {
            ...process.env,
            ...NPMX_E2E_ENV,
            NODE_ENV: "production",
          },
          stdio: "inherit",
          timeout: 300_000,
        });
        ensureNpmxPreviewVueRuntime(npmxDir);
      }
    },
  };
}

export function createNpmxVisualParityApps(mode: VisualParityMode = "dev"): {
  candidate: AppConfig;
  reference: AppConfig;
} {
  const referencePort = mode === "preview" ? 5330 : 5320;
  const candidatePort = mode === "preview" ? 5331 : 5321;

  return {
    reference: createNpmxVisualParityApp("reference", referencePort, mode),
    candidate: createNpmxVisualParityApp("candidate", candidatePort, mode),
  };
}

export const frontendPhpconApp: AppConfig = {
  name: "frontend-phpcon-do-website",
  cwd: FRONTEND_PHPCON_WORK_DIR,
  command: "npx",
  args: ["-y", "pnpm@10", "exec", "nuxt", "dev", "--port", "3007", "--host", "0.0.0.0"],
  port: 3007,
  url: "http://127.0.0.1:3007",
  mountSelector: "#__nuxt",
  readyPattern: /Local:\s+http:\/\/(localhost|127\.0\.0\.1|0\.0\.0\.0):3007/,
  allowNon200: true,
  waitUntil: "load",
  readyDelay: 15_000,
  env: FRONTEND_PHPCON_E2E_ENV,
  startupTimeout: 240_000,
  setup() {
    setupFrontendPhpconWorktree();
  },
  build: {
    command: "npx",
    args: ["-y", "pnpm@10", "build"],
    timeout: 300_000,
  },
  preview: {
    command: "npx",
    args: ["-y", "pnpm@10", "exec", "nuxt", "preview", "--port", "3008"],
    port: 3008,
    url: "http://127.0.0.1:3008",
    readyPattern: /Listening on/,
  },
  check: {
    cwd: path.join(GIT_DIR, "frontend-phpcon-do-website"),
    patterns: ["app/**/*.vue"],
  },
  lint: {
    cwd: path.join(GIT_DIR, "frontend-phpcon-do-website"),
    patterns: ["app/**/*.vue"],
  },
};

function patchFrontendPhpconVisualFixture(frontendDir: string): void {
  const configPath = path.join(frontendDir, "nuxt.config.ts");
  const source = fs.readFileSync(configPath, "utf-8");
  let nextSource = source.replace('    preset: "cloudflare_module",', '    preset: "node-server",');
  nextSource = nextSource.replace(
    `  content: {
    database: {
      type: "d1",
      bindingName: "DB",
    },
  },`,
    "  content: {},",
  );
  nextSource = nextSource.replace(
    "  devtools: { enabled: true },",
    "  devtools: { enabled: false },",
  );
  if (nextSource !== source) {
    fs.writeFileSync(configPath, nextSource);
  }

  ensureFileContent(
    path.join(frontendDir, "server", "routes", "__vize_e2e", "api", "sponsors.get.ts"),
    `export default defineEventHandler(() => ({
  sponsor_plans: [
    {
      name: "Platinum",
      name_en: "Platinum",
      tier: "A",
      sponsors: [
        {
          id: "e3f03260-28a2-4cd8-8f4c-d6cc61774ca5",
          name: "Frontend PHP Labs",
          pr: "Building reliable web platforms with Vue and PHP.",
          url: "https://example.com/frontend-php-labs",
        },
        {
          id: "ea9a7096-2de4-407e-8e60-ef69fc8fa588",
          name: "Hokkaido Web Studio",
          pr: "Local engineering team supporting the conference community.",
          url: "https://example.com/hokkaido-web-studio",
        },
      ],
    },
    {
      name: "Gold",
      name_en: "Gold",
      tier: "B",
      sponsors: [
        {
          id: "2ad98278-5a9a-4a17-bbe6-924be25534a9",
          name: "Sapporo Type Systems",
          pr: "Tooling, design systems, and product engineering.",
          url: "https://example.com/sapporo-type-systems",
        },
      ],
    },
    {
      name: "Booth",
      name_en: "Booth",
      tier: "C",
      sponsors: [
        {
          id: "8e55bc66-2146-4115-8d7c-55eb4cfc83bc",
          name: "Filtered Booth Sponsor",
          url: "https://example.com/booth",
        },
      ],
    },
  ],
}));
`,
  );

  for (const imageDir of [
    path.join(frontendDir, "public", "individual-sponsors"),
    path.join(frontendDir, "public", "sponsors"),
  ]) {
    replacePngFilesWithTransparentFixture(imageDir);
  }
}

function replacePngFilesWithTransparentFixture(dir: string): void {
  if (!fs.existsSync(dir)) {
    return;
  }

  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const entryPath = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      replacePngFilesWithTransparentFixture(entryPath);
      continue;
    }
    if (entry.isFile() && entry.name.endsWith(".png")) {
      fs.writeFileSync(entryPath, TRANSPARENT_PNG);
    }
  }
}

function setupFrontendPhpconWorktree(opts?: { enableVize?: boolean; variant?: string }): string {
  const enableVize = opts?.enableVize ?? true;
  const frontendDir = syncGitFixtureWorktree("frontend-phpcon-do-website", opts?.variant);

  if (enableVize) {
    ensureLocalVizePackagesBuilt();
  }

  addPnpmOverrides(path.join(frontendDir, "package.json"), {
    ...VUE_BETA_OVERRIDES,
    vite: "^8.0.0",
  });
  patchNuxtConfig(path.join(frontendDir, "nuxt.config.ts"), {
    enableVize,
    removeModules: ["@nuxt/fonts", "@nuxt/test-utils"],
  });
  patchFrontendPhpconVisualFixture(frontendDir);

  console.log(
    `[frontend-phpcon-do-website:${enableVize ? "candidate" : "reference"}:setup] pnpm install...`,
  );
  execSync("npx -y pnpm@10 install --no-frozen-lockfile", {
    cwd: frontendDir,
    stdio: "inherit",
    timeout: 300_000,
    env: {
      ...process.env,
      ...FRONTEND_PHPCON_E2E_ENV,
    },
  });

  if (enableVize) {
    createVizeSymlinks(path.join(frontendDir, "node_modules"));
  }

  console.log(
    `[frontend-phpcon-do-website:${enableVize ? "candidate" : "reference"}:setup] nuxt prepare...`,
  );
  execSync("npx -y pnpm@10 exec nuxt prepare", {
    cwd: frontendDir,
    stdio: "inherit",
    timeout: 180_000,
    env: {
      ...process.env,
      ...FRONTEND_PHPCON_E2E_ENV,
    },
  });

  return frontendDir;
}

type FrontendPhpconVisualParityMode = "dev" | "preview";

function createFrontendPhpconVisualParityApp(
  kind: "candidate" | "reference",
  port: number,
  mode: FrontendPhpconVisualParityMode,
): AppConfig {
  const variant = `vrt-${mode}-${kind}`;
  const command =
    mode === "preview"
      ? ["exec", "nuxt", "preview", "--port", String(port)]
      : ["exec", "nuxt", "dev", "--port", String(port), "--host", "0.0.0.0"];

  return {
    name: `frontend-phpcon-do-website:${mode}:${kind}`,
    cwd: getMutableGitFixtureDir("frontend-phpcon-do-website", variant),
    command: "npx",
    args: ["-y", "pnpm@10", ...command],
    port,
    url: `http://127.0.0.1:${port}`,
    mountSelector: "#__nuxt",
    readyPattern:
      mode === "preview"
        ? /Listening on/
        : new RegExp(`Local:\\s+http:\\/\\/(localhost|127\\.0\\.0\\.1|0\\.0\\.0\\.0):${port}`),
    allowNon200: true,
    waitUntil: "load",
    readyDelay: 15_000,
    env: {
      ...FRONTEND_PHPCON_E2E_ENV,
      NODE_ENV: mode === "preview" ? "production" : "development",
    },
    startupTimeout: 300_000,
    setup() {
      const frontendDir = setupFrontendPhpconWorktree({
        enableVize: kind === "candidate",
        variant,
      });
      if (mode === "preview") {
        execSync("npx -y pnpm@10 build", {
          cwd: frontendDir,
          env: {
            ...process.env,
            ...FRONTEND_PHPCON_E2E_ENV,
            NODE_ENV: "production",
          },
          stdio: "inherit",
          timeout: 300_000,
        });
      }
    },
  };
}

export function createFrontendPhpconVisualParityApps(
  mode: FrontendPhpconVisualParityMode = "dev",
): {
  candidate: AppConfig;
  reference: AppConfig;
} {
  const referencePort = mode === "preview" ? 5338 : 5336;
  const candidatePort = mode === "preview" ? 5339 : 5337;

  return {
    reference: createFrontendPhpconVisualParityApp("reference", referencePort, mode),
    candidate: createFrontendPhpconVisualParityApp("candidate", candidatePort, mode),
  };
}

export const vuefesApp: AppConfig = {
  name: "vuefes-2025",
  cwd: VUEFES_WORK_DIR,
  command: "npx",
  args: ["-y", "pnpm@10", "exec", "nuxt", "dev", "--port", "3003", "--host", "0.0.0.0"],
  port: 3003,
  url: "http://127.0.0.1:3003",
  mountSelector: "#__nuxt",
  readyPattern: /Local:\s+http:\/\/(localhost|127\.0\.0\.1|0\.0\.0\.0):3003/,
  allowNon200: true,
  waitUntil: "load",
  readyDelay: 30_000,
  startupTimeout: 180_000,
  setup() {
    setupVuefesWorktree();
  },
  build: {
    command: "npx",
    args: ["-y", "pnpm@10", "build"],
    timeout: 300_000,
  },
  preview: {
    command: "npx",
    args: ["-y", "pnpm@10", "exec", "nuxt", "preview", "--port", "3004"],
    port: 3004,
    url: "http://127.0.0.1:3004",
    readyPattern: /Listening on/,
  },
  check: {
    cwd: path.join(GIT_DIR, "vuefes-2025"),
    patterns: ["app/**/*.vue"],
  },
  lint: {
    cwd: path.join(GIT_DIR, "vuefes-2025"),
    patterns: ["app/**/*.vue"],
  },
};

function setupVuefesWorktree(opts?: { enableVize?: boolean; variant?: string }): string {
  const enableVize = opts?.enableVize ?? true;
  const vuefesDir = syncGitFixtureWorktree("vuefes-2025", opts?.variant);

  if (enableVize) {
    ensureLocalVizePackagesBuilt();
  }

  // Ensure pnpm-workspace.yaml exists so pnpm doesn't resolve the parent workspace
  const wsYaml = path.join(vuefesDir, "pnpm-workspace.yaml");
  if (!fs.existsSync(wsYaml)) {
    fs.writeFileSync(wsYaml, "packages: []\n");
  }

  // Relax packageManager and engines constraints for e2e environment
  const vuefesPackageJson = path.join(vuefesDir, "package.json");
  const pkg = JSON.parse(fs.readFileSync(vuefesPackageJson, "utf-8"));
  let changed = false;
  if (pkg.packageManager) {
    delete pkg.packageManager;
    changed = true;
  }
  if (pkg.engines?.node) {
    pkg.engines = { pnpm: pkg.engines.pnpm ?? ">=10" };
    changed = true;
  }
  if (changed) {
    fs.writeFileSync(vuefesPackageJson, JSON.stringify(pkg, null, "\t") + "\n");
  }

  addPnpmOverrides(vuefesPackageJson, {
    vite: "^8.0.0",
  });
  patchVuefesVisualFixture(vuefesDir);

  console.log(`[vuefes-2025:${enableVize ? "candidate" : "reference"}:setup] pnpm install...`);
  execSync("npx -y pnpm@10 install --no-frozen-lockfile", {
    cwd: vuefesDir,
    stdio: "inherit",
    timeout: 300_000,
  });

  if (enableVize) {
    createVizeSymlinks(path.join(vuefesDir, "node_modules"));
  }
  patchNuxtConfig(path.join(vuefesDir, "nuxt.config.ts"), {
    enableVize,
    removeModules: ["@nuxtjs/storybook"],
  });

  console.log(`[vuefes-2025:${enableVize ? "candidate" : "reference"}:setup] nuxt prepare...`);
  execSync("npx -y pnpm@10 exec nuxt prepare", {
    cwd: vuefesDir,
    stdio: "inherit",
    timeout: 180_000,
  });

  return vuefesDir;
}

type VuefesVisualParityMode = "dev" | "preview";

function createVuefesVisualParityApp(
  kind: "candidate" | "reference",
  port: number,
  mode: VuefesVisualParityMode,
): AppConfig {
  const variant = `vrt-${mode}-${kind}`;
  const command =
    mode === "preview"
      ? ["exec", "nuxt", "preview", "--port", String(port)]
      : ["exec", "nuxt", "dev", "--port", String(port), "--host", "0.0.0.0"];

  return {
    name: `vuefes-2025:${mode}:${kind}`,
    cwd: getMutableGitFixtureDir("vuefes-2025", variant),
    command: "npx",
    args: ["-y", "pnpm@10", ...command],
    port,
    url: mode === "preview" ? `http://127.0.0.1:${port}/2025` : `http://127.0.0.1:${port}`,
    mountSelector: "#__nuxt",
    readyPattern:
      mode === "preview"
        ? /Listening on/
        : new RegExp(`Local:\\s+http:\\/\\/(localhost|127\\.0\\.0\\.1|0\\.0\\.0\\.0):${port}`),
    allowNon200: true,
    waitUntil: "load",
    readyDelay: 30_000,
    env: {
      ...VUEFES_E2E_ENV,
      CONTEXT: "production",
      NODE_ENV: mode === "preview" ? "production" : "development",
    },
    startupTimeout: 300_000,
    setup() {
      const vuefesDir = setupVuefesWorktree({ enableVize: kind === "candidate", variant });
      if (mode === "preview") {
        execSync("npx -y pnpm@10 build", {
          cwd: vuefesDir,
          env: {
            ...process.env,
            ...VUEFES_E2E_ENV,
            CONTEXT: "production",
            NODE_ENV: "production",
          },
          stdio: "inherit",
          timeout: 300_000,
        });
      }
    },
  };
}

export function createVuefesVisualParityApps(mode: VuefesVisualParityMode = "dev"): {
  candidate: AppConfig;
  reference: AppConfig;
} {
  const referencePort = mode === "preview" ? 5332 : 5326;
  const candidatePort = mode === "preview" ? 5333 : 5327;

  return {
    reference: createVuefesVisualParityApp("reference", referencePort, mode),
    candidate: createVuefesVisualParityApp("candidate", candidatePort, mode),
  };
}

export const antDesignVueApp: AppConfig = {
  name: "ant-design-vue",
  cwd: path.join(GIT_DIR, "ant-design-vue"),
  command: "npx",
  args: ["pnpm@10", "dev"],
  port: 5316,
  url: "http://localhost:5316",
  mountSelector: "#app",
  readyPattern: /Local:\s+http:\/\/localhost:5316/,
  allowNon200: true,
  waitUntil: "load",
  readyDelay: 10_000,
  startupTimeout: 120_000,
  check: {
    cwd: path.join(GIT_DIR, "ant-design-vue"),
    patterns: ["components/**/*.vue", "site/**/*.vue"],
  },
  lint: {
    cwd: path.join(GIT_DIR, "ant-design-vue"),
    patterns: ["components/**/*.vue", "site/**/*.vue"],
  },
};

export const nuxtUiApp: AppConfig = {
  name: "nuxt-ui",
  cwd: NUXT_UI_WORK_DIR,
  command: "npx",
  args: ["-y", "pnpm@10", "dev", "--port", "5317", "--host", "0.0.0.0"],
  port: 5317,
  url: "http://localhost:5317",
  mountSelector: "#__nuxt",
  readyPattern: /Local:\s+http:\/\/localhost:5317/,
  allowNon200: true,
  waitUntil: "load",
  readyDelay: 10_000,
  startupTimeout: 180_000,
  setup() {
    const nuxtUiDir = syncGitFixtureWorktree("nuxt-ui", "playground");
    const nuxtConfigPath = path.join(nuxtUiDir, "playgrounds", "nuxt", "nuxt.config.ts");
    const nuxtUiLinkPath = path.join(nuxtUiDir, "src", "runtime", "components", "Link.vue");
    const nuxtUiFormPath = path.join(nuxtUiDir, "src", "runtime", "components", "Form.vue");

    ensureLocalVizePackagesBuilt();
    installPnpmDependencies(nuxtUiDir);
    createVizeSymlinks(path.join(nuxtUiDir, "node_modules"));
    patchNuxtConfig(nuxtConfigPath);
    patchNuxtUiLinkComponent(nuxtUiLinkPath);
    patchNuxtUiFormComponent(nuxtUiFormPath);
    let nuxtConfig = fs.readFileSync(nuxtConfigPath, "utf-8");
    if (!nuxtConfig.includes("@nuxt/content")) {
      nuxtConfig = nuxtConfig.replace(
        "modules: [\n    '@vizejs/nuxt',",
        "modules: [\n    '@vizejs/nuxt',\n    '@nuxt/content',",
      );
      fs.writeFileSync(nuxtConfigPath, nuxtConfig);
    }
    if (!nuxtConfig.includes("handleNodeModulesVue: false")) {
      nuxtConfig = nuxtConfig.replace(
        "vize: {\n    musea: false,\n  },",
        "vize: {\n    musea: false,\n    compiler: {\n      handleNodeModulesVue: false,\n    },\n  },",
      );
      fs.writeFileSync(nuxtConfigPath, nuxtConfig);
    }
    console.log("[nuxt-ui:setup] pnpm dev:prepare...");
    execSync("npx -y pnpm@10 dev:prepare", {
      cwd: nuxtUiDir,
      stdio: "inherit",
      timeout: 900_000,
    });
  },
  check: {
    cwd: path.join(GIT_DIR, "nuxt-ui"),
    patterns: ["src/**/*.vue"],
  },
  lint: {
    cwd: path.join(GIT_DIR, "nuxt-ui"),
    patterns: ["src/**/*.vue"],
  },
};

export const rekaUiApp: AppConfig = {
  name: "reka-ui",
  cwd: path.join(GIT_DIR, "reka-ui"),
  command: "npx",
  args: ["pnpm@10", "dev"],
  port: 5318,
  url: "http://localhost:5318",
  mountSelector: "#app",
  readyPattern: /Local:\s+http:\/\/localhost:5318/,
  allowNon200: true,
  waitUntil: "load",
  readyDelay: 10_000,
  startupTimeout: 120_000,
  check: {
    cwd: path.join(GIT_DIR, "reka-ui"),
    patterns: ["packages/**/*.vue"],
  },
  lint: {
    cwd: path.join(GIT_DIR, "reka-ui"),
    patterns: ["packages/**/*.vue"],
  },
};

export const rekaUiDocsApp: AppConfig = {
  name: "reka-ui-docs",
  cwd: REKA_UI_DOCS_WORK_DIR,
  command: "npx",
  args: ["-y", "pnpm@10", "--filter", "docs", "docs:dev", "--port", "5318", "--host", "0.0.0.0"],
  port: 5318,
  url: "http://localhost:5318",
  mountSelector: "#app",
  readyPattern: /Local:\s+http:\/\/localhost:5318/,
  allowNon200: true,
  waitUntil: "load",
  readyDelay: 10_000,
  startupTimeout: 180_000,
  setup() {
    const rekaUiDir = syncGitFixtureWorktree("reka-ui", "docs");

    ensureLocalVizePackagesBuilt();
    installPnpmDependencies(rekaUiDir);
    createVizeSymlinks(path.join(rekaUiDir, "node_modules"));
    patchVitepressConfig(path.join(rekaUiDir, "docs", ".vitepress", "config.ts"));
    console.log("[reka-ui-docs:setup] pnpm --filter ./packages/core build...");
    execSync("npx -y pnpm@10 --filter ./packages/core build", {
      cwd: rekaUiDir,
      stdio: "inherit",
      timeout: 900_000,
    });
  },
};

export const typecheckErrorsApp: AppConfig = {
  name: "typecheck-errors",
  cwd: path.join(PROJECTS_DIR, "typecheck-errors"),
  command: "",
  args: [],
  port: 0,
  url: "",
  mountSelector: "",
  readyPattern: /./,
  startupTimeout: 0,
  check: {
    cwd: path.join(PROJECTS_DIR, "typecheck-errors"),
    patterns: ["src/**/*.vue"],
  },
};

export const compilerMacrosApp: AppConfig = {
  name: "compiler-macros",
  cwd: path.join(PROJECTS_DIR, "compiler-macros"),
  command: "",
  args: [],
  port: 0,
  url: "",
  mountSelector: "",
  readyPattern: /./,
  startupTimeout: 0,
  check: {
    cwd: path.join(PROJECTS_DIR, "compiler-macros"),
    patterns: ["src/**/*.vue"],
  },
};

export const stylePreprocessorsApp: AppConfig = {
  name: "style-preprocessors",
  cwd: path.join(PROJECTS_DIR, "style-preprocessors"),
  command: "",
  args: [],
  port: 0,
  url: "",
  mountSelector: "",
  readyPattern: /./,
  startupTimeout: 0,
  check: {
    cwd: path.join(PROJECTS_DIR, "style-preprocessors"),
    patterns: ["src/**/*.vue"],
  },
};

export const ecosystemProductsApp: AppConfig = {
  name: "ecosystem-products",
  cwd: path.join(PROJECTS_DIR, "ecosystem-products"),
  command: "",
  args: [],
  port: 0,
  url: "",
  mountSelector: "",
  readyPattern: /./,
  startupTimeout: 0,
  check: {
    cwd: path.join(PROJECTS_DIR, "ecosystem-products"),
    patterns: ["src/**/*.vue"],
  },
};

export const SCREENSHOT_DIR = path.resolve(TESTS_DIR, "app", "screenshots");
const BIN_EXT = process.platform === "win32" ? ".exe" : "";
const VIZE_CI_BIN = path.resolve(TESTS_DIR, `../target/ci/vize${BIN_EXT}`);
const VIZE_RELEASE_BIN = path.resolve(TESTS_DIR, `../target/release/vize${BIN_EXT}`);
const VIZE_DEBUG_BIN = path.resolve(TESTS_DIR, `../target/debug/vize${BIN_EXT}`);
const VIZE_BIN_OVERRIDE = process.env.VIZE_BIN;
const VIZE_BIN_FALLBACKS = [VIZE_CI_BIN, VIZE_RELEASE_BIN, VIZE_DEBUG_BIN];
export const VIZE_BIN =
  VIZE_BIN_OVERRIDE && VIZE_BIN_OVERRIDE.length > 0
    ? VIZE_BIN_OVERRIDE
    : (VIZE_BIN_FALLBACKS.find((candidate) => fs.existsSync(candidate)) ?? VIZE_RELEASE_BIN);
const CORSA_PRIMARY_BIN = path.resolve(TESTS_DIR, "../node_modules/.bin/corsa");
const CORSA_LEGACY_BIN = path.resolve(TESTS_DIR, "../node_modules/.bin/tsgo");
const CORSA_BIN_OVERRIDE = process.env.CORSA_BIN;
export const CORSA_BIN =
  CORSA_BIN_OVERRIDE && CORSA_BIN_OVERRIDE.length > 0
    ? CORSA_BIN_OVERRIDE
    : fs.existsSync(CORSA_PRIMARY_BIN)
      ? CORSA_PRIMARY_BIN
      : CORSA_LEGACY_BIN;

export function requireVizeBin(): void {
  requireFile(VIZE_BIN, "vize CLI", "Build it with `cargo build --profile ci -p vize`.");
}

export function requireVizeAndCorsaBins(): void {
  requireVizeBin();
  requireFile(
    CORSA_BIN,
    "Corsa/tsgo",
    "Install JS dependencies with `vp install` so node_modules/.bin/corsa is available.",
  );
}

function requireFile(filePath: string, label: string, hint: string): void {
  if (fs.existsSync(filePath)) {
    return;
  }
  throw new Error(`${label} binary not found at ${filePath}. ${hint}`);
}
