# Character references in template expressions

The shipped HTML parser decodes character references in attribute values and
interpolations before the compiler reads an expression; L2 decodes them when
it admits the expression and keeps the authored padding. The production-reach
gate compiles these sources on both DOM lanes and requires byte-identical
modules.

- `Entities.vue`: named, decimal and hexadecimal references in `v-if`,
  `v-show`, `v-for`, `v-on`, `v-bind`, `v-model` and interpolations, with
  padded and unpadded values.
- `PugEntities.vue`: the pug dialect escapes `"`, `&`, `<` and `>` in every
  attribute value, so arrow handlers and logical operators reach the
  template as references (reported by the dho-web-client corpus project).
