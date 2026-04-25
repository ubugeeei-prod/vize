export interface DeclarationOptions {
  filename?: string;
}

export interface DeclarationResult {
  code: string;
}

export function generateDeclaration(
  source: string,
  options?: DeclarationOptions,
): DeclarationResult;
