import { execFileSync } from "node:child_process";
import type { AppConfig } from "./apps.ts";
import { execNpxCommand } from "./app-fixture-runtime.ts";

/** Use the same package runner and environment as fixture setup and dev. */
export function runAppBuild(app: Pick<AppConfig, "cwd" | "env" | "build">): void {
  const build = app.build;
  if (!build) throw new Error(`No build command configured for ${app.cwd}`);
  const options = {
    cwd: app.cwd,
    env: { ...process.env, NODE_ENV: "production", ...app.env },
    timeout: build.timeout,
  };
  console.log(`Running: ${build.command} ${build.args.join(" ")} (cwd: ${app.cwd})`);
  if (build.command === "npx") {
    execNpxCommand(build.args, options);
  } else {
    execFileSync(build.command, build.args, { ...options, stdio: "inherit" });
  }
}
