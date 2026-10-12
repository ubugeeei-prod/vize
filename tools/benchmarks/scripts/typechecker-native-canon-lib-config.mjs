import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { existsSync, lstatSync, readFileSync, realpathSync } from "node:fs";
import path from "node:path";
import TOML from "@iarna/toml";

const executableEnvironment =
  /^(?:RUSTC(?:_WRAPPER|_WORKSPACE_WRAPPER)?|RUSTDOC|RUSTFLAGS|RUSTDOCFLAGS|CARGO_ENCODED_RUSTFLAGS|CARGO_BUILD_RUSTC(?:_WRAPPER|_WORKSPACE_WRAPPER)?|CARGO_TARGET_.*_RUNNER|CARGO_SOURCE_.*|CARGO_RESOLVER_LOCKFILE_PATH)$/u;

// Cargo merges configs from the invocation directory through its ancestors and
// CARGO_HOME. Read metadata only; no executable or registration source is run.
export function cargoConfigurations(root, env = process.env) {
  const directories = new Set();
  let directory = realpathSync(root);
  while (true) {
    directories.add(path.join(directory, ".cargo"));
    const parent = path.dirname(directory);
    if (parent === directory) break;
    directory = parent;
  }
  directories.add(
    env.CARGO_HOME ? path.resolve(root, env.CARGO_HOME) : path.join(env.HOME, ".cargo"),
  );
  const configurations = [];
  const seen = new Set();
  for (const directory of directories) {
    if (!existsSync(directory)) continue;
    assert.ok(!lstatSync(directory).isSymbolicLink(), "symlink Cargo configuration directory");
    for (const name of ["config", "config.toml"]) {
      const file = path.join(directory, name);
      if (!existsSync(file)) continue;
      const stat = lstatSync(file);
      assert.ok(stat.isFile() && !stat.isSymbolicLink(), "nonphysical Cargo configuration");
      const physical = realpathSync(file);
      if (seen.has(physical)) continue;
      seen.add(physical);
      const text = readFileSync(file, "utf8");
      configurations.push({
        file: physical,
        text,
        sha256: createHash("sha256").update(text).digest("hex"),
      });
    }
  }
  return {
    configurations,
    environment: Object.keys(env).filter((key) => env[key] && executableEnvironment.test(key)),
    runnerTemp: env.RUNNER_TEMP,
  };
}

export function qualifyCargoConfigurations(source) {
  const context = source.cargoContext ?? {
    configurations: [],
    environment: [],
    runnerTemp: undefined,
  };
  assert.deepEqual(context.environment, [], "unreviewed Cargo executable environment override");
  const configurations = [...context.configurations];
  for (const file of [".cargo/config", ".cargo/config.toml"]) {
    if (source.paths.has(file)) configurations.push({ file, text: source.read(file) });
  }
  for (const { text } of configurations) {
    let config;
    try {
      config = TOML.parse(text);
    } catch {
      throw new Error("invalid Cargo configuration");
    }
    for (const key of ["source", "paths", "patch", "include", "unstable"]) {
      assert.ok(config[key] === undefined, `unreviewed Cargo ${key} override`);
    }
    assert.ok(config.resolver?.["lockfile-path"] === undefined, "redirected Cargo lockfile");
    for (const key of ["rustc", "rustc-wrapper", "rustc-workspace-wrapper", "rustdoc"]) {
      assert.ok(config.build?.[key] === undefined, "unreviewed Cargo compiler override");
    }
    assert.ok(config.build?.rustflags === undefined, "unreviewed Cargo compiler flags");
    for (const key of Object.keys(config.env ?? {})) {
      assert.ok(
        !executableEnvironment.test(key) && key !== "CARGO_HOME",
        "unreviewed Cargo executable config environment",
      );
    }
    for (const target of Object.values(config.target ?? {})) {
      assert.ok(target.runner === undefined, "unreviewed Cargo target runner");
      assert.ok(
        target.linker === undefined || ["clang", "rust-lld"].includes(target.linker),
        "unreviewed Cargo linker",
      );
      if (target.rustflags !== undefined) {
        assert.ok(context.runnerTemp, "unreviewed Cargo target compiler flags");
        // Exact flag emitted by the unchanged Wild0.9.0 action at0bbbfa5.
        assert.ok(
          Array.isArray(target.rustflags) &&
            target.rustflags.length === 1 &&
            target.rustflags[0] === `-Clink-arg=--ld-path=${context.runnerTemp}/wild-install/wild`,
          "unreviewed Cargo target compiler flags",
        );
      }
    }
  }
}
