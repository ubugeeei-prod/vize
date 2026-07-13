# vize_atelier_template

Compatibility follows the [Rust crate support tiers](https://github.com/ubugeeei-prod/vize/blob/main/docs/content/stability.md#rust-crate-support-tiers).

`vize_atelier_template` owns the raw Vue-template frontend. It parses a
standalone template into source-faithful Relief syntax, projects the cached
syntax into Croquis semantics, Flow, and frontend-neutral Rendu, and routes
its compile recipe to the requested DOM, SSR, or Vapor product through Atlas.
The application host registers those peer backends explicitly.

This crate does not create a synthetic SFC. `vize_atelier_sfc` consumes the
same generic Relief-to-Flow/Rendu lowering for real `.vue` template blocks.

## Key Entry Points

- `register_atlas_providers`
- `install_template_compile_request`
- `TemplateCompileProduct`
- `TemplateCompileRequest`
- `TemplateRenderTarget`

`register_atlas_providers` installs only raw-template-owned providers and the
compile recipe. It does not register Croquis projection or any render backend.
