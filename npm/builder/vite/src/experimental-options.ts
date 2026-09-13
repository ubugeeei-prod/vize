/**
 * Switch shape for experimental Vue compiler features.
 *
 * Missing keys, `false`, and `null` are disabled. `true` and object values are
 * enabled; object values are accepted so future per-feature options can be
 * added without changing the top-level config shape.
 */
export type ExperimentalSwitch = boolean | null | Record<string, unknown>;

/**
 * Fully opt-in Vue RFC and backend experiments.
 *
 * Direct `vize({ experimentals })` values take precedence over shared
 * `vize.config.*` values. Passing `false` or `null` directly disables a shared
 * opt-in for that plugin invocation. Prefer stable `compiler` options when a
 * backend choice is no longer experimental for the project.
 */
export interface ExperimentalOptions {
  /**
   * Enable experimental SFC Vapor backend routing unless stable
   * `compiler.vapor` or direct `vize({ vapor })` is set.
   */
  vapor?: ExperimentalSwitch;
  /**
   * Default JSX/TSX output to Vapor unless `compiler.jsxMode` or direct
   * `jsxMode` is set.
   */
  jsxVapor?: ExperimentalSwitch;
  /** Compatibility alias for `inTagComment`; new TS configs should use `inTagComment`. */
  intagComment?: ExperimentalSwitch;
  /** Vue RFC #831: parse compile-time-only `//` comments inside start tags. */
  inTagComment?: ExperimentalSwitch;
  /** Legacy typo alias for `patternedTemplate`; do not set both names. */
  pattenedTemplate?: ExperimentalSwitch;
  /** Vue RFC #823: enable `v-match` containers and direct `v-when` pattern branches. */
  patternedTemplate?: ExperimentalSwitch;
  /** Vue RFC #833: reserve `<Self>` for recursive component resolution. */
  selfComponent?: ExperimentalSwitch;
  /** Vue RFC #734: enable virtual-TypeScript checks for strict slot child contracts. */
  strictSlotChildren?: ExperimentalSwitch;
  /** Enable the reserved server-script compiler experiment for hosts that test it. */
  serverScript?: ExperimentalSwitch;
  /** Compatibility alias for `serverScript`. */
  "server script"?: ExperimentalSwitch;
}

/** Native compiler flags resolved from `experimentals` and direct plugin options. */
export interface ExperimentalCompileFlags {
  experimentalInTagComments?: boolean;
  experimentalPatternedTemplate?: boolean;
  experimentalSelfComponent?: boolean;
  experimentalStrictSlotChildren?: boolean;
  experimentalServerScript?: boolean;
}

export interface ExperimentalPluginOptions extends ExperimentalCompileFlags {
  /** Experimental RFC opt-ins. Values other than `false` or `null` enable keys. */
  experimentals?: ExperimentalOptions;
}
