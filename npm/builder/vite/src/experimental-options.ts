/**
 * Switch shape for experimental Vue compiler features.
 *
 * Missing keys, `false`, and `null` are disabled. `true` and object values are
 * enabled; object values are accepted so future per-feature options can be
 * added without changing the top-level config shape. Prefer `{}` over an
 * arbitrary object until a specific feature documents nested options.
 */
export type ExperimentalSwitch = boolean | null | Record<string, unknown>;

/**
 * Fully opt-in Vue RFC and backend experiments.
 *
 * Direct `vize({ experimentals })` values take precedence over shared
 * `vize.config.*` values. Passing `false` or `null` directly disables a shared
 * opt-in for that plugin invocation. Prefer stable `compiler` options when a
 * backend choice is no longer experimental for the project. See
 * https://vizejs.dev/guide/experimentals for examples, aliases, and the full
 * surface matrix.
 */
export interface ExperimentalOptions {
  /**
   * Enable experimental SFC Vapor backend routing unless stable
   * `compiler.vapor` or direct `vize({ vapor })` is set. Use this for trials;
   * use the stable `compiler` or direct plugin option for an intentional
   * project-wide backend choice.
   */
  vapor?: ExperimentalSwitch;
  /**
   * Default JSX/TSX output to Vapor unless `compiler.jsxMode` or direct
   * `jsxMode` is set. Per-file `"use vue:vapor"` and `"use vue:vdom"`
   * directives still describe file-local intent.
   */
  jsxVapor?: ExperimentalSwitch;
  /** Compatibility alias for `inTagComment`; new configs should use `inTagComment`. */
  intagComment?: ExperimentalSwitch;
  /**
   * Vue RFC #831: parse compile-time-only `//` comments inside start tags.
   * These comments are kept for tooling/source mapping and produce no runtime
   * output.
   */
  inTagComment?: ExperimentalSwitch;
  /** Legacy typo alias for `patternedTemplate`; do not set both names. */
  pattenedTemplate?: ExperimentalSwitch;
  /**
   * Vue RFC #823: enable `v-match` containers and direct `v-when` pattern
   * branches. Vize lowers them to ordinary conditional render code and reports
   * invalid placements instead of passing unknown directives through.
   */
  patternedTemplate?: ExperimentalSwitch;
  /**
   * Vue RFC #833: reserve exact `<Self>` for recursive component resolution in
   * DOM, SSR, and Vapor compilation.
   */
  selfComponent?: ExperimentalSwitch;
  /**
   * Vue RFC #734: enable virtual-TypeScript checks for strict slot child
   * contracts. This affects `vize check`/LSP virtual code, not runtime
   * compilation.
   */
  strictSlotChildren?: ExperimentalSwitch;
  /**
   * Enable the reserved server-script compiler experiment for hosts that test
   * it. Some runtimes intentionally keep the switch reserved until their
   * compiler stage owns the experiment.
   */
  serverScript?: ExperimentalSwitch;
  /** Compatibility alias for `serverScript`. */
  "server script"?: ExperimentalSwitch;
}

/**
 * Native compiler flags resolved from `experimentals` and direct plugin
 * options. These are already booleans: they do not understand aliases, switch
 * objects, or shared-config precedence.
 */
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
