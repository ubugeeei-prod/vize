import {
  cacheInputs,
  checkedPackages,
  checkedPackagesViaVpRun,
  ciCheckedPackages,
  directCheckPackages,
  packedPackages,
  testedPackages,
} from "./task-inputs.ts";
import {
  defineTasks,
  devApp,
  installVscodeExtensionDependencies,
  localVp,
  moonScript,
  noCacheTask,
  runInPackages,
  runInVscodeExtension,
  runPackageScriptDirectly,
  runTask,
  runTasks,
  task,
} from "./task-helpers.ts";

/**
 * Root setup and development tasks.
 *
 * These tasks are intentionally thin wrappers around the underlying package or
 * MoonBit automation. Keeping them in a dedicated group makes the root config a
 * table of capabilities rather than a long list of shell snippets.
 */
const setupTasks = defineTasks({
  setup: noCacheTask("vp install"),
});

const devTasks = defineTasks({
  dev: noCacheTask(runTask("dev:app")),
  "dev:app": noCacheTask(devApp()),
  "dev:playground": noCacheTask(devApp("playground")),
  "dev:misskey": noCacheTask(devApp("misskey")),
  "dev:npmx": noCacheTask(devApp("npmx")),
  "dev:elk": noCacheTask(devApp("elk")),
  "dev:vuefes": noCacheTask(devApp("vuefes")),
  example: noCacheTask(runInPackages("dev", ["./npm/vite-plugin-vize/example"])),
});

/**
 * Build tasks for Rust crates, npm packages, WASM outputs, and editor
 * extensions.
 */
const buildTasks = defineTasks({
  build: noCacheTask(runTasks("build:rust", "build:all")),
  "build:all": noCacheTask(runTasks("build:runtime", "package:editor-extensions")),
  "build:rust": task("cargo build --workspace", { input: cacheInputs.rust }),
  "build:runtime": noCacheTask(runTasks("build:native", "build:wasm", "build:packages")),
  "build:packages": noCacheTask(runInPackages("build", packedPackages)),
  "build:native": noCacheTask(runPackageScriptDirectly("build", ["./npm/vize-native"])),
  "build:wasm": task(moonScript("build_vitrine_wasm", "nodejs", "npm/vite-plugin-vize/wasm")),
  "build:wasm-web": task(moonScript("build_vitrine_wasm", "web", "playground/src/wasm")),
  "build:vite-plugin": noCacheTask(
    `${runInPackages("build", ["./npm/vize"])} && ${runInPackages("build", ["./npm/vite-plugin-vize"])}`,
  ),
  "build:plugin": noCacheTask(runTask("build:vite-plugin")),
  "build:cli": task("cargo build --release -p vize"),
  "build:vscode-extension": noCacheTask(runInVscodeExtension("pnpm exec vp pack")),
  "build:editor-extensions": noCacheTask(runTasks("build:vscode-extension", "check:zed-extension")),
  "package:vscode-extension": noCacheTask(
    runInVscodeExtension("pnpm exec vsce package --no-dependencies --out dist/vize.vsix"),
  ),
  "check:zed-extension": task("cargo check --manifest-path npm/zed-vize/Cargo.toml", {
    input: ["npm/zed-vize/**"],
  }),
  "package:zed-extension": noCacheTask(
    "tar --exclude 'zed-vize/target' -czf zed-vize-extension.tar.gz -C npm zed-vize",
  ),
  "package:editor-extensions": noCacheTask(
    `${runInVscodeExtension(
      "pnpm exec tsgo --noEmit",
      "pnpm exec vp check src vite.config.ts",
      "pnpm exec vsce package --no-dependencies --out dist/vize.vsix",
    )} && ${runTask("check:zed-extension")} && ${runTask("package:zed-extension")}`,
  ),
  "install:plugin": noCacheTask("vp install --filter './npm/vite-plugin-vize'"),
});

/**
 * Code generation tasks.
 *
 * JSON Schema remains the source of truth for public config declarations, while
 * MoonBit performs the post-processing that encodes Vize-specific API details
 * which the schema generator cannot express on its own.
 */
const generationTasks = defineTasks({
  "gen:schema": noCacheTask(
    "pnpm exec pkl eval -f json npm/vize/pkl/jsonschema/generate.pkl -o npm/vize/schemas/vize.config.schema.json",
  ),
  "gen:types": noCacheTask(
    `${runTask("gen:schema")} && pnpm exec json2ts -i npm/vize/schemas/vize.config.schema.json -o npm/vize/src/types/generated.ts && ${moonScript("postprocess_types")}`,
  ),
  gen: noCacheTask(runTask("gen:types")),
});

const cliTasks = defineTasks({
  cli: noCacheTask(
    'sh -c \'if [ "${usage_debug:-$1}" = "true" ] || [ "$1" = "--debug" ]; then cargo install --path crates/vize --force --locked --debug && echo "Installed vize CLI (debug build)"; else cargo install --path crates/vize --force --locked && echo "Installed vize CLI (release build)"; fi\' --',
  ),
  "cli:help": noCacheTask("vize --help"),
  "cli:example": noCacheTask("vize './**/*.vue' -o . -v"),
  "cli:example-json": noCacheTask("vize './**/*.vue' -o . -f json -v"),
  "cli:example-ssr": noCacheTask("vize './**/*.vue' -o . -f json --ssr -v"),
  "cli:example-stats": noCacheTask("vize './**/*.vue' -f stats -v"),
});

/**
 * Test and snapshot tasks for the Rust compiler, JavaScript packages, MoonBit
 * automation, browser-facing playground checks, and fixture-driven coverage.
 */
const testTasks = defineTasks({
  test: noCacheTask(runTasks("test:rust", "test:js", "test:scripts")),
  "test:rust": task("cargo test --workspace", { input: cacheInputs.rust }),
  "test:js": noCacheTask(`${runTask("build:native")} && ${runInPackages("test", testedPackages)}`),
  "test:scripts": task("node --test --test-concurrency=1 tests/tooling/*.test.ts", {
    input: cacheInputs.rust,
  }),
  "test:playground": task(runInPackages("test:browser", ["./playground"]), {
    input: cacheInputs.jsChecks,
  }),
  "test:e2e": noCacheTask(runTasks("test:e2e:dev", "test:e2e:preview")),
  "test:e2e:dev": task(runInPackages("test:dev", ["./tests"]), { input: cacheInputs.e2e }),
  "test:e2e:preview": task(runInPackages("test:preview", ["./tests"]), {
    input: cacheInputs.e2e,
  }),
  "test:e2e:vrt": task(runInPackages("test:vrt", ["./tests"]), { input: cacheInputs.e2e }),
  "test:vue": task("cargo test -p vize_test_runner", { input: cacheInputs.rust }),
  coverage: task("cargo run -p vize_test_runner --bin coverage", { input: cacheInputs.rust }),
  "coverage:verbose": task("cargo run -p vize_test_runner --bin coverage -- -v", {
    input: cacheInputs.rust,
  }),
  "coverage:diff": task("cargo run -p vize_test_runner --bin coverage -- -vv", {
    input: cacheInputs.rust,
  }),
  "generate:rule-types": task(moonScript("generate_rule_types"), {
    input: cacheInputs.rust,
  }),
  "expected:generate": task(moonScript("generate_expected")),
  "expected:generate:sfc": task(moonScript("generate_expected", "--mode", "sfc")),
  "expected:generate:vdom": task(moonScript("generate_expected", "--mode", "vdom")),
  "expected:generate:vapor": task(moonScript("generate_expected", "--mode", "vapor")),
  snapshot: noCacheTask(runTasks("snapshot:test", "snapshot:review")),
  "snapshot:test": task("cargo insta test -p vize_atelier_sfc -- snapshot_tests"),
  "snapshot:review": noCacheTask("cargo insta review"),
  "snapshot:accept": noCacheTask("cargo insta accept"),
});

const benchmarkTasks = defineTasks({
  bench: noCacheTask(moonScript("bench", "run")),
  "bench:quick": noCacheTask(moonScript("bench", "run", "1000")),
  "bench:generate": noCacheTask(moonScript("bench", "generate", "15000")),
  "bench:lint": noCacheTask(moonScript("bench", "lint")),
  "bench:fmt": noCacheTask(moonScript("bench", "fmt")),
  "bench:check": noCacheTask(moonScript("bench", "check")),
  "bench:vite": noCacheTask(moonScript("bench", "vite")),
  "bench:all": noCacheTask(
    runTasks("bench", "bench:lint", "bench:fmt", "bench:check", "bench:vite"),
  ),
  "bench:rust": noCacheTask("cargo bench -p vize_atelier_sfc"),
});

const ciPackageCheckCommand = runInPackages("check", ciCheckedPackages, {
  concurrencyLimit: 1,
});
const directPackageCheckCommand = runPackageScriptDirectly("check", directCheckPackages);

/**
 * Repository-wide formatting, linting, package checks, and CI aggregate tasks.
 */
const checkTasks = defineTasks({
  check: noCacheTask(runTasks("check:repo", "check:rust", "check:js", "check:editor-extensions")),
  "check:js": noCacheTask(runTasks("check:js:packages", "check:js:direct-packages")),
  "check:js:packages": task(
    runInPackages("check", checkedPackagesViaVpRun, { concurrencyLimit: 1 }),
    {
      input: cacheInputs.jsChecks,
    },
  ),
  "check:js:direct-packages": noCacheTask(directPackageCheckCommand),
  "check:repo": noCacheTask(`${localVp} check`),
  // The oxlint example intentionally exits non-zero for its default lint script,
  // so CI checks every package except that runnable failure-case fixture.
  "check:ci": noCacheTask(
    `${runTask("check:repo")} && ${ciPackageCheckCommand} && ${directPackageCheckCommand}`,
  ),
  "check:fix": noCacheTask(runInPackages("check:fix", checkedPackages)),
  "check:rust": noCacheTask("cargo check --workspace"),
  "check:vscode-extension": noCacheTask(
    runInVscodeExtension("pnpm exec tsgo --noEmit", "pnpm exec vp check src vite.config.ts"),
  ),
  "check:editor-extensions": noCacheTask(runTasks("check:vscode-extension", "check:zed-extension")),
  clippy: task("cargo clippy --workspace -- -D warnings", { input: cacheInputs.rust }),
  fmt: noCacheTask(runTasks("fmt:repo", "fmt:rust", "fmt:js")),
  "fmt:repo": noCacheTask(`${localVp} fmt --write`),
  "fmt:js": noCacheTask(runInPackages("fmt", checkedPackages)),
  "fmt:rust": task("cargo fmt --all", { input: cacheInputs.rust }),
  "fmt:all": noCacheTask(runTask("fmt")),
  lint: noCacheTask(runTask("check")),
  "lint:fix": noCacheTask(runTask("check:fix")),
  "lint:rust": task("cargo clippy --workspace -- -D warnings", { input: cacheInputs.rust }),
  "lint:all": noCacheTask(runTasks("lint:rust", "check")),
  "fmt:check": noCacheTask(runTask("check")),
  ci: noCacheTask(runTasks("fmt:all", "clippy", "test", "check:ci")),
});

const releaseTasks = defineTasks({
  release: noCacheTask(moonScript("release")),
  "publish:wasm": noCacheTask(
    `${moonScript("build_vize_wasm_package")} && ${moonScript("publish_npm_package", "npm/vize-wasm")}`,
  ),
  "publish:native": noCacheTask(
    `${runTask("build:native")} && ${moonScript("publish_npm_package", "npm/vize-native")}`,
  ),
  "publish:vite-plugin": noCacheTask(
    `${runTask("build:vite-plugin")} && ${moonScript("publish_npm_package", "npm/vite-plugin-vize")}`,
  ),
  "publish:oxlint-plugin": noCacheTask(
    `${runInPackages("build", ["./npm/oxlint-plugin-vize"])} && ${moonScript("inject_native_optional_deps", "npm/oxlint-plugin-vize/package.json", "npm/vize-native/package.json")} && ${moonScript("publish_npm_package", "npm/oxlint-plugin-vize")}`,
  ),
  "publish:npm": noCacheTask(
    runTasks("publish:wasm", "publish:native", "publish:vite-plugin", "publish:oxlint-plugin"),
  ),
  "publish:crates": noCacheTask(moonScript("publish_crates")),
  "publish:vscode-extension": noCacheTask(
    `${installVscodeExtensionDependencies} && ${moonScript("publish_vscode_extension", "npm/vscode-vize/dist/vize.vsix")}`,
  ),
  publish: noCacheTask(runTasks("publish:npm", "publish:crates")),
});

/**
 * Fully assembled root Vite+ task catalog.
 *
 * Exporting the catalog as one value keeps `vite.config.ts` compact while still
 * letting every group above retain local names, comments, and type checking.
 */
export const taskCatalog = defineTasks({
  ...setupTasks,
  ...devTasks,
  ...buildTasks,
  ...generationTasks,
  ...cliTasks,
  ...testTasks,
  ...benchmarkTasks,
  ...checkTasks,
  ...releaseTasks,
});
