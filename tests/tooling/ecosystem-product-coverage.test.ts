import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const fixtureDir = path.join(root, "tests", "_fixtures", "_projects", "ecosystem-products");
const sourceDir = path.join(fixtureDir, "src");

const requestedProducts = [
  { name: "Reka UI", tokens: ["reka-ui", "DialogRoot"] },
  { name: "PrimeVue", tokens: ["primevue/button", "PrimeButton"] },
  { name: "Ant Design Vue", tokens: ["ant-design-vue", "ASelect"] },
  { name: "Nuxt UI", tokens: ["@nuxt/ui/components/Button.vue", "UButton"] },
  { name: "Quasar", tokens: ["quasar", "QSelect"] },
  { name: "VueUse", tokens: ["@vueuse/core", "useDark"] },
  { name: "Vue I18n", tokens: ["vue-i18n", "useI18n"] },
  { name: "Swiper.js", tokens: ["swiper/vue", "SwiperSlide"] },
  { name: "Vue Router", tokens: ["vue-router", "RouterLink"] },
  { name: "Vee Validate", tokens: ["vee-validate", "useForm"] },
  { name: "Element Plus", tokens: ["element-plus", "ElForm"] },
  { name: "Tiptap Vue", tokens: ["@tiptap/vue-3", "EditorContent"] },
  { name: "Vue Chart.js", tokens: ["vue-chartjs", "Line"] },
  { name: "Vue Virtual Scroller", tokens: ["vue-virtual-scroller", "RecycleScroller"] },
  { name: "Vue Flow", tokens: ["@vue-flow/core", "VueFlow"] },
  { name: "Vue Apollo", tokens: ["@vue/apollo-composable", "useApolloQuery"] },
  { name: "Vue Select", tokens: ["vue-select", "VSelect"] },
  { name: "TanStack Query Vue", tokens: ["@tanstack/vue-query", "useTanStackQuery"] },
  { name: "Ionic Vue", tokens: ["@ionic/vue", "IonButton"] },
  { name: "Vant", tokens: ["vant", "VanButton"] },
  { name: "Naive UI", tokens: ["naive-ui", "NCard"] },
  { name: "FormKit", tokens: ["@formkit/vue", "FormKit"] },
  { name: "TresJS", tokens: ["@tresjs/core", "TresCanvas"] },
] as const;

function readJsonFile<T>(...segments: string[]): T {
  return JSON.parse(fs.readFileSync(path.join(root, ...segments), "utf8")) as T;
}

function readSourceTree(dir: string): string {
  return fs
    .readdirSync(dir, { withFileTypes: true })
    .flatMap((entry) => {
      const entryPath = path.join(dir, entry.name);
      if (entry.isDirectory()) {
        return readSourceTree(entryPath);
      }
      if (/\.(vue|ts|d\.ts)$/.test(entry.name)) {
        return fs.readFileSync(entryPath, "utf8");
      }
      return "";
    })
    .join("\n");
}

test("ecosystem product fixture covers the requested Vue ecosystem packages", () => {
  const source = readSourceTree(sourceDir);

  for (const product of requestedProducts) {
    for (const token of product.tokens) {
      assert.match(
        source,
        new RegExp(token.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")),
        `${product.name} should include ${token}`,
      );
    }
  }
});

test("ecosystem product fixture is wired into the fixture check script", () => {
  const pkg = readJsonFile<{ scripts: Record<string, string> }>("tests", "package.json");
  const appsSource = fs.readFileSync(path.join(root, "tests", "_helpers", "apps.ts"), "utf8");
  const runnerSource = fs.readFileSync(
    path.join(root, "tests", "snapshots", "check", "ecosystem-products.ts"),
    "utf8",
  );

  assert.match(pkg.scripts["test:check:fixtures"], /snapshots\/check\/ecosystem-products\.ts/);
  assert.match(appsSource, /export const ecosystemProductsApp/);
  assert.match(runnerSource, /ecosystemProductsApp/);
});
