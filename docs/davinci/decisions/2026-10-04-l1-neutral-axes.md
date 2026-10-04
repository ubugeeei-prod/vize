# Framework-neutral L1 axes (2026-10-04)

Decision for [#6855](https://github.com/ubugeeei-prod/vize/issues/6855), a
Stage 2 design-only slice of [#6829](https://github.com/ubugeeei-prod/vize/issues/6829).
This defines the destination of the five axes. It does not report their
implementation, any new framework parser, or product acceptance.

## Present boundary

The reviewed `davinci/vize_l1/src/` surface has:

| Current API                                                                | What it actually supplies                                              | Remaining boundary                                                     |
| -------------------------------------------------------------------------- | ---------------------------------------------------------------------- | ---------------------------------------------------------------------- |
| `container::{ContainerFormat, Container, Block}` and `container::vue::Vue` | Lossless top-level Vue block spans and recoverable split errors        | Other containers and the full SFC descriptor switch remain #6837       |
| `markup::{MarkupGrammar, Vue}`                                             | A grammar identity and directive-hook associated type                  | Only Vue is registered; `VueDirectives::decompose` is a #6836 skeleton |
| `markup::profile::{Profile, Document, Component}`                          | Static host-rule constants                                             | The independent profile axis is nested under markup                    |
| `embed::{Shape, Lang, Grammar, Embed}`                                     | Typed role/language and source-coordinate records                      | Language resolution and retained typed embed trees remain #6836        |
| `parse::{parse, SurfaceParseOptions}`                                      | A lossless Vue surface tree using the L1-owned compatibility tokenizer | A five-axis registry and native provider composition are not this API  |
| `lang::moonbit`                                                            | Feature-gated source-position facts                                    | It does not establish JS/TS language-provider completion               |

The generic markup lexer is opt-in. Ownership of a moved compatibility
parser is distinct from native shared-artifact acceptance. The source audit
here does not count legacy-backed product paths as native work.

## Axis ownership

Keep the five axes independently selected. The registry validates their
combination once and gives the parser static capabilities; there is no
per-token registry lookup and no dynamically tagged universal syntax tree.

| Axis      | Question it answers                                                 | Destination in L1                                            | It must not decide                                               |
| --------- | ------------------------------------------------------------------- | ------------------------------------------------------------ | ---------------------------------------------------------------- |
| Container | Where are the file's authored regions and block attributes?         | `container::{plain, vue, ...}` with shared span-only records | Meaning of `defineProps`, runtime target, or directive semantics |
| Markup    | Which concrete delimiters, tags and directive spellings are parsed? | `markup::{html_core, vue, pug, jsx, ...}`                    | Binding resolution, update effects, or runtime helpers           |
| Profile   | Which host-document rules apply to markup?                          | `profile::{document, component}`                             | Vue version or JS versus TS                                      |
| Lang      | Which grammar parses each script and embed?                         | `lang::{js, ts, ...}`; shared embed roles stay in `embed`    | Framework macro meaning or DOM/Vapor selection                   |
| Framework | Which feature composition supplies framework syntax hooks?          | `framework::vue::{feature, version, quirks, jsx}`            | HTML source preservation or language syntax ownership            |

These are proposed module destinations. `plain` is a pass-through container
selection, not a new parse or allocated copy. `pug` already has its own
parser; moving its module is implementation work. Other frameworks are
reserved design examples, without placeholder parsers or supported claims.

`framework::vue` in L1 supplies only syntax-level features: directive
spellings, interpolation-mode changes such as `v-pre`, and dialect quirks.
Macro recognition and binding meaning belong to L2. A Vue version is a const
feature composition, and quirks are orthogonal to that version. Container
`vue` and markup `vue` are syntax choices, not an implicit Vue runtime.

A framework-neutral core owns spans, tokens, holes, authored bytes and
structural containers without importing axis implementations. Registries
may name all axes; adapters may consume core. Each axis adapter imports core
and the explicit capabilities it needs, rather than another adapter's
private implementation. The existing #6841 import-path gate must check this
direction when the module move lands.

## Composition and diagnostics

A file has one resolved JS/TS host-language selection and may contain
several syntax regions. Every region keeps its own markup selection,
optional profile, source range and embed role. A pure JS/TS file has no
markup profile; do not pretend its program is an HTML component.

The following are intended compositions, not current support assertions:

| Input                         | Container                   | Markup regions                                             | Profile                      | Language                             | Framework syntax features              |
| ----------------------------- | --------------------------- | ---------------------------------------------------------- | ---------------------------- | ------------------------------------ | -------------------------------------- |
| Vue SFC                       | Vue                         | Vue template, or its explicitly selected template language | Component                    | JS or TS resolved from script blocks | Selected Vue version and quirks        |
| In-DOM Vue / petite-vue       | Plain                       | Vue on HTML core                                           | Document                     | JS or TS from configuration          | Selected Vue document feature set      |
| Standalone JS/TS              | Plain                       | None                                                       | None                         | JS or TS                             | None, unless explicitly requested      |
| Vue JSX/TSX                   | Plain or a Vue script block | JSX within the retained host-language AST                  | Component at the UI boundary | JS or TS with JSX syntax enabled     | Selected Vue JSX feature composition   |
| Future non-Vue component file | Its registered container    | Its registered markup                                      | Explicit host profile        | Its registered language              | Its own registered feature composition |

JSX is parsed as part of the host-language AST once. `markup::jsx` is its
lossless UI projection and syntax adapter, not a second parser or an HTML
repair pass. JSX closing-tag constraints belong to that grammar. HTML-only
profile constants are applied only by grammars supporting HTML host rules;
`Component` must not silently make JSX accept malformed closing tags.

For Vue, template embeds follow the resolved script language. Conflicting
`<script>` and `<script setup>` language declarations produce a source-bound
diagnostic before semantic lowering; no provider quietly reparses as JS.
Missing syntax capabilities produce a typed hole or an explicit unsupported
diagnostic. An opaque expression remains opaque in downstream levels.
Uncertain container split ranges are not compilation or acceptance evidence.

The five axes do not encode a JSX execution model. Re-render/run-once and
interpretation depth remain L2 semantics under #6885/#6859. Neither the file
extension nor the L1 markup selection silently chooses an L4 target.

## Concrete migrations

1. Move `markup::profile` to `profile` in a move-only commit, then update
   imports separately. Preserve the `Document` and `Component` constants and
   their present meaning; no behavior change hides in a module move.
2. Place the shared directive-hook capability in the markup boundary, but
   move Vue-specific prefixes (`v-`, `:`, `.`, `@`, `#`) and decomposition
   payloads to the Vue syntax adapter. Neutral hook results carry authored
   spans and typed dialect syntax, not a closed Vue prefix enum.
3. Move `experimental_in_tag_comments` from a generic parse-switch record
   into Vue's feature/quirk composition when the new registry replaces that
   entry. Preserve explicit option compatibility on published product APIs.
4. Resolve container, region grammar, profile, language and framework
   capabilities once. Build embeds from authored spans and the decode map;
   retain the provider tree and comments in the original artifact. Add no
   reparses, serialization, or extra pipeline stage.
5. Register JS/TS, Vue dialect and JSX/TSX combinations before claiming Vue
   Fes coverage. Register future axes only with their actual provider;
   unimplemented combinations stay unsupported and visible.

Implementation remains tracked by #6835 (markup), #6836 (embeds), #6837
(container), #6841 (features/registries), #6843 (MoonBit) and #6844 (script
artifacts). This design closes only #6855 after review and merge.

## Implementation acceptance

- Preserve `render(tree) == source` for valid and malformed input, comments,
  typed missing tokens, authored projections, UTF-8 spans and CRLF.
- Test Document versus Component rules directly; test JSX grammar rules
  separately. Include Vue versions/quirks and configured language mismatch
  diagnostics, rather than testing only the default SFC combination.
- Verify core-to-axis imports are absent and no normal/build dependency from
  `davinci/` to `crates/` is introduced. A dev-only legacy differential oracle
  never supplies product facts.
- Demonstrate shared same-run retained syntax and downstream diagnostics for
  JS/TS and JSX/TSX. Product selection still requires each fix-history gate;
  #6880 remains the compiler gate.
- Run exact-head Actions and the protected merge queue with unchanged
  instruction budgets. A design, skeleton or moved parser earns no native
  acceptance credit.
