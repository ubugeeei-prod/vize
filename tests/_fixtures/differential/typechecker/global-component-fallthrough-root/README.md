# Global component fallthrough root

Jimmy Chen ([Q16solver](https://github.com/Q16solver)) supplied the reproductions
for a [native listener and a native attribute rejected on a component whose root only `GlobalComponents` declares](https://github.com/ubugeeei-prod/vize/issues/13721).

A root tag no setup binding owns is the component Vue resolves from the
registry, so `Bar` forwards what `GlobalComponents["Foo"]` accepts: the
listener is typed by `Foo`'s `<input>` root and the attribute is known, while a
name `Foo` does not declare stays unknown.
