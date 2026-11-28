import { spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

const packageRoot = fileURLToPath(new URL("../", import.meta.url));
const repositoryRoot = path.resolve(packageRoot, "../..");
const env = {
  ...process.env,
  VIZE_NATIVE_BUILD_RECEIPT: path.join(packageRoot, ".build/compiler-build.json"),
};
const commands: readonly (readonly [string, readonly string[]])[] = [
  [process.execPath, ["tools/ci/workflows/06-native-application-host/scripts/qualify-sdk.ts"]],
  [process.execPath, ["npm/native-host-fixture/scripts/build-compiler.ts"]],
  ["vp", ["run", "--filter", "./npm/marquette", "build"]],
  ["vp", ["run", "--filter", "./npm/native-host-fixture", "check:types"]],
  [process.execPath, ["--test", "npm/native-host-fixture/test/application.test.ts"]],
  [process.execPath, ["npm/native-host-fixture/scripts/qualify-host.ts"]],
];
for (const [program, args] of commands) {
  const result = spawnSync(program, args, { cwd: repositoryRoot, env, stdio: "inherit" });
  if (result.error) throw result.error;
  if (result.status !== 0) {
    process.exitCode = result.status ?? 1;
    break;
  }
}
