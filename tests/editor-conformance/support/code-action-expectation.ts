import { isDeepStrictEqual } from "node:util";

function record(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

/** Resolve explicit diagnostic references from the request, never the response. */
export function codeActionExpectation(expected: unknown, request: unknown): unknown {
  if (!Array.isArray(expected)) return expected;
  const context = record(request) ? request.context : undefined;
  const diagnostics = record(context) ? context.diagnostics : undefined;
  return expected.map((action: unknown) => {
    if (!record(action) || !Array.isArray(action.diagnostics)) return action;
    return {
      ...action,
      diagnostics: action.diagnostics.map((reference: unknown) => {
        if (!record(reference) || !("$requestDiagnostic" in reference)) return reference;
        const identity = reference.$requestDiagnostic;
        if (
          Object.keys(reference).length !== 1 ||
          !record(identity) ||
          !isDeepStrictEqual(Object.keys(identity).sort(), ["code", "range", "source"])
        ) {
          throw new Error(
            "A request diagnostic reference requires exactly source, code, and range",
          );
        }
        const selected = Array.isArray(diagnostics)
          ? diagnostics.filter(
              (diagnostic: unknown) =>
                record(diagnostic) &&
                isDeepStrictEqual(
                  { source: diagnostic.source, code: diagnostic.code, range: diagnostic.range },
                  identity,
                ),
            )
          : [];
        if (selected.length !== 1) {
          throw new Error(`Expected one matching request diagnostic, found ${selected.length}`);
        }
        // Preserve every incoming field, including optional metadata. The judge
        // still compares the entire response against this resolved expectation.
        return selected[0];
      }),
    };
  });
}
