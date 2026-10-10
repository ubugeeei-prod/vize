//! Generating plain `.ts` virtual TypeScript for `.jsx`/`.tsx` Vue components
//! (issue #1497, opt-in).
//!
//! This is the JSX/TSX parallel to [`super::vue_codegen`]. It is reached only
//! when the user explicitly enables `typeChecker.jsxTypecheck` (default off):
//! mixed Vue/React repositories may contain React `.tsx` files that must *not*
//! be type-checked as Vue JSX.
//!
//! # Authoring convention (#1502)
//!
//! A Vize JSX/TSX component is a plain function whose parameters carry the
//! component contract, with no macros and no runtime validation:
//!
//! ```tsx
//! const Comp = (
//!     props: { msg: string; count?: number },
//!     { emit }: Ctx<{ change: [value: number] }>,
//! ) => <div>{props.msg}</div>;
//! ```
//!
//! The **typed first parameter is the props type**; the optional typed second
//! parameter is the `Ctx<Emits, Slots>` context. Defaults are plain
//! destructuring defaults.
//!
//! # Why a textual JSX → plain-TS lowering
//!
//! `vize_canon` virtual TypeScript stays plain `.ts` (never JSX-format virtual
//! documents — standing directive). A `.tsx` Vue component is, syntactically,
//! already valid TypeScript *except* for the JSX elements themselves. So this
//! pass keeps every non-JSX byte verbatim (component functions, the typed props
//! parameter, the setup body) and replaces only the JSX render roots with a
//! synthesized plain-TS expression that re-lists every embedded JSX expression.
//!
//! The result type-checks exactly what this first cut promises:
//! - the **typed first parameter** stays verbatim, so every `props.X` access is
//!   checked against the declared props type;
//! - the **typed second parameter** (`{ emit, slots }: Ctx<Emits, Slots>`) stays
//!   verbatim, and an ambient [`Ctx<Emits, Slots>`](CTX_HELPER) type is injected
//!   so `emit(name, ...args)` checks `name` against `keyof Emits` and the payload
//!   against the tuple `Emits[name]` (Vue's emits-as-tuple convention), and
//!   `slots` is typed as `Slots`;
//! - the **setup-scope** statements above the `return <jsx/>` stay verbatim, so
//!   their declarations and uses are checked;
//! - each **JSX expression** (`{props.msg}`, `class={cls}`, `{count + 1}`, …) is
//!   re-emitted as real TypeScript at — and source-mapped back to — its original
//!   byte range, so a wrong type inside a JSX expression is reported at the right
//!   location;
//! - **component tags and props** are preserved as type-only calls. Imported
//!   SFC constructors reuse their generated `$props`/raw-props contract, while
//!   local functional components reuse their first parameter. This checks
//!   required, excess, static, bound, kebab-case, listener, and spread props;
//! - **directive expressions** are checked too (#1497): a `v-model` binding
//!   target is re-emitted as an assignment to itself, so binding to a `const`,
//!   a `readonly`/computed value, or a non-lvalue is reported at the binding; a
//!   `v-for` (idiomatic `items.map(…)`) body is re-emitted *inside* the `.map()`
//!   callback so the loop aliases bind with their inferred element types; and
//!   `v-show`/`v-if` conditions, directive arg/value expressions, and event
//!   handlers are re-emitted as plain reads.
//! - **style-block expressions** are checked too (#1497): a `<style scoped>` JSX
//!   block (#1495) is extracted out of the rendered children, but its
//!   template-literal interpolations (`${expr}`, e.g. `color: ${props.color}`)
//!   reference script values and are re-emitted through the same sink and
//!   component scope as that root's JSX expressions, so a wrong type inside a
//!   style interpolation is reported at the interpolation.
//!
//! Deferred (see issue #1497): CSS `v-bind(expr)` references inside a
//! `<style scoped>` block (their spans live in cooked CSS text whose offsets no
//! longer map to source bytes, so recovering them needs dedicated extraction);
//! the stateful `defineComponent(() => () => VNode)` form; and full source-map
//! fidelity for the synthesized wrapper scaffolding.

use std::path::Path;

use vize_atelier_jsx::{JsxDiagnostic, JsxLang, lower_source_for_typecheck};
use vize_carton::{Allocator, String as CompactString, cstr};

use crate::batch::error::CorsaResult;
use crate::batch::{Diagnostic, SfcBlockType};
use crate::virtual_ts::VizeMapping;

use super::diagnostics::diagnostic_for_offset;

mod collect;
mod component;
mod render;
mod slot;

use collect::{collect_root_expressions, collect_style_expressions, expr_of};
use render::{push_mapped_expr, render_plain_ts};

/// The generated plain-`.ts` virtual file for one `.jsx`/`.tsx` source.
pub struct GeneratedJsxFile {
    pub code: CompactString,
    pub mappings: Vec<VizeMapping>,
    pub diagnostics: Vec<Diagnostic>,
}

/// Name of the synthesized helper that swallows every re-emitted JSX
/// expression. Declaring it ambient and `any`-returning lets each argument be
/// type-checked independently while the whole call stays a valid render return.
const JSX_EXPR_SINK: &str = "__vize_jsx_expr__";

/// Ambient `Ctx<Emits, Slots>` type injected at module scope so the typed
/// second parameter of a Vize JSX/TSX component (`{ emit, slots }: Ctx<…>`)
/// resolves and type-checks (#1502).
///
/// `emit` reuses the very same emits-as-tuple convention as the `.vue` path's
/// `defineEmits<E>()` (see `crate::virtual_ts::helpers`): the `__EmitFn<E>`
/// alias resolves `E = { change: [value: number] }` to an event overload, so
/// `emit('change', 1)`
/// checks the payload against the declared tuple and an unknown event name or a
/// wrong payload is reported at the `emit(...)` call site. `slots` is typed as
/// the second type argument so slot access/usage type-checks. Both fall back to
/// `{}` when omitted (`Ctx`, `Ctx<Emits>`). The type is purely ambient and fully
/// erased — no runtime is emitted.
///
/// Kept self-contained (the emit trio is duplicated rather than pulling in the
/// broader Vue helper blob) so JSX/TSX virtual TS never depends on resolving the
/// `vue` module, matching the minimal, fully-erased intent of this path.
const CTX_HELPER: &str = "type __EmitShape<T> = T extends (...args: any[]) => any ? T : T extends Record<string, any> ? { [K in keyof T]: T[K] extends (...args: infer A) => any ? A : T[K] extends any[] ? T[K] : any[]; } : Record<string, any[]>;\n\
type __EmitArgs<T, K extends keyof T> = T[K] extends any[] ? T[K] : any[];\n\
type __EmitFn<T, __S = __EmitShape<T>, __K extends keyof __S & string = keyof __S & string, __U = { [K in __K]: (event: K, ...args: __EmitArgs<__S, K>) => void }[__K]> = __S extends (...args: any[]) => any ? __S : [__K] extends [never] ? (event: never, ...args: any[]) => void : (__U extends unknown ? (fn: __U) => void : never) extends (fn: infer __I) => void ? __I : never;\n\
type Ctx<Emits = {}, Slots = {}> = { emit: __EmitFn<Emits>; slots: Slots; attrs: Record<string, unknown>; };\n";

/// A dynamic JSX expression recovered from the lowered tree: its original source
/// text plus the byte range it occupied in the `.jsx`/`.tsx` source.
#[derive(Clone)]
struct JsxExpr {
    content: CompactString,
    start: u32,
    end: u32,
}

/// One re-emitted unit recovered from a lowered JSX root, in source order.
///
/// The render pass turns these into the arguments of a `__vize_jsx_expr__(…)`
/// call. Most are plain [`Expr`](JsxEmit::Expr) reads, but two directive forms
/// need structured re-emission so their checks match Vue semantics:
///
/// - [`ModelTarget`](JsxEmit::ModelTarget): a `v-model` binding target re-emitted
///   as an assignment to itself so TypeScript checks the target is a writable
///   lvalue (binding to a `const`, a `readonly`/computed value, or a non-lvalue
///   expression is reported at the binding).
/// - [`ForScope`](JsxEmit::ForScope): a `v-for` (idiomatic `items.map(…)`) whose
///   body is re-emitted *inside* the `.map()` callback so the loop aliases are
///   bound with their inferred element types — both fixing a spurious
///   "Cannot find name '<alias>'" and checking the body against the real type.
enum JsxEmit {
    /// A plain dynamic expression (interpolation, bound attribute, directive
    /// value, `v-if`/`v-show` condition, event handler, …).
    Expr(JsxExpr),
    /// A `v-model` binding target, re-emitted as `(<lvalue> = <lvalue>)`.
    ModelTarget(JsxExpr),
    /// A component tag plus its authored JSX attributes. Unlike intrinsic
    /// elements, these participate in the imported/local component's props
    /// contract and therefore cannot be reduced to value expressions alone.
    Component(component::JsxComponent),
    /// A scoped-slot scope: the slot's binding pattern plus the body units
    /// evaluated with that pattern in scope, typed from the host component's
    /// declared `$slots`.
    SlotScope(slot::JsxSlotScope),
    /// A `v-for` scope: the iterated `source` plus the alias patterns and the
    /// body units evaluated with those aliases in scope.
    ForScope {
        source: JsxExpr,
        value_alias: Option<JsxExpr>,
        key_alias: Option<JsxExpr>,
        body: Vec<JsxEmit>,
    },
}

/// Lower a `.jsx`/`.tsx` Vize component to plain virtual TypeScript.
pub fn generate_jsx_virtual_ts(
    path: &Path,
    source: &str,
    lang: JsxLang,
) -> CorsaResult<GeneratedJsxFile> {
    let allocator = Allocator::new();
    let lowered = lower_source_for_typecheck(&allocator, allocator.as_oxc(), source, lang);

    // Collect every outermost JSX root's byte range together with the dynamic
    // expressions inside it, in source order.
    let mut roots: Vec<(u32, u32, Vec<JsxEmit>)> = Vec::with_capacity(lowered.roots.len());
    for root in &lowered.roots {
        let mut exprs = Vec::new();
        collect_root_expressions(&root.root, &mut exprs);
        // The `<style scoped>` block is extracted out of the rendered children
        // (#1495), so its template-literal interpolations (`${expr}`) never reach
        // the lowered tree above. Append them as plain reads so they type-check
        // against the very same component scope (props, setup vars, ctx) as the
        // root's JSX expressions, source-mapped back to their `.tsx` ranges
        // (#1497).
        collect_style_expressions(&root.scoped_style_exprs, &mut exprs);
        roots.push((root.root.loc.span.start, root.root.loc.span.end, exprs));
    }
    // Outermost roots never overlap and are produced in source order, but guard
    // the rewrite against any accidental disorder.
    roots.sort_by_key(|(start, _, _)| *start);

    let mut diagnostics = Vec::new();
    for diagnostic in &lowered.diagnostics {
        if !diagnostic.is_error() {
            continue;
        }
        diagnostics.push(diagnostic_for_offset(
            path,
            source,
            diagnostic.start,
            jsx_parse_message(diagnostic),
            SfcBlockType::Script,
        ));
    }

    let (code, mappings) = render_plain_ts(source, &roots);

    Ok(GeneratedJsxFile {
        code,
        mappings,
        diagnostics,
    })
}

fn jsx_parse_message(diagnostic: &JsxDiagnostic) -> CompactString {
    cstr!("JSX parse error: {}", diagnostic.message)
}

#[cfg(test)]
#[path = "jsx_codegen_tests.rs"]
mod tests;
