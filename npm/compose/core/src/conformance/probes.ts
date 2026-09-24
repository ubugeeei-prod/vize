import { probesGroupA } from "./probes-a.ts";
import { probesGroupB } from "./probes-b.ts";
import { probesGroupC } from "./probes-c.ts";
import { probesGroupD } from "./probes-d.ts";
import { probesGroupE } from "./probes-e.ts";
import { probesGroupF } from "./probes-f.ts";
import type { ComposableProbeMap } from "./probe-types.ts";

/** Conformance probes for every catalog entry, merged from the group files (test-only). */
export const composableProbes: ComposableProbeMap = {
  ...probesGroupA,
  ...probesGroupB,
  ...probesGroupC,
  ...probesGroupD,
  ...probesGroupE,
  ...probesGroupF,
};
