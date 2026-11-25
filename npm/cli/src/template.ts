import { createRequire } from "node:module";
import type {
  TemplateAstOptions,
  TemplateAstResult,
  TemplateAttributeNode,
  TemplateCommentNode,
  TemplateDirectiveNode,
  TemplateElementNode,
  TemplateInterpolationNode,
  TemplateRootNode,
  TemplateSimpleExpressionNode,
  TemplateTextNode,
} from "@vizejs/native";

export type {
  TemplateAstDiagnostic,
  TemplateAstOptions,
  TemplateAstResult,
  TemplateAttributeNode,
  TemplateCommentNode,
  TemplateDirectiveNode,
  TemplateElementNode,
  TemplateInterpolationNode,
  TemplatePosition,
  TemplateRootNode,
  TemplateSimpleExpressionNode,
  TemplateSourceLocation,
  TemplateTextNode,
} from "@vizejs/native";

/** A node in an element's or the root's `children`. */
export type TemplateChildNode =
  | TemplateElementNode
  | TemplateTextNode
  | TemplateCommentNode
  | TemplateInterpolationNode;
/** A node in an element's `props`. */
export type TemplatePropNode = TemplateAttributeNode | TemplateDirectiveNode;
/** Any node of the template tree, discriminated by `type`. */
export type TemplateNode =
  | TemplateRootNode
  | TemplateChildNode
  | TemplatePropNode
  | TemplateSimpleExpressionNode;

const require = createRequire(import.meta.url);

/**
 * Parse a Vue template into its full authored tree: every element, text,
 * comment, interpolation, attribute and directive in place, each with a
 * `loc` whose offsets and columns count UTF-16 code units, so
 * `source.slice(loc.start.offset, loc.end.offset)` is the node's text.
 *
 * Parser diagnostics are returned in `errors` and `warnings`, never thrown.
 */
export function parseTemplateAst(source: string, options?: TemplateAstOptions): TemplateAstResult {
  // Loaded on first use, so `vize` config helpers never load the native binding.
  const native = require("@vizejs/native") as typeof import("@vizejs/native");
  return native.parseTemplateAst(source, options);
}
