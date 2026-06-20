import type { ConfigEnv } from "vite";

export function shouldApplyMuseaPlugin(env: Pick<ConfigEnv, "command" | "mode">): boolean {
  return (env.command === "serve" || env.command === "build") && env.mode !== "test";
}
