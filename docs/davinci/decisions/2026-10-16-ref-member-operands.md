# Ref member access is a value operand

Related qualification: [#8849](https://github.com/ubugeeei-prod/vize/issues/8849).

The complete Matrix found two missed `vue/no-ref-as-operand` findings in pinned
`graphieros/vue-data-ui`: `timeLabels.map(...)` in `vue-ui-ridgeline.vue` and
`drawingArea?.bottom.value` in `vue-ui-xy.vue`. Both objects are locally declared
Vue refs. The first must read `timeLabels.value.map(...)`; the second must first
unwrap `drawingArea.value` before accessing its object properties.

The native script visitor already resolves imported factories, aliases,
namespaces and lexical shadows. Member reads now reuse that same binding
resolution for a direct identifier object. Static property names other than
`value` and `effect` require unwrapping. Unknown computed keys remain allowed,
as do non-ref objects, shadowed names and refs passed to functions.

The change adds no engine or type-provider surface. Plain assignments, emit
recognition and the existing boolean-operand checks retain their current
behavior. In particular, this does not suppress the native boolean/let-ref
guard findings that the reference currently omits. The possible raw-ref
fallback finding in `vue-ui-hill.vue` needs a separate context-specific proof.

`fixtures/ref-member-access/source.json` binds both full originals to their
gitlink pin, byte counts, SHA-256 hashes and Git blob IDs. The complete original
Matrix producer report is retained with a raw-byte hash, all messages, stdout,
stderr and counts. The oracle retains complete official packets, including
the `.value` fixes, for both originals and 14 positive/negative controls.
The only recording step replaces the ephemeral checkout prefix in `filePath`.

The reference is ESLint 10.4.1, eslint-plugin-vue 10.9.2, vue-eslint-parser
10.4.1 and @typescript-eslint/parser 8.65.0. The Node replay asserts every
provider packet field. `ref_member_access.rs` asserts every public native
result field, including authored UTF-8 ranges, labels, help, counts and the
existing absence of an automatic native fix. The official fixes remain intact
in their original packets; this change introduces diagnostics only.
