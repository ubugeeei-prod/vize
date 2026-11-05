/** Authored parser/project diagnostics are not configurable catalogue rules. */
export const operationalDiagnosticPages = [
  ["parser/template", "parser-template"],
  ["parser/sfc", "parser-sfc"],
  ["parser/jsx", "parser-jsx"],
  ["type/project-configuration", "type-project-configuration"],
] as const;

export function isOperationalDiagnosticReference(file: string): boolean {
  return operationalDiagnosticPages.some(([, slug]) => file === `${slug}.md`);
}
