# @vizejs/parcel-resolver

Keep native Vue component modules in independent Parcel requests so CSS and Sass
dependency updates do not execute script module side effects again.

Install the resolver with the component transformer:

```sh
npm install --save-dev @vizejs/parcel-transformer @vizejs/parcel-resolver
```

Register both in `.parcelrc`:

```json
{
  "extends": "@parcel/config-default",
  "resolvers": ["@vizejs/parcel-resolver", "..."],
  "transformers": {
    "*.vue": ["@vizejs/parcel-transformer"],
    "*.css": ["@vizejs/parcel-transformer/style", "..."]
  }
}
```

The resolver receives immutable generated module bytes through Parcel's own
metadata. It performs no parsing, code generation, filesystem reads, or global
caching. Normal dependencies continue through Parcel's configured resolvers.
The package is a compatibility preview.

See the [transformer documentation](../parcel/README.md) for compiler options,
styling, SSR, and opt-in Vapor output.
