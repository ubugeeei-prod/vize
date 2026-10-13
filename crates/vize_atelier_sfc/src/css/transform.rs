//! Shared SFC CSS variable transforms.

pub(crate) mod vars;

#[cfg(test)]
pub(crate) use vize_croquis::sfc::__internal::extract_and_transform_v_bind;
pub(crate) use vize_croquis::sfc::__internal::{
    extract_and_transform_v_bind_with_context, extract_and_transform_v_bind_with_scope,
    find_matching_paren, prod_scoped_v_bind_name, scoped_v_bind_name,
};
