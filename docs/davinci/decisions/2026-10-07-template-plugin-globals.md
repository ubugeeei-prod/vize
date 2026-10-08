# Template plugin globals

Issue: [#7896](https://github.com/ubugeeei-prod/vize/issues/7896).

`vue/no-undefined-refs` treats `$`-prefixed template identifiers as component
proxy properties. Vue plugins may provide them through
`app.config.globalProperties` and `ComponentCustomProperties`; the linter's
single-file binding facts cannot enumerate that application configuration.
The rule continues to report undefined bare identifiers, including in the
same expression or file as plugin properties.

The regression corpus contains vue-i18n, Vuetify, filters, router, built-in
instance properties, an arbitrary plugin property, and an undefined bare
identifier with its exact diagnostic span. The enum half of #7896 is tracked
with the compiler binding correction in #7893 and remains separate from this
slice. This legacy regression adds no Davinci native acceptance credit.
