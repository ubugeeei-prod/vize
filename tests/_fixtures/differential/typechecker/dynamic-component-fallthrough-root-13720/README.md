# Dynamic component fallthrough root

Jimmy Chen ([Q16solver](https://github.com/Q16solver)) supplied the reproduction
for a [setup binding named `component` typing a `<component :is>` root](https://github.com/ubugeeei-prod/vize/issues/13720).

Vue's compiler resolves `<component is>` and its built-in components before it
reads setup bindings, so a binding spelled like the root's tag never types the
attributes that fall through to it. The spread on `Foo` stays valid, while a
wrong value for `Foo`'s own prop is still reported.
