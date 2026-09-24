/** Member extraction from interfaces, aliases, and type literals. */
import ts from "typescript";

import type { ApiMember } from "./api-types.ts";
import { type Declaration, type TypeScope, memberName, oneLine, readDoc } from "./ts-scope.ts";

export function membersOfType(
  node: ts.TypeNode,
  scope: TypeScope,
  depth = 0,
): readonly ts.TypeElement[] | null {
  if (depth > 24) return null;
  if (ts.isTypeLiteralNode(node)) return node.members;
  if (ts.isParenthesizedTypeNode(node)) return membersOfType(node.type, scope, depth + 1);
  if (ts.isIntersectionTypeNode(node) || ts.isUnionTypeNode(node)) {
    const parts = node.types.map((part) => membersOfType(part, scope, depth + 1));
    return parts.every((part) => part != null) ? parts.flatMap((part) => part ?? []) : null;
  }
  if (ts.isTypeReferenceNode(node) && ts.isIdentifier(node.typeName)) {
    const resolved = scope.resolveWithScope(node.typeName.text);
    return resolved == null
      ? null
      : declarationMembers(resolved.declaration, resolved.scope, depth + 1);
  }
  return null;
}

export function declarationMembers(
  declaration: Declaration,
  scope: TypeScope,
  depth: number,
): readonly ts.TypeElement[] | null {
  if (depth > 24) return null;
  if (ts.isTypeAliasDeclaration(declaration)) {
    return membersOfType(declaration.type, scope, depth + 1);
  }
  const inherited = (declaration.heritageClauses ?? []).flatMap((clause) =>
    clause.types.flatMap((heritage) => {
      const parent = ts.isIdentifier(heritage.expression)
        ? scope.resolveWithScope(heritage.expression.text)
        : undefined;
      return parent == null
        ? []
        : (declarationMembers(parent.declaration, parent.scope, depth + 1) ?? []);
    }),
  );
  return [...inherited, ...declaration.members];
}

/** Members of a named interface or alias; `null` when it is not an object shape. */
export function namedTypeMembers(name: string, scope: TypeScope): readonly ApiMember[] | null {
  const resolved = scope.resolveWithScope(name);
  if (resolved == null) return null;
  const members = declarationMembers(resolved.declaration, resolved.scope, 0);
  return members == null ? null : membersToApi(members, new Map());
}

export function toMember(
  element: ts.TypeElement,
  defaults: ReadonlyMap<string, string>,
): ApiMember {
  const doc = readDoc(element);
  const name = memberName(element.name);
  let type = "unknown";
  if (ts.isPropertySignature(element) && element.type != null) {
    type = oneLine(element.type.getText());
  } else if (ts.isMethodSignature(element)) {
    const parameters = element.parameters
      .map((parameter) => oneLine(parameter.getText()))
      .join(", ");
    type = `(${parameters}) => ${oneLine(element.type?.getText() ?? "void")}`;
  }
  return {
    name,
    type,
    optional: element.questionToken != null,
    description: doc.description,
    defaultValue: doc.defaultValue ?? defaults.get(name) ?? null,
    deprecated: doc.deprecated,
  };
}

export function typeMembers(
  node: ts.TypeNode | undefined,
  scope: TypeScope,
  defaults: ReadonlyMap<string, string> = new Map(),
): readonly ApiMember[] {
  if (node == null) return [];
  const members = membersOfType(node, scope);
  if (members == null) {
    return [
      {
        name: "(all)",
        type: oneLine(node.getText()),
        optional: false,
        description: "",
        defaultValue: null,
        deprecated: false,
      },
    ];
  }
  return membersToApi(members, defaults);
}

export function membersToApi(
  members: readonly ts.TypeElement[],
  defaults: ReadonlyMap<string, string>,
): readonly ApiMember[] {
  // Union variants repeat a member with different types (often `never` in the
  // variants that forbid it): merge them into one row.
  const merged = new Map<string, ApiMember>();
  for (const member of members) {
    if (!ts.isPropertySignature(member) && !ts.isMethodSignature(member)) continue;
    const next = toMember(member, defaults);
    if (next.name === "") continue;
    const previous = merged.get(next.name);
    if (previous == null) {
      merged.set(next.name, next);
      continue;
    }
    const types = [...new Set([...previous.type.split(" | "), ...next.type.split(" | ")])];
    const useful = types.filter((type) => type !== "never");
    merged.set(next.name, {
      name: next.name,
      type: (useful.length === 0 ? types : useful).join(" | "),
      optional: previous.optional || next.optional,
      description: previous.description === "" ? next.description : previous.description,
      defaultValue: previous.defaultValue ?? next.defaultValue,
      deprecated: previous.deprecated && next.deprecated,
    });
  }
  return [...merged.values()];
}

export function slotMembers(node: ts.TypeNode | undefined, scope: TypeScope): readonly ApiMember[] {
  return typeMembers(node, scope).map((member) => {
    const props = /^\((?:props\??: )?([^)]*)\) =>/.exec(member.type)?.[1];
    return { ...member, type: props == null || props === "" ? "—" : props };
  });
}

export function emitMembers(node: ts.TypeNode | undefined, scope: TypeScope): readonly ApiMember[] {
  if (node != null && ts.isTypeLiteralNode(node)) {
    const calls = node.members.filter(ts.isCallSignatureDeclaration);
    if (calls.length > 0) {
      return calls.map((call) => {
        const [event, ...payload] = call.parameters;
        const eventType = event?.type;
        const name =
          eventType != null &&
          ts.isLiteralTypeNode(eventType) &&
          ts.isStringLiteral(eventType.literal)
            ? eventType.literal.text
            : "(event)";
        const doc = readDoc(call);
        return {
          name,
          type: `[${payload.map((parameter) => oneLine(parameter.getText())).join(", ")}]`,
          optional: false,
          description: doc.description,
          defaultValue: null,
          deprecated: doc.deprecated,
        };
      });
    }
  }
  return typeMembers(node, scope);
}
