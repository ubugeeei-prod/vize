/** One native-reported import binding that can participate in HMR snapshots. */
export type ModuleImportInfo = {
  /** Module specifier used by the emitted import. */
  source: string;
  /** Local binding name in the emitted module. */
  local: string;
  /** Whether the binding represents an import namespace. */
  namespace: boolean;
};

/** Native module-shape facts needed to decide whether state can be preserved. */
export type HmrModuleShape = {
  /** Imported bindings whose current values are snapshotted before replacement. */
  imports?: ModuleImportInfo[];
  /** Whether a bare side-effect import makes preservation unsafe. */
  hasSideEffectImport?: boolean;
};

/** Snapshot native-reported imported values before a hot replacement. */
export function hmrImportSnapshot(
  shape: HmrModuleShape | undefined,
  hostImports: readonly ModuleImportInfo[] = [],
): string | null {
  if (!shape?.imports || shape.hasSideEffectImport) return null;
  const entries: string[] = [];
  for (const { source, local, namespace } of [...shape.imports, ...hostImports]) {
    // Vue helpers can change with the template. CSS has its own HMR boundary.
    if (source === "vue" || source.includes("?vue&type=style&")) continue;
    const key = JSON.stringify(`${source}:${local}`);
    entries.push(
      namespace
        ? `...Object.keys(${local}).sort().map(key => [${key} + ':' + key, ${local}[key]])`
        : `[${key}, ${local}]`,
    );
  }
  return `[${entries.join(", ")}]`;
}
