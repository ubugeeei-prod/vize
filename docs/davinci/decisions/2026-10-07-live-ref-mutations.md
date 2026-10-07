### 2026-10-07: live ref values and imperative template nodes (#7898)

A member write through `ref(...).value` still targets Vue's deep reactive proxy;
an imperative write through `useTemplateRef(...).value` targets its host node.
Neither loses a reactive subscription. Record those source kinds in the existing
declaration passes and suppress only member/method mutation losses through them.
Source facts follow lexical scopes, and captured snapshots preserve the source
identity from extraction time even when a callback shadows the original ref.
Primitive snapshot updates remain reportable. `shallowRef` and `markRaw` values
remain plain; direct/named markRaw initializers retain positive mutation controls.

API aliases follow the existing Vue API alias facts. No new pipeline stage or
serialization is introduced, and raw initializer analysis only visits `ref`
arguments. Public parser debug output remains unchanged. The checked-in legacy
corpus contains both complete reporter examples and deep arrays/nested objects,
API aliases, shallow/raw values, scalar-extraction controls and inverse lexical
shadow/capture controls. Hosted Actions
must qualify the exact head before protected queue admission.

The protected parent/child candidates failed the unchanged Croquis instruction
ceilings and were removed from the queue. The reviewed parent bootstrap repair
reuses the existing universal root and reserves the original 16 scope slots;
this child inherits that exact root, whole-state control and 350-line central
record through its real refreshed parent. The complete original fifteen
live-ref controls and all nine inherited lexical-snapshot controls remain
byte-exact. Fresh source/native/Corsa and protected unchanged-ceiling results
are required before actual merge; prior green source runs remain historical.
