import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

assert.equal(
  process.env.GITHUB_ACTIONS,
  "true",
  "This comparison runs only in an isolated Actions checkout",
);
const root = process.cwd();
const capture = process.env.VIZE_PACKAGE_REFRESH_CAPTURE;
assert(capture);
fs.mkdirSync(capture, { recursive: true });
const hash = (file) => createHash("sha256").update(fs.readFileSync(file)).digest("hex");
const git = (...args) => execFileSync("git", args, { cwd: root, encoding: "utf8" }).trim();
const observer = git("rev-parse", "HEAD");
const sdk = path.join(capture, "sdk");
fs.mkdirSync(sdk);
fs.writeFileSync(path.join(sdk, "package.json"), '{"private":true}');
const npm = path.resolve(path.dirname(process.execPath), "../lib/node_modules/npm/bin/npm-cli.js");
execFileSync(
  process.execPath,
  [
    npm,
    "install",
    "--ignore-scripts",
    "--no-audit",
    "--no-fund",
    "--save-exact",
    "@typescript/typescript-linux-x64@7.0.2",
  ],
  { cwd: sdk, stdio: "inherit" },
);
const executable = fs.realpathSync(
  path.join(sdk, "node_modules/@typescript/typescript-linux-x64/lib/tsc"),
);
const expectedImage = "4f2de678286401759b3fb4475bafe35b8f32b4b3a07d92642bbf37eadc9b34a4";
assert.equal(hash(executable), expectedImage);
const receipt = {
  observer,
  executable,
  sha256: expectedImage,
  version: execFileSync(executable, ["--version"], { encoding: "utf8" }).trim(),
  results: [],
};
fs.writeFileSync(path.join(capture, "attempt.json"), JSON.stringify(receipt, null, 2));
const cases = [
  [
    "bridge",
    "crates/vize_canon/src/corsa_bridge/vue_document_package_negative_tests.rs",
    "corsa_bridge::vue_document_package_negative_tests::named_only_and_missing_vue_exports_report_on_the_authored_host",
  ],
  [
    "server",
    "crates/vize_canon/src/corsa_server/request.rs",
    "corsa_server::request::tests::persistent_check_server_refreshes_package_target_create_and_delete",
  ],
];
const refs =
  process.env.VIZE_COMPARE_HISTORICAL === "true"
    ? [
        ["before-policy", "13fa41356d075047a229f580014b637d3bbeb57f"],
        ["runtime-policy", "9469f272b189d0220a9838539f2c878fc5d70edd"],
        ["fixed", observer],
      ]
    : [["fixed", observer]];
for (const [label, source] of refs) {
  git("checkout", "--detach", source);
  assert.equal(git("rev-parse", "HEAD"), source);
  const originals = new Map();
  try {
    for (const [name, file] of cases) {
      const original = fs.readFileSync(file, "utf8");
      originals.set(file, original);
      fs.writeFileSync(path.join(capture, `${label}-${name}-test.rs`), original);
      let observed = original;
      if (name === "bridge") {
        for (const variable of ["diagnostics", "created_diagnostics", "deleted_diagnostics"]) {
          const declaration = new RegExp(`(let ${variable} = [^;]+;)`);
          assert(declaration.test(observed));
          observed = observed.replace(
            declaration,
            `$1\n        eprintln!("OBSERVED ${variable}: {${variable}:#?}");`,
          );
        }
        observed = observed.replace(
          "let deleted_diagnostics =",
          'eprintln!("OBSERVED_DISK source={} generated={}", missing.join("Missing.vue").is_file(), created_shadow.join("Missing.d.vue.ts").is_file());\n        let deleted_diagnostics =',
        );
      } else {
        for (const variable of ["missing", "created", "deleted"]) {
          const declaration = new RegExp(`(let ${variable} = server\\.check_vue_sfc[^;]+;)`);
          assert(declaration.test(observed));
          observed = observed.replace(
            declaration,
            `$1\n        eprintln!("OBSERVED ${variable}: {}", serde_json::to_string(&${variable}).unwrap());`,
          );
        }
      }
      fs.writeFileSync(file, observed);
    }
    for (const [file, needle, trace] of [
      [
        "crates/vize_canon/src/lsp_client/workspace_project.rs",
        "        match self.activate_workspace_project_with_reload(",
        '        eprintln!("OBSERVED_DELTA {changes:#?}");\n',
      ],
      [
        "crates/vize_canon/src/lsp_client/initial_diagnostics.rs",
        "        let document_uri = self.session_document_uri(uri);",
        '        eprintln!("OBSERVED_OWNER attached={} materialized={} uri={uri} documents={:?}", self.project_session_attached_to_editor, self.materialized_project_session, self.document_texts.keys());\n',
      ],
    ]) {
      const original = fs.readFileSync(file, "utf8");
      assert(original.includes(needle));
      originals.set(file, original);
      fs.writeFileSync(file, original.replace(needle, trace + needle));
    }
    for (const [name, , test] of cases) {
      assert.equal(
        hash(executable),
        expectedImage,
        "Both revisions must use the same physical SDK",
      );
      const result = spawnSync(
        "cargo",
        [
          "test",
          "--locked",
          "--profile",
          "ci",
          "-p",
          "vize_canon",
          test,
          "--",
          "--exact",
          "--nocapture",
        ],
        {
          cwd: root,
          env: { ...process.env, CORSA_PATH: executable },
          encoding: "utf8",
          timeout: 900_000,
          maxBuffer: 32_000_000,
        },
      );
      fs.writeFileSync(path.join(capture, `${label}-${name}.stdout`), result.stdout || "");
      fs.writeFileSync(path.join(capture, `${label}-${name}.stderr`), result.stderr || "");
      const row = {
        label,
        source,
        test,
        exit: result.status,
        signal: result.signal,
        error: result.error?.message,
        sdkSha256: hash(executable),
        expectedFailure: label !== "fixed",
      };
      receipt.results.push(row);
      fs.writeFileSync(path.join(capture, "receipt.json"), JSON.stringify(receipt, null, 2));
      process.stdout.write(`${JSON.stringify(row)}\n`);
      assert.equal(result.error, undefined);
      assert.match(result.stdout, /running 1 test/);
      if (label === "fixed") {
        assert.match(result.stdout, /test result: ok/);
        assert.equal(
          result.status,
          0,
          "The real deletion contract must pass after invalidation is repaired",
        );
      } else {
        assert.match(result.stdout, /test result: FAILED/);
        assert.match(
          result.stderr,
          name === "bridge"
            ? /deleting the package target did not refresh Corsa/
            : /assertion failed: has_code\(&deleted, "TS2307"\)/,
        );
        assert.equal(
          result.status,
          101,
          "Attribution must reproduce the same genuine delete failure before and after runtime policy",
        );
      }
    }
  } finally {
    for (const [file, original] of originals) fs.writeFileSync(file, original);
  }
}
