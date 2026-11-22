//! An isolated Node entry for consumers requesting all four original block products.
use super::super::types::{CustomBlockNapi, SfcSrcInfoNapi, StyleBlockNapi};
use napi_derive::napi;
use vize_atelier_sfc::bundler::BundlerBlockProducts;

#[napi(object)]
pub struct BundlerBlockProductsNapi {
    pub styles: Vec<StyleBlockNapi>,
    pub custom_blocks: Vec<CustomBlockNapi>,
    pub src: SfcSrcInfoNapi,
    pub has_scoped_style: bool,
}

#[napi(js_name = "inspectSfcBundlerBlocks")]
pub fn inspect_sfc_bundler_blocks(
    source: String,
    filename: Option<String>,
) -> BundlerBlockProductsNapi {
    let products = BundlerBlockProducts::read(&source, filename.as_deref());
    BundlerBlockProductsNapi {
        styles: products.styles.into_iter().map(Into::into).collect(),
        custom_blocks: products.custom_blocks.into_iter().map(Into::into).collect(),
        src: products.src.into(),
        has_scoped_style: products.has_scoped_style,
    }
}
