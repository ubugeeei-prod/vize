/**
 * SSR lane (child process): render every runtime fixture with Vue 3.5's
 * server renderer, using components compiled by the native SSR lane, and
 * write `{ name, sourceFile, html }` records as JSON to the path in argv[2].
 */
import { writeFileSync } from "node:fs";

import { installDomGlobals, withoutBrowserGlobals } from "./dom-env.ts";
import { compileFailures, registerSfcHooks } from "./sfc-hooks.ts";

// Fixture modules may reference DOM constructors in their assertions, so the
// globals exist while modules load; every render below runs without them.
installDomGlobals();
registerSfcHooks("ssr");
const { loadRuntimeFixtures } = await import("./fixtures.ts");
const { createSSRApp, defineComponent } = await import("vue");
const { renderToString } = await import("vue/server-renderer");

const records: { name: string; sourceFile: string; html: string | null; error: string | null }[] =
  [];
for (const fixture of await loadRuntimeFixtures()) {
  try {
    const root = defineComponent({ name: `VaporSsr${fixture.name}`, setup: () => fixture.render });
    records.push({
      name: fixture.name,
      sourceFile: fixture.sourceFile,
      html: await withoutBrowserGlobals(() => renderToString(createSSRApp(root))),
      error: null,
    });
  } catch (error) {
    records.push({
      name: fixture.name,
      sourceFile: fixture.sourceFile,
      html: null,
      error: error instanceof Error ? (error.stack ?? error.message) : String(error),
    });
  }
}
// Large payloads are written to a file: `process.exit` can truncate piped stdout.
writeFileSync(process.argv[2] ?? "", JSON.stringify({ records, compileFailures }));
process.exit(0);
