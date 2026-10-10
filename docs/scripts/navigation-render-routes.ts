import { ruleRenderRoutes } from "./rule-render-assertions.ts";

const pages = [
  "/",
  "/getting-started",
  "/guide/configuration",
  "/guide/migration",
  "/guide/vite-plus",
  "/guide/vite-plugin",
  "/guide/configuration-reference",
  "/guide/compiler-configuration-reference",
  ...ruleRenderRoutes,
];

export const navigationRenderRoutes = [
  ...pages.flatMap((route) => [route, `/ja${route}`]),
  ...["", "/ja", "/zh-CN", "/pt-BR", "/fr"].flatMap((locale) => [
    `${locale}/philosophy`,
    `${locale}/guide/content-mapper`,
  ]),
  "/zh-CN/getting-started",
  "/pt-BR/getting-started",
  "/fr/getting-started",
];
