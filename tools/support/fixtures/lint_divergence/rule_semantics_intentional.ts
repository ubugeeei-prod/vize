import {
  callsVueExport,
  type Finding,
  type Node,
  node,
  nodes,
  offset,
  setupScript,
  type TemplateNode,
  templateRoot,
} from "./sfc_ast.ts";
import { mutableTemplateRefReason } from "./template_ref_intentional.ts";

const elementNode = 1,
  attributeNode = 6,
  directiveNode = 7;

function templateElements(root: TemplateNode): TemplateNode[] {
  const result: TemplateNode[] = [];
  const visit = (current: TemplateNode) => {
    if (current.type === elementNode) result.push(current);
    for (const child of current.children ?? []) visit(child);
    // v-if / v-for wrappers keep their elements in branches.
    for (const branch of (current as { branches?: TemplateNode[] }).branches ?? []) visit(branch);
  };
  visit(root);
  return result;
}

/** The reported `<component>` carries a non-empty static `is` and no bound one. */
function staticComponentIs(finding: Finding, source: string): boolean {
  const start = offset(source, finding.line, finding.column),
    root = start === null ? null : templateRoot(source);
  if (root === null) return false;
  return templateElements(root).some((element) => {
    const props = element.props ?? [];
    return (
      element.tag === "component" &&
      element.loc.start.offset === start &&
      props.some(
        (prop) =>
          prop.type === attributeNode &&
          prop.name === "is" &&
          (prop.value?.content.trim() ?? "") !== "",
      ) &&
      !props.some(
        (prop) => prop.type === directiveNode && prop.name === "bind" && prop.arg?.content === "is",
      )
    );
  });
}

/** Declared types that cannot contain `undefined`, read at the top level only. */
const definiteTypes = new Set([
  "TSTypeLiteral",
  "TSNullKeyword",
  "TSStringKeyword",
  "TSNumberKeyword",
  "TSBooleanKeyword",
  "TSBigIntKeyword",
  "TSObjectKeyword",
  "TSLiteralType",
  "TSArrayType",
  "TSTupleType",
  "TSFunctionType",
]);
function excludesUndefined(type: Node | null): boolean {
  if (type?.type === "TSParenthesizedType") return excludesUndefined(node(type.typeAnnotation));
  if (type?.type === "TSUnionType")
    return (
      Array.isArray(type.types) && type.types.every((member) => excludesUndefined(node(member)))
    );
  return type !== null && definiteTypes.has(type.type);
}
function endsInExit(statement: Node | null): boolean {
  if (statement?.type === "BlockStatement" && Array.isArray(statement.body))
    return endsInExit(node(statement.body.at(-1)));
  return statement?.type === "ReturnStatement" || statement?.type === "ThrowStatement";
}

/**
 * The reported getter is `computed<T>(…)` or `(): T =>` with a `T` that
 * excludes `undefined`, and its only path out is a final default-less
 * `switch` whose last case returns or throws and which no `break` leaves.
 */
function typedComputedSwitch(finding: Finding, source: string): boolean {
  const setup = setupScript(source),
    start = offset(source, finding.line, finding.column);
  if (setup === null || start === null) return false;
  return nodes(setup.program).some((call) => {
    if (call.type !== "CallExpression" || !callsVueExport(setup, call.callee, ["computed"]))
      return false;
    const getter = Array.isArray(call.arguments) ? node(call.arguments[0]) : null;
    if (
      getter === null ||
      !["ArrowFunctionExpression", "FunctionExpression"].includes(getter.type) ||
      getter.start !== start - setup.start
    )
      return false;
    const typeArguments = node(call.typeParameters) ?? node(call.typeArguments),
      declared = Array.isArray(typeArguments?.params)
        ? node(typeArguments.params[0])
        : node(node(getter.returnType)?.typeAnnotation);
    const body = node(getter.body),
      last =
        body?.type === "BlockStatement" && Array.isArray(body.body) ? node(body.body.at(-1)) : null;
    const cases = last?.type === "SwitchStatement" && Array.isArray(last.cases) ? last.cases : [];
    const lastCase = node(cases.at(-1));
    return (
      excludesUndefined(declared) &&
      last !== null &&
      lastCase !== null &&
      cases.every((value) => node(value)?.test !== null) &&
      Array.isArray(lastCase.consequent) &&
      endsInExit(node(lastCase.consequent.at(-1))) &&
      !nodes(last).some((current) => current.type === "BreakStatement")
    );
  });
}

/** Mapped upstream findings Patina deliberately does not report. */
export function ruleSemanticsBaselineReason(finding: Finding, source: string): string | null {
  const mutable = mutableTemplateRefReason(finding, source);
  if (mutable !== null) return mutable;
  try {
    if (finding.ruleId === "vue/require-component-is" && staticComponentIs(finding, source))
      return "Vue renders the component a static is attribute names on <component>, in Vue 2 and Vue 3 alike; eslint-plugin-vue requires v-bind:is even then.";
    if (
      finding.ruleId === "vue/return-in-computed-property" &&
      typedComputedSwitch(finding, source)
    )
      return "Patina leaves a default-less switch to TypeScript when the getter declares a return type without undefined: TypeScript rejects any reachable implicit return there, while eslint-plugin-vue has no types and counts the missing default as a path.";
  } catch {
    // Unavailable parsers or invalid source cannot establish an exception.
  }
  return null;
}
