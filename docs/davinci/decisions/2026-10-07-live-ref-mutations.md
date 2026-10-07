### 2026-10-07: live ref values and imperative template nodes (#7898)

A member write through `ref(...).value` still targets Vue's deep reactive proxy;
an imperative write through `useTemplateRef(...).value` targets its host node.
Neither loses a reactive subscription. Record those source kinds in the existing
declaration pass and suppress only member/method mutation losses through them.
Primitive snapshot updates remain reportable. `shallowRef` and `markRaw` values
remain plain; direct/named markRaw initializers retain positive mutation controls.

API aliases follow the existing Vue API alias facts. No new pipeline stage or
serialization is introduced, and raw initializer analysis only visits `ref`
arguments. Public parser debug output remains unchanged. The checked-in legacy
corpus contains both complete reporter examples and deep arrays/nested objects,
API aliases, shallow/raw values and scalar-extraction controls. Hosted Actions
must qualify the exact head before protected queue admission.
