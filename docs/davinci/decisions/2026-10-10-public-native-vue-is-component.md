# Public native Vue component selectors

Issue: [#8328](https://github.com/ubugeeei-prod/vize/issues/8328).
Reporter: Danila Poyarkov (`dannote`, `dev@dannote.net`).
Repair base: signed actual main `4c2bebb9a586e3eaab698b41e5b58ed5f8181852`.

The original [component-selector repair](./2026-10-09-native-vue-is-component.md)
passed its compiler entry regressions. Fresh installed v0.439.0 acceptance
[38018568206](https://github.com/ubugeeei-prod/vize/actions/runs/38018568206),
[job 114114274775](https://github.com/ubugeeei-prod/vize/actions/runs/38018568206/job/114114274775),
then failed the first actual public `@vizejs/native.compile` comparison:

```ts
compile('<div><div is="vue:my-thing">x</div></div>', {
  mode: "function",
  ssr: false,
  hoistStatic: false,
});
```

It emitted a native `div` and retained `is: "vue:my-thing"`; the equivalent
named component emitted `resolveComponent`, `createVNode` and a default slot.
The public consumer verified the installed platform module was actually loaded
and matched the umbrella exports before calling the compiler. The final
`native.json` receipt was not written because this first DOM case aborted;
subsequent DOM, SSR and browser acceptance remain unexecuted on that run.

The direct N-API compiler constructs `ParserOptions` separately from the
DOM, SSR and Vapor compiler entries. It omitted their existing
`is_native_tag: Some(vize_l0::is_native_tag)` policy. The retained parser
intentionally requires that compiler policy for static `vue:` casts, so
default lint and authored parse-only classification remain unchanged.

Install that same policy at `compile` without introducing a compiler route,
pipeline stage, parse, allocation, dependency or public field. `parseTemplate`
retains its authored classification. Existing custom-element admission,
plain/bound/case-sensitive selectors and `v-pre` retain native elements.
The repair does not establish completion of the independent Davinci native
compiler or replace its unfinished providers with a retained shortcut.

`tests/tooling/native-vue-is-component-8328.test.ts` calls the actual native
package export. It compares complete code, preamble and helper packets with
named components in DOM/SSR and function/module modes, using the exact public
failure and unchanged original corpus, props, encoded and ordinary-template
inputs. Native, bound, case-sensitive, verbatim and explicit custom-element
controls run through the same API; `parseTemplate` pins authored tag behavior.
The added `public-native.json` corpus fixture preserves the exact failing
public call and options; all prior selector corpus files remain unchanged.
The fresh installed public failure supplies the genuine before observation;
the additional cases require fresh source-built Actions and are not claimed
as observed before or after locally.

Local work is limited to source custody, formatting and policy checks because
disk space is constrained. Source Actions must build and execute this native
API regression. Root owns protected merge admission and the next official
release. The immutable v0.439.0 tag and release run are preserved. Keep #8328
open until the released artifact passes actual DOM, SSR and browser acceptance,
then record the available version and verify this reporter trailer on the
actual protected squash:

```text
Co-authored-by: Danila Poyarkov <dev@dannote.net>
```
