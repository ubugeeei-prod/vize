import type {
  AdapterCapabilityCompatibilityChange,
  AdapterCapabilityCompatibilityReport,
  AdapterCapabilityManifest,
  AdapterCapabilitySupport,
  CompatibilityChangeKind,
} from "./adapter-model.js";

/**
 * Compares adapter capability support from older to newer.
 *
 * Adding support or widening an inclusive version range is additive.
 * Removing support or narrowing either bound is breaking.
 */
export function compareAdapterCapabilities(
  previous: AdapterCapabilityManifest,
  next: AdapterCapabilityManifest,
): AdapterCapabilityCompatibilityReport {
  const changes: AdapterCapabilityCompatibilityChange[] = [];
  if (previous.adapter !== next.adapter) {
    changes.push(change("breaking", "adapter", "adapter identity changed"));
  }

  const oldSupport = byId(previous);
  const newSupport = byId(next);
  for (const id of [...oldSupport.keys()].filter((id) => !newSupport.has(id)).sort()) {
    changes.push(change("breaking", `capabilities.${id}`, "capability support was removed"));
  }
  for (const id of [...newSupport.keys()].filter((id) => !oldSupport.has(id)).sort()) {
    changes.push(change("additive", `capabilities.${id}`, "capability support was added"));
  }
  for (const [id, old] of oldSupport) {
    const current = newSupport.get(id);
    if (current === undefined) continue;
    if (old.minVersion !== current.minVersion) {
      changes.push(
        change(
          current.minVersion < old.minVersion ? "additive" : "breaking",
          `capabilities.${id}.minVersion`,
          current.minVersion < old.minVersion
            ? "minimum supported version decreased"
            : "minimum supported version increased",
        ),
      );
    }
    if (old.maxVersion !== current.maxVersion) {
      changes.push(
        change(
          current.maxVersion > old.maxVersion ? "additive" : "breaking",
          `capabilities.${id}.maxVersion`,
          current.maxVersion > old.maxVersion
            ? "maximum supported version increased"
            : "maximum supported version decreased",
        ),
      );
    }
  }

  changes.sort(
    (left, right) => compareText(left.path, right.path) || compareText(left.kind, right.kind),
  );
  return { changes };
}

function byId(manifest: AdapterCapabilityManifest): Map<string, AdapterCapabilitySupport> {
  return new Map((manifest.capabilities ?? []).map((value) => [value.id, value] as const));
}

function change(
  kind: CompatibilityChangeKind,
  path: string,
  message: string,
): AdapterCapabilityCompatibilityChange {
  return { kind, path, message };
}

function compareText(left: string, right: string): number {
  return left < right ? -1 : left > right ? 1 : 0;
}
