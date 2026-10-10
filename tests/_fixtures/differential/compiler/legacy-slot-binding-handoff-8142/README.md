# Legacy slot binding handoff (#8142)

These 17 independently authored complete templates fix declaration names,
original physical header byte spans, dialect gates and parser-attempt counts
before executing the product. Existing fixture and golden bytes stay untouched.

The serialized integration judge captures 196 real compilations: legacy and
equivalent modern source, Function and Module modes, and Vue 2 / 2.7 / 3 (the
native HTML `scope` control is Vue 3 only). It retains the original parsed root,
both conversion roots, the transformed root, every compiler diagnostic, retained
parameter/diagnostic debug fields, actual pointers, every profiler counter and
complete generated code/preamble/map **before** checking any product law in
`target/differential/legacy-slot-binding-handoff-8142/whole-packets.json`.

For Vue 2 and 2.7, the first dialect-selected conversion owns one direct original
parameter parse for each admitted value. Unbalanced/depth guards own zero
attempts but keep a refused parameter role; the second conversion owns zero.
Modern payload identity, ordinary expression payload identity, Vue 3 inertness,
valueless attributes and mixed modern/legacy directives are separate laws.
Attribute/Text/SimpleExpression layouts remain 56/24/88 bytes on 64-bit targets.
Pointers establish local arena custody; they are not portable golden values.

Valid Vue 2/2.7 pairs compare the whole generated result, with additional
independent callback, filter, shadowing, and outside-identifier controls.
The additive structural controls describe every callback declaration, display
argument and semantic read in order. A test-only OXC observer reads the complete
generated Module after the product counter windows; its full inventory and
parse diagnostics are retained before comparison with authored expectations.
Malformed pairs compare complete parameter diagnostics and retain all compiler
errors without normalizing physical locations. TypeScript output is retained as
compiler output; this test does not claim executable JavaScript or mounted
runtime correctness. Source maps are disabled for both variants, while original
physical header spans are checked separately.

The entity case checks decoded raw bytes and the original value's identity.
It does not claim that decoded declaration offsets map to encoded physical
bytes, or that the selected consumer has complete entity parity. Croquis keeps
its existing separate legacy raw-attribute boundary; this is not a universal
whole-pipeline parse-once claim. Allocation costs, SSR/runtime behavior, original
DOM corpus checks and public package acceptance remain separate evidence.

Run the new binary with `cargo test -p vize_atelier_core --features legacy
--test legacy_slot_binding_handoff_8142`. A single test owns the process-global
profiler. The same additive test and fixture can be replayed against frozen
predecessor source without calling the newly added producer API directly.
The n8n compiler companion runs this explicit legacy-feature target together
with the original `legacy_template_sugar` controls and uploads its complete raw
observations. The shared Rust differential recipe and its expected command
vectors retain their original bytes.
