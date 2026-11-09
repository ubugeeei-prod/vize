use super::{DirectiveArgStyle, LegacyDialectCapabilities};

pub const fn clause(mut features: LegacyDialectCapabilities) -> LegacyDialectCapabilities {
    features.directive_arg_style = DirectiveArgStyle::Clause;
    features
}
