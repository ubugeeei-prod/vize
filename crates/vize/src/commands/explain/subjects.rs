//! Every code `vize explain` knows, generated from the producers' own
//! metadata — the compiler's `ErrorCode::ALL` and every registered lint rule's
//! `RuleMeta` family — never from a hand-written list.

use vize_patina::rules::css::{
    CssRule, NoDisplayNone, NoHardcodedValues, NoIdSelectors, NoImportant, NoUtilityClasses,
    NoVBindPerformance, PreferLogicalProperties, PreferNestedSelectors, PreferSlotted,
    RequireFontDisplay,
};
use vize_patina::{
    RuleCategory, RuleRegistry, Severity, builtin_musea_rules, builtin_script_rules,
};
use vize_relief::ErrorCode;
use vize_s0::FxHashSet;

/// A lint rule, as its metadata declares it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Rule {
    pub(crate) name: &'static str,
    /// The rule's own English description.
    pub(crate) description: &'static str,
    pub(crate) category: &'static str,
    pub(crate) fixable: bool,
    pub(crate) default_severity: Severity,
}

/// A code with a page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Subject {
    Compiler(ErrorCode),
    Rule(Rule),
}

impl Subject {
    pub(crate) const fn code(&self) -> &'static str {
        match self {
            Self::Compiler(code) => code.code(),
            Self::Rule(rule) => rule.name,
        }
    }
}

const fn category_name(category: RuleCategory) -> &'static str {
    match category {
        RuleCategory::Essential => "essential",
        RuleCategory::StronglyRecommended => "strongly-recommended",
        RuleCategory::Recommended => "recommended",
        RuleCategory::Vapor => "vapor",
        RuleCategory::Musea => "musea",
        RuleCategory::Accessibility => "accessibility",
        RuleCategory::HtmlConformance => "html-conformance",
        RuleCategory::TypeAware => "type-aware",
        RuleCategory::Ecosystem => "ecosystem",
    }
}

/// Every lint rule, once each, sorted by name.
pub(crate) fn rules() -> Vec<Rule> {
    let mut rules = Vec::new();
    let registries = [
        RuleRegistry::with_all(),
        RuleRegistry::with_opt_in_rules(),
        RuleRegistry::with_nuxt(),
    ];
    for registry in &registries {
        for rule in registry.rules() {
            let meta = rule.meta();
            rules.push(Rule {
                name: meta.name,
                description: meta.description,
                category: category_name(meta.category),
                fixable: meta.fixable,
                default_severity: meta.default_severity,
            });
        }
    }
    for rule in builtin_script_rules() {
        rules.push(Rule {
            name: rule.name,
            description: rule.description,
            category: rule.category,
            fixable: rule.fixable,
            default_severity: rule.default_severity,
        });
    }
    for rule in builtin_musea_rules() {
        rules.push(Rule {
            name: rule.name,
            description: rule.description,
            category: "musea",
            fixable: false,
            default_severity: rule.default_severity,
        });
    }
    let css: [&dyn CssRule; 10] = [
        &NoDisplayNone,
        &NoHardcodedValues::default(),
        &NoIdSelectors,
        &NoImportant,
        &NoUtilityClasses,
        &NoVBindPerformance,
        &PreferLogicalProperties,
        &PreferNestedSelectors,
        &PreferSlotted,
        &RequireFontDisplay,
    ];
    for rule in css {
        let meta = rule.meta();
        rules.push(Rule {
            name: meta.name,
            description: meta.description,
            category: "css",
            fixable: false,
            default_severity: meta.default_severity,
        });
    }
    let mut seen = FxHashSet::default();
    rules.retain(|rule| seen.insert(rule.name));
    rules.sort_by_key(|rule| rule.name);
    rules
}

/// Every code with a page: compiler codes, then rules, each sorted.
pub(crate) fn all() -> Vec<Subject> {
    let mut compiler: Vec<Subject> = ErrorCode::ALL.into_iter().map(Subject::Compiler).collect();
    compiler.sort_by_key(Subject::code);
    compiler.extend(rules().into_iter().map(Subject::Rule));
    compiler
}

/// The subject whose code is exactly `code`.
pub(crate) fn find(code: &str) -> Option<Subject> {
    if let Some(error) = ErrorCode::from_code(code) {
        return Some(Subject::Compiler(error));
    }
    rules()
        .into_iter()
        .find(|rule| rule.name == code)
        .map(Subject::Rule)
}
