// The tests workspace declares the source-pinned TypeScript dependency.
// Keep this bridge type-only; the campaign loads its admitted API explicitly.
import type * as ts from "typescript";

export type TypeScriptApi = typeof ts;
export type { Diagnostic, SourceFile } from "typescript";
