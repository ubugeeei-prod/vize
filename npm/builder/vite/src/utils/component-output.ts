import type { CompiledModule } from "../types.ts";
import {
  analyzeModuleOutput,
  insertBeforeSfcMainDefaultExport,
  rewriteDefaultExportToSfcMain,
} from "./module-output.ts";
import { MappedModule, parseSourceMap } from "./source-map.ts";

/** Assemble the compiler's component export before CSS and HMR decoration. */
export function assembleComponentOutput(
  compiled: Pick<CompiledModule, "code" | "map" | "moduleShape" | "hasScoped" | "scopeId">,
): { emitted: MappedModule; hasExportDefault: boolean } {
  const emitted = new MappedModule(compiled.code, parseSourceMap(compiled.map));

  // The native compiler already parsed this module and reports its shape, so
  // the oxc parse here is redundant (#3425). `??` rather than a required field:
  // a `.vpc` cache entry written before this existed reads back without it and
  // simply pays the parse it always paid, so no cache-format bump is needed and
  // the fallback stays exercised.
  const moduleInfo = compiled.moduleShape ?? analyzeModuleOutput(emitted.code);
  const hasExportDefault = moduleInfo.hasDefaultExport;
  const hasNamedRenderExport = moduleInfo.hasNamedRenderExport;
  const hasNamedSsrRenderExport = moduleInfo.hasNamedSsrRenderExport;
  const hasSfcMainDefined = moduleInfo.hasSfcMainDefined;

  if (hasExportDefault && !hasSfcMainDefined) {
    emitted.edit(rewriteDefaultExportToSfcMain(emitted.code, moduleInfo));
    // Add __scopeId for scoped CSS support
    if (compiled.hasScoped && compiled.scopeId) {
      emitted.edit(`${emitted.code}\n_sfc_main.__scopeId = "data-v-${compiled.scopeId}";`);
    }
    emitted.edit(`${emitted.code}\nexport default _sfc_main;`);
  } else if (hasExportDefault && hasSfcMainDefined) {
    // _sfc_main already defined, just add scopeId if needed
    if (compiled.hasScoped && compiled.scopeId) {
      // `emitted.code` is still `compiled.code` on this branch -- nothing above
      // it rewrote the module -- so `moduleInfo`'s offsets describe it exactly and
      // the insertion can reuse them instead of parsing the module again
      // (#3425). The later CSS-modules insertion below cannot: by then the
      // module has been rewritten and the offsets are stale.
      emitted.edit(
        insertBeforeSfcMainDefaultExport(
          emitted.code,
          `_sfc_main.__scopeId = "data-v-${compiled.scopeId}";`,
          { moduleInfo },
        ),
      );
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
  }

  return { emitted, hasExportDefault };
}
