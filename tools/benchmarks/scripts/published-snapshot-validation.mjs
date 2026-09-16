import { rerenderData } from "./render-results.mjs";

const REQUIRED_SURFACES = [
  "compile",
  "large-compile",
  "large-check",
  "lint",
  "fmt",
  "check",
  "vite",
  "nuxt",
];
const HASH = /^[a-f0-9]{64}$/u;
const PLANTS = ["script", "templateProp", "templateEvent", "componentProp"];

function requireValue(condition, message) {
  if (!condition) throw new Error(`benchmark publication: ${message}`);
}

function median(runs) {
  const sorted = [...runs].sort((a, b) => a - b);
  const middle = Math.floor(sorted.length / 2);
  return Number(
    (sorted.length % 2 ? sorted[middle] : (sorted[middle - 1] + sorted[middle]) / 2).toFixed(3),
  );
}

/** Validate recorded work and provenance without manufacturing replacement measurements. */
export function validatePublishedSnapshot(input) {
  requireValue(
    input?.schemaVersion === 1 && input.kind === "tool-comparison",
    "unsupported artifact",
  );
  requireValue(Number.isFinite(Date.parse(input.generatedAt)), "missing measurement date");
  requireValue(/^[a-f0-9]{40}$/u.test(input.commit?.sha ?? ""), "missing commit SHA");
  requireValue(input.commit?.repository === "ubugeeei-prod/vize", "unexpected source repository");
  requireValue(
    /^https:\/\/github\.com\/ubugeeei-prod\/vize\/actions\/runs\/\d+$/u.test(
      input.commit?.runUrl ?? "",
    ),
    "missing Actions provenance",
  );
  requireValue(
    input.runner?.platform === "linux" &&
      ["blacksmith-32vcpu-ubuntu-2404", "ubuntu-24.04"].includes(input.runner?.label),
    "not a reference-runner measurement",
  );
  requireValue(
    input.backend?.ready === true && input.backend.engine === "tsgo-native",
    "native backend was not ready",
  );
  for (const binary of ["vize", "tsgo", "vueTsc", "verterTsc"]) {
    requireValue(
      typeof input.versions?.[binary] === "string" && input.versions[binary].trim(),
      `missing ${binary} version`,
    );
    requireValue(HASH.test(input.binaries?.[binary] ?? ""), `missing ${binary} binary hash`);
  }
  requireValue(
    Number.isInteger(input.settings?.runs) && input.settings.runs >= 3,
    "at least three measured runs are required",
  );
  requireValue(
    Number.isInteger(input.settings?.warmups) && input.settings.warmups >= 1,
    "missing warmup",
  );
  requireValue(Array.isArray(input.surfaces), "missing surfaces");
  requireValue(
    new Set(input.surfaces.map((s) => s.id)).size === input.surfaces.length,
    "duplicate surface",
  );
  for (const id of REQUIRED_SURFACES)
    requireValue(
      input.surfaces.some((s) => s.id === id),
      `missing ${id} surface`,
    );
  const data = rerenderData(input);
  for (const surface of data.surfaces) {
    requireValue(
      Number.isInteger(surface.files) && surface.files > 0,
      `invalid ${surface.id} corpus size`,
    );
    requireValue(
      new Set(surface.variants.map((v) => v.id)).size === surface.variants.length,
      `duplicate ${surface.id} variant`,
    );
    requireValue(
      surface.variants.some((v) => v.id === surface.vizeMaxId),
      `missing ${surface.id} Vize lane`,
    );
    if (REQUIRED_SURFACES.includes(surface.id)) {
      requireValue(
        surface.primarySpeedup > 0 && Number.isFinite(surface.primarySpeedup),
        `missing ${surface.id} measured baseline`,
      );
    }
    for (const variant of surface.variants) {
      requireValue(
        Array.isArray(variant.runs) &&
          variant.runs.length === data.settings.runs &&
          variant.runs.every((ms) => Number.isFinite(ms) && ms > 0),
        `invalid ${surface.id}/${variant.id} samples`,
      );
      requireValue(
        variant.medianMs === median(variant.runs),
        `inconsistent ${surface.id}/${variant.id} median`,
      );
      if (!["check", "large-check"].includes(surface.id)) continue;
      const proof = variant.correctness;
      requireValue(
        proof?.corpusPlant === true &&
          proof.strictTemplates === true &&
          JSON.stringify(proof.minimalPlants) === JSON.stringify(PLANTS),
        `unverified ${surface.id}/${variant.id} diagnostic work`,
      );
      requireValue(
        HASH.test(proof.diagnosticFingerprint ?? "") &&
          HASH.test(proof.corpusBaselineFingerprint ?? ""),
        `missing ${surface.id}/${variant.id} diagnostic identity`,
      );
      requireValue(
        [0, 1, 2].includes(proof.status) &&
          Number.isInteger(proof.diagnosticCount) &&
          proof.diagnosticCount >= 0,
        `invalid ${surface.id}/${variant.id} diagnostic result`,
      );
    }
  }
  return data;
}

/** Freshness applies to imports, not to reproducible rendering of a historical snapshot. */
export function assertFreshSnapshot(data, previous, now = Date.now()) {
  const measuredAt = Date.parse(data.generatedAt);
  requireValue(
    now - measuredAt <= 7 * 24 * 60 * 60 * 1000 && measuredAt <= now + 60_000,
    "import requires a measurement from the last seven days",
  );
  requireValue(
    !previous || measuredAt >= Date.parse(previous.generatedAt),
    "refusing an older measurement than the published snapshot",
  );
}
