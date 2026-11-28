import { spawn } from "node:child_process";
import { createHash } from "node:crypto";
import { cpSync, mkdirSync, readFileSync, symlinkSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { setTimeout as pause } from "node:timers/promises";
import type { HMRPayload } from "vite-plus";
import { startNativeDevelopment } from "./development.ts";

const packageRoot = fileURLToPath(new URL("../", import.meta.url));
const refreshMarker = "VIZE_NATIVE_TEMPLATE_REFRESH_READY";
const maximumLogBytes = 64 * 1024 * 1024;
const maximumInteractionMilliseconds = 10 * 60 * 1000;

/** An invocation-owned application copy used by the real native refresh test. */
export interface NativeRefreshFixture {
  readonly origin: string;
  readonly close: () => Promise<void>;
  /** Apply only a template change after the native test has observed live state. */
  readonly observe: (output: string) => void;
  /** Wait for the real SFC compiler's template rerender notification. */
  readonly waitForTemplateUpdate: () => Promise<void>;
  /** Wait for a real HMR error after a source save violates native admission. */
  readonly waitForSourceRefusal: () => Promise<void>;
  /** Assert that the fixture actually changed the intended canonical source. */
  readonly finish: () => void;
}

/** Creates a real development server without editing tracked application sources. */
export async function createNativeRefreshFixture(
  output: string,
  compilerPath?: string,
): Promise<NativeRefreshFixture> {
  const root = path.join(output, "development-source");
  mkdirSync(root, { recursive: true });
  cpSync(path.join(packageRoot, "src"), path.join(root, "src"), { recursive: true });
  symlinkSync(path.join(packageRoot, "node_modules"), path.join(root, "node_modules"), "dir");
  writeFileSync(path.join(root, "package.json"), '{"private":true,"type":"module"}\n');
  const applicationFile = path.join(root, "src/App.vue");
  const original = readFileSync(applicationFile, "utf8");
  const replacement = original.replace("`Count: ${count}`", "`Refreshed count: ${count}`");
  if (replacement === original)
    throw new Error("The native refresh fixture requires its counter template.");
  const { server, origin, application } = await startNativeDevelopment({
    root,
    ...(compilerPath ? { compilerPath } : {}),
  });
  let changed = false;
  const client = server.environments.client;
  if (!client) {
    await server.close();
    throw new Error("Native development requires a client environment.");
  }
  let rerenderObserved = false;
  let sourceRefusalObserved = false;
  const originalSend = client.hot.send.bind(client.hot);
  client.hot.send = (payload: HMRPayload | string, data?: unknown) => {
    if (typeof payload === "string") {
      if (changed && payload === "vize:rerender") rerenderObserved = true;
      originalSend(payload, data);
    } else {
      if (changed && payload.type === "custom" && payload.event === "vize:rerender")
        rerenderObserved = true;
      if (
        payload.type === "error" &&
        payload.err.message.includes("canonical SFC sections and empty native styles")
      ) {
        sourceRefusalObserved = true;
      }
      originalSend(payload);
    }
  };
  let pending = "";
  return {
    origin,
    close: () => server.close(),
    observe(chunk) {
      if (changed) return;
      const observed = `${pending}${chunk}`;
      pending = observed.slice(-refreshMarker.length);
      if (!observed.includes(refreshMarker)) return;
      writeFileSync(applicationFile, replacement);
      changed = true;
    },
    async waitForTemplateUpdate() {
      const deadline = Date.now() + 10000;
      while (!rerenderObserved) {
        if (Date.now() >= deadline)
          throw new Error("Native template refresh did not reach the compiler's HMR boundary.");
        await pause(20);
      }
    },
    async waitForSourceRefusal() {
      const deadline = Date.now() + 10000;
      while (!sourceRefusalObserved) {
        if (Date.now() >= deadline)
          throw new Error("Native style save did not produce its HMR admission diagnostic.");
        await pause(20);
      }
    },
    finish() {
      if (!changed || !rerenderObserved || readFileSync(applicationFile, "utf8") !== replacement) {
        throw new Error("The native interaction test did not exercise its template refresh.");
      }
      writeFileSync(
        path.join(output, "refresh-source.json"),
        `${JSON.stringify(
          {
            format: "vize.native-host.template-refresh",
            formatVersion: 1,
            source: application.source,
            compilerSourceVerified: application.compilerSourceVerified,
            compilerSha256: application.compilerSha256,
            compilerBuildToken: application.compilerBuildToken,
            applicationSha256: application.sha256,
            origin,
            templateRerenderObserved: rerenderObserved,
            before: createHash("sha256").update(original).digest("hex"),
            after: createHash("sha256").update(replacement).digest("hex"),
          },
          null,
          2,
        )}\n`,
      );
    },
  };
}

/** Streams native test output so source changes follow an observed native state. */
export async function runNativeRefreshInteractions(
  args: readonly string[],
  output: string,
  fixture: NativeRefreshFixture,
): Promise<void> {
  let transcript = "";
  let capturedBytes = 0;
  let observationFailure: unknown;
  console.log(
    JSON.stringify({
      event: "native-qualification-phase",
      phase: "native-template-interactions",
      state: "started",
    }),
  );
  const result = await new Promise<number | null>((resolve, reject) => {
    const child = spawn("xcodebuild", args, {
      env: { ...process.env, TEST_RUNNER_VIZE_NATIVE_REFRESH_ORIGIN: fixture.origin },
      stdio: ["ignore", "pipe", "pipe"],
    });
    const timeout = setTimeout(() => {
      observationFailure = new Error("Native refresh interaction qualification timed out.");
      child.kill("SIGTERM");
    }, maximumInteractionMilliseconds);
    const observe = (chunk: Buffer) => {
      if (observationFailure) return;
      const text = chunk.toString("utf8");
      capturedBytes += chunk.byteLength;
      try {
        if (capturedBytes > maximumLogBytes) {
          throw new Error("The native interaction log exceeds its capture budget.");
        }
        transcript += text;
        fixture.observe(text);
      } catch (error) {
        observationFailure = error;
        child.kill("SIGTERM");
      }
    };
    child.stdout.on("data", observe);
    child.stderr.on("data", observe);
    child.once("error", (error) => {
      clearTimeout(timeout);
      reject(error);
    });
    child.once("close", (code) => {
      clearTimeout(timeout);
      resolve(code);
    });
  });
  writeFileSync(path.join(output, "interaction-tests.log"), transcript);
  if (observationFailure) throw observationFailure;
  if (result !== 0)
    throw new Error(`Native refresh interactions failed: ${transcript.slice(-12000)}`);
  fixture.finish();
  console.log(
    JSON.stringify({
      event: "native-qualification-phase",
      phase: "native-template-interactions",
      state: "passed",
    }),
  );
}
