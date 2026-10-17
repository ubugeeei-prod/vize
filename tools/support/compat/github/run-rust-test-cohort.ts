import { spawn } from "node:child_process";
import { createWriteStream, mkdirSync, openSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { finished } from "node:stream/promises";
import { setTimeout as delay } from "node:timers/promises";
import { pathToFileURL } from "node:url";
import { readCargoMetadata } from "./plan-affected-rust.ts";
import type { CargoMetadata } from "./plan-affected-rust.ts";
import { rustCommand } from "./run-affected-rust.ts";
import { isRustTestCohort, rustTestCohortForBinary, rustTestMatrix } from "./rust-test-cohorts.ts";
import { prDefaultFeatures } from "./rust-test-default-features.ts";
import type { PrDefaultFeatures } from "./rust-test-default-features.ts";

type Target = {
  name: string;
  kind: string[];
  test: boolean;
  "required-features"?: string[];
};
type Package = { name: string; targets: Target[] };
type Projection = { args: string[]; reason?: string };
const libraryKinds = new Set(["lib", "rlib", "dylib", "staticlib", "cdylib", "proc-macro"]);
const ordinaryKinds = new Set(["bin", "test", "example", "bench", "custom-build"]);
const isLibrary = (target: Target) => target.kind.every((kind) => libraryKinds.has(kind));
const defaultTargets = (reason: string): Projection => ({ args: [], reason });

function selectedTargets(metadata: CargoMetadata, selected: readonly string[]): Package[] {
  return metadata.packages
    .filter((pkg) => metadata.workspace_members.includes(pkg.id) && selected.includes(pkg.name))
    .map((pkg) => {
      const targets: unknown = (pkg as unknown as Package).targets;
      if (!Array.isArray(targets) || !targets.length)
        throw new Error(`missing Cargo targets: ${pkg.name}`);
      const identities = new Set<string>();
      for (const target of targets as Target[]) {
        if (
          !target ||
          typeof target.name !== "string" ||
          !/^[A-Za-z0-9_][A-Za-z0-9_.-]*$/.test(target.name) ||
          !Array.isArray(target.kind) ||
          !target.kind.length ||
          target.kind.some((kind) => typeof kind !== "string" || !kind) ||
          typeof target.test !== "boolean" ||
          (target["required-features"] !== undefined &&
            (!Array.isArray(target["required-features"]) ||
              target["required-features"].some(
                (feature) => typeof feature !== "string" || !feature,
              )))
        )
          throw new Error(`invalid Cargo target metadata: ${pkg.name}`);
        const identity = `${target.kind.join(",")}:${target.name}`;
        if (identities.has(identity))
          throw new Error(`duplicate Cargo target: ${pkg.name}:${identity}`);
        identities.add(identity);
      }
      return { name: pkg.name, targets: targets as Target[] };
    });
}

export function rustTestTargetProjection(
  metadata: CargoMetadata,
  selected: readonly string[],
  cohort: string,
  features?: PrDefaultFeatures,
): Projection {
  const matrix = rustTestMatrix(metadata, selected).include;
  if (!isRustTestCohort(cohort) || !matrix.some((entry) => entry.cohort === cohort)) {
    throw new Error(`Rust test cohort is not selected: ${cohort}`);
  }
  const packages = selectedTargets(metadata, selected);
  const targets = packages.flatMap((pkg) => pkg.targets);
  // Explicit --lib/--bins would override test=false. Unusual runnable target
  // kinds also retain Cargo's complete default selection, never silently drop.
  if (
    targets.some(
      (target) =>
        (!isLibrary(target) && (target.kind.length !== 1 || !ordinaryKinds.has(target.kind[0]))) ||
        ((isLibrary(target) || target.kind.includes("bin")) && !target.test) ||
        (target.test && !isLibrary(target) && !["bin", "test"].includes(target.kind[0])),
    )
  )
    return defaultTargets("nonstandard default test targets");

  const projections = new Map(
    matrix.map((entry) => {
      const names = [
        ...new Set(
          packages.flatMap((pkg) =>
            pkg.targets
              .filter(
                (target) =>
                  target.kind.includes("test") &&
                  target.test &&
                  rustTestCohortForBinary(pkg.name, target.name) === entry.cohort,
              )
              .map((target) => target.name),
          ),
        ),
      ].sort();
      // --test names apply across all selected packages. Every same-name target
      // must be active, or every instance inactive, before projecting that name.
      let gated = false;
      const active = names.filter((name) => {
        const states = packages.flatMap((pkg) =>
          pkg.targets
            .filter((target) => target.kind.includes("test") && target.name === name)
            .map((target) =>
              !target.test
                ? "unknown"
                : target["required-features"]?.length
                  ? (features?.requiredFeatures(pkg.name, target["required-features"]) ?? "unknown")
                  : "active",
            ),
        );
        if (states.every((state) => state === "inactive")) return false;
        if (states.some((state) => state !== "active")) gated = true;
        return true;
      });
      const args = [
        ...(targets.some(isLibrary) ? ["--lib"] : []),
        ...(targets.some((target) => target.kind.includes("bin")) ? ["--bins"] : []),
        ...active.flatMap((name) => ["--test", name]),
      ];
      const projection: Projection = gated
        ? defaultTargets("feature-gated or disabled integration targets")
        : args.length
          ? { args }
          : defaultTargets("no explicit test targets");
      return [entry.cohort, projection] as const;
    }),
  );
  // The ordinary full-workspace PR recipe explicitly builds normal examples in the
  // required Clippy/doctest job. Other invocations retain the original fallback.
  if (
    !features &&
    targets.some((target) => target.kind.includes("example")) &&
    ![...projections.values()].some((projection) => projection.reason)
  ) {
    projections.set(matrix[0].cohort, defaultTargets("retain default example compilation"));
  }
  return projections.get(cohort)!;
}

function captureStdout(filename: string) {
  // Open before spawning Cargo: an invalid destination must not leave a child.
  const stream = createWriteStream(filename, { fd: openSync(filename, "w") });
  let error: Error | undefined;
  let failed!: () => void;
  const failure = new Promise<void>((resolve) => {
    failed = resolve;
  });
  const fail = (cause: Error) => {
    error ??= cause;
    failed();
  };
  stream.on("error", fail);
  process.stdout.on("error", fail);
  const drained = finished(stream).catch(fail);
  let finishing: Promise<void> | undefined;
  return {
    stream,
    failure,
    fail,
    get error() {
      return error;
    },
    write(chunk: string) {
      process.stdout.write(chunk);
      stream.write(chunk);
    },
    finish() {
      finishing ??= (async () => {
        stream.end();
        await Promise.all([
          drained,
          new Promise<void>((resolve) => {
            process.stdout.write("", (cause) => {
              if (cause) fail(cause);
              resolve();
            });
          }),
        ]);
      })();
      return finishing;
    },
    dispose() {
      process.stdout.removeListener("error", fail);
    },
  };
}

async function runCargo(
  executable: string,
  args: string[],
  capture?: ReturnType<typeof captureStdout>,
): Promise<number> {
  // Only this child and descendants inherit this new POSIX process group.
  // Keep the event loop live: spawnSync prevents cancellation handlers running
  // while nextest/Cargo are compiling, leaving their children for runner cleanup.
  const grouped = process.platform !== "win32";
  const child = spawn(executable, args, {
    detached: grouped,
    stdio: capture ? ["inherit", "pipe", "inherit"] : "inherit",
    shell: false,
  });
  if (capture && child.stdout) {
    // Both destinations receive the same bytes with stream backpressure. Keep
    // stdout open, and finish the file only after the child's output closes.
    child.stdout.on("error", capture.fail);
    child.stdout.pipe(process.stdout, { end: false });
    child.stdout.pipe(capture.stream, { end: false });
  }
  let closed = false;
  const completion = new Promise<{ status: number | null; error?: Error }>((resolve) => {
    child.once("error", (error) => {
      closed = true;
      resolve({ status: null, error });
    });
    child.once("close", (status) => {
      closed = true;
      resolve({ status });
    });
  });
  function signalOwned(signal: "SIGINT" | "SIGTERM" | "SIGKILL" | 0): boolean {
    if (child.pid === undefined) return false;
    try {
      if (grouped) process.kill(-child.pid, signal);
      else if (closed) return false;
      else if (signal !== 0) child.kill(signal);
      return true;
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code === "ESRCH") return false;
      throw error;
    }
  }
  let cancelled: "SIGINT" | "SIGTERM" | undefined;
  let interrupted!: () => void;
  const cancellation = new Promise<void>((resolve) => {
    interrupted = resolve;
  });
  const cancel = (signal: "SIGINT" | "SIGTERM") => {
    cancelled ??= signal;
    interrupted();
  };
  const onInterrupt = () => cancel("SIGINT");
  const onTerminate = () => cancel("SIGTERM");
  process.on("SIGINT", onInterrupt);
  process.on("SIGTERM", onTerminate);
  try {
    const flushed = completion.then(() => capture?.finish());
    await Promise.race([flushed, cancellation, ...(capture ? [capture.failure] : [])]);
    if (cancelled || capture?.error) {
      const started = performance.now();
      process.stderr.write(
        cancelled
          ? `Rust cohort received ${cancelled}; stopping its owned Cargo group.\n`
          : "Rust cohort stdout capture failed; stopping its owned Cargo group.\n",
      );
      signalOwned(cancelled ?? "SIGTERM");
      let repeated = false;
      let killed = false;
      // The group can outlive the direct Cargo child. Do not stop draining when
      // that child exits, or clear escalation while its compiler children live.
      while (signalOwned(0) || !closed) {
        const elapsed = performance.now() - started;
        // Nextest owns separate test groups. A second shutdown signal makes
        // nextest kill/reap those groups before we consider killing nextest.
        // This is required even when the first signal was already SIGTERM.
        if (!repeated && elapsed >= 1_000) {
          signalOwned("SIGTERM");
          repeated = true;
        }
        if (!killed && elapsed >= 4_000) {
          signalOwned("SIGKILL");
          killed = true;
        }
        if (elapsed >= 6_000) {
          child.unref();
          throw new Error("owned Cargo process group did not drain after cancellation and SIGKILL");
        }
        await delay(25);
      }
      await capture?.finish();
      if (!cancelled) throw capture!.error;
      // A child may catch a cancellation signal and exit zero. Cancellation
      // must still fail the step and its required Rust aggregate.
      return cancelled === "SIGINT" ? 130 : 143;
    }
    const result = await completion;
    if (result.error) throw result.error;
    return result.status ?? 1;
  } finally {
    process.removeListener("SIGINT", onInterrupt);
    process.removeListener("SIGTERM", onTerminate);
  }
}

export async function main(argv = process.argv.slice(2)) {
  const started = performance.now();
  const [planPath, cohort, ...options] = argv;
  let status = 1;
  let capture: ReturnType<typeof captureStdout> | undefined;
  try {
    if (!planPath || !cohort) throw new Error("expected Rust plan path and semantic cohort");
    const plan = JSON.parse(readFileSync(planPath, "utf8"));
    // Validate the unchanged package roots before reading metadata or launching Cargo.
    rustCommand(plan, ["cargo", "nextest", "run", "@packages@"]);
    const metadata = readCargoMetadata();
    const features = prDefaultFeatures(
      metadata,
      plan.packages,
      options,
      process.env.GITHUB_EVENT_NAME,
    );
    const projection = rustTestTargetProjection(metadata, plan.packages, cohort, features);
    const stdoutPath = process.env.VIZE_RUST_BUILD_STDOUT_PATH;
    if (stdoutPath) capture = captureStdout(stdoutPath);
    const banner = `Cargo targets (${cohort}): ${projection.reason ?? `${projection.args.filter((arg) => arg === "--test").length} integration targets and shared unit targets`}\n`;
    if (capture) capture.write(banner);
    else process.stdout.write(banner);
    const [executable, ...args] = rustCommand(plan, [
      "cargo",
      "nextest",
      "run",
      "@packages@",
      ...projection.args,
      ...options,
    ]);
    status = await runCargo(executable, args, capture);
    process.exitCode = status;
  } finally {
    await capture?.finish();
    capture?.dispose();
    // The workflow execs this CLI so Actions signals Node directly. Keep its
    // existing receipt here instead of leaving a waiting Bash process in front.
    const timingPath = process.env.VIZE_RUST_COHORT_TIMING_PATH;
    if (timingPath) {
      mkdirSync(dirname(timingPath), { recursive: true });
      writeFileSync(
        timingPath,
        `${JSON.stringify({
          phase: "workspace-cohort",
          cohort,
          elapsed_seconds: Math.floor((performance.now() - started) / 1_000),
          exit_code: status,
        })}\n`,
      );
    }
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href)
  await main();
