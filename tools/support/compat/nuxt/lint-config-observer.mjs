// The #7904 wrapper reports original-source byte ranges and keeps an
// absolute reported filename for files outside the process cwd.
import path from "node:path";

export function expectedCliDiagnostic(corpus, project, name) {
  const absolute = path.resolve(project, name);
  const relative = path.relative(project, absolute);
  const outside = relative === ".." || relative.startsWith(`..${path.sep}`);
  return {
    code: corpus.expectedRule,
    filename: outside ? absolute : relative.replaceAll(path.sep, "/"),
    message:
      "Avoid using inline style attributes\n    Help:\n      Use CSS classes or scoped styles instead",
    severity: corpus.expectedSeverity,
    labels: [{ span: { offset: 76, length: 18, line: 6, column: 7 } }],
  };
}
