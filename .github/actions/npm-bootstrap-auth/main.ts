import { appendFileSync, writeFileSync } from "node:fs";
import path from "node:path";

const token = process.env.INPUT_TOKEN ?? "";
if (token !== "") {
  const runnerTemp = process.env.RUNNER_TEMP;
  const githubEnv = process.env.GITHUB_ENV;
  const githubState = process.env.GITHUB_STATE;
  if (!runnerTemp || !githubEnv || !githubState) {
    throw new Error("npm bootstrap authentication requires runner-owned environment files");
  }
  const authFile = path.join(runnerTemp, "npm-bootstrap.npmrc");
  writeFileSync(authFile, `//registry.npmjs.org/:_authToken=${token}\n`, {
    encoding: "utf8",
    flag: "wx",
    mode: 0o600,
  });
  appendFileSync(githubEnv, `NPM_CONFIG_USERCONFIG=${authFile}\n`);
  appendFileSync(githubState, `auth=${authFile}\n`);
}
