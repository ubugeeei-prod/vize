export function validatedFileCount(tool, payload) {
  if (tool === "compiler") return payload.compilerArtifacts.inputFileCount;
  if (tool === "typechecker") return payload.parsed.fileCount;
  if (tool === "linter") return payload.parsed.length;
  if (tool === "formatter") return payload.formatterCheck.checkedFileCount;
  throw new Error(`Unsupported fixture matrix tool: ${tool}`);
}
