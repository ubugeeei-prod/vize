# Deprecated instance API receiver audit

All 239 original Matrix false-positive rows are retained in `provenance.json.gz`; classifications describe source truth separately from pinned ESLint silence. Every row retains the original report fields plus immutable source URL, revision, source SHA256, binding context, and provider evidence references.

| Rule and construct | Rows | Evidence and interpretation |
| --- | ---: | --- |
| events: `this.$root` | 94 | Vue public root identity. |
| events: direct `this` | 35 | Plain exported Vue Options hooks/methods; lexical Promise arrows preserve `this`. No MdComponent constructors in these 35. |
| events: local/imported constructor provenance | 34 | Explicit `new Vue`, Vue.extend constructor, or imported module exporting such instance. |
| events: plugin-provided instance | 10 | Plugin/boot assignment establishes intended Vue instance; no property-name-only inference. |
| events: injected GridItem eventBus | 20 | Intended GridLayout provider is `new Vue`; standalone SFC could receive arbitrary external provider. |
| events: arbitrary action callback return | 4 | Unresolved; caller-defined Function prop may return any emitter. |
| events: dangling `./EventHub` import | 3 | Unresolved; complete immutable repository tree has no target. |
| events: local tiny-emitter wrapper | 1 | Proven foreign: Vue3 B.vue line46. |
| events: out-of-scope eventHub identifier | 1 | Unresolved: Vue3 B.vue line51, sibling hook cannot access mounted-local variable. |
| **events total** | **202** | **173 direct/plain/public/plugin +20 intended GridLayout injection +1 foreign +8 unresolved.** |
| listeners: plain exported Options | 23 | Public component `this.$listeners` in methods/computed/render. |
| listeners: MdComponent-returned Options | 14 | Imported factory returns deepmerge(defaults,newComponent), i.e. Vue Options. |
| **listeners total** | **37** | **All37 actual component public-instance migration concerns.** |

The 193 Vue-backed events comprise 94 public root +35 direct public this +34 direct/module constructor +10 plugin +20 intended injection. The last two tiers require provider context; they are not proof that arbitrary standalone properties always have Vue identity.

## Product decision and remaining frontier

`script/no-deprecated-events-api` reports only receivers proven to be a Vue
instance within the file: `this` and its aliases, their `$root` / `$parent`, and
local `new Vue` / `Vue.extend` instances that never escape. An imported or
injected hub may be the Vue 3 migration shim that re-exposes `$on` over
`tiny-emitter`, so the import, plugin and injection tiers above are no longer
reported, and a file that defines a `$on` / `$off` / `$once` itself (object key,
member write, `defineProperty` key, merge into `this`) is not reported for that
name. Against the 202 events rows this keeps the 94 `$root` rows, 31 direct
`this` rows (four use `extend(this, options)`) and two local constructors.

The tiny-emitter call and all eight unresolved identities are silent; [issue
#9217](https://github.com/ubugeeei-prod/vize/issues/9217) is resolved by that
policy. Exact source and diagnostic packets remain separate from it.

The tiny-emitter source is Gogocode's
`packages/gogocode-vue-playground/packages/vue3/src/components/events-api/B.vue`,
revision `6b94d4d05ee4facc935fe2c3e4965655913afd6e`, SHA256
`136f82332bfa8b7795c5802015fc64859d3fbb794ce3720a73fe7093ecede165`.
Only line 46 has the mounted-local tiny-emitter wrapper; line 51 in the sibling
`beforeUnmount` hook has no accessible binding for that local variable.

`provenance.json.gz` contains all 239 row classifications, original report
coordinates/context, immutable source URLs/revisions/hashes, 23 complete primary
source files and the complete untruncated Gogocode Git tree. Its packet bytes
are base64 encoded without alteration. The 17 cited complete Matrix reports
are retained under `originals/`; the 18th original report covers the separately
tracked scoped-slots rule. The Node replay verifies source, provider and whole
report hashes. No `.vue` inventory entries are added for these source captures.
