import fs from "node:fs";
import path from "node:path";

/**
 * Places a failing `vize` command before the inherited PATH.
 *
 * Package-manager script runners should still prefer the fresh project's own
 * `node_modules/.bin`; reaching this executable means the release smoke stopped
 * proving the generated command against project-local artifacts.
 */
export function withPoisonedVizePath(
  projectRoot: string,
  env: NodeJS.ProcessEnv = process.env,
): NodeJS.ProcessEnv & { PATH: string } {
  const poisonDir = path.join(projectRoot, ".vize-path-poison");
  fs.mkdirSync(poisonDir, { recursive: true });
  const executable = path.join(poisonDir, "vize");
  fs.copyFileSync(new URL("./path-poison/vize.sh", import.meta.url), executable);
  if (process.platform !== "win32") fs.chmodSync(executable, 0o755);
  fs.copyFileSync(
    new URL("./path-poison/vize.cmd", import.meta.url),
    path.join(poisonDir, "vize.cmd"),
  );
  return {
    ...env,
    PATH: [poisonDir, env.PATH ?? ""].filter((segment) => segment.length > 0).join(path.delimiter),
  };
}
