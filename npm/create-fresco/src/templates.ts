import type { PackageManager } from "./types.js";
import { CREATE_FRESCO_VERSION } from "./version.js";

/** Inputs that make generated project files deterministic. */
export interface ProjectTemplateOptions {
  /** Normalized npm package name written to the generated manifest. */
  packageName: string;
  /** Package manager whose commands are written to generated instructions. */
  packageManager: PackageManager;
}

const packageManagerCommands: Record<
  PackageManager,
  { install: string; run: (script: string) => string }
> = {
  npm: { install: "npm install", run: (script) => `npm run ${script}` },
  pnpm: { install: "pnpm install", run: (script) => `pnpm ${script}` },
  yarn: { install: "yarn", run: (script) => `yarn ${script}` },
  bun: { install: "bun install", run: (script) => `bun run ${script}` },
};

function projectPackageJson(packageName: string): string {
  return `${JSON.stringify(
    {
      name: packageName,
      version: "0.0.0",
      private: true,
      type: "module",
      scripts: {
        dev: "vite-node --watch src/main.ts",
        build: "vite build",
        start: "node dist/main.mjs",
        check: "vue-tsc --noEmit -p tsconfig.json",
      },
      dependencies: {
        "@vizejs/fresco": `^${CREATE_FRESCO_VERSION}`,
        vue: "^3.5.42",
      },
      devDependencies: {
        "@vitejs/plugin-vue": "^6.0.7",
        "@types/node": "^25.0.0",
        typescript: "^6.0.3",
        vite: "^8.2.4",
        "vite-node": "^6.0.0",
        "vue-tsc": "^3.3.11",
      },
      engines: { node: ">=22" },
    },
    null,
    2,
  )}\n`;
}

function projectReadme(options: ProjectTemplateOptions): string {
  const commands = packageManagerCommands[options.packageManager];
  return `# ${options.packageName}

A Vue-native terminal application built with [Fresco](https://github.com/ubugeeei-prod/vize).

## Develop

\`\`\`sh
${commands.install}
${commands.run("dev")}
\`\`\`

\`vite-node --watch\` reloads the SFC application when source files change. Press the arrow keys
to update the counter and \`q\` to exit.

## Check and build

\`\`\`sh
${commands.run("check")}
${commands.run("build")}
${commands.run("start")}
\`\`\`

The build produces one Node entry at \`dist/main.mjs\`. To use it as an executable launcher on
macOS or Linux, prepend a Node shebang or invoke it from a small package \`bin\` entry. Fresco's
platform-native addon remains an installed runtime dependency; this is intentionally not described
as a static, dependency-free binary.
`;
}

const appVue = `<script setup lang="ts">
import { ref } from "vue";
import { Box, Text, useApp, useInput } from "@vizejs/fresco";

const count = ref(0);
const { exit } = useApp();

useInput({
  onArrow(direction) {
    if (direction === "up" || direction === "right") count.value += 1;
    if (direction === "down" || direction === "left") count.value -= 1;
  },
  onChar(character) {
    if (character === "q") exit();
  },
});
</script>

<template>
  <Box flex-direction="column" :padding="1" border="rounded" border-color="cyan">
    <Text :bold="true" fg="cyan">Fresco</Text>
    <Text>Count: {{ count }}</Text>
    <Text :dim="true">Use arrow keys; press q to exit.</Text>
  </Box>
</template>
`;

const mainTs = `import { createApp } from "@vizejs/fresco";
import App from "./App.vue";

const app = createApp(App, {
  alternateScreen: true,
  exitOnCtrlC: true,
});

if (import.meta.hot) {
  import.meta.hot.accept();
  import.meta.hot.dispose(async () => {
    await app.unmount();
  });
}

try {
  await app.mount();
  await app.waitUntilExit();
} catch (error) {
  console.error(error);
  process.exitCode = 1;
}
`;

const viteConfig = `import vue from "@vitejs/plugin-vue";
import { defineConfig } from "vite";

export default defineConfig({
  plugins: [vue()],
  build: {
    ssr: "src/main.ts",
    target: "node22",
    outDir: "dist",
    rollupOptions: {
      external: ["@vizejs/fresco-native"],
      output: {
        entryFileNames: "main.mjs",
      },
    },
  },
});
`;

const tsconfig = `{
  "compilerOptions": {
    "target": "ES2022",
    "module": "NodeNext",
    "moduleResolution": "NodeNext",
    "strict": true,
    "noEmit": true,
    "isolatedModules": true,
    "skipLibCheck": true,
    "types": ["node"]
  },
  "include": ["src/**/*.ts", "src/**/*.vue", "vite.config.ts"]
}
`;

const vueTypes = `/// <reference types="vite/client" />

declare module "*.vue" {
  import type { DefineComponent } from "vue";
  const component: DefineComponent<Record<string, never>, Record<string, never>, unknown>;
  export default component;
}
`;

/** Materialize every deterministic file in a new Fresco project. */
export function projectTemplates(options: ProjectTemplateOptions): ReadonlyMap<string, string> {
  return new Map([
    [".gitignore", "node_modules\ndist\n*.log\n"],
    ["README.md", projectReadme(options)],
    ["package.json", projectPackageJson(options.packageName)],
    ["tsconfig.json", tsconfig],
    ["vite.config.ts", viteConfig],
    ["src/App.vue", appVue],
    ["src/env.d.ts", vueTypes],
    ["src/main.ts", mainTs],
  ]);
}

/** Return the install and development commands for a supported package manager. */
export function commandsFor(packageManager: PackageManager): { install: string; dev: string } {
  const commands = packageManagerCommands[packageManager];
  return { install: commands.install, dev: commands.run("dev") };
}
