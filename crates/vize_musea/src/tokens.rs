//! Design token parsing, reference resolution, and documentation helpers.

mod markdown;
mod parse;
mod resolve;
#[cfg(test)]
mod tests;
mod types;

pub use markdown::generate_tokens_markdown;
pub use parse::{parse_tokens_from_json, parse_tokens_from_path, parse_tokens_from_value};
pub use resolve::{
    build_token_map, find_dependent_tokens, flatten_token_categories, resolve_token_categories,
    validate_reference,
};
pub use types::{
    DesignToken, FlattenedToken, ResolvedTokens, TokenCategory, TokenError, TokenResult,
    ValidationResult,
};
