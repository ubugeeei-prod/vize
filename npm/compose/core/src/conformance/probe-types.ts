/**
 * Renderer conformance probe contract.
 *
 * Test-only module: every catalog entry of `@vizejs/composable` has one probe.
 * `scripts/check-conformance.ts` turns each probe into a `<script setup>` SFC,
 * compiles it through the SSR, DOM, and Vapor lanes of `@vizejs/native`, and
 * verifies server rendering, hydration, and Vapor execution.
 */

/** Serializable JSON value used for expected probe states. */
export type ProbeJson =
  | string
  | number
  | boolean
  | null
  | readonly ProbeJson[]
  | { readonly [key: string]: ProbeJson };

/** A probe that mounts a component exercising the entry. */
export interface ComponentProbe {
  /** Discriminant. */
  readonly kind: "component";

  /**
   * `<script setup lang="ts">` body. Import package modules through the `@/`
   * alias, for example `import { useToggle } from "@/use-toggle.ts";`.
   */
  readonly setup: string;

  /**
   * Template expression producing the probe state. It is serialized with
   * `serializeProbeState` into the `data-state` attribute of the probe root.
   */
  readonly state: string;

  /**
   * Extra template markup rendered inside the probe root, for example
   * elements bound to template refs.
   *
   * @default ""
   */
  readonly markup?: string;

  /**
   * Exact state expected from server rendering.
   *
   * @default not asserted
   */
  readonly server?: ProbeJson;

  /**
   * Exact state expected once the client has hydrated (or Vapor has
   * mounted) and effects have flushed, proving client activation.
   *
   * @default not asserted
   */
  readonly client?: ProbeJson;

  /**
   * Reason why the Vapor lane cannot execute this probe. The probe is still
   * compiled through the Vapor lane.
   *
   * @default undefined (Vapor executes the probe)
   */
  readonly vaporSkip?: string;
}

/** An entry with no runtime behavior to mount (types only or pure metadata). */
export interface ExemptProbe {
  /** Discriminant. */
  readonly kind: "exempt";

  /** Why the entry has nothing to mount. */
  readonly reason: string;
}

/** Conformance probe for one catalog entry. */
export type ComposableProbe = ComponentProbe | ExemptProbe;

/** Probes keyed by catalog subpath (for example `"./use-toggle"`). */
export type ComposableProbeMap = Readonly<Record<`./${string}`, ComposableProbe>>;
