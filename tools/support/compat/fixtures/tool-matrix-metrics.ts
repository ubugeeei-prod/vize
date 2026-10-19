export interface ToolMatrixPayload {
  readonly compilerArtifacts?: { readonly inputFileCount: number };
  readonly parsed?: { readonly fileCount?: number; readonly length?: number };
  readonly formatterCheck?: { readonly checkedFileCount: number };
}

function required(value: number | undefined, tool: string): number {
  if (value === undefined) throw new Error(`Missing ${tool} fixture matrix file count`);
  return value;
}

export function validatedFileCount(tool: string, payload: ToolMatrixPayload): number {
  if (tool === "compiler") return required(payload.compilerArtifacts?.inputFileCount, tool);
  if (tool === "typechecker") return required(payload.parsed?.fileCount, tool);
  if (tool === "linter") return required(payload.parsed?.length, tool);
  if (tool === "formatter") return required(payload.formatterCheck?.checkedFileCount, tool);
  throw new Error(`Unsupported fixture matrix tool: ${tool}`);
}
