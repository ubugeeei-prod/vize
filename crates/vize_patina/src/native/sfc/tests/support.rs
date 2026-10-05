use super::super::*;
use crate::Locale;
use crate::rules::script::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::{HelpLevel, LintDiagnostic, LintResult, Linter, RuleRegistry, Severity};
use std::sync::{Arc, Mutex};
use vize_l0::{Span, config::VueVersion};

pub const RULE: &str = "script/prefer-use-template-ref";
pub const SECOND_RULE: &str = "script/no-next-tick";
pub const SOURCE: &str = "<script setup lang=\"ts\">\nconst text = \"ref(null)\"\n</script>\n<template><div ref=\"text\" /></template>\n";
pub const FILE: &str = "/元/History.vue";
pub const LOCALES: [Locale; 3] = [Locale::En, Locale::Ja, Locale::Zh];
pub static FIRST: ScriptRuleMeta = ScriptRuleMeta {
    name: RULE,
    description: "Actual configured setup law",
    default_severity: Severity::Warning,
};
pub static SECOND: ScriptRuleMeta = ScriptRuleMeta {
    name: SECOND_RULE,
    description: "Second actual configured setup law",
    default_severity: Severity::Error,
};
pub type Log = Arc<Mutex<Vec<serde_json::Value>>>;

pub struct Audit {
    pub meta: &'static ScriptRuleMeta,
    pub marker: &'static str,
    pub locale: Locale,
    pub help: HelpLevel,
    pub log: Log,
    pub provided: bool,
    pub runs_setup: bool,
    pub fail: bool,
}
impl ScriptRule for Audit {
    fn meta(&self) -> &'static ScriptRuleMeta {
        self.meta
    }
    fn check(&self, _: &str, offset: usize, result: &mut ScriptLintResult) {
        let mut diagnostic = LintDiagnostic::warn(
            self.meta.name,
            message(self.locale, self.marker),
            offset as u32,
            offset as u32,
        );
        diagnostic.severity = self.meta.default_severity;
        diagnostic.help = help(self.locale, self.help).map(Into::into);
        result.add_diagnostic(diagnostic);
    }
    fn as_native_sfc_setup_rule(&self) -> Option<&dyn NativeSfcSetupRule> {
        self.provided.then_some(self)
    }
    fn runs_on_script_setup(&self) -> bool {
        self.runs_setup
    }
}
impl NativeSfcSetupRule for Audit {
    fn run_on_setup<'a>(
        &self,
        context: &mut NativeSfcLintContext<'_, 'a>,
        setup: &NativeSfcSetup<'_, 'a>,
    ) -> Result<(), NativeSfcLintRefusal> {
        assert!(core::ptr::eq(context.owner(), setup.owner()));
        assert!(core::ptr::eq(
            setup.semantic().exposure().file(),
            context.owner().file()
        ));
        let bindings: Vec<_> = setup
            .semantic()
            .bindings()
            .map(|binding| {
                let d = binding.declaration().unwrap();
                serde_json::json!({ "id": d.id.index(), "unit": d.unit.index(),
                "scope": d.scope.index(), "name": d.name.as_str(), "span": d.span,
                "namespace": format!("{:?}", d.namespace), "kind": format!("{:?}", d.kind),
                "initializer": format!("{:?}", d.initializer), "import_source": d.import_source,
                "imported_name": d.imported_name, "direct": d.is_direct_program() })
            })
            .collect();
        self.log.lock().unwrap().push(serde_json::json!({
            "marker": self.marker, "rule": self.meta.name, "filename": context.filename(),
            "source": context.source(), "script": setup.semantic().source().span(),
            "template": setup.owner().template().component().block().span(),
            "template_index": setup.owner().template().template_index(),
            "unit": setup.semantic().exposure().unit().index(), "bindings": bindings,
            "vue": context.requested_vue_version().map(|v| format!("{v:?}")),
            "vapor": context.requested_vapor_mode(), "locale": format!("{:?}", context.locale()),
            "help": format!("{:?}", context.help_level()),
        }));
        context.warn_setup_with_help(
            setup,
            "vue/component-definition-name-casing.message",
            &[("name", self.marker)],
            "vue/component-definition-name-casing.help",
        )?;
        if self.fail {
            return Err(NativeSfcLintRefusal::UnsupportedEnvelope {
                span: setup.semantic().source().span(),
            });
        }
        Ok(())
    }
}
pub fn audit(
    meta: &'static ScriptRuleMeta,
    marker: &'static str,
    locale: Locale,
    help: HelpLevel,
    log: &Log,
) -> Audit {
    Audit {
        meta,
        marker,
        locale,
        help,
        log: log.clone(),
        provided: true,
        runs_setup: true,
        fail: false,
    }
}
pub fn log() -> Log {
    Arc::new(Mutex::new(Vec::new()))
}
pub fn configured(rules: Vec<Audit>, locale: Locale, help: HelpLevel) -> Linter {
    let names = rules.iter().map(|rule| rule.meta.name.into()).collect();
    let mut linter = Linter::with_registry(RuleRegistry::new())
        .with_enabled_rules(Some(names))
        .with_locale(locale)
        .with_help_level(help);
    for rule in rules {
        linter
            .script_rule_overrides
            .insert(rule.meta.name, Box::new(rule));
    }
    linter
}
pub fn single(log: &Log, locale: Locale) -> Linter {
    configured(
        vec![audit(&FIRST, "actual", locale, HelpLevel::Full, log)],
        locale,
        HelpLevel::Full,
    )
}
pub fn span(source: &str, text: &str) -> Span {
    let start = source.find(text).unwrap() as u32;
    Span::new(start, start + text.len() as u32)
}
pub fn script_span(source: &str) -> Span {
    let opener = source.find("<script").unwrap();
    let start = (opener + source[opener..].find('>').unwrap() + 1) as u32;
    Span::new(
        start,
        source
            .find("</script>")
            .or_else(|| source.find("</SCRIPT >"))
            .unwrap() as u32,
    )
}
pub fn expected_event(
    source: &str,
    file: &str,
    meta: &ScriptRuleMeta,
    marker: &str,
    locale: Locale,
    help_level: HelpLevel,
    vue: Option<VueVersion>,
    vapor: Option<bool>,
) -> serde_json::Value {
    let script = script_span(source);
    let script_first = source.find("<script").unwrap() < source.find("<template>").unwrap();
    let unit = u32::from(!script_first);
    let template_index = usize::from(script_first);
    let declaration_start = script.start
        + source[script.start as usize..script.end as usize]
            .find("text")
            .unwrap() as u32;
    let declaration = Span::new(declaration_start, declaration_start + 4);
    let template_start = source.find("<template>").unwrap() as u32 + 10;
    serde_json::json!({
        "marker": marker, "rule": meta.name, "filename": file, "source": source,
        "script": script, "template": Span::new(template_start, source.find("</template>").or_else(|| source.find("</TEMPLATE\t>")).unwrap() as u32),
        "template_index": template_index, "unit": unit,
        "bindings": [{ "id": 0, "unit": unit, "scope": 1, "name": "text", "span": declaration,
            "namespace": "Value", "kind": "Const", "initializer": "PrimitiveLiteral",
            "import_source": null, "imported_name": null, "direct": true }],
        "vue": vue.map(|v| format!("{v:?}")), "vapor": vapor,
        "locale": format!("{locale:?}"), "help": format!("{help_level:?}"),
    })
}
pub fn complete(result: &LintResult) -> serde_json::Value {
    let diagnostics: Vec<_> =
        result
            .diagnostics
            .iter()
            .map(|d| {
                let labels: Vec<_> = d.labels.iter().map(|label| serde_json::json!({
            "message": label.message.as_str(), "start": label.start, "end": label.end,
        })).collect();
                serde_json::json!({ "rule_name": d.rule_name, "severity": d.severity,
            "message": d.message.as_str(), "start": d.start, "end": d.end,
            "help": d.help.as_ref().map(|help| help.as_str()), "labels": labels, "fix": d.fix })
            })
            .collect();
    serde_json::json!({ "filename": result.filename.as_str(), "diagnostics": diagnostics,
        "error_count": result.error_count, "warning_count": result.warning_count })
}
pub fn empty(file: &str) -> serde_json::Value {
    serde_json::json!({ "filename": file, "diagnostics": [], "error_count": 0, "warning_count": 0 })
}
pub fn diagnostic(
    rule: &str,
    locale: Locale,
    level: HelpLevel,
    marker: &str,
    point: u32,
    severity: Severity,
) -> serde_json::Value {
    serde_json::json!({ "rule_name": rule, "severity": severity, "message": message(locale, marker),
        "start": point, "end": point, "help": help(locale, level), "labels": [], "fix": null })
}
pub fn expected(file: &str, diagnostics: Vec<serde_json::Value>) -> serde_json::Value {
    let errors = diagnostics
        .iter()
        .filter(|d| d["severity"] == "error")
        .count();
    serde_json::json!({ "filename": file, "error_count": errors,
        "warning_count": diagnostics.len() - errors, "diagnostics": diagnostics })
}
pub fn message(locale: Locale, marker: &str) -> std::string::String {
    match locale {
        Locale::En => format!("Component file name '{marker}' should be PascalCase or kebab-case"),
        Locale::Ja => {
            format!("コンポーネントファイル名'{marker}'はPascalCaseまたはkebab-caseにすべきです")
        }
        Locale::Zh => format!("组件文件名'{marker}'应使用PascalCase或kebab-case"),
    }
}
pub fn help(locale: Locale, level: HelpLevel) -> Option<std::string::String> {
    let first = match locale {
        Locale::En => "Rename the file to PascalCase or kebab-case:",
        Locale::Ja => "ファイル名をPascalCaseまたはkebab-caseに変更してください:",
        Locale::Zh => "将文件名改为PascalCase或kebab-case:",
    };
    match level {
        HelpLevel::None => None,
        HelpLevel::Short => Some(first.into()),
        HelpLevel::Full => Some(format!(
            "{first}\n```\nmyComponent.vue  -> MyComponent.vue\nmy-Component.vue -> my-component.vue\n```"
        )),
    }
}
