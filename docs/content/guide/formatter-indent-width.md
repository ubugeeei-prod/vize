# Indentation width

`tabWidth` controls the number of spaces per indentation level. Its default is
`2`. Set it in the existing formatter configuration:

```json
{
  "formatter": {
    "tabWidth": 3
  }
}
```

For `<template><p>hello</p></template>`, this produces:

```text
<template>
   <p>hello</p>
</template>
```

`useTabs: true` selects tabs for indentation instead. Quote preferences keep
their existing behavior. Whitespace inside raw template regions such as `<pre>`,
`<textarea>` and `v-pre` retains its authored bytes.
