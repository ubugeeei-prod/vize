/**
 * Vapor lane (child process): hydrate the SSR output of every runtime
 * fixture with Vue 3.6, where each `.vue` component is compiled by the
 * native Vapor lane and mounted through `vaporInteropPlugin` inside the
 * fixture's VDOM root. Warnings, errors, root replacement, or a failing
 * `assertHydratedDom` fail the fixture.
 */
import { readFileSync, writeFileSync } from "node:fs";

import { installDomGlobals } from "./dom-env.ts";
import { compileFailures, registerSfcHooks } from "./sfc-hooks.ts";

interface SsrRecord {
  readonly name: string;
  readonly html: string | null;
}

const records = JSON.parse(readFileSync(process.argv[2] ?? "", "utf8")) as readonly SsrRecord[];
/** `hydrate` adopts the server markup; `mount` renders from scratch. */
const mode = process.argv[3] === "mount" ? "mount" : "hydrate";
installDomGlobals();
registerSfcHooks("vapor");
const vue = await import("vue");
const { loadRuntimeFixtures } = await import("./fixtures.ts");
// `VIZE_VAPOR_FIXTURES=a,b` narrows the lane to named fixtures while debugging.
const only = process.env.VIZE_VAPOR_FIXTURES?.split(",").filter(Boolean);
const allFixtures = await loadRuntimeFixtures();
if (
  only !== undefined &&
  (only.length === 0 || only.some((name) => !allFixtures.some((fixture) => fixture.name === name)))
) {
  throw new Error("VIZE_VAPOR_FIXTURES must name existing fixtures");
}
const fixtures = allFixtures.filter((fixture) => only === undefined || only.includes(fixture.name));

function describe(error: unknown): string {
  return error instanceof Error ? (error.stack ?? error.message) : String(error);
}

async function settle(): Promise<void> {
  for (let index = 0; index < 5; index += 1) await Promise.resolve();
  await new Promise((resolve) => setTimeout(resolve, 10));
}

// Errors thrown from scheduled effects escape `mount()`; attribute them to
// the fixture that is currently hydrating instead of crashing the lane.
let asyncProblems: string[] = [];
process.on("uncaughtException", (error) => {
  asyncProblems.push(`uncaught: ${describe(error)}`);
});
process.on("unhandledRejection", (reason) => {
  asyncProblems.push(`unhandled rejection: ${describe(reason)}`);
});

// Vue 3.6 keeps the pending insertion parent in module state and clears it
// only when the next block locates its hydration node, so a fixture that
// fails mid-hydration would leak it into the next fixture. Each fixture starts
// from a cleared state, as a fresh page would.
const setInsertionState: unknown = Reflect.get(vue, "setInsertionState");
function clearInsertionState(): void {
  if (typeof setInsertionState === "function") setInsertionState(null);
}

const results: { name: string; problems: string[] }[] = [];
for (const fixture of fixtures) {
  clearInsertionState();
  const problems: string[] = [];
  asyncProblems = problems;
  const html = records.find((record) => record.name === fixture.name)?.html;
  if (typeof html !== "string") {
    results.push({ name: fixture.name, problems: ["no server markup to hydrate"] });
    continue;
  }
  const host = document.createElement("div");
  if (mode === "hydrate") host.innerHTML = html;
  document.body.append(host);
  const serverRoot = host.firstElementChild;
  const messages: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...values: unknown[]) => messages.push(values.map(String).join(" "));
  console.error = (...values: unknown[]) => messages.push(values.map(String).join(" "));
  const root = vue.defineComponent({
    name: `VaporHydrate${fixture.name}`,
    setup: () => fixture.render,
  });
  const app = mode === "hydrate" ? vue.createSSRApp(root) : vue.createApp(root);
  app.config.errorHandler = (error, _instance, info) => {
    problems.push(`${info}: ${describe(error)}`);
  };
  let mounted = false;
  try {
    app.use(vue.vaporInteropPlugin);
    app.mount(host);
    mounted = true;
    await settle();
    if (mode === "hydrate" && host.firstElementChild !== serverRoot) {
      problems.push("hydration replaced the server-rendered root");
    }
    fixture.assertHydratedDom(host);
  } catch (error) {
    problems.push(describe(error));
  } finally {
    console.warn = originalWarn;
    console.error = originalError;
    try {
      if (mounted) app.unmount();
    } catch (error) {
      problems.push(`unmount failed: ${describe(error)}`);
    }
    host.remove();
  }
  if (messages.length > 0) problems.unshift(...messages);
  results.push({ name: fixture.name, problems });
}
// Large payloads are written to a file: `process.exit` can truncate piped stdout.
writeFileSync(process.argv[4] ?? "", JSON.stringify({ results, compileFailures }));
process.exit(0);
