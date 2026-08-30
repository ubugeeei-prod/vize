import { test } from "node:test";

import {
  assertReleasePlatformCommandsUseRefEnv,
  readRepoFile,
} from "./support/github-workflows.ts";

test("release workflow passes release refs to shell commands through env", () => {
  assertReleasePlatformCommandsUseRefEnv(readRepoFile(".github", "workflows", "release.yml"), [
    "plan-release-platforms",
    "smoke-release-packages",
    "release-npm-native",
    "release-npm-oxlint-plugin",
  ]);
});
