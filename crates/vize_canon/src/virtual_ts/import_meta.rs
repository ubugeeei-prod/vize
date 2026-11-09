//! ImportMeta augmentation for generated virtual TypeScript.

pub(super) fn emit_import_meta_augmentation(
    output: &mut crate::virtual_ts::TextBuf,
    include_vite: bool,
) {
    output.push_str("// ImportMeta augmentation (reference existing framework types)\n");
    if include_vite {
        output.push_str("/// <reference types=\"vite/client\" />\n");
    }
    output.push_str(
        r#"declare global {
  // Extend ImportMeta with Nuxt-specific properties not covered by vite/client
  interface ImportMeta {
    client: boolean;
    server: boolean;
    dev: boolean;
    prod: boolean;
    ssr: boolean;
  }
}
"#,
    );
}
