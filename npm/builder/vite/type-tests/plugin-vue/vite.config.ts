import vue from "@vitejs/plugin-vue";
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";
import { defineVizeLintConfig, flatConfigs } from "oxlint-plugin-vize";

export default defineConfig({
  compiler: false,
  plugins: [vue(), Promise.resolve([{ name: "async-plugin" }, false]), null],
  server: { port: 4321 },
});

defineConfig(async () => ({
  compiler: false,
  plugins: [vue()],
}));

defineConfig({
  // @ts-expect-error unnamed plugins are invalid
  plugins: [123],
});

// Standard Vite+ exports compose without changing their existing plugin types.
const fluent = defineConfig(Promise.resolve({ plugins: [vue()] })).withVue({
  formatter: { singleQuote: true },
  typeChecker: { strict: true },
});
fluent.withVue({ compiler: { sourceMap: true } });
defineConfig(async () => ({ plugins: [vue()] })).withVue({ version: "3" });
defineConfig({}).withVue({
  compile: { sourceMap: true },
  typeCheck: { strict: true },
  lint: defineVizeLintConfig(...flatConfigs.recommended, ...flatConfigs.ecosystem, {
    ignorePatterns: ["dist/**"],
    rules: { "no-console": "warn" },
  }),
  fmt: { singleQuote: true },
  entries: [
    {
      files: ["apps/legacy/**"],
      linter: { rules: { "vue/no-v-html": "off" } },
    },
  ],
});
defineConfig({
  // @ts-expect-error configuration composition uses .withVue(), not custom extends
  extends: {},
});
// @ts-expect-error native Vue options retain their own types
fluent.withVue({ formatter: { singleQuote: "yes" } });
// @ts-expect-error lint owns the host Vite+ block; native-only settings use linter
fluent.withVue({ lint: { preset: "essential" } });

defineConfig({
  // @ts-expect-error shared native settings use vue:
  vize: {},
});
defineConfig({
  lint: {
    // @ts-expect-error per-tool native settings use lint.vue
    vize: {},
  },
});
defineConfig({
  fmt: {
    // @ts-expect-error per-tool native settings use fmt.vue
    vize: {},
  },
});
defineConfig({
  lint: { vue: { typecheck: true, locale: "ja" } },
  fmt: { vue: { singleQuote: true } },
  pack: { vue: { dts: true } },
}).withVue({ linter: { preset: "essential" } });
