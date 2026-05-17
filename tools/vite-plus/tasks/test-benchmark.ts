import { cacheInputs, testedPackages } from "../task-inputs.ts";
import {
  defineTasks,
  moonScript,
  noCacheTask,
  runInPackages,
  runTask,
  runTasks,
  task,
} from "../task-helpers.ts";

/**
 * Test, snapshot, coverage, and benchmark tasks.
 *
 * These commands validate observable behavior across Rust compiler crates,
 * JavaScript packages, browser-facing playground flows, and generated fixture
 * output. Snapshot and benchmark entries live beside normal tests because they
 * are both part of the same feedback loop: keep correctness visible and keep
 * performance regressions difficult to miss.
 */
export const testAndBenchmarkTasks = defineTasks({
  test: noCacheTask(runTasks("test:rust", "test:js", "test:scripts")),
  "test:rust": task("cargo test --workspace", { input: cacheInputs.rust }),
  "test:js": noCacheTask(`${runTask("build:native")} && ${runInPackages("test", testedPackages)}`),
  "test:scripts": noCacheTask("node --test --test-concurrency=1 tests/tooling/*.test.ts"),
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
