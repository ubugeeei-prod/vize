import ts from "typescript";
import { Rolldown } from "vite-plus/pack";

/** Remove generated source-region labels, preserving runtime tokens and source maps. */
export function stripCatalogRegions(code: string, fileName: string) {
  const ast = ts.createSourceFile(fileName, code, ts.ScriptTarget.Latest, true, ts.ScriptKind.JS);
  const edited = new Rolldown.RolldownMagicString(code);
  for (const node of [...ast.statements, ast.endOfFileToken]) {
    for (const range of ts.getLeadingCommentRanges(code, node.pos) ?? []) {
      if (/^\/\/#(?:end)?region(?:[ \t].*)?$/.test(code.slice(range.pos, range.end))) {
        let end = range.end;
        if (code[end] === "\r") end++;
        if (code[end] === "\n") end++;
        edited.remove(range.pos, end);
      }
    }
  }
  return {
    code: edited.toString(),
    map: edited.generateMap({ hires: true, source: fileName, includeContent: true }).toString(),
  };
}
