import assert from "node:assert/strict";
// Paths are resolved from the package cwd: the runner virtualizes import.meta.url.
import path from "node:path";

import { test } from "vite-plus/test";

import { formatSfcLintResults, lintSfcFiles } from "@vizejs/ui-tooling/lint-sfc";
import type { SfcLintFunction } from "@vizejs/ui-tooling/lint-sfc";

test("discovers every SFC with the opinionated Vize contract", async () => {
  const requests: Parameters<SfcLintFunction>[1][] = [];
  const lint: SfcLintFunction = (_source, options) => {
    requests.push(options);
    return { diagnostics: [] };
  };
  const results = await lintSfcFiles(lint);

  assert.deepEqual(
    results.map((result) => result.filename),
    [
      "src/action-button.vue",
      "src/alert-dialog-content.vue",
      "src/alert.vue",
      "src/announcer-provider.vue",
      "src/aspect-ratio.vue",
      "src/badge.vue",
      "src/checkbox-control.vue",
      "src/cluster.vue",
      "src/collapsible-content.vue",
      "src/collapsible-root.vue",
      "src/collapsible-trigger.vue",
      "src/container.vue",
      "src/deterministic-id-provider.vue",
      "src/dialog-close.vue",
      "src/dialog-content.vue",
      "src/dialog-description.vue",
      "src/dialog-overlay.vue",
      "src/dialog-portal.vue",
      "src/dialog-root.vue",
      "src/dialog-title.vue",
      "src/dialog-trigger.vue",
      "src/error-summary.vue",
      "src/field-description.vue",
      "src/field-error-message.vue",
      "src/field-label.vue",
      "src/field.vue",
      "src/link-anchor.vue",
      "src/live-region.vue",
      "src/locale-provider.vue",
      "src/meter.vue",
      "src/portal.vue",
      "src/positioner-arrow.vue",
      "src/positioner.vue",
      "src/presence.vue",
      "src/primitive-element.vue",
      "src/progress-bar.vue",
      "src/search-field.vue",
      "src/separator.vue",
      "src/skeleton.vue",
      "src/spacer.vue",
      "src/stack.vue",
      "src/switch-control.vue",
      "src/text-input.vue",
      "src/textarea-control.vue",
      "src/toggle-button.vue",
      "src/transition.vue",
      "src/visually-hidden.vue",
    ],
  );
  assert.deepEqual(
    requests,
    [
      "action-button.vue",
      "alert-dialog-content.vue",
      "alert.vue",
      "announcer-provider.vue",
      "aspect-ratio.vue",
      "badge.vue",
      "checkbox-control.vue",
      "cluster.vue",
      "collapsible-content.vue",
      "collapsible-root.vue",
      "collapsible-trigger.vue",
      "container.vue",
      "deterministic-id-provider.vue",
      "dialog-close.vue",
      "dialog-content.vue",
      "dialog-description.vue",
      "dialog-overlay.vue",
      "dialog-portal.vue",
      "dialog-root.vue",
      "dialog-title.vue",
      "dialog-trigger.vue",
      "error-summary.vue",
      "field-description.vue",
      "field-error-message.vue",
      "field-label.vue",
      "field.vue",
      "link-anchor.vue",
      "live-region.vue",
      "locale-provider.vue",
      "meter.vue",
      "portal.vue",
      "positioner-arrow.vue",
      "positioner.vue",
      "presence.vue",
      "primitive-element.vue",
      "progress-bar.vue",
      "search-field.vue",
      "separator.vue",
      "skeleton.vue",
      "spacer.vue",
      "stack.vue",
      "switch-control.vue",
      "text-input.vue",
      "textarea-control.vue",
      "toggle-button.vue",
      "transition.vue",
      "visually-hidden.vue",
    ].map((basename) => ({
      filename: path.resolve("src", basename),
      preset: "opinionated" as const,
      typeAware: true as const,
      helpLevel: "short" as const,
    })),
  );
  assert.equal(formatSfcLintResults(results), "");
});
