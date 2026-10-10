# Native JSX nested expression roots

Issue: [#8441](https://github.com/ubugeeei-prod/vize/issues/8441).
Source dependency: the native slot annotation repair in
[#8428](https://github.com/ubugeeei-prod/vize/pull/8428).

The unchanged `check_tsx_story_allows_slot_object_with_kebab_update_handler`
input produces twelve native parse errors with explicit JSX checking. Native
lowering retains an `Array.from` callback as a plain expression, leaving its
inner JSX syntax in the plain virtual `.ts` file. A second original direct JSX
expression inside the typed slot body exposes the same defect. The original
fixture bytes, complete twelve-error report, argv and process outcome are
retained in the differential corpus.

The existing OXC visitor now retains previously unlowered JSX nodes from plain
child expressions, plain slot-body expressions and raw conditional arms. Their
actual AST ranges and lowered units reach the existing Canon renderer. Only
those exact ranges are replaced inside the unchanged surrounding call and
callback bytes, preserving lexical scope and fine source mappings. Scoped style
expressions are retained on that same native root, rather than drained into an
outer callback scope. No parse stage, serialized representation or synthetic
provider is added. Sorted range
windows exclude unrelated sibling roots before examining expression bytes.

An earlier separate commit mechanically extracts the unchanged rendering
helpers. The behavior repair is a separate commit. Normal compiler lowering
does not collect these additional analysis roots.

## Qualification

- The original authored input and clean whole-report expectation remain
  unchanged. The mandatory native CLI law compares every file diagnostic,
  compiler option, program member, count, stdout, stderr and exit outcome with
  explicit JSX enabled.
- Independent stock TypeScript 6.0.3 and public native TypeScript 7.0.2 accept
  the original conditional `Array.from`, direct `Array.from` and structural
  `.map` controls. The pre-fix source CLI rejects the first two with TS1161;
  all three complete repaired CLI packets must be clean.
- A broken number callback member reports only TS2339 at authored line 28,
  column 34, independently confirmed by both original engines. Multiple nested
  roots, adjacent roots, Unicode/CRLF coordinates, reverse mappings and a
  JSX-looking string literal have native source controls.
- A nested scoped-style interpolation retains its callback owner. The valid
  input has a clean complete packet; replacing only the numeric interpolation
  with `i.toUpperCase()` produces only TS2339 at line 2, column 121, matching
  both original engines. The additional pre-repair TS2304 scope regression and
  its whole process evidence are frozen in `native-style-scope-red.tar.gz`
  (SHA-256 `4d96f84cf71a343c0127171aef0718f4707e30d8ee2fcc3c84402f59741d9b59`).
  This local red receipt explicitly records an unsealed build recipe; exact
  committed-source qualification is supplied separately by Actions.
- Complete normal VDOM and Vapor module text, component text, preambles,
  metadata, styles, diagnostics and maps are frozen from the parent production
  implementation and compared without normalization. Existing snapshots stay
  unchanged.
- Current source Actions, protected native Stack admission, actual merge and a
  subsequent installed release remain required. Local acceptance alone does
  not qualify the proposed default switch.

Broader JSX attributes/spreads, JSX inside recognized conditional tests or loop
sources, generic/multiple/rest slot parameters and full JSX ownership remain
unfinished under #1497. This bounded repair makes no throughput or 10x claim.
