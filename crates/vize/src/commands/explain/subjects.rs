//! Every code `vize explain` knows, generated from the producers' own
//! metadata — the compiler's `ErrorCode::ALL`, every registered lint rule's
//! `RuleMeta` family, and the catalogue's checked mirror of the cross-file and
//! stage-verifier codes — never from a hand-written list.

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
use vize_s0::i18n::{Locale, translator};

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

/// A code whose producer builds each message from the facts at hand, so the
/// catalogue describes the code instead of templating the message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Described {
    /// `vize:croquis/cf/<name>`.
    CrossFile(&'static str),
    /// `S2V…` / `S3V…`.
    Verifier(&'static str),
}

/// A code with a page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Subject {
    Compiler(ErrorCode),
    Rule(Rule),
    Described(Described),
}

impl Subject {
    pub(crate) const fn code(&self) -> &'static str {
        match self {
            Self::Compiler(code) => code.code(),
            Self::Rule(rule) => rule.name,
            Self::Described(Described::CrossFile(code) | Described::Verifier(code)) => code,
        }
    }
}

/// Every described code, read from the catalogue's `<code>.description`
/// keys. The TS-53 catalog check derives the same list from the producers'
/// sources (`CrossFileDiagnostic::code()`, both `ViolationCode::as_str()`) and
/// fails when the two disagree, so the catalogue is a checked mirror.
pub(crate) fn described() -> Vec<Described> {
    let mut codes: Vec<Described> = translator()
        .keys(Locale::En)
        .filter_map(|key| key.strip_suffix(".description"))
        .filter_map(|code| {
            if code.starts_with("vize:croquis/cf/") {
                Some(Described::CrossFile(code))
            } else if is_verifier_code(code) {
                Some(Described::Verifier(code))
            } else {
                None
            }
        })
        .collect();
    codes.sort_by_key(|described| Subject::Described(*described).code());
    codes
}

/// `S2V` or `S3V` followed by three digits.
fn is_verifier_code(code: &str) -> bool {
    let bytes = code.as_bytes();
    bytes.len() == 6
        && (code.starts_with("S2V") || code.starts_with("S3V"))
        && bytes[3..].iter().all(u8::is_ascii_digit)
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

/// Every code with a page: compiler codes, rules, then described codes, each
/// sorted.
pub(crate) fn all() -> Vec<Subject> {
    let mut all: Vec<Subject> = ErrorCode::ALL.into_iter().map(Subject::Compiler).collect();
    all.sort_by_key(Subject::code);
    all.extend(rules().into_iter().map(Subject::Rule));
    all.extend(described().into_iter().map(Subject::Described));
    all
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
        .or_else(|| {
            described()
                .into_iter()
                .map(Subject::Described)
                .find(|subject| subject.code() == code)
        })
}
