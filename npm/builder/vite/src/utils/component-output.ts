import { bindDefaultExport } from "../../../shared/native-shape.ts";
import type { CompiledModule } from "../types.ts";
import { unparsedModuleOutput } from "./module-output.ts";
import { MappedModule, parseSourceMap } from "./source-map.ts";

/** What the assembled module exports, for the decorations appended to it. */
export type AssembledComponent = {
  emitted: MappedModule;
  /**
   * Whether the assembled module has a default export. This describes the
   * output, not `compiled.code`: a render-only module gains one here.
   */
  exportsDefault: boolean;
  /**
   * Whether that default export is `_sfc_main`; decorations that attach to it
   * are appended after the module instead of located in it.
   */
  exportsSfcMain: boolean;
  /**
   * Whether `_sfc_main.render` is known to be a standalone render function,
   * which template-only HMR can swap. `false` leaves the question to the
   * emitted code.
   */
  standaloneRender: boolean;
};

/** Assemble the compiler's component export before CSS and HMR decoration. */
export function assembleComponentOutput(
  compiled: Pick<CompiledModule, "code" | "map" | "moduleShape" | "hasScoped" | "scopeId">,
): AssembledComponent {
  const emitted = new MappedModule(compiled.code, parseSourceMap(compiled.map));

  // The native compiler already parsed this module and reports its shape. It
  // reports none only for output it could not parse, which is not parsed here.
  const moduleInfo = compiled.moduleShape ?? unparsedModuleOutput(emitted.code);
  const hasExportDefault = moduleInfo.hasDefaultExport;
  const hasNamedRenderExport = moduleInfo.hasNamedRenderExport;
  const hasNamedSsrRenderExport = moduleInfo.hasNamedSsrRenderExport;
  const hasSfcMainDefined = moduleInfo.hasSfcMainDefined;
  let exportsDefault = true;
  let exportsSfcMain = true;
  let standaloneRender = false;

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
    // bindings: it runs before any importer reads the export.
    if (exportsSfcMain && compiled.hasScoped && compiled.scopeId) {
      emitted.edit(`${emitted.code}\n_sfc_main.__scopeId = "data-v-${compiled.scopeId}";`);
    }
  } else if (!hasExportDefault && !hasSfcMainDefined && hasNamedRenderExport) {
    // A render-only module (an SFC without `<script>`) gets a component that
    // carries the exported render function itself.
    standaloneRender = true;
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
    exportsDefault = false;
    exportsSfcMain = false;
  }

  return { emitted, exportsDefault, exportsSfcMain, standaloneRender };
}
