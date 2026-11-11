//! vue/block-lang
//!
//! Disallow an SFC block language outside the configured set. The default
//! policies match eslint-plugin-vue when the rule is enabled with no options:
//! missing `lang` is allowed, and an explicit default language is reported.

use crate::context::LintContext;
use crate::diagnostic::{LintDiagnostic, Severity};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_l0::config::BlockLangOptions;
use vize_l0::{String, cstr};

static META: RuleMeta = RuleMeta {
    name: "vue/block-lang",
    description: "Disallow languages other than the configured lang on an SFC block",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

const SCRIPT_DEFAULTS: &[&str] = &["js", "javascript"];
const TEMPLATE_DEFAULTS: &[&str] = &["html"];
const STYLE_DEFAULTS: &[&str] = &["css"];

#[derive(Clone)]
struct Policy {
    langs: Vec<String>,
    allow_no_lang: bool,
    required: bool,
}

/// `vue/block-lang`.
pub struct BlockLang {
    script: Option<Policy>,
    template: Option<Policy>,
    style: Option<Policy>,
}

impl Default for BlockLang {
    fn default() -> Self {
        let open = Policy {
            langs: Vec::new(),
            allow_no_lang: true,
            required: false,
        };
        Self {
            script: Some(open.clone()),
            template: Some(open.clone()),
            style: Some(open),
        }
    }
}

impl BlockLang {
    /// Build the rule from `linter.ruleOptions["vue/block-lang"]`.
    ///
    /// An empty object checks nothing. Only the blocks present in the object
    /// are checked, matching eslint-plugin-vue's option keys.
    pub fn from_options(options: &BlockLangOptions) -> Self {
        Self {
            script: options
                .script
                .as_ref()
                .map(|block| normalize("script", block)),
            template: options
                .template
                .as_ref()
                .map(|block| normalize("template", block)),
            style: options
                .style
                .as_ref()
                .map(|block| normalize("style", block)),
        }
    }

    fn policy(&self, tag: &str) -> Option<&Policy> {
        match tag {
            "script" => self.script.as_ref(),
            "template" => self.template.as_ref(),
            "style" => self.style.as_ref(),
            _ => None,
        }
    }
}

fn normalize(tag: &str, block: &vize_l0::config::BlockLangBlock) -> Policy {
    let mut langs = match &block.lang {
        Some(vize_l0::config::BlockLangNames::One(lang)) => vec![String::from(lang.as_str())],
        Some(vize_l0::config::BlockLangNames::Many(langs)) => langs
            .iter()
            .map(|lang| String::from(lang.as_str()))
            .collect(),
        None => Vec::new(),
    };
    let mut seen = Vec::new();
    langs.retain(|lang| {
        if seen.iter().any(|kept: &String| kept == lang) {
            return false;
        }
        seen.push(lang.clone());
        true
    });
    let defaults = defaults_for(tag);
    let mut has_default = false;
    langs.retain(|lang| {
        if defaults.contains(&lang.as_str()) {
            has_default = true;
            false
        } else {
            true
        }
    });
    let required = block.required == Some(true);
    if langs.is_empty() {
        return Policy {
            langs,
            allow_no_lang: true,
            required,
        };
    }
    Policy {
        langs,
        allow_no_lang: has_default || block.allow_no_lang == Some(true),
        required,
    }
}

fn defaults_for(tag: &str) -> &'static [&'static str] {
    match tag {
        "script" => SCRIPT_DEFAULTS,
        "template" => TEMPLATE_DEFAULTS,
        "style" => STYLE_DEFAULTS,
        _ => &[],
    }
}

fn allows_phrase(langs: &[String]) -> String {
    let mut phrase = String::default();
    for (index, lang) in langs.iter().enumerate() {
        if index > 0 {
            phrase.push_str(if index + 1 == langs.len() {
                ", and "
            } else {
                ", "
            });
        }
        phrase.push('\'');
        phrase.push_str(lang);
        phrase.push('\'');
    }
    phrase
}

impl Rule for BlockLang {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_sfc<'a>(&self, ctx: &mut LintContext<'a>) {
        // `sfc_descriptor` borrows `ctx`, so own the block facts before reporting.
        let (seen, missing) = {
            let Some(descriptor) = ctx.sfc_descriptor() else {
                return;
            };
            let mut seen = Vec::new();
            let mut present = Vec::new();
            if let Some(block) = &descriptor.script {
                note_block(
                    &mut seen,
                    &mut present,
                    "script",
                    block.lang.as_deref(),
                    block.loc.tag_start,
                    block.loc.start,
                );
            }
            if let Some(block) = &descriptor.script_setup {
                note_block(
                    &mut seen,
                    &mut present,
                    "script",
                    block.lang.as_deref(),
                    block.loc.tag_start,
                    block.loc.start,
                );
            }
            if let Some(block) = &descriptor.template {
                note_block(
                    &mut seen,
                    &mut present,
                    "template",
                    block.lang.as_deref(),
                    block.loc.tag_start,
                    block.loc.start,
                );
            }
            for block in &descriptor.styles {
                note_block(
                    &mut seen,
                    &mut present,
                    "style",
                    block.lang.as_deref(),
                    block.loc.tag_start,
                    block.loc.start,
                );
            }
            let missing = ["script", "template", "style"]
                .into_iter()
                .filter(|tag| {
                    self.policy(tag)
                        .is_some_and(|policy| policy.required && !present.contains(tag))
                })
                .collect::<Vec<_>>();
            (seen, missing)
        };
        for block in &seen {
            check_lang(
                ctx,
                self.policy(block.tag),
                block.tag,
                block.lang.as_deref(),
                block.tag_start,
                block.content_start,
            );
        }
        for tag in missing {
            ctx.report_in_sfc(LintDiagnostic::warn(
                META.name,
                cstr!("The '<{tag}>' block is missing."),
                0,
                0,
            ));
        }
    }
}

fn note_block(
    seen: &mut Vec<SeenBlock>,
    present: &mut Vec<&'static str>,
    tag: &'static str,
    lang: Option<&str>,
    tag_start: usize,
    content_start: usize,
) {
    present.push(tag);
    seen.push(SeenBlock {
        tag,
        lang: lang.map(String::from),
        tag_start,
        content_start,
    });
}

struct SeenBlock {
    tag: &'static str,
    lang: Option<String>,
    tag_start: usize,
    content_start: usize,
}

fn check_lang(
    ctx: &mut LintContext<'_>,
    policy: Option<&Policy>,
    tag: &str,
    lang: Option<&str>,
    tag_start: usize,
    content_start: usize,
) {
    let Some(policy) = policy else {
        return;
    };
    let lang = lang.map(str::trim).filter(|lang| !lang.is_empty());
    let Some(lang) = lang else {
        if !policy.allow_no_lang {
            ctx.report_in_sfc(LintDiagnostic::warn(
                META.name,
                cstr!("The 'lang' attribute of '<{tag}>' is missing."),
                tag_start as u32,
                content_start as u32,
            ));
        }
        return;
    };
    if policy.langs.iter().any(|allowed| allowed == lang) {
        return;
    }
    let allows = allows_phrase(&policy.langs);
    let message = if !policy.allow_no_lang {
        cstr!("Only {allows} can be used for the 'lang' attribute of '<{tag}>'.")
    } else if policy.langs.is_empty() && defaults_for(tag).contains(&lang) {
        cstr!(
            "Do not explicitly specify the default language for the 'lang' attribute of '<{tag}>'."
        )
    } else if policy.langs.is_empty() {
        cstr!("Do not specify the 'lang' attribute of '<{tag}>'.")
    } else {
        cstr!(
            "Only {allows} can be used for the 'lang' attribute of '<{tag}>'. Or, not specifying the 'lang' attribute is allowed."
        )
    };
    let (start, end) = lang_attribute_span(ctx.source, tag_start, content_start);
    ctx.report_in_sfc(LintDiagnostic::warn(META.name, message, start, end));
}

/// Span of the `lang` attribute inside the opening tag, from the name through
/// the value. Falls back to the opening tag when the attribute text is absent.
fn lang_attribute_span(source: &str, tag_start: usize, content_start: usize) -> (u32, u32) {
    let opening = source.get(tag_start..content_start).unwrap_or("");
    let Some(index) = opening.find("lang") else {
        return (tag_start as u32, content_start as u32);
    };
    let mut cursor = index + "lang".len();
    let bytes = opening.as_bytes();
    let at = |cursor: usize| bytes.get(cursor).copied();
    while at(cursor).is_some_and(|byte| byte.is_ascii_whitespace()) {
        cursor += 1;
    }
    if at(cursor) == Some(b'=') {
        cursor += 1;
        while at(cursor).is_some_and(|byte| byte.is_ascii_whitespace()) {
            cursor += 1;
        }
        if let Some(quote @ (b'"' | b'\'')) = at(cursor) {
            cursor += 1;
            while at(cursor).is_some_and(|byte| byte != quote) {
                cursor += 1;
            }
            if cursor < bytes.len() {
                cursor += 1;
            }
        } else {
            while at(cursor).is_some_and(|byte| !byte.is_ascii_whitespace() && byte != b'>') {
                cursor += 1;
            }
        }
    }
    (
        tag_start as u32 + index as u32,
        tag_start as u32 + cursor as u32,
    )
}

#[cfg(test)]
mod tests;
