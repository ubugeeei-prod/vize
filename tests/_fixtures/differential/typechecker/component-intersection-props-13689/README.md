These fixtures reproduce #13689 with Vue `DefineComponent` and actual SFC imports, without Nuxt. Both intersection orders, empty components, required wrapper props, defaults, repeated usages, a closure, and both root-slot constructor-pair paths are exercised.

The original `.vue` inputs were checked with vue-tsc 3.3.14, TypeScript 6.0.3, and Vue 3.5.46 using `vue-tsc -p tsconfig.oracle.json --pretty false`. `Valid.vue` has no diagnostics. `Invalid.vue` has these complete `(line, column, code)` findings:

```
(6, 4, 2345)
(7, 11, 2322)
(8, 24, 2322)
(9, 24, 2353)
(10, 4, 2345)
(11, 33, 2322)
(12, 24, 2353)
(13, 25, 2322)
(14, 15, 2322)
(15, 15, 2353)
```

`controls.ts` checks the private generated helper contract separately: exact optional props, raw metadata precedence, legacy components without static metadata, two generated SFCs, additional raw constructors, any/unknown/never/unions, and the generic `__vizeCheck` path. It is not an input to the vue-tsc oracle. The production batch test and the complete inline/shared mapper module test execute it against the current helpers.

The oracle config includes the original Vue inputs and `lazy.ts`; the production config also includes the direct helper controls. `Authored.vue` checks inherited static metadata, `Emit.vue` checks mixed native/generated unions, and `Fresh.vue` checks a setup-local binding with the generated component name.

The callable self witness carries the actual direct instance raw contract and the canonical owned props separately. `Exposed*.vue` and the isolated `globals/` projects retain raw metadata contributed by expose/global augmentation, including any/unknown/never. `Static*.vue` proves that authored static-only literal, never, open, and pattern metadata cannot hide hydration props. `Own*.vue` keeps real owned literal/pattern/index values strict. Each global variant is compiled in its own program to avoid contaminating unrelated fixtures.

`RawMethods`, `RawComputed`, and `RawSetup` preserve Options API raw metadata. Aliased, union, open-index, and pattern-index exposed surfaces preserve the original instance-first helper contract; `OptionalSource` checks optional-field inference. Raw metadata always comes from the complete default instance.

`ExposedConflictingProps`, `ExposedConflictingEmit`, and `ExposedConflictingSlots` retain full-instance `never` behavior when an unrelated exposed member conflicts with an owned field.
