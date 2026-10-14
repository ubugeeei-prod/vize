# Admit self-closed Vapor markup without matching legacy element numbering

Issue: [#8859](https://github.com/ubugeeei-prod/vize/issues/8859).

The native Vapor lane used to send markup that the legacy parser diagnoses,
such as a self-closed `<div/>`, to the retained lane. The two lanes number
elements differently. The retained lane names the root after its children
(`const n1 = t0()`, `const n0 = _child(n1)`), and the native lane names it
first (`const n0 = t0()`, `const n1 = _child(n0)`). So admitting the markup
changes the bytes of the compiled output, even though the DOM and the runtime
behavior stay the same.

The maintainer decided to admit this markup on the native lane and accept the
change in output. The native lane does not adopt the legacy numbering. This
is an approved exception to the rule in `AGENTS.md` that legacy products keep
byte-exact output. It covers only renumbering and reordered reference lookups
in sources that move to the native lane. Any other change in output still
needs its own fix and fixture.

One side effect is not a renumbering. The retained lane merges a static
`style` before a `:style` written ahead of it, while the native lane keeps
the authored order (`[{ color: _ctx.c }, "top: 0"]`), as upstream
`@vue/compiler-vapor` 3.6.0-rc.11 does. A self-closed element now compiles
exactly like its explicitly closed form, and
`self_closed_elements_compile_like_explicitly_closed_ones` covers that case.

The native lane still reports exactly what the legacy parser would. That is
one recovery diagnostic per self-closed non-void element, and
`parser_agreement.rs` checks it over the fixture corpus and its malformed
variants.

The same exception may later cover elements that HTML tree construction
closes early (`<li>a<li>b</li>`). It does not cover an empty directive
modifier (`@click..stop`). Today that is a fatal compile error, so admitting
it would turn an error into output, which is a different change.
