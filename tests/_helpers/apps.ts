import type { Page } from "@playwright/test";
import { execSync } from "node:child_process";
import * as fs from "node:fs";
import * as path from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const TESTS_DIR = path.resolve(__dirname, "..");
const GIT_DIR = path.join(TESTS_DIR, "_fixtures", "_git");
const PROJECTS_DIR = path.join(TESTS_DIR, "_fixtures", "_projects");
const NPM_DIR = path.resolve(__dirname, "../../npm");

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
const MISSKEY_FLUENT_EMOJI_RE = /\/fluent-emoji(?:s)?\/([0-9a-z-]+\.png)\b/g;
const TRANSPARENT_PNG = Buffer.from(
  "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVQIW2P8z/C/HwAFgwJ/lE6nWQAAAABJRU5ErkJggg==",
  "base64",
);

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
  opts?: { removeModules?: string[] },
): void {
  let config = fs.readFileSync(configPath, "utf-8");
  let changed = false;

  if (!config.includes("@vizejs/nuxt")) {
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
      const re = new RegExp(
        `\\s*'${mod.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}',?\\n?`,
      );
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
  const candidates = fs
    .readdirSync(pnpmDir)
    .filter((d) => d.startsWith(`${packageName}@`));
  for (const candidate of candidates) {
    const target = path.join(pnpmDir, candidate, "node_modules", packageName);
    if (fs.existsSync(target)) {
      fs.symlinkSync(target, link, "dir");
      return;
    }
  }
}

function addPnpmOverrides(
  packageJsonPath: string,
  overrides: Record<string, string>,
): void {
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

// --- App configurations ---

export const elkApp: AppConfig = {
  name: "elk",
  cwd: path.join(GIT_DIR, "elk"),
  command: "npx",
  args: [
    "pnpm@10",
    "exec",
    "nuxt",
    "dev",
    "--port",
    "5314",
    "--host",
    "0.0.0.0",
  ],
  port: 5314,
  url: "http://127.0.0.1:5314",
  mountSelector: "#__nuxt",
  readyPattern: /Local:\s+http:\/\/(localhost|127\.0\.0\.1|0\.0\.0\.0):5314/,
  allowNon200: true,
  waitUntil: "load",
  readyDelay: 15_000,
  startupTimeout: 120_000,
  setup() {
    const elkDir = path.join(GIT_DIR, "elk");

    addPnpmOverrides(path.join(elkDir, "package.json"), {
      vite: "^8.0.0-beta.0",
    });

    console.log("[elk:setup] pnpm install...");
    execSync("npx pnpm@10 install --no-frozen-lockfile", {
      cwd: elkDir,
      stdio: "inherit",
      timeout: 300_000,
    });

    createVizeSymlinks(path.join(elkDir, "node_modules"));
    patchNuxtConfig(path.join(elkDir, "nuxt.config.ts"));
  },
  build: {
    command: "npx",
    args: ["pnpm@10", "build"],
    timeout: 300_000,
  },
  preview: {
    command: "npx",
    args: ["pnpm@10", "start"],
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

export const misskeyApp: AppConfig = {
  name: "misskey",
  cwd: path.join(GIT_DIR, "misskey", "packages", "frontend"),
  command: "npx",
  args: ["pnpm@10", "exec", "vite"],
  port: 5173,
  url: "http://localhost:5173/vite/",
  mountSelector: "#misskey_app",
  readyPattern: /Local:\s+http:\/\//,
  allowNon200: true,
  waitUntil: "domcontentloaded",
  startupTimeout: 180_000,
  setup() {
    const misskeyDir = path.join(GIT_DIR, "misskey");
    const frontendDir = path.join(misskeyDir, "packages", "frontend");

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
      vite: "^8.0.0-beta.0",
    });

    console.log("[misskey:setup] pnpm install...");
    execSync("npx pnpm@10 install --no-frozen-lockfile", {
      cwd: misskeyDir,
      stdio: "inherit",
      timeout: 300_000,
    });

    ensureMisskeyFluentEmojiAssets(misskeyDir);

    // Build workspace packages needed by frontend
    for (const pkg of [
      "i18n",
      "icons-subsetter",
      "misskey-js",
      "misskey-bubble-game",
      "misskey-reversi",
      "frontend-shared",
    ]) {
      console.log(`[misskey:setup] building ${pkg} package...`);
      execSync(`npx pnpm@10 --filter ${pkg} build`, {
        cwd: misskeyDir,
        stdio: "inherit",
        timeout: 120_000,
      });
    }

    createVizeSymlinks(path.join(misskeyDir, "node_modules"));

    // Patch vite.config.ts
    const viteConfigPath = path.join(frontendDir, "vite.config.ts");
    let viteConfig = fs.readFileSync(viteConfigPath, "utf-8");
    if (!viteConfig.includes("@vizejs/vite-plugin")) {
      viteConfig = viteConfig.replace(
        "import pluginVue from '@vitejs/plugin-vue';",
        "import { vize as pluginVue } from '@vizejs/vite-plugin';",
      );
      fs.writeFileSync(viteConfigPath, viteConfig);
    }

    removeManualChunksObject(viteConfigPath);
    removeManualChunksObject(path.join(misskeyDir, "packages", "frontend-embed", "vite.config.ts"));
  },
  async setupPage(page) {
    await page.addInitScript(() => {
      const _origFetch = window.fetch;
      window.fetch = function (input, init) {
        const url =
          typeof input === "string"
            ? input
            : input instanceof URL
              ? input.toString()
              : input.url;
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
                serverIsDead:
                  "Server is not responding. Please wait a moment and try again.",
                youShouldUpgradeClient:
                  "Please refresh the page to use the updated client.",
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
    args: ["pnpm@10", "exec", "vite", "build"],
    timeout: 180_000,
  },
  preview: {
    command: "npx",
    args: ["pnpm@10", "exec", "vite", "preview", "--port", "5174"],
    port: 5174,
    url: "http://localhost:5174/vite/",
    readyPattern: /Local:\s+http:\/\//,
  },
  check: {
    cwd: path.join(GIT_DIR, "misskey", "packages", "frontend"),
    patterns: ["src/**/*.vue"],
  },
  lint: {
    cwd: path.join(GIT_DIR, "misskey", "packages", "frontend"),
    patterns: ["src/**/*.vue"],
  },
};

export const npmxApp: AppConfig = {
  name: "npmx.dev",
  cwd: path.join(GIT_DIR, "npmx.dev"),
  command: "npx",
  args: [
    "pnpm@10",
    "exec",
    "nuxt",
    "dev",
    "--port",
    "3001",
    "--host",
    "0.0.0.0",
  ],
  port: 3001,
  url: "http://127.0.0.1:3001",
  mountSelector: "#__nuxt",
  readyPattern: /Local:\s+http:\/\/(localhost|127\.0\.0\.1|0\.0\.0\.0):3001/,
  allowNon200: true,
  waitUntil: "load",
  readyDelay: 30_000,
  env: {
    NUXT_SESSION_PASSWORD: "e2e-test-dummy-session-password-32chars!",
  },
  startupTimeout: 120_000,
  setup() {
    const npmxDir = path.join(GIT_DIR, "npmx.dev");
    const nmDir = path.join(npmxDir, "node_modules");

    console.log("[npmx.dev:setup] pnpm install...");
    execSync("npx pnpm@10 install --no-frozen-lockfile", {
      cwd: npmxDir,
      stdio: "inherit",
      timeout: 300_000,
      env: {
        ...process.env,
        NUXT_SESSION_PASSWORD: "e2e-test-dummy-session-password-32chars!",
      },
    });

    createVizeSymlinks(nmDir);
    patchNuxtConfig(path.join(npmxDir, "nuxt.config.ts"), {
      removeModules: ["@nuxtjs/html-validator"],
    });
    const npmxAppPath = path.join(npmxDir, "app", "app.vue");
    const npmxAppSource = fs.readFileSync(npmxAppPath, "utf-8");
    const nextNpmxAppSource = npmxAppSource.replace(/\n\s*<NuxtPwaAssets\s*\/>\s*/g, "\n");
    if (nextNpmxAppSource !== npmxAppSource) {
      fs.writeFileSync(npmxAppPath, nextNpmxAppSource);
    }
    hoistPnpmPackage(nmDir, "vue-i18n");

    // Ensure .nuxt/tsconfig.server.json exists (vite 8 needs it at startup)
    console.log("[npmx.dev:setup] nuxt prepare...");
    execSync("npx pnpm@10 exec nuxt prepare", {
      cwd: npmxDir,
      stdio: "inherit",
      timeout: 60_000,
      env: {
        ...process.env,
        NUXT_SESSION_PASSWORD: "e2e-test-dummy-session-password-32chars!",
      },
    });
  },
  build: {
    command: "npx",
    args: ["pnpm@10", "build"],
    timeout: 300_000,
  },
  preview: {
    command: "npx",
    args: ["pnpm@10", "exec", "nuxt", "preview", "--port", "3002"],
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

export const vuefesApp: AppConfig = {
  name: "vuefes-2025",
  cwd: path.join(GIT_DIR, "vuefes-2025"),
  command: "npx",
  args: [
    "pnpm@10",
    "exec",
    "nuxt",
    "dev",
    "--port",
    "3003",
    "--host",
    "0.0.0.0",
  ],
  port: 3003,
  url: "http://127.0.0.1:3003",
  mountSelector: "#__nuxt",
  readyPattern: /Local:\s+http:\/\/(localhost|127\.0\.0\.1|0\.0\.0\.0):3003/,
  allowNon200: true,
  waitUntil: "load",
  readyDelay: 30_000,
  startupTimeout: 180_000,
  setup() {
    const vuefesDir = path.join(GIT_DIR, "vuefes-2025");

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
      fs.writeFileSync(
        vuefesPackageJson,
        JSON.stringify(pkg, null, "\t") + "\n",
      );
    }

    addPnpmOverrides(vuefesPackageJson, {
      vite: "^8.0.0-beta.0",
    });

    console.log("[vuefes-2025:setup] pnpm install...");
    execSync("npx pnpm@10 install --no-frozen-lockfile", {
      cwd: vuefesDir,
      stdio: "inherit",
      timeout: 300_000,
    });

    createVizeSymlinks(path.join(vuefesDir, "node_modules"));
    patchNuxtConfig(path.join(vuefesDir, "nuxt.config.ts"));
  },
  build: {
    command: "npx",
    args: ["pnpm@10", "build"],
    timeout: 300_000,
  },
  preview: {
    command: "npx",
    args: ["pnpm@10", "exec", "nuxt", "preview", "--port", "3004"],
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
  cwd: path.join(GIT_DIR, "nuxt-ui"),
  command: "npx",
  args: ["pnpm@10", "dev"],
  port: 5317,
  url: "http://localhost:5317",
  mountSelector: "#app",
  readyPattern: /Local:\s+http:\/\/localhost:5317/,
  allowNon200: true,
  waitUntil: "load",
  readyDelay: 10_000,
  startupTimeout: 120_000,
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

export const SCREENSHOT_DIR = path.resolve(TESTS_DIR, "app", "screenshots");
export const VIZE_BIN = path.resolve(TESTS_DIR, "../target/release/vize");
export const TSGO_BIN = path.resolve(TESTS_DIR, "../node_modules/.bin/tsgo");
