import { rmSync } from "node:fs";
import path from "node:path";

const authFile = process.env.STATE_auth;
if (authFile) {
  const runnerTemp = process.env.RUNNER_TEMP;
  if (!runnerTemp) throw new Error("npm bootstrap cleanup requires RUNNER_TEMP");
  const expected = path.resolve(runnerTemp, "npm-bootstrap.npmrc");
  if (path.resolve(authFile) !== expected) {
    throw new Error("refusing to remove an npm bootstrap file outside RUNNER_TEMP");
  }
  rmSync(expected, { force: true });
}
