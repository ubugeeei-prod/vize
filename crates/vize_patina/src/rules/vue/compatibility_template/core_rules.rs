//! Core JavaScript rules applied to retained template expressions.

use super::core_checks::{CoreKind, check_expression};
use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_l1::markup::entity::EntityContext;
use vize_relief::{DirectiveNode, ElementNode, InterpolationNode};

macro_rules! core_rule {
    ($type:ident, $meta:ident, $name:literal, $description:literal, $kind:ident) => {
        static $meta: RuleMeta = RuleMeta {
            name: $name,
            description: $description,
            category: RuleCategory::Recommended,
            fixable: false,
            default_severity: Severity::Warning,
        };
        #[doc = $description]
        pub struct $type;

        impl Rule for $type {
            fn meta(&self) -> &'static RuleMeta {
                &$meta
            }

            fn check_interpolation<'a>(
                &self,
                ctx: &mut LintContext<'a>,
                interpolation: &InterpolationNode<'a>,
            ) {
                check_expression(
                    ctx,
                    &interpolation.content,
                    CoreKind::$kind,
                    EntityContext::Text,
                );
            }

            fn check_directive<'a>(
                &self,
                ctx: &mut LintContext<'a>,
                _element: &ElementNode<'a>,
                directive: &DirectiveNode<'a>,
            ) {
                if let Some(expression) = &directive.exp {
                    check_expression(ctx, expression, CoreKind::$kind, EntityContext::Attribute);
                }
            }
        }
    };
}

core_rule!(
    Camelcase,
    CAMELCASE_META,
    "vue/camelcase",
    "Require camelCase identifiers in template expressions",
    Camelcase
);
core_rule!(
    DotNotation,
    DOT_NOTATION_META,
    "vue/dot-notation",
    "Prefer dot notation in template expressions",
    DotNotation
);
core_rule!(
    Eqeqeq,
    EQEQEQ_META,
    "vue/eqeqeq",
    "Require strict equality in template expressions",
    Eqeqeq
);
core_rule!(
    NoConsole,
    NO_CONSOLE_META,
    "vue/no-console",
    "Disallow console access in template expressions",
    NoConsole
);
core_rule!(
    NoConstantCondition,
    NO_CONSTANT_CONDITION_META,
    "vue/no-constant-condition",
    "Disallow constant conditions in template expressions",
    NoConstantCondition
);
core_rule!(
    NoEmptyPattern,
    NO_EMPTY_PATTERN_META,
    "vue/no-empty-pattern",
    "Disallow empty destructuring patterns in template expressions",
    NoEmptyPattern
);
core_rule!(
    NoImplicitCoercion,
    NO_IMPLICIT_COERCION_META,
    "vue/no-implicit-coercion",
    "Disallow implicit coercion in template expressions",
    NoImplicitCoercion
);
core_rule!(
    NoIrregularWhitespace,
    NO_IRREGULAR_WHITESPACE_META,
    "vue/no-irregular-whitespace",
    "Disallow irregular whitespace in template expressions",
    NoIrregularWhitespace
);
core_rule!(
    NoLossOfPrecision,
    NO_LOSS_OF_PRECISION_META,
    "vue/no-loss-of-precision",
    "Disallow numeric literals that lose precision",
    NoLossOfPrecision
);
core_rule!(
    NoNegatedCondition,
    NO_NEGATED_CONDITION_META,
    "vue/no-negated-condition",
    "Disallow negated ternary conditions in template expressions",
    NoNegatedCondition
);
core_rule!(
    NoSparseArrays,
    NO_SPARSE_ARRAYS_META,
    "vue/no-sparse-arrays",
    "Disallow sparse arrays in template expressions",
    NoSparseArrays
);
core_rule!(
    NoUselessConcat,
    NO_USELESS_CONCAT_META,
    "vue/no-useless-concat",
    "Disallow useless string concatenation in template expressions",
    NoUselessConcat
);
core_rule!(
    ObjectShorthand,
    OBJECT_SHORTHAND_META,
    "vue/object-shorthand",
    "Require object shorthand in template expressions",
    ObjectShorthand
);
core_rule!(
    PreferTemplate,
    PREFER_TEMPLATE_META,
    "vue/prefer-template",
    "Prefer template literals in template expressions",
    PreferTemplate
);
core_rule!(
    QuoteProps,
    QUOTE_PROPS_META,
    "vue/quote-props",
    "Disallow unnecessary quoted object properties in template expressions",
    QuoteProps
);
