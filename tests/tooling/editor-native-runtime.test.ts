import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";

test("real editor fixtures select the installed platform runtime without searching unrelated compilers", () => {
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "editor-native-runtime-"));
  try {
    const support = fileURLToPath(new URL("../../tools/support", import.meta.url));
    const launcher = path.join(temporary, "resolve-runtime.rs");
    fs.writeFileSync(
      launcher,
      `//! \x60\x60\x60cargo
//! [package]
//! edition = "2024"
//! [dependencies]
//! vize_tool_support = { path = ${JSON.stringify(support)} }
//! \x60\x60\x60
fn main() {
    let root = std::path::PathBuf::from(std::env::args_os().nth(1).unwrap());
    match vize_tool_support::editors::e2e::resolve_corsa_path(&root) {
        Ok(runtime) => println!("{}", runtime.display()),
        Err(message) => { eprintln!("{message}"); std::process::exit(1); }
    }
}
`,
    );
    const packageName = `@typescript/typescript-${process.platform}-${process.arch}`;
    const executable = process.platform === "win32" ? "tsc.exe" : "tsc";
    const runtime = (root: string) =>
      path.join(root, "node_modules", packageName, "lib", executable);
    const write = (filename: string) => {
      fs.mkdirSync(path.dirname(filename), { recursive: true });
      fs.writeFileSync(filename, "inert runtime lookup fixture; never executed\n");
    };
    const resolve = (root: string) => {
      const result = spawnSync("rust-script", ["--debug", launcher, root], {
        encoding: "utf8",
        timeout: 120_000,
      });
      assert.ifError(result.error);
      return { status: result.status, stdout: result.stdout.trim(), stderr: result.stderr };
    };

    // The exact CI failure: the root install owns TypeScript 7 while the
    // standalone VS Code node_modules has no native compiler dependency.
    const rootOnly = path.join(temporary, "root-only"),
      rootRuntime = runtime(rootOnly);
    write(rootRuntime);
    assert.deepEqual(resolve(rootOnly), {
      status: 0,
      stdout: fs.realpathSync(rootRuntime),
      stderr: "",
    });

    const standalone = path.join(temporary, "standalone");
    const extensionRuntime = runtime(path.join(standalone, "editors/vscode"));
    write(extensionRuntime);
    assert.deepEqual(resolve(standalone), {
      status: 0,
      stdout: fs.realpathSync(extensionRuntime),
      stderr: "",
    });
    const preferred = runtime(standalone);
    write(preferred);
    assert.deepEqual(resolve(standalone), {
      status: 0,
      stdout: fs.realpathSync(preferred),
      stderr: "",
    });

    for (const [name, unrelated] of [
      ["nested", `other/node_modules/${packageName}/lib/${executable}`],
      ["wrong-platform", `@typescript/typescript-other-other/lib/${executable}`],
      ["plain-typescript", `typescript/bin/${executable}`],
    ]) {
      const root = path.join(temporary, name);
      write(path.join(root, "editors/vscode/node_modules", unrelated));
      const refused = resolve(root);
      assert.equal(refused.status, 1, `${name}: unrelated compiler must never be selected`);
      assert.equal(refused.stdout, "");
      assert.ok(refused.stderr.includes("missing TypeScript 7 runtime"));
      assert.ok(refused.stderr.includes(runtime(root)));
      assert.ok(refused.stderr.includes(runtime(path.join(root, "editors/vscode"))));
    }
    if (process.platform !== "win32") {
      const linked = path.join(temporary, "linked"),
        physical = path.join(temporary, "physical-package");
      write(path.join(physical, "lib", executable));
      const packageLink = path.dirname(path.dirname(runtime(linked)));
      fs.mkdirSync(path.dirname(packageLink), { recursive: true });
      fs.symlinkSync(physical, packageLink, "dir");
      assert.deepEqual(resolve(linked), {
        status: 0,
        stdout: fs.realpathSync(path.join(physical, "lib", executable)),
        stderr: "",
      });
    }
  } finally {
    fs.rmSync(temporary, { recursive: true, force: true });
  }
});
