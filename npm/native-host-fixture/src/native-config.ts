import { defineApplicationMarquette } from "@vizejs/marquette";
import {
  negotiateAdapterCapabilities,
  type AdapterCapabilityManifest,
} from "@vizejs/marquette/adapter";

/** Actual primitives exercised by the first native application host. */
export const firstHostCapabilities = [
  "native.rendering",
  "native.events",
  "native.text",
  "native.accessibility",
  "native.layout.stack",
] as const;

/** Experimental first-host application contract; it claims no unsupported engine baseline. */
export const nativeApplication = defineApplicationMarquette({
  application: "native-first-host",
  targets: ["native"],
  environments: [{ id: "device", target: "native", consumer: "client", runtime: "native" }],
  capabilities: {
    "native.rendering": {
      id: "native.rendering",
      description: "Install native view-tree snapshots",
    },
    "native.events": { id: "native.events", description: "Deliver native button activations" },
    "native.text": { id: "native.text", description: "Display native labels" },
    "native.accessibility": {
      id: "native.accessibility",
      description: "Expose native label and button semantics",
    },
    "native.layout.stack": {
      id: "native.layout.stack",
      description: "Lay out a vertical native stack with bounded gap and padding",
    },
    "native.images": { id: "native.images", description: "Display native images" },
  },
});

/** Engine support implemented by the first host, rather than the complete native baseline. */
export const firstHostAdapter: AdapterCapabilityManifest = {
  adapter: "native.first-host",
  capabilities: firstHostCapabilities.map((id) => ({ id, minVersion: 1, maxVersion: 1 })),
};

/** Refuses unsupported fixture requirements before any native view can mount. */
export function admitFirstHost(scenario: string) {
  const requirements =
    scenario === "unsupported-capability"
      ? [...firstHostCapabilities, "native.images"]
      : firstHostCapabilities;
  return negotiateAdapterCapabilities(nativeApplication, requirements, firstHostAdapter);
}
