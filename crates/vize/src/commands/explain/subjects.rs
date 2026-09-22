//! Every code `vize explain` knows, generated from the producers' own
//! metadata — the compiler's `ErrorCode::ALL` and every registered lint rule —
//! never from a hand-written list.

use std::sync::OnceLock;

use vize_davinci::diagnostic::Severity as Claim;
use vize_patina::rule_contracts::contract_for;
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

/// A lint rule, as its metadata and its [`RuleContract`](vize_davinci::diagnostic::RuleContract)
/// declare it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Rule {
    pub(crate) name: &'static str,
    /// The rule's own English description.
    pub(crate) description: &'static str,
    pub(crate) category: &'static str,
    pub(crate) fixable: bool,
    /// The severity the unified channel reports at.
    pub(crate) severity: Claim,
    pub(crate) tier: Option<&'static str>,
    pub(crate) domain: Option<&'static str>,
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

fn claim(severity: Severity) -> Claim {
    match severity {
        Severity::Error => Claim::Error,
        Severity::Warning => Claim::Warning,
    }
}

fn push(
    rules: &mut Vec<Rule>,
    name: &'static str,
    description: &'static str,
    category: &'static str,
    fixable: bool,
    meta_severity: Severity,
) {
    let entry = contract_for(name);
    rules.push(Rule {
        name,
        description,
        category,
        fixable,
        severity: entry.map_or_else(|| claim(meta_severity), |entry| entry.contract.severity()),
        tier: entry.map(|entry| entry.contract.tier().as_str()),
        domain: entry.map(|entry| entry.contract.domain().as_str()),
    });
}

/// Every lint rule, once each, sorted by name.
pub(crate) fn rules() -> Vec<Rule> {
    let mut rules = Vec::new();
    for registry in [RuleRegistry::with_all(), RuleRegistry::with_opt_in_rules()] {
        for rule in registry.rules() {
            let meta = rule.meta();
            push(
                &mut rules,
                meta.name,
                meta.description,
                category_name(meta.category),
                meta.fixable,
                meta.default_severity,
            );
        }
    }
    for rule in builtin_script_rules() {
        push(
            &mut rules,
            rule.name,
            rule.description,
            rule.category,
            rule.fixable,
            rule.default_severity,
        );
    }
    let hardcoded = NoHardcodedValues::default();
    let css: [&dyn CssRule; 10] = [
        &NoDisplayNone,
        &hardcoded,
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
        push(
            &mut rules,
            meta.name,
            meta.description,
            "css",
            false,
            meta.default_severity,
        );
    }
    for rule in builtin_musea_rules() {
        push(
            &mut rules,
            rule.name,
            rule.description,
            "musea",
            false,
            rule.default_severity,
        );
    }
    let mut seen = FxHashSet::default();
    rules.retain(|rule| seen.insert(rule.name));
    rules.sort_by(|left, right| left.name.cmp(right.name));
    rules
}

fn load() -> Vec<Subject> {
    let mut subjects: Vec<Subject> = ErrorCode::ALL.into_iter().map(Subject::Compiler).collect();
    subjects.sort_by(|left, right| left.code().cmp(right.code()));
    subjects.extend(rules().into_iter().map(Subject::Rule));
    subjects
}

/// Every code with a page: compiler codes, then rules, each sorted.
pub(crate) fn all() -> &'static [Subject] {
    static ALL: OnceLock<Vec<Subject>> = OnceLock::new();
    ALL.get_or_init(load)
}

/// The subject whose code is exactly `code`.
pub(crate) fn find(code: &str) -> Option<Subject> {
    if let Some(error) = ErrorCode::from_code(code) {
        return Some(Subject::Compiler(error));
    }
    all().iter().copied().find(|subject| subject.code() == code)
}
