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

use vize_atelier_jsx::{JsxDiagnostic, JsxLang, lower_source};
use vize_carton::{Allocator, String as CompactString, cstr};

use crate::batch::error::CorsaResult;
use crate::batch::{Diagnostic, SfcBlockType};
use crate::virtual_ts::VizeMapping;

use super::diagnostics::diagnostic_for_offset;

mod collect;
mod component;
mod slot;

use collect::{collect_root_expressions, collect_style_expressions, expr_of};

/// The generated plain-`.ts` virtual file for one `.jsx`/`.tsx` source.
pub(super) struct GeneratedJsxFile {
    pub(super) code: CompactString,
    pub(super) mappings: Vec<VizeMapping>,
    pub(super) diagnostics: Vec<Diagnostic>,
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
pub(super) fn generate_jsx_virtual_ts(
    path: &Path,
    source: &str,
    lang: JsxLang,
) -> CorsaResult<GeneratedJsxFile> {
    let allocator = Allocator::new();
    let lowered = lower_source(&allocator, allocator.as_oxc(), source, lang);

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
        roots.push((root.root.loc.start.offset, root.root.loc.end.offset, exprs));
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

/// Build the plain-`.ts` text and its source mappings.
///
/// Every byte outside a JSX render root is copied verbatim; each render root is
/// replaced by `__vize_jsx_expr__(<unit>, <unit>, …)`, with each re-emitted
/// expression mapped back to its original byte range.
fn render_plain_ts(
    source: &str,
    roots: &[(u32, u32, Vec<JsxEmit>)],
) -> (CompactString, Vec<VizeMapping>) {
    let mut out = CompactString::default();
    let mut mappings: Vec<VizeMapping> = Vec::new();

    // Ambient helpers: declared once at module scope so the re-emitted JSX
    // expressions and the synthesized render returns both type-check.
    out.push_str("declare function ");
    out.push_str(JSX_EXPR_SINK);
    out.push_str("(...args: unknown[]): any;\n");
    // Ambient `Ctx<Emits, Slots>` so the typed second parameter resolves and the
    // `emit`/`slots` usages in the setup body and JSX expressions type-check.
    out.push_str(CTX_HELPER);
    if roots
        .iter()
        .any(|(_, _, emits)| emits.iter().any(emit_contains_component))
    {
        out.push_str(component::HELPER);
    }

    let mut cursor = 0usize;
    for (start, end, emits) in roots {
        let start = (*start as usize).min(source.len());
        let end = (*end as usize).min(source.len());
        if start < cursor {
            // Overlapping/disordered root: skip defensively.
            continue;
        }
        // Verbatim prefix (component function header, typed params, setup body).
        // Emit an identity mapping so diagnostics in this region (e.g. a wrong
        // `props.X` use in the setup body) map back to their true source range
        // despite the prepended ambient-helper preamble.
        push_verbatim(&mut out, &mut mappings, source, cursor, start);

        render_sink_call(&mut out, &mut mappings, emits);
        cursor = end.max(start);
    }
    // Trailing verbatim suffix (e.g. `export default Comp;`).
    push_verbatim(&mut out, &mut mappings, source, cursor, source.len());

    (out, mappings)
}

/// Emit `__vize_jsx_expr__(<unit>, <unit>, …)` for one render scope, recursing
/// into `v-for` bodies so their loop aliases stay in scope.
fn render_sink_call(out: &mut CompactString, mappings: &mut Vec<VizeMapping>, emits: &[JsxEmit]) {
    out.push_str(JSX_EXPR_SINK);
    out.push('(');
    for (index, emit) in emits.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        render_emit(out, mappings, emit);
    }
    out.push(')');
}

/// Re-emit one [`JsxEmit`] unit as a `__vize_jsx_expr__` argument, recording the
/// source mappings that point its diagnostics back at the original JSX.
fn render_emit(out: &mut CompactString, mappings: &mut Vec<VizeMapping>, emit: &JsxEmit) {
    match emit {
        JsxEmit::Expr(expr) => push_mapped_expr(out, mappings, expr),
        JsxEmit::ModelTarget(expr) => {
            // `v-model` binds a writable lvalue. Re-emit the target as an
            // assignment to itself so TypeScript reports binding to a `const`,
            // `readonly`/computed value, or a non-lvalue at the binding. Only the
            // left-hand side is mapped: assignability and name-resolution errors
            // land on the LHS, so the unmapped RHS copy never double-reports.
            out.push('(');
            push_mapped_expr(out, mappings, expr);
            out.push_str(" = ");
            out.push_str(&expr.content);
            out.push(')');
        }
        JsxEmit::Component(component) => component::render(out, mappings, component),
        JsxEmit::SlotScope(scope) => {
            // `__vize_jsx_component_slot__(<Host>, "<name>", (<pattern>) =>
            //  __vize_jsx_expr__(<body…>))`: the body is re-emitted inside the
            // callback so the slot pattern binds with the payload type declared
            // by the host component's `$slots`. `render_open` leaves the helper
            // call open; the trailing `)` below closes it.
            slot::render_open(out, mappings, scope);
            render_sink_call(out, mappings, scope.body());
            out.push(')');
        }
        JsxEmit::ForScope {
            source,
            value_alias,
            key_alias,
            body,
        } => {
            // `(<source>).map((<value>, <key>) => __vize_jsx_expr__(<body…>))`:
            // the body is re-emitted inside the callback so the loop aliases bind
            // with their inferred element types. The `.map` scaffolding is left
            // unmapped (its diagnostics, if any, point at the mapped `source`).
            out.push('(');
            push_mapped_expr(out, mappings, source);
            out.push_str(").map((");
            if let Some(value) = value_alias {
                push_mapped_expr(out, mappings, value);
            } else {
                out.push_str("__vize_v");
            }
            if let Some(key) = key_alias {
                out.push_str(", ");
                push_mapped_expr(out, mappings, key);
            }
            out.push_str(") => ");
            render_sink_call(out, mappings, body);
            out.push(')');
        }
    }
}

fn emit_contains_component(emit: &JsxEmit) -> bool {
    match emit {
        // A slot scope is only ever produced under a component host, and its
        // opening call needs the same helper block.
        JsxEmit::Component(_) | JsxEmit::SlotScope(_) => true,
        JsxEmit::ForScope { body, .. } => body.iter().any(emit_contains_component),
        JsxEmit::Expr(_) | JsxEmit::ModelTarget(_) => false,
    }
}

/// Copy a re-emitted expression's text into `out` and record the mapping from
/// its generated range back to its original `.jsx`/`.tsx` byte range.
fn push_mapped_expr(out: &mut CompactString, mappings: &mut Vec<VizeMapping>, expr: &JsxExpr) {
    let gen_start = out.len();
    out.push_str(&expr.content);
    let gen_end = out.len();
    mappings.push(VizeMapping {
        gen_range: gen_start..gen_end,
        src_range: expr.start as usize..expr.end as usize,
        sub_spans: Vec::new(),
    });
}

/// Copy `source[src_start..src_end)` verbatim into `out`, recording an identity
/// mapping (generated range -> original range) for diagnostics in the region.
fn push_verbatim(
    out: &mut CompactString,
    mappings: &mut Vec<VizeMapping>,
    source: &str,
    src_start: usize,
    src_end: usize,
) {
    if src_start >= src_end {
        return;
    }
    let gen_start = out.len();
    out.push_str(&source[src_start..src_end]);
    let gen_end = out.len();
    mappings.push(VizeMapping {
        gen_range: gen_start..gen_end,
        src_range: src_start..src_end,
        sub_spans: Vec::new(),
    });
}

#[cfg(test)]
#[path = "jsx_codegen_tests.rs"]
mod tests;
