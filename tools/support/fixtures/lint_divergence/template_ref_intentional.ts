import {
  callsVueExport,
  type Finding,
  type Node,
  node,
  nodes,
  offset,
  type SetupScript,
  setupScript,
  statements,
} from "./sfc_ast.ts";

const refFactories = ["ref", "shallowRef"];

/** The declarators of every top-level `const` statement of the setup block. */
function constDeclarators(setup: SetupScript): Node[] {
  return statements(setup.program)
    .filter((statement) => statement.type === "VariableDeclaration" && statement.kind === "const")
    .flatMap((statement) => (Array.isArray(statement.declarations) ? statement.declarations : []))
    .flatMap((declaration) => node(declaration) ?? []);
}

/** The `[start, end)` of a finding relative to the setup block content. */
function setupRange(finding: Finding, source: string, setup: SetupScript): [number, number] | null {
  const start = offset(source, finding.line, finding.column),
    end = offset(source, finding.endLine, finding.endColumn);
  return start === null || end === null ? null : [start - setup.start, end - setup.start];
}

/** Refuse ambiguous bindings: only a proven write makes this migration unsafe. */
export function mutableTemplateRefReason(finding: Finding, source: string): string | null {
  if (finding.ruleId !== "vue/prefer-use-template-ref") return null;
  try {
    const setup = setupScript(source);
    const range = setup === null ? null : setupRange(finding, source, setup);
    if (setup === null || range === null) return null;
    const declarations = constDeclarators(setup).filter((declaration) => {
      const init = node(declaration.init);
      return (
        init?.type === "CallExpression" &&
        callsVueExport(setup, init.callee, refFactories) &&
        init.start === range[0] &&
        init.end === range[1]
      );
    });
    if (declarations.length !== 1) return null;
    const binding = node(declarations[0]?.id);
    if (binding?.type !== "Identifier" || typeof binding.name !== "string") return null;
    const all = nodes(setup.program),
      allowed = new Set<Node>([binding]);
    let written = false;
    for (const current of all) {
      // A call argument is a plain read; it cannot declare a shadowing binding.
      if (current.type === "CallExpression" && Array.isArray(current.arguments))
        for (const argument of current.arguments) {
          const read = node(argument);
          if (read?.type === "Identifier" && read.name === binding.name) allowed.add(read);
        }
      if (current.type !== "MemberExpression") continue;
      const object = node(current.object),
        property = node(current.property);
      if (object?.type !== "Identifier" || object.name !== binding.name) continue;
      allowed.add(object);
      const value = current.computed
        ? property?.type === "StringLiteral" && property.value === "value"
        : property?.type === "Identifier" && property.name === "value";
      if (
        value &&
        all.some(
          (parent) =>
            (parent.type === "AssignmentExpression" && parent.left === current) ||
            (parent.type === "UpdateExpression" && parent.argument === current),
        )
      )
        written = true;
    }
    if (
      !written ||
      all.some(
        (current) =>
          current.type === "Identifier" && current.name === binding.name && !allowed.has(current),
      )
    )
      return null;
    return "Patina retains a mutable template ref whose .value is assigned by its unshadowed setup binding; useTemplateRef() returns a readonly ref, so the upstream migration suggestion would introduce a TypeScript error.";
  } catch {
    // Unavailable parsers or invalid source cannot establish an exception.
    return null;
  }
}

const quotedName = (message: string): string | undefined => /'([^']*)'/u.exec(message)?.[1];

export function templateRefIsOtherwiseUnread(finding: Finding, source: string): boolean {
  const name = quotedName(finding.message);
  if (name === undefined) return false;
  const escaped = name.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
  const script = /<script\b[^>]*\bsetup\b[^>]*>([\s\S]*?)<\/script>/iu.exec(source)?.[1];
  if (script === undefined) return false;
  const references = script.match(new RegExp(`\\b${escaped}\\b`, "gu"))?.length ?? 0;
  return references === 1;
}

const assertionWrappers = new Set([
  "TSAsExpression",
  "TSSatisfiesExpression",
  "TSNonNullExpression",
]);

/**
 * The reported Vue `ref()` call is the initializer of a top-level setup
 * `const`, but under a TypeScript assertion (`ref(null) as Ref<T>`, `ref()!`).
 */
function assertedTemplateRef(finding: Finding, source: string): boolean {
  const setup = setupScript(source);
  const range = setup === null ? null : setupRange(finding, source, setup);
  if (setup === null || range === null) return false;
  return constDeclarators(setup).some((declaration) => {
    let init = node(declaration.init);
    if (init === null || !assertionWrappers.has(init.type)) return false;
    while (init !== null && assertionWrappers.has(init.type)) init = node(init.expression);
    return (
      init?.type === "CallExpression" &&
      callsVueExport(setup, init.callee, refFactories) &&
      init.start === range[0] &&
      init.end === range[1]
    );
  });
}

/**
 * The template ref's same-named setup binding is a `useTemplateRef()` handle
 * for a different static key, so the binding never reads this element.
 */
function templateRefNamesOtherHandle(finding: Finding, source: string): boolean {
  const name = /^\[vize:vue\/no-unused-refs\] Template ref '([^']+)' is never referenced/u.exec(
    finding.message,
  )?.[1];
  const setup = name === undefined ? null : setupScript(source);
  if (setup === null) return false;
  return constDeclarators(setup).some((declaration) => {
    const init = node(declaration.init),
      key = Array.isArray(init?.arguments) ? node(init.arguments[0]) : null;
    return (
      node(declaration.id)?.name === name &&
      init?.type === "CallExpression" &&
      callsVueExport(setup, init.callee, ["useTemplateRef"]) &&
      key?.type === "StringLiteral" &&
      typeof key.value === "string" &&
      key.value !== name
    );
  });
}

/** Template-ref findings Patina reports deliberately beyond eslint-plugin-vue. */
export function templateRefPatinaReason(finding: Finding, source: string): string | null {
  try {
    if (finding.ruleId === "script/prefer-use-template-ref" && assertedTemplateRef(finding, source))
      return "Patina pairs a template ref whose ref() initializer sits under a TypeScript `as`, `satisfies` or `!` assertion; eslint-plugin-vue only pairs a declarator whose initializer is the bare call.";
    if (finding.ruleId === "vue/no-unused-refs" && templateRefNamesOtherHandle(finding, source))
      return "Patina reports a template ref whose same-named setup binding is a useTemplateRef() handle for another key: in development Vue does not assign that readonly handle from the ref attribute, so the element stays unreachable; eslint-plugin-vue counts the same-named identifier as a use.";
  } catch {
    // Unavailable parsers or invalid source cannot establish an exception.
  }
  return null;
}
