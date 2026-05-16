// Croquis (SFC analysis) type definitions

// Binding source (where it comes from)
export type BindingSource =
  | "props"
  | "emits"
  | "model"
  | "slots"
  | "ref"
  | "reactive"
  | "computed"
  | "import"
  | "local"
  | "function"
  | "class"
  | "templateRef"
  | "unknown";

// Binding metadata
export interface BindingMetadata {
  fromMacro?: string;
  isExported: boolean;
  isImported: boolean;
  isComponent: boolean;
  isDirective: boolean;
  needsValue: boolean;
  usedInTemplate: boolean;
  usedInScript: boolean;
  scopeDepth: number;
}

export interface BindingDisplay {
  name: string;
  kind: string;
  source: BindingSource;
  metadata: BindingMetadata;
  typeAnnotation?: string;
  start: number;
  end: number;
  isUsed: boolean;
  isMutated: boolean;
  referenceCount: number;
  // Template binding info
  bindable: boolean; // Can be referenced from template
  usedInTemplate: boolean; // Actually used in template
  fromScriptSetup: boolean; // Comes from <script setup>
}

// Scope kind (abbreviated)
export type ScopeKind =
  | "mod" // module
  | "setup" // scriptSetup
  | "plain" // nonScriptSetup
  | "extern" // externalModule
  | "vue" // vueGlobal
  | "universal" // runs on both server and client
  | "server" // server only (Node.js)
  | "client" // client only (browser)
  | "function"
  | "arrowFunction"
  | "block"
  | "vFor"
  | "vSlot"
  | "class"
  | "staticBlock"
  | "catch";

export interface ScopeDisplay {
  id: number;
  parentIds?: number[]; // Multiple parent scopes (e.g., setup can access mod, universal, etc.)
  kind: ScopeKind;
  kindStr: string;
  start: number;
  end: number;
  bindings: string[];
  children: number[];
  depth: number;
}

export interface MacroDisplay {
  name: string;
  start: number;
  end: number;
  type_args?: string;
  args?: string;
  binding?: string;
}

// Type export (hoisted from script setup)
export interface TypeExportDisplay {
  name: string;
  kind: "type" | "interface";
  start: number;
  end: number;
  hoisted: boolean; // true if hoisted from script setup to module level
}

// Invalid export in script setup
export interface InvalidExportDisplay {
  name: string;
  kind: "const" | "let" | "var" | "function" | "class" | "default";
  start: number;
  end: number;
  message: string;
}

export interface PropDisplay {
  name: string;
  type_annotation?: string;
  required: boolean;
  has_default: boolean;
}

export interface EmitDisplay {
  name: string;
  payload_type?: string;
}

// Provide key (string or symbol)
export interface ProvideKey {
  type: "string" | "symbol";
  value: string;
}

// Provide entry from Rust analysis
export interface ProvideDisplay {
  key: ProvideKey;
  value: string;
  valueType?: string;
  fromComposable?: string;
  start: number;
  end: number;
}

// Inject pattern
export type InjectPattern = "simple" | "objectDestructure" | "arrayDestructure";

// Inject entry from Rust analysis
export interface InjectDisplay {
  key: ProvideKey;
  localName: string;
  defaultValue?: string;
  expectedType?: string;
  pattern: InjectPattern;
  destructuredProps?: string[];
  fromComposable?: string;
  start: number;
  end: number;
}

export interface CssDisplay {
  selector_count: number;
  unused_selectors: Array<{ text: string; start: number; end: number }>;
  v_bind_count: number;
  is_scoped: boolean;
}

export interface CroquisStats {
  binding_count: number;
  unused_binding_count: number;
  scope_count: number;
  macro_count: number;
  type_export_count: number;
  invalid_export_count: number;
  error_count: number;
  warning_count: number;
}

export interface CroquisDiagnostic {
  severity: "error" | "warning" | "info" | "hint";
  message: string;
  start: number;
  end: number;
  code?: string;
  related: Array<{ message: string; start: number; end: number }>;
}

export interface Croquis {
  component_name?: string;
  is_setup: boolean;
  bindings: BindingDisplay[];
  scopes: ScopeDisplay[];
  macros: MacroDisplay[];
  props: PropDisplay[];
  emits: EmitDisplay[];
  provides: ProvideDisplay[];
  injects: InjectDisplay[];
  typeExports: TypeExportDisplay[];
  invalidExports: InvalidExportDisplay[];
  css?: CssDisplay;
  diagnostics: CroquisDiagnostic[];
  stats: CroquisStats;
}

export interface CroquisOptions {
  filename?: string;
}

export interface CroquisResult {
  croquis: Croquis;
  diagnostics: CroquisDiagnostic[];
  /** VIR (Vize Intermediate Representation) text format */
  vir?: string;
}
