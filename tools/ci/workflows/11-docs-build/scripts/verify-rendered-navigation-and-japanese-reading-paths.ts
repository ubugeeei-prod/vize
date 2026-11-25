/// <reference lib="dom" />
import { pathToFileURL } from "node:url";
import { resolve as resolveWorkflowInput } from "node:path";
const { defaultNavigationRenderRoutes } = (await import(
  pathToFileURL(resolveWorkflowInput("./docs/scripts/navigation-render-routes.ts")).href
)) as typeof import("../../../../../docs/scripts/navigation-render-routes.ts");
process.stdout.write(
  defaultNavigationRenderRoutes.filter((route) => route.startsWith("/ja/")).join(","),
);
