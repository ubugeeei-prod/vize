// Disposable Actions checkout only. No product switch or protocol override.
import {
  assert,
  readFile,
  writeFile,
  mkdir,
  copyFile,
  readdir,
  unlink,
  path,
  sha256,
  json,
  run,
  checked,
  inventory,
  transforms,
} from "./owner-process.mjs";

const root = path.resolve(process.env.RUNNER_TEMP, "directive-owners");
const baseline = "8d3ca7bbc544ec1d8cf8e275cc758037c5289237";
const overlay = [
  "crates/vize/tests/content_mapper_tsgo_directives.rs",
  "crates/vize/tests/content_mapper_tsgo_directives/owners.rs",
  "crates/vize/tests/content_mapper_tsgo_directives/owner_lsp.rs",
];
const owned = [
  ...overlay,
  ".github/workflows/content-mapper-conformance.yml",
  "crates/vize_canon/src/template_diagnostic_directives.rs",
  "crates/vize_canon/src/template_diagnostic_directives/tests.rs",
  "crates/vize_canon/src/batch/virtual_project/content_mapper_directive_tests.rs",
  ...[
    "scope.rs",
    "scope/closures.rs",
    "scope/component_props.rs",
    "scope/component_prop_navigation.rs",
    "scope/root_owner_packets.rs",
    "scope/root_owner_packets/tests.rs",
  ].map((file) => `crates/vize_canon/src/virtual_ts/${file}`),
  "docs/davinci/decisions/2026-10-17-content-mapper-static-owner-packets.md",
  ...["owner-process.mjs", "owner-verify.mjs", "owner-qualify.mjs"].map(
    (file) => `tools/support/compat/content-mapper/${file}`,
  ),
];
const stock = [
  "content_mapper_tsgo_lsp",
  "content_mapper_lsp_shutdown",
  "content_mapper_tsgo_declaration_lsp",
  "content_mapper_tsgo_lsp_event_forms",
];
const unitFilters = [
  "root_owner_packets",
  "static_component_expect_owns_one_packet_without_its_child",
  "borrowed_root_preserves_the_complete_canonical_policy_with_offsets",
  "unrelated_marker_text_never_creates_owner_directives",
];
const env = {
  VIZE_TEST_CONTENT_MAPPER_TSGO: path.resolve(process.env.RUNNER_TEMP, "tsgo"),
  CORSA_PATH: path.resolve(process.env.RUNNER_TEMP, "tsgo"),
  CORSA_EXECUTABLE: path.resolve(process.env.RUNNER_TEMP, "tsgo"),
  TSGO_PATH: path.resolve(process.env.RUNNER_TEMP, "tsgo"),
  TSGO_EXECUTABLE: path.resolve(process.env.RUNNER_TEMP, "tsgo"),
  VIZE_TEST_CONTENT_MAPPER_VUE: path.resolve("npm/cli/node_modules/vue"),
};
let sequence = 0;
const git = (...args) => checked("git", args, path.join(root, "git", String(sequence++)));
async function build(phase, args) {
  const directory = path.join(root, phase, "build");
  await checked(
    "cargo",
    [
      "test",
      "--locked",
      "--profile",
      "ci",
      ...args,
      "--no-run",
      "--message-format=json-render-diagnostics",
    ],
    directory,
  );
  const packets = (await readFile(path.join(directory, "stdout.bin"), "utf8"))
    .trim()
    .split("\n")
    .map(JSON.parse);
  const artifacts = packets.filter(
    (packet) => packet.reason === "compiler-artifact" && packet.executable,
  );
  await json(path.join(directory, "executables.json"), artifacts);
  return artifacts;
}
function artifact(artifacts, name, kind) {
  const matches = artifacts.filter(
    (packet) => packet.target.name === name && packet.target.kind.includes(kind),
  );
  assert.equal(matches.length, 1, `${kind} ${name} executable identity`);
  return matches[0].executable;
}
async function preserve(executable, destination) {
  await copyFile(executable, destination);
  return {
    compiledPath: executable,
    retainedPath: destination,
    sha256: sha256(await readFile(executable)),
  };
}
async function test(executable, args, directory, extraEnv = {}) {
  return run(executable, args, directory, { env: { ...env, ...extraEnv } });
}
async function testList(executable, directory) {
  const list = await test(executable, ["--list", "--format", "terse"], directory);
  assert.equal(list.status, 0);
  const names = list.stdout
    .toString()
    .split("\n")
    .filter((line) => line.endsWith(": test"))
    .map((line) => line.slice(0, -6));
  assert(names.length > 0);
  return names;
}
// The baseline still links each CLI test file as its own binary; the candidate
// links them as modules of one `integration` binary.
const layouts = {
  old: { target: (name) => name, prefix: () => "" },
  new: { target: () => "integration", prefix: (name) => `${name}::` },
};
async function phase(name, artifacts) {
  const directory = path.join(root, name);
  const layout = layouts[name];
  const cli = artifact(artifacts, "vize", "bin");
  const owner = "content_mapper_tsgo_directives";
  const harness = artifact(artifacts, layout.target(owner), "test");
  await json(path.join(directory, "binaries.json"), {
    cli: await preserve(cli, path.join(directory, "vize")),
    harness: await preserve(harness, path.join(directory, "owner-harness")),
  });
  const names = await testList(harness, path.join(directory, "test-list"));
  for (const [file, selector] of [
    ["cli-test", "owners::parent_expect_keeps_child_visible_after_each_parent_repair"],
    ["lsp-test", "owner_lsp::parent_expect_editor_repairs_preserve_whole_reports"],
  ].map(([file, test]) => [file, layout.prefix(owner) + test])) {
    assert(names.includes(selector));
    await test(harness, ["--exact", selector, "--nocapture"], path.join(directory, file), {
      VIZE_TEST_OWNER_CAPTURE_DIR: path.join(directory, "packets"),
    }); // retain a baseline assertion failure, then continue to the other boundary
  }
  try {
    await transforms(cli, directory);
  } catch (error) {
    await json(path.join(directory, "mapper-failure.json"), {
      message: String(error),
      stack: error.stack,
    });
  }
}
async function stockGates(artifacts, canonArtifacts) {
  const results = [];
  for (const name of [...stock, "lsp_import_resolution"]) {
    const canon = name === "lsp_import_resolution";
    const prefix = canon ? "" : layouts.new.prefix(name);
    const executable = canon
      ? artifact(canonArtifacts, name, "test")
      : artifact(artifacts, layouts.new.target(name), "test");
    const directory = path.join(root, "new/stock", name);
    await mkdir(directory, { recursive: true });
    const binary = await preserve(executable, path.join(directory, "harness"));
    const names = (await testList(executable, path.join(directory, "list")))
      .filter((test) => test.startsWith(prefix))
      .map((test) => test.slice(prefix.length));
    const filter = prefix ? [prefix] : [];
    const result = await test(executable, [...filter, "--nocapture"], path.join(directory, "run"));
    const pureNames = names.filter(
      (test) =>
        test.includes("leak_assertions::tests::") ||
        test === "graceful_close_omits_shutdown_and_exit_params",
    );
    results.push({
      name,
      names,
      pureNames,
      backendDependentNames: names.filter((test) => !pureNames.includes(test)),
      binary,
      status: result.status,
      signal: result.signal,
    });
  }
  await json(path.join(root, "new/stock-results.json"), results);
}
async function units(artifacts) {
  const executable = artifact(artifacts, "vize_canon", "lib");
  const allNames = await testList(executable, path.join(root, "new/unit-list"));
  const results = [];
  for (const filter of unitFilters) {
    const names = allNames.filter((name) => name.includes(filter));
    assert.equal(names.length, filter === "root_owner_packets" ? 3 : 1);
    const result = await test(
      executable,
      [filter, "--nocapture"],
      path.join(root, "new/units", filter),
    );
    results.push({ filter, names, status: result.status, signal: result.signal });
  }
  await json(path.join(root, "new/unit-results.json"), results);
}

await mkdir(root, { recursive: true });
try {
  assert.equal(process.env.GITHUB_ACTIONS, "true", "requires disposable Actions checkout");
  const candidate = await git("rev-parse", "HEAD");
  assert.equal(candidate, process.env.GITHUB_SHA);
  assert.equal(await git("status", "--porcelain", "--untracked-files=no"), "");
  await git("merge-base", "--is-ancestor", baseline, candidate);
  const changed = (await git("diff", "--name-only", baseline, candidate))
    .split("\n")
    .filter(Boolean)
    .sort();
  assert.deepEqual(changed, owned.toSorted(), "exact reviewed change boundary");
  const tree = await git("rev-parse", `${candidate}^{tree}`);
  const oldTree = await git("rev-parse", `${baseline}^{tree}`);
  const helper = "crates/vize/tests/content_mapper_lsp_support";
  assert.equal(
    await git("rev-parse", `${baseline}:${helper}`),
    await git("rev-parse", `${candidate}:${helper}`),
  );
  const saved = new Map();
  for (const file of overlay) saved.set(file, await readFile(file));
  const patch = await run(
    "git",
    ["diff", "--binary", baseline, candidate],
    path.join(root, "patch"),
  );
  assert.equal(patch.status, 0);
  await writeFile(path.join(root, "source.patch"), patch.stdout);
  const sourceFiles = await git("ls-tree", "-r", candidate, "--", ...owned, helper);
  await json(path.join(root, "identity.json"), {
    baseline,
    candidate,
    oldTree,
    tree,
    changed,
    sourceFiles,
    env,
    overlay: [...saved].map(([file, bytes]) => ({ file, sha256: sha256(bytes) })),
  });
  const stockSources = (
    await git(
      "ls-files",
      "--",
      ...stock.flatMap((name) => [`crates/vize/tests/${name}.rs`, `crates/vize/tests/${name}`]),
      "crates/vize_canon/tests/lsp_import_resolution.rs",
      "crates/vize_canon/tests/lsp_import_resolution",
      helper,
    )
  )
    .split("\n")
    .filter(Boolean);
  for (const file of stockSources) {
    const destination = path.join(root, "stock-source", file);
    await mkdir(path.dirname(destination), { recursive: true });
    await copyFile(file, destination);
  }
  const sourceQuerySites = [];
  for (const file of stockSources) {
    (await readFile(file, "utf8")).split("\n").forEach((text, index) => {
      if (
        /\b(?:request::<|notify::<|pull_diagnostics\(|hover\(|references\(|rename\(|assert_completion\(|signature_help\(|definition\()/.test(
          text,
        )
      ) {
        sourceQuerySites.push({ file, line: index + 1, text });
      }
    });
  }
  await json(path.join(root, "stock-source-inventory.json"), {
    files: stockSources,
    sourceQuerySites,
    boundary:
      "Source call sites and complete assertions, not an invented runtime RPC count. Pure law tests are counted separately.",
  });
  const provider = path.join(process.env.GITHUB_WORKSPACE, "typescript-content-mapper");
  assert.equal(
    await git("-C", provider, "rev-parse", "HEAD"),
    process.env.CONTENT_MAPPER_TYPESCRIPT_SHA,
  );
  await copyFile(env.VIZE_TEST_CONTENT_MAPPER_TSGO, path.join(root, "tsgo"));
  await mkdir(path.join(root, "stdlib"), { recursive: true });
  const libraries = (await readdir(process.env.RUNNER_TEMP)).filter((file) =>
    file.endsWith(".d.ts"),
  );
  assert(libraries.length > 0);
  for (const file of libraries) {
    if (file.endsWith(".d.ts"))
      await copyFile(path.join(process.env.RUNNER_TEMP, file), path.join(root, "stdlib", file));
  }
  await json(path.join(root, "provider.json"), {
    source: process.env.CONTENT_MAPPER_TYPESCRIPT_SHA,
    binary: sha256(await readFile(env.VIZE_TEST_CONTENT_MAPPER_TSGO)),
    libraries,
  });
  await copyFile("pnpm-lock.yaml", path.join(root, "pnpm-lock.yaml"));
  await copyFile(
    path.join(env.VIZE_TEST_CONTENT_MAPPER_VUE, "package.json"),
    path.join(root, "vue-package.json"),
  );
  for (const [command, args] of [
    ["rustc", ["-Vv"]],
    ["go", ["version"]],
    ["node", ["--version"]],
  ]) {
    await checked(command, args, path.join(root, "versions", command));
  }
  // Only the exact authored integration tests are overlaid. The old producer,
  // Cargo graph, helper sources and compiled CARGO_BIN_EXE remain genuine B.
  await git("checkout", "--detach", baseline);
  for (const [file, bytes] of saved) {
    await mkdir(path.dirname(file), { recursive: true });
    await writeFile(file, bytes);
    assert.equal(sha256(await readFile(file)), sha256(bytes));
  }
  const overlayPatch = await run(
    "git",
    ["diff", "--binary", "HEAD"],
    path.join(root, "overlay-patch"),
  );
  assert.equal(overlayPatch.status, 0);
  await writeFile(path.join(root, "old-test-overlay.patch"), overlayPatch.stdout);
  assert.equal((await git("ls-tree", "-r", baseline, "--", ...overlay)).split("\n").length, 1);
  for (const [file, bytes] of saved) {
    const destination = path.join(root, "test-overlay", file);
    await mkdir(path.dirname(destination), { recursive: true });
    await writeFile(destination, bytes);
  }
  const trackedOverlay = await git("diff", "--name-only", "HEAD");
  assert.equal(trackedOverlay, overlay[0]);
  const untrackedOverlay = (
    await git("ls-files", "--others", "--exclude-standard", "--", ...overlay)
  )
    .split("\n")
    .sort();
  assert.deepEqual(untrackedOverlay, overlay.slice(1).toSorted());
  await json(path.join(root, "old-test-overlay-status.json"), {
    trackedOverlay,
    untrackedOverlay,
    producer: await git("rev-parse", "HEAD"),
  });
  await phase(
    "old",
    await build("old", ["-p", "vize", "--test", "content_mapper_tsgo_directives"]),
  );
  for (const [file, bytes] of saved) assert.equal(sha256(await readFile(file)), sha256(bytes));
  await git("restore", "--source", baseline, "--worktree", "--", overlay[0]);
  for (const file of overlay.slice(1)) await unlink(file);
  await git("checkout", "--detach", candidate);
  assert.equal(await git("status", "--porcelain", "--untracked-files=no"), "");
  const artifacts = await build("new", ["-p", "vize", "--test", "integration"]);
  await phase("new", artifacts);
  const canon = await build("canon", [
    "-p",
    "vize_canon",
    "--lib",
    "--test",
    "lsp_import_resolution",
  ]);
  await units(canon);
  await stockGates(artifacts, canon);
  await json(path.join(root, "capture-complete.json"), { baseline, candidate });
} catch (error) {
  await json(path.join(root, "capture-failure.json"), {
    message: String(error),
    stack: error.stack,
  });
  process.exitCode = 1;
} finally {
  await json(
    path.join(process.env.RUNNER_TEMP, "directive-owners-inventory.json"),
    await inventory(root),
  );
}
