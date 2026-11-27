import { spawnSync } from "node:child_process";
import { copyFileSync, mkdirSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

const directory = fileURLToPath(new URL("../", import.meta.url));
const root = resolve(directory, "../../..");
const target = "wasm32-wasip1";
function run(command: string, args: string[], cwd: string, env = process.env): void {
  const result = spawnSync(command, args, { cwd, env, stdio: "inherit" });
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error(`${command} exited with ${result.status}`);
}

run("vp", ["pack"], directory);
run(
  "cargo",
  ["build", "--release", "--locked", "-p", "vize_swc_plugin", "--target", target],
  root,
  {
    ...process.env,
    CARGO_TARGET_WASM32_WASIP1_RUSTFLAGS: `${process.env.CARGO_TARGET_WASM32_WASIP1_RUSTFLAGS ?? ""} --cfg=swc_ast_unknown`,
  },
);
mkdirSync(resolve(directory, "dist"), { recursive: true });
copyFileSync(
  resolve(root, process.env.CARGO_TARGET_DIR ?? "target", target, "release/vize_swc_plugin.wasm"),
  resolve(directory, "dist/vize_swc_plugin.wasm"),
);
