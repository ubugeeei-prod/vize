# Indentation width

Choose the indentation width that matches your project. Space or tab settings change the layout around template content; authored raw text stays intact.

Check [shared formatter configuration and current release availability](./configuration.md)
before choosing where to put these settings.

## Configure indentation

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
