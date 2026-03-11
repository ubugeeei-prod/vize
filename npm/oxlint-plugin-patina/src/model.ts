export interface PatinaPosition {
  line: number;
  column: number;
  offset: number;
}

export interface PatinaLocation {
  start: PatinaPosition;
  end: PatinaPosition;
}

export interface PatinaDiagnostic {
  rule: string;
  severity: "error" | "warning";
  message: string;
  location: PatinaLocation;
  help: string | null;
}

export interface PatinaLintResult {
  filename: string;
  errorCount: number;
  warningCount: number;
  diagnostics: PatinaDiagnostic[];
}

export interface PatinaRuleMeta {
  name: string;
  description: string;
  category: string;
  fixable: boolean;
  defaultSeverity: "error" | "warning";
}

export interface PatinaBinding {
  lintPatinaSfc(
    source: string,
    options?: {
      filename?: string;
      locale?: string;
      enabled_rules?: string[];
    },
  ): PatinaLintResult;
  getPatinaRules(): PatinaRuleMeta[];
}

export interface PatinaSettings {
  locale?: string;
  showHelp?: boolean;
}

export interface LineColumn {
  line: number;
  column: number;
}

export interface ScriptBlock {
  content: string;
  contentStart: LineColumn;
  contentEnd: LineColumn;
}

export interface SingleScriptMap {
  block: ScriptBlock;
}
