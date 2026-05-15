// WASM output transformation layer
// Transforms raw WASM analysis output to the expected TypeScript format

import type {
  CroquisOptions,
  CroquisResult,
  BindingDisplay,
  BindingSource,
  ScopeDisplay,
  ScopeKind,
  MacroDisplay,
  PropDisplay,
  EmitDisplay,
  ProvideDisplay,
  InjectDisplay,
} from "./types";

// Raw WASM scope type
interface RawWasmScope {
  id: number;
  kind: string;
  kindStr?: string;
  parentIds?: number[];
  start: number;
  end: number;
  bindings: string[];
  depth?: number;
  isTemplateScope?: boolean;
}

// Create a transformAnalyzeSfc wrapper for WASM output
export function createTransformAnalyzeSfc(
  wasmAnalyzeSfc: (source: string, options: CroquisOptions) => any,
): (source: string, options: CroquisOptions) => CroquisResult {
  return (source: string, options: CroquisOptions): CroquisResult => {
    const rawResult = wasmAnalyzeSfc(source, options);

    // WASM returns data under 'croquis' key
    const croquis = rawResult.croquis || rawResult;

    const rawScopes: RawWasmScope[] = croquis.scopes || [];

    // Build children map from parentIds
    const childrenMap = new Map<number, number[]>();
    for (const scope of rawScopes) {
      const parentIds = scope.parentIds || [];
      for (const parentId of parentIds) {
        const existing = childrenMap.get(parentId) || [];
        existing.push(scope.id);
        childrenMap.set(parentId, existing);
      }
    }

    // Convert to ScopeDisplay format
    const scopes: ScopeDisplay[] = rawScopes.map((scope) => ({
      id: scope.id,
      parentIds: scope.parentIds || [],
      kind: scope.kind as ScopeKind,
      kindStr: scope.kindStr || scope.kind,
      start: scope.start,
      end: scope.end,
      bindings: scope.bindings,
      children: childrenMap.get(scope.id) || [],
      depth: scope.depth || 0,
    }));

    // Transform bindings to match BindingDisplay interface
    const bindings: BindingDisplay[] = (croquis.bindings || []).map(
      (b: { name: string; type: string; start?: number; end?: number }) => ({
        name: b.name,
        kind: b.type,
        source: "script" as BindingSource,
        metadata: {
          isExported: false,
          isImported: false,
          isComponent: false,
          isDirective: false,
          needsValue: true,
          usedInTemplate: true,
          usedInScript: true,
          scopeDepth: 0,
        },
        typeAnnotation: undefined,
        start: b.start ?? 0,
        end: b.end ?? 0,
        isUsed: true,
        isMutated: false,
        referenceCount: 1,
        bindable: true,
        usedInTemplate: true,
        fromScriptSetup: croquis.is_setup || false,
      }),
    );

    // Transform macros from WASM
    const macros: MacroDisplay[] = (croquis.macros || []).map(
      (m: {
        name: string;
        kind: string;
        start: number;
        end: number;
        typeArgs?: string;
        runtimeArgs?: string;
      }) => ({
        name: m.name,
        start: m.start,
        end: m.end,
        type_args: m.typeArgs,
        args: m.runtimeArgs,
      }),
    );

    // Transform props from WASM
    const props: PropDisplay[] = (croquis.props || []).map(
      (p: { name: string; required: boolean; hasDefault: boolean }) => ({
        name: p.name,
        required: p.required,
        has_default: p.hasDefault,
      }),
    );

    // Transform emits from WASM
    const emits: EmitDisplay[] = (croquis.emits || []).map((e: { name: string }) => ({
      name: e.name,
    }));

    // Pass through provides and injects from WASM
    const provides: ProvideDisplay[] = croquis.provides || [];
    const injects: InjectDisplay[] = croquis.injects || [];

    // Build CroquisResult in expected format
    const result: CroquisResult = {
      croquis: {
        is_setup: croquis.is_setup || false,
        bindings,
        scopes,
        macros,
        props,
        emits,
        provides,
        injects,
        typeExports: croquis.typeExports || [],
        invalidExports: croquis.invalidExports || [],
        diagnostics: croquis.diagnostics || [],
        stats: croquis.stats || {
          binding_count: bindings.length,
          unused_binding_count: 0,
          scope_count: scopes.length,
          macro_count: macros.length,
          type_export_count: 0,
          invalid_export_count: 0,
          error_count: 0,
          warning_count: 0,
        },
      },
      diagnostics: rawResult.diagnostics || [],
      // VIR (Vize Intermediate Representation) text from WASM
      vir: rawResult.vir || "",
    };

    return result;
  };
}
