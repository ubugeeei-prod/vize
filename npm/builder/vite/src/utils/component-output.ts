import { bindDefaultExport } from "../../../shared/native-shape.ts";
import type { CompiledModule } from "../types.ts";
import { analyzeModuleOutput } from "./module-output.ts";
import { MappedModule, parseSourceMap } from "./source-map.ts";

/** Assemble the compiler's component export before CSS and HMR decoration. */
export function assembleComponentOutput(
  compiled: Pick<CompiledModule, "code" | "map" | "moduleShape" | "hasScoped" | "scopeId">,
): { emitted: MappedModule; hasExportDefault: boolean; exportsSfcMain: boolean } {
  const emitted = new MappedModule(compiled.code, parseSourceMap(compiled.map));

  // The native compiler already parsed this module and reports its shape, so
  // the oxc parse here is redundant (#3425, #11544). It reports none only for
  // output it could not parse, which pays the parse here.
  const moduleInfo = compiled.moduleShape ?? analyzeModuleOutput(emitted.code);
  const hasExportDefault = moduleInfo.hasDefaultExport;
  const hasNamedRenderExport = moduleInfo.hasNamedRenderExport;
  const hasNamedSsrRenderExport = moduleInfo.hasNamedSsrRenderExport;
  const hasSfcMainDefined = moduleInfo.hasSfcMainDefined;
  // Whether the assembled module default-exports `_sfc_main`; decorations that
  // attach to it are appended after the module instead of located in it.
  let exportsSfcMain = true;

  if (hasExportDefault && !hasSfcMainDefined) {
    // The shape describes `compiled.code`, so this is the first edit.
    bindDefaultExport(emitted, moduleInfo, "_sfc_main");
    // Add __scopeId for scoped CSS support
    if (compiled.hasScoped && compiled.scopeId) {
      emitted.edit(`${emitted.code}\n_sfc_main.__scopeId = "data-v-${compiled.scopeId}";`);
    }
    emitted.edit(`${emitted.code}\nexport default _sfc_main;`);
  } else if (hasExportDefault && hasSfcMainDefined) {
    exportsSfcMain = moduleInfo.defaultExportIsSfcMain;
    // Attach to the exported `_sfc_main` after the module, like the style
    // bindings: it runs before any importer reads the export (#11583).
    if (exportsSfcMain && compiled.hasScoped && compiled.scopeId) {
      emitted.edit(`${emitted.code}\n_sfc_main.__scopeId = "data-v-${compiled.scopeId}";`);
    }
  } else if (!hasExportDefault && !hasSfcMainDefined && hasNamedRenderExport) {
    const scope =
      compiled.hasScoped && compiled.scopeId ? `, __scopeId: "data-v-${compiled.scopeId}"` : "";
    emitted.edit(
      `${emitted.code}\nconst _sfc_main = { render${scope} };\nexport default _sfc_main;`,
    );
  } else if (!hasExportDefault && !hasSfcMainDefined && hasNamedSsrRenderExport) {
    const scope =
      compiled.hasScoped && compiled.scopeId ? `, __scopeId: "data-v-${compiled.scopeId}"` : "";
    emitted.edit(
      `${emitted.code}\nconst _sfc_main = { ssrRender${scope} };\nexport default _sfc_main;`,
    );
  } else {
    exportsSfcMain = false;
  }

  return { emitted, hasExportDefault, exportsSfcMain };
}
