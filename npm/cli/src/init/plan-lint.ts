import path from "node:path";

import type { ProjectDetection } from "./detect.js";
import { injectViteLint } from "./edit-config.js";
import { INIT_OXLINT_CONFIG_FILE, type LintTarget } from "./lint-target.js";
import type { PlanDraft } from "./plan-types.js";
import { INIT_OXLINT_CONFIG } from "./templates.js";

/**
 * Plans the Oxlint wiring into whichever file the project's lint command reads.
 *
 * The single rule this function exists to enforce: never write an Oxlint config
 * the project's own lint command ignores. `vp lint` and `vp check` read only the
 * `lint` block of the Vite config; `oxlint` and `oxlint-vize` read only their own
 * config file. Writing the wrong one produces a project that looks configured,
 * reports zero `vize/*` diagnostics and exits `0` -- #3389, fixed in #3407.
 *
 * When the required file cannot be edited safely this returns a `blocked`
 * result and writes nothing at all. Falling back to the *other* file would be
 * the bug: the user would see a success message and get silence from the linter.
 *
 * @returns the possibly-edited Vite config source, or the input unchanged.
 */
export function planLint(
  detection: ProjectDetection,
  lintTarget: LintTarget,
  viteDraft: string | null,
  draft: PlanDraft,
): string | null {
  if (lintTarget.blockedReason !== null) {
    draft.features.push({
      id: "lint",
      outcome: "blocked",
      detail:
        `vp lint reads the \`lint\` block in the Vite config, but ${lintTarget.blockedReason}. ` +
        "Nothing was written: an unconfigured project fails loudly, while an Oxlint config vp " +
        "lint never reads reports zero Vize diagnostics and exits 0",
      snippet: lintTarget.blockedSnippet,
    });
    return viteDraft;
  }

  let source = viteDraft;
  const wrote: string[] = [];
  if (lintTarget.viteConfig !== null && !detection.hasVitePlusLintBlock && source !== null) {
    const injected = injectViteLint(source);
    if (injected !== null) {
      source = injected;
      wrote.push(lintTarget.viteConfig);
    }
  }
  if (lintTarget.oxlintConfig !== null) {
    draft.files.push({
      filename: path.join(detection.root, INIT_OXLINT_CONFIG_FILE),
      source: INIT_OXLINT_CONFIG,
    });
    draft.createdFiles.push(INIT_OXLINT_CONFIG_FILE);
    wrote.push(INIT_OXLINT_CONFIG_FILE);
  }
  draft.features.push({
    id: "lint",
    outcome: wrote.length > 0 ? "configured" : "unchanged",
    detail:
      wrote.length > 0 ? `${lintTarget.reason}; writes ${wrote.join(" and ")}` : lintTarget.reason,
    snippet: null,
  });
  return source;
}
