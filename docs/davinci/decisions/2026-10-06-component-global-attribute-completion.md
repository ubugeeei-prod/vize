# Common attribute completion on component tags

Issue: [#8015](https://github.com/ubugeeei-prod/vize/issues/8015). Reporter: ubugeeei (public account 71201308). Paired decision: [6019220960](https://github.com/ubugeeei-prod/vize/issues/8015#issuecomment-6019220960).

## Scope and source authority

The original report combines common HTML attributes on component tags, arbitrary `data-*` names, Vue instance/template globals, JavaScript globals, and generic component prop hover. This change handles the existing nine common attribute candidates only: `id`, `class`, `style`, `title`, `role`, `tabindex`, `aria-label`, `ref`, and `key`. Issue #8015 remains open for the other original cases. It makes no claim that generic native hover is fixed or that every global HTML/ARIA attribute is enumerated.

`completion/template/native.rs` previously refused every component tag before returning its existing common table. Return that same table for components and retain element-specific attributes only on native tags. Assemble the already existing component surface once and suppress a fallback common attribute only when the surface actually declares that property name. The complete declared property type, documentation, ordering, insertion and resolve data retain authority. Event/modifier candidates and their existing union remain unchanged.

Native component documentation still attaches only to declared component properties. The same component surface already evaluated by its guard identifies those properties; newly exposed common fallback items retain the complete existing HTML documentation and snippets. There is no extra component export, native request, parse, cache, pipeline stage, hover work, provider fallback or type authority inference.

## Original custody and whole controls

The differential directory `component-global-attributes-8015` retains the complete public Issue body and all four original Child, Comp, List and App SFC blocks. The original common-class position is independently derived and fixed at zero-based 7:11. Every original reported position, including the pending expression and generic-hover cases, is recorded. The supplied full tsconfig is authored from the report's prose; the report supplied no exact package/config bytes. Hosted dependency versions are recorded separately and do not imply the original Vue 3.5/TypeScript 5 environment was replayed.

The complete nine-item bank comes from the unchanged historical native-button oracle, and each new property item keeps every original field. Four new Rust laws compare the complete original class response, the full common bank and declared property authority, unchanged native-button attributes and value/event/slot refusal, and the complete declared generic-prop synchronous hover. The latter checks existing source metadata only; it grants no native generic instantiation acceptance.

One normally discovered, source-bound stdio test sends the full original and authored controls with native typechecking disabled and enabled. Each session requires the actual physical Vue package and pinned native runtime, preserving complete dependency/config bytes and hashes. It compares thirty complete responses: all nine globals, native-only property refusal, static-value refusal, dirty Unicode/CRLF and restore, unchanged resolution and closed-document resolution. Every result or request error, publication and stderr observation is saved before assertions. Full initialization and terminal shutdown responses are retained; cleanup removes only the authored test workspace.

The existing #8005 event test keeps its entire immutable 29-item original oracle, native item/mirror witness, both complete input sources, full45 response/hover law and every prefix/control bank. Its two component opening-tag expected arrays explicitly insert only the nine whole common items before declared properties, becoming38 items; nothing is filtered from actual responses or silently regolded. All event/native/resolve fields and original historical banks remain exact.

## Qualification and remaining work

This is prepared source. Rust compilation, all four new laws, thirty real stdio results, the unchanged event/native controls, protected full suites and104 instruction windows, signed merge and installed/public release acceptance are pending. Static formatting, inventory or source review grant no runtime, speed or release credit. Root alone publishes.

The existing incomplete-prefix text-edit range behavior is unchanged: this slice adds candidates using the same opening-tag range/snippet contract as native elements. Arbitrary data attributes, additional ARIA names, expression globals and native generic prop hover need separate source diagnosis and complete response qualification. Do not close the entire Issue based on this common-table slice.

## Actual security-main replay

The [paired replay decision](https://github.com/ubugeeei-prod/vize/issues/8015#issuecomment-6020554889) binds actual signed security main `ef84821d30fa0d8538b2472fef34418e75380523`. Preserve every production, source, full oracle, opaque native item and resolve field during the single genuine replay. All incoming decisions remain intact.

Previous source `0c5815aa7d5e4fa5ef9a607a1ef6624bf2376218` passed four new Rust laws, thirty whole stdio responses (proof SHA256 `c6db79916874e748b9095cf5eac17b2de9047601262b1afc6069c9797728c818`) and43 whole completion arrays plus two complete hover answers across seven original sessions (proof SHA256 `8dea17434113e6c04742336011b61c210cfabfb6f891d79a066d607e352e2ea6`). These remain historical exact-source evidence after replay. Fresh exact-head source/native Actions, protected full suites and unchanged instruction ceilings, actual signed merge and installed/public acceptance remain required. The remaining original Issue cases stay open.
