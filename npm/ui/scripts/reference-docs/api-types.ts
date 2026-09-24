/** Shapes produced by the reference-doc extractor. */

export interface ApiMember {
  readonly name: string;
  /** Type text as written in source (payload tuple for emits, props type for slots). */
  readonly type: string;
  readonly optional: boolean;
  readonly description: string;
  readonly defaultValue: string | null;
  readonly deprecated: boolean;
}

/** Public contract of one SFC. */
export interface ComponentApi {
  readonly file: string;
  readonly generic: string | null;
  readonly props: readonly ApiMember[];
  readonly emits: readonly ApiMember[];
  readonly slots: readonly ApiMember[];
  readonly expose: readonly ApiMember[];
}

/** One exported function or constant of a TypeScript module. */
export interface ModuleExport {
  readonly name: string;
  readonly signature: string;
  readonly description: string;
  /** Bodies of `@example` tags. */
  readonly examples: readonly string[];
}

/** One exported interface of a TypeScript module. */
export interface ModuleInterface {
  readonly name: string;
  readonly description: string;
  readonly members: readonly ApiMember[];
}
