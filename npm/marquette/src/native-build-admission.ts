import type {
  AdapterCapabilityDiagnostic,
  AdapterCapabilityManifest,
  AdapterCapabilityMismatch,
} from "./adapter-model.js";
import { negotiateAdapterCapabilities } from "./adapter-negotiate.js";
import { parseAdapterCapabilityManifest } from "./adapter-validate.js";
import type { ApplicationMarquette } from "./model.js";
import type { NativeBuildProfile, NativeBuildProfileDiagnostic } from "./native-build-model.js";
import { parseNativeBuildProfile, validateNativeBuildProfile } from "./native-build-validate.js";
import { validateApplicationMarquette, type MarquetteDiagnostic } from "./validate.js";

/** Application-to-build binding failure that must prevent executor scheduling. */
export interface NativeBuildBindingDiagnostic {
  /** Stable machine-readable binding refusal. */
  readonly code: "unknown-environment" | "environment-not-native" | "environment-not-client";
  /** Location of the profile or application value that prevents admission. */
  readonly path: string;
  /** Human-readable explanation shared by local and hosted executors. */
  readonly message: string;
}

/** A source-qualified finding retained from every native build admission layer. */
export type NativeBuildAdmissionDiagnostic =
  | (MarquetteDiagnostic & {
      /** Contract layer that owns this finding. */
      readonly source: "application";
    })
  | (NativeBuildProfileDiagnostic & {
      /** Contract layer that owns this finding. */
      readonly source: "profile";
      /** Build refusal severity. */
      readonly severity: "error";
    })
  | (NativeBuildBindingDiagnostic & {
      /** Contract layer that owns this finding. */
      readonly source: "binding";
      /** Build refusal severity. */
      readonly severity: "error";
    })
  | (AdapterCapabilityDiagnostic & {
      /** Contract layer that owns this finding. */
      readonly source: "adapter";
      /** Build refusal severity. */
      readonly severity: "error";
    })
  | (AdapterCapabilityMismatch & {
      /** Contract layer that owns this finding. */
      readonly source: "capability";
      /** Build refusal severity. */
      readonly severity: "error";
    });

/** Deterministic decision computed before a local or hosted native build runs. */
export interface NativeBuildAdmission {
  /** Whether every contract, environment binding, and capability requirement passed. */
  readonly allowed: boolean;
  /** Profile whose configuration was admitted or refused. */
  readonly profile: string;
  /** Application environment selected by that profile. */
  readonly environment: string;
  /** Adapter whose capability ranges were checked. */
  readonly adapter: string;
  /** Sorted union of profile, environment, and selected-route capability requirements. */
  readonly capabilities: readonly string[];
  /** Source-qualified findings in stable source, path, code, and message order. */
  readonly diagnostics: readonly NativeBuildAdmissionDiagnostic[];
}

/**
 * Binds a native build profile to an application and a concrete adapter manifest.
 *
 * Standalone profile validation cannot establish that the selected environment
 * exists, is a native client, or that its routes can execute with the adapter.
 * Admission checks all three contracts and negotiates the deduplicated union of
 * profile, environment, and selected-route capabilities. Invalid application,
 * profile, or adapter contracts and unsupported capability versions fail closed.
 * Strict profile and adapter parsing runs first: malformed fields, unknown
 * executor or secret fields, and unsafe numeric bounds throw `TypeError` before
 * graph inspection. Semantically invalid contracts return `allowed: false`.
 *
 * This experimental synchronous decision neither schedules a build nor handles
 * secrets. Executors must inspect `allowed` immediately before scheduling, using
 * the same inputs they pass to the build. Application warnings remain visible
 * without becoming errors. Inputs are not mutated; cost is linear in contract
 * collections plus sorting their diagnostics and required capability identifiers.
 */
export function admitNativeBuildProfile(
  marquette: ApplicationMarquette,
  profile: NativeBuildProfile,
  adapter: AdapterCapabilityManifest,
): NativeBuildAdmission {
  parseNativeBuildProfile(profile);
  parseAdapterCapabilityManifest(adapter);
  const environment = marquette.environments?.find(({ id }) => id === profile.environment);
  const capabilities = [
    ...new Set([
      ...(profile.capabilities ?? []),
      ...(environment?.capabilities ?? []),
      ...(marquette.routes ?? [])
        .filter((route) => route.environment === profile.environment)
        .flatMap((route) => route.capabilities ?? []),
    ]),
  ].sort();
  const negotiation = negotiateAdapterCapabilities(marquette, capabilities, adapter);
  const diagnostics: NativeBuildAdmissionDiagnostic[] = [
    ...validateApplicationMarquette(marquette).map((diagnostic) => ({
      ...diagnostic,
      source: "application" as const,
    })),
    ...validateNativeBuildProfile(profile).map((diagnostic) => ({
      ...diagnostic,
      source: "profile" as const,
      severity: "error" as const,
    })),
    ...bindingDiagnostics(environment, profile.environment).map((diagnostic) => ({
      ...diagnostic,
      source: "binding" as const,
      severity: "error" as const,
    })),
    ...negotiation.diagnostics.map((diagnostic) => ({
      ...diagnostic,
      source: "adapter" as const,
      severity: "error" as const,
    })),
    ...negotiation.mismatches.map((diagnostic) => ({
      ...diagnostic,
      source: "capability" as const,
      severity: "error" as const,
    })),
  ];
  diagnostics.sort(
    (left, right) =>
      compareText(left.source, right.source) ||
      compareText(left.path, right.path) ||
      compareText(left.code, right.code) ||
      compareText(left.message, right.message),
  );
  return {
    allowed: negotiation.compatible && !diagnostics.some(({ severity }) => severity === "error"),
    profile: profile.profile,
    environment: profile.environment,
    adapter: adapter.adapter,
    capabilities,
    diagnostics,
  };
}

function bindingDiagnostics(
  environment: { readonly target: string; readonly consumer: string } | undefined,
  id: string,
): NativeBuildBindingDiagnostic[] {
  if (environment === undefined) {
    return [
      {
        code: "unknown-environment",
        path: "environment",
        message: "native build environment is not declared by the application",
      },
    ];
  }
  const diagnostics: NativeBuildBindingDiagnostic[] = [];
  if (environment.target !== "native") {
    diagnostics.push({
      code: "environment-not-native",
      path: `environments.${id}.target`,
      message: "native builds require an environment with the native target",
    });
  }
  if (environment.consumer !== "client") {
    diagnostics.push({
      code: "environment-not-client",
      path: `environments.${id}.consumer`,
      message: "native builds require a client environment",
    });
  }
  return diagnostics;
}

function compareText(left: string, right: string): number {
  return left < right ? -1 : left > right ? 1 : 0;
}
