import assert from "node:assert/strict";
import test from "node:test";

import { resolveTokenPreview, type MuseaTokenPreviewConfig } from "./preview.js";
import type { DesignToken } from "./parser.js";

void test("token preview uses referenced token metadata and resolved value", () => {
  const tokenMap: Record<string, DesignToken> = {
    "palette.red.500": { value: "#ef4444", type: "color", $tier: "primitive" },
    "semantic.danger.bg": {
      value: "{palette.red.500}",
      $reference: "palette.red.500",
      $resolvedValue: "#ef4444",
      $tier: "semantic",
    },
  };

  const preview = resolveTokenPreview({
    tokenPath: "semantic.danger.bg",
    token: tokenMap["semantic.danger.bg"]!,
    tokenMap,
  });

  assert.equal(preview.kind, "color");
  assert.equal(preview.value, "#ef4444");
  assert.deepEqual(preview.reference, {
    path: "palette.red.500",
    token: tokenMap["palette.red.500"],
    value: "#ef4444",
  });
});

void test("token preview includes default spacing, opacity, radius, z-index, and letter-spacing previews", () => {
  const cases: Array<[string, DesignToken, string]> = [
    ["spacing.4", { value: "1rem", type: "dimension" }, "spacing"],
    ["opacity.disabled", { value: 0.48 }, "opacity"],
    ["shape.round.full", { value: "9999px", type: "dimension" }, "radius"],
    ["zIndex.modal", { value: 1000 }, "zIndex"],
    ["typography.tracking.tight", { value: "-0.02em", type: "letter-spacing" }, "letterSpacing"],
  ];

  for (const [tokenPath, token, expectedKind] of cases) {
    assert.equal(resolveTokenPreview({ tokenPath, token }).kind, expectedKind);
  }
});

void test("token preview config can override and disable preview kinds", () => {
  const token: DesignToken = { value: 80, type: "number" };
  const config: MuseaTokenPreviewConfig = {
    rules: [{ pathIncludes: "elevation.overlay", kind: "zIndex" }],
  };

  assert.equal(
    resolveTokenPreview({ tokenPath: "elevation.overlay", token, config }).kind,
    "zIndex",
  );
  assert.equal(
    resolveTokenPreview({
      tokenPath: "elevation.overlay",
      token,
      config: { ...config, disabledKinds: ["zIndex"] },
    }).kind,
    "generic",
  );
});

void test("token preview config can match reference paths explicitly", () => {
  const tokenMap: Record<string, DesignToken> = {
    "layers.modal": { value: 1000 },
    "semantic.surface.modal": {
      value: "{layers.modal}",
      $reference: "layers.modal",
      $resolvedValue: 1000,
    },
  };

  const preview = resolveTokenPreview({
    tokenPath: "semantic.surface.modal",
    token: tokenMap["semantic.surface.modal"]!,
    tokenMap,
    config: { rules: [{ referencePathIncludes: "layers", kind: "zIndex" }] },
  });

  assert.equal(preview.kind, "zIndex");
  assert.equal(preview.value, 1000);
});
