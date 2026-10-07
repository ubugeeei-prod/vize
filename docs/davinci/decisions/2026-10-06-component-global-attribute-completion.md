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

## Protected audit failure and next qualification

The [paired failure decision](https://github.com/ubugeeei-prod/vize/issues/8015#issuecomment-6033322697) preserves the actual protected GHSA-6qxp-vccf-f47h MCP SDK audit failure. The failed candidate was removed and the unchanged 7c6 source was restored Ready/offqueue after source-grounded diagnosis; the later concrete reference correction below returns this PR to Draft/offqueue. Retain the whole current source success separately; unfinished protected work is failed/canceled, and fresh actual signed dependency-main composition plus exact-source/protected qualification remains required. No dependency waiver, output change or runtime transfer.

## Complete real-project reference correction

The [paired reference decision](https://github.com/ubugeeei-prod/vize/issues/8015#issuecomment-6033682081) retains the actual protected #8140 / 80aa Tooling2 failure. Its complete 997,122-byte log has SHA256 `570f2b9a149d38b6034ced2d1cb15ac1ba2da86e21738cf3c0cfddd7fda19fff`; ant-design-vue, create-vue, vue-router and vue-select expose the common candidates absent from their old ranked banks. This is a missing reference update in #8131, with no parser or performance cause inferred.

Retain the entire old 261,842-byte ecosystem registry as `real-project-before.json` (SHA256 `cfce6418b777f0668063c424ae74c74f4b223147046abf3f69e0ff4effb3c8c5`). Independently author all 51 complete current component banks from their unchanged 28 directive/event items, the nine fixed historical common items, and the full old declared-property/event suffix. Only a declared property suppresses a same-label common item; seven banks have one such collision. Counts and every rank derive from the complete authored bank, never runtime recapture. Every source, revision, config, dependency edit, other oracle, gate and budget stays exact, including all six pinned event-declaration inputs and historical event banks. A whole-registry pure guard verifies this complete scope and the original/common SHA256 authorities; the existing runtime checks still require every ranked label and complete repaired bank equality.

All three pure registry laws pass without Vize or native execution. Actual 7c6 source Actions previously passed the four new Rust laws, thirty whole stdio responses (proof `6c5571154ca006356386d195bc75042274d94f1bcd66b2cb6b9c7a08ff4655ef`) and45 original answers across seven sessions (proof `862d7f241226c19620f450ee4a644bba7d6aaa1cba9a3501bf8bed0cb88fa010`). Those results remain historical for the new reference correction. The PR is Draft/offqueue while this correction is prepared; existing #8148 must actually deliver the MCP repair before one genuine main composition and fresh exact-head source/native and protected qualification. Production, full original thirty-response and45-answer controls, opaque resolve authority and instruction ceilings remain unchanged. Installed/public acceptance remains unclaimed and the other original #8015 cases remain open.
