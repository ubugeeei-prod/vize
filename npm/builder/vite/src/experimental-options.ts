export type ExperimentalSwitch = boolean | null | Record<string, unknown>;

export interface ExperimentalOptions {
  vapor?: ExperimentalSwitch;
  jsxVapor?: ExperimentalSwitch;
  intagComment?: ExperimentalSwitch;
  inTagComment?: ExperimentalSwitch;
  pattenedTemplate?: ExperimentalSwitch;
  patternedTemplate?: ExperimentalSwitch;
  selfComponent?: ExperimentalSwitch;
  strictSlotChildren?: ExperimentalSwitch;
  serverScript?: ExperimentalSwitch;
  "server script"?: ExperimentalSwitch;
}

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
