mod assets;
mod blocks;
mod css;
mod scope;

#[cfg(test)]
mod tests;

pub use assets::{
    TemplateAssetTagRule, TemplateAssetUrl, collect_template_asset_urls, is_importable_asset_url,
};
pub use blocks::{
    BundlerCustomBlock, BundlerStyleBlock, SfcBlockAttribute, SfcSrcInfo, extract_custom_blocks,
    extract_src_info, extract_style_blocks, has_scoped_style,
};
pub use css::{strip_css_comments_for_scoped, wrap_scoped_preprocessor_style};
pub use scope::generate_bundler_scope_id;
