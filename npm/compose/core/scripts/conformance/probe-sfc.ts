/** Probe SFC generation shared by the conformance lanes. */
import type { ComponentProbe } from "../../src/conformance/probe-types.ts";

/** One probe handed to the Vapor lane. */
export interface VaporLaneJob {
  /** Catalog subpath. */
  readonly subpath: string;
  /** Compiled Vapor module path. */
  readonly module: string;
  /** Server-rendered HTML to hydrate. */
  readonly html: string;
  /** Expected client state, or `null` when not asserted. */
  readonly client: unknown;
}

/** Outcome of one Vapor lane job. */
export interface VaporLaneResult {
  /** Catalog subpath. */
  readonly subpath: string;
  /** Problems found; empty when the probe passed. */
  readonly problems: readonly string[];
}

/** Stable SFC filename for a catalog subpath. */
export function probeFileName(subpath: string): string {
  const base = subpath
    .slice(2)
    .split("-")
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join("");
  return `${base}Probe.vue`;
}

/** Build the probe SFC source; `@/` imports resolve to `sourceRoot`. */
export function buildProbeSfc(probe: ComponentProbe, sourceRoot: string): string {
  const setup = probe.setup.replaceAll('"@/', `"${sourceRoot}/`);
  return [
    '<script setup lang="ts">',
    `import { serializeProbeState as probeState } from "${sourceRoot}/conformance/serialize.ts";`,
    setup,
    "</script>",
    "",
    "<template>",
    // The state is rendered as text: Vue reports text mismatches during
    // hydration, but it does not compare `data-*` attributes.
    `  <div data-conformance-probe><output>{{ probeState(${probe.state}) }}</output>${probe.markup ?? ""}</div>`,
    "</template>",
    "",
  ].join("\n");
}

/** Read the serialized probe state from rendered HTML. */
export function readProbeState(html: string): unknown {
  const match = /<output>([^<]*)<\/output>/.exec(html);
  if (match?.[1] === undefined) return undefined;
  return JSON.parse(
    match[1]
      .replaceAll("&quot;", '"')
      .replaceAll("&#39;", "'")
      .replaceAll("&lt;", "<")
      .replaceAll("&gt;", ">")
      .replaceAll("&amp;", "&"),
  );
}
