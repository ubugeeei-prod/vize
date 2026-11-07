/** Fixture contracts for the generated LSP scaling projects. */

import {
  FOCUS_SCRIPT_PROBE_AFTER,
  FOCUS_TEMPLATE_PROBE_AFTER,
  focusRequests,
} from "./lsp-vs-volar-scale-core.ts";
import type { FixtureSpec } from "./lsp-vs-volar-fixtures.ts";
import {
  MONOREPO_APPS,
  MONOREPO_DOMAINS,
  giantSfcProject,
  monorepoProject,
  scaleProject,
} from "./lsp-vs-volar-scale.ts";

const two = (index: number) => String(index).padStart(2, "0");

function scaleSpec(sfcs: number): FixtureSpec {
  return {
    id: `scale-${sfcs / 1000}k`,
    file: "src/core/OrderBoard.vue",
    tsconfigs: ["tsconfig.json"],
    source: { kind: "generated", files: (vue) => scaleProject(sfcs, vue) },
    curve: { name: "scale", sfcs },
    scriptProbeAfter: FOCUS_SCRIPT_PROBE_AFTER,
    templateProbeAfter: FOCUS_TEMPLATE_PROBE_AFTER,
    templateIndent: "    ",
    requests: focusRequests({ chain: "src/core/chain", model: "src/core/model" }),
  };
}

/** Members holding `.vue`, with the edited SFC's app first. */
const MONOREPO_VUE_PROJECTS = [
  ...Array.from({ length: MONOREPO_APPS }, (_, app) => `apps/app-${app}`),
  "packages/ui-kit",
  "packages/board",
  ...Array.from({ length: MONOREPO_DOMAINS }, (_, index) => `packages/domain-${two(index)}`),
];

export const SCALE_FIXTURES: FixtureSpec[] = [
  scaleSpec(1000),
  scaleSpec(5000),
  scaleSpec(20000),
  {
    id: "scale-giant-sfc",
    file: "src/core/OrderBoard.vue",
    tsconfigs: ["tsconfig.json"],
    source: { kind: "generated", files: (vue) => giantSfcProject(vue) },
    scriptProbeAfter: FOCUS_SCRIPT_PROBE_AFTER,
    templateProbeAfter: FOCUS_TEMPLATE_PROBE_AFTER,
    templateIndent: "    ",
    requests: focusRequests({ chain: "src/core/chain", model: "src/core/model" }),
  },
  {
    id: "scale-monorepo",
    file: "apps/app-0/src/core/OrderBoard.vue",
    // Every project holding .vue declares the mapper, as the content-mapper guide does for references.
    tsconfigs: MONOREPO_VUE_PROJECTS.map((dir) => `${dir}/tsconfig.json`),
    source: { kind: "generated", files: monorepoProject },
    scriptProbeAfter: FOCUS_SCRIPT_PROBE_AFTER,
    templateProbeAfter: FOCUS_TEMPLATE_PROBE_AFTER,
    templateIndent: "    ",
    requests: focusRequests({ chain: "packages/board/src", model: "packages/model/src" }),
  },
];
