// Analysis type definitions (Canon/TypeCheck, Cross-file)

// TypeCheck types (Canon)
export interface TypeCheckOptions {
  filename?: string;
  strict?: boolean;
  includeVirtualTs?: boolean;
  checkProps?: boolean;
  checkEmits?: boolean;
  checkTemplateBindings?: boolean;
}

export interface TypeCheckRelatedLocation {
  message: string;
  start: number;
  end: number;
  filename?: string;
}

export interface TypeCheckDiagnostic {
  severity: "error" | "warning" | "info" | "hint";
  message: string;
  start: number;
  end: number;
  code?: string;
  help?: string;
  related: TypeCheckRelatedLocation[];
}

export interface TypeCheckResult {
  diagnostics: TypeCheckDiagnostic[];
  virtualTs?: string;
  errorCount: number;
  warningCount: number;
  analysisTimeMs?: number;
}

export interface TypeCheckCapability {
  name: string;
  description: string;
  severity: string;
}

export interface TypeCheckCapabilities {
  mode: string;
  description: string;
  checks: TypeCheckCapability[];
  notes: string[];
}

// Cross-file analysis types
export interface CrossFileOptions {
  all?: boolean;
  fallthroughAttrs?: boolean;
  componentEmits?: boolean;
  eventBubbling?: boolean;
  provideInject?: boolean;
  uniqueIds?: boolean;
  serverClientBoundary?: boolean;
  errorSuspenseBoundary?: boolean;
  reactivityTracking?: boolean;
  setupContext?: boolean;
  circularDependencies?: boolean;
  maxImportDepth?: number;
  componentResolution?: boolean;
  propsValidation?: boolean;
}

export interface CrossFileDiagnostic {
  type: string;
  code: string;
  severity: "error" | "warning" | "info" | "hint";
  message: string;
  file: string;
  offset: number;
  endOffset: number;
  relatedLocations?: Array<{
    file: string;
    offset: number;
    message: string;
  }>;
  suggestion?: string;
}

export interface CrossFileStats {
  filesAnalyzed: number;
  vueComponents: number;
  dependencyEdges: number;
  errorCount: number;
  warningCount: number;
  infoCount: number;
  analysisTimeMs: number;
}

export interface CrossFileResult {
  diagnostics: CrossFileDiagnostic[];
  circularDependencies: string[][];
  stats: CrossFileStats;
  filePaths: string[];
}

export interface CrossFileInput {
  path: string;
  source: string;
}
