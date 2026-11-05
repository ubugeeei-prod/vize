import { ruleRenderRoutes } from "./rule-render-assertions.ts";
import { ruleGroups } from "./rules/rule-groups.ts";

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
  "/ja/architecture/performance",
  "/zh-CN/getting-started",
  "/pt-BR/getting-started",
  "/fr/getting-started",
];

/** The CLI and coverage checks consume the same complete native route manifest. */
export const defaultNavigationRenderRoutes = [
  ...navigationRenderRoutes,
  ...["zh-CN", "pt-BR", "fr"].flatMap((locale) =>
    ["all", ...ruleGroups, "cross-file"].map((file) => `/${locale}/rules/${file}`),
  ),
];
