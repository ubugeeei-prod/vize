import { ruleRenderRoutes } from "./rule-render-assertions.ts";
import { categoryFiles } from "./rules/catalogue-routes.ts";
import { catalogueSubgroups } from "./rules/catalogue-subgroups.ts";

const pages = [
  "/",
  "/getting-started",
  "/guide/configuration",
  "/guide/migration",
  "/guide/vite-plus",
  "/guide/vite-plugin",
  "/guide/configuration-reference",
  "/guide/compiler-configuration-reference",
  "/guide/musea",
  "/architecture/performance-blacksmith",
  ...ruleRenderRoutes,
];

export const navigationRenderRoutes = [
  ...pages.flatMap((route) => [route, `/ja${route}`]),
  ...["", "/ja", "/zh-CN", "/pt-BR", "/fr"].flatMap((locale) => [
    `${locale}/philosophy`,
    `${locale}/guide/content-mapper`,
    `${locale}/guide/testing`,
    `${locale}/guide/jsx-babel-compat`,
  ]),
  "/zh-CN/getting-started",
  "/pt-BR/getting-started",
  "/fr/getting-started",
];

/** The CLI and coverage checks consume the same complete native route manifest. */
export const defaultNavigationRenderRoutes = [
  ...navigationRenderRoutes,
  ...["zh-CN", "pt-BR", "fr"].flatMap((locale) =>
    ["all", "vue", ...categoryFiles, "cross-file"].map((file) => `/${locale}/rules/${file}`),
  ),
  ...Object.keys(catalogueSubgroups).map((file) => `/rules/${file}`),
];
