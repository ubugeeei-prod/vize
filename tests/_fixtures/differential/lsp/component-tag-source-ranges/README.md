# Authored component tag ranges

The parser owns each element's start/end offsets. Its retained source owns
the opening tag's bytes; an Options API expression, named `:is` value or
generated dynamic alias can have a different length from `<component>`.

These complete script/template inputs freeze the authored tag and first-byte
query ranges. Actual native projection tests retain navigation, whole-props
and required-slot anchors on the entire authored tag, while querying dynamic
values at their copied expressions. Static tags, nesting, UTF-8 prefixes and
CRLF are separate controls. The underscore-starting reserved-looking spelling
is retained as opaque text by the original tokenizer; it creates no component
mapping or generated alias, as the negative case explicitly requires.

The distinct `options-method-definition/Canvas.vue.txt` native fixture keeps
its original first-byte query at offset 29 and complete method target at
line 10, characters 4–20. No extra props are added to that minimal source.
