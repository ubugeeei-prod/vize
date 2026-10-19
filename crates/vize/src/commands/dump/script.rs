//! Explicit native Program inspection; no product compile or syntax reparse.

use oxc_ast::ast::Statement;
use oxc_span::GetSpan;
use vize_l0::{Allocator, Span, String, append};
use vize_l1::embed::syntax::{ProgramGoal, ProgramOptions, parse_program_once};
use vize_l1::embed::{EmbedSource, Lang, SourceError};

pub(super) struct Inspected {
    pub printed: String,
    pub facts: String,
}

pub(super) fn inspect(input: &str, options: ProgramOptions) -> Result<Inspected, SourceError> {
    let end = u32::try_from(input.len()).map_err(|_| SourceError::SourceTooLarge)?;
    let source = EmbedSource::authored(input, Span::new(0, end))?;
    let allocator = Allocator::default();
    let language = match (options.lang, options.jsx) {
        (Lang::Js, false) => "js",
        (Lang::Js, true) => "jsx",
        (Lang::Ts, false) => "ts",
        (Lang::Ts, true) => "tsx",
    };
    let goal = match options.goal {
        ProgramGoal::Module => "module",
        ProgramGoal::Script => "script",
    };
    let syntax = parse_program_once(&allocator, source, options);
    let mut facts = String::default();
    append!(
        facts,
        "script: {language} {goal}; hole={:?}\n",
        syntax.hole()
    );
    if let Some(program) = syntax.program() {
        // Only direct Program children are observed. Do not add an unbounded
        // recursive Debug traversal merely to print this standalone inspection.
        for directive in &program.directives {
            append!(
                facts,
                "directive @{:?} {:?}\n",
                syntax.authored_span(directive.span)?,
                directive.directive
            );
        }
        for statement in &program.body {
            append!(
                facts,
                "statement {} @{:?}\n",
                statement_kind(statement),
                syntax.authored_span(statement.span())?
            );
        }
    }
    for comment in syntax.comments() {
        append!(
            facts,
            "comment {:?} @{:?} {:?}\n",
            comment.kind(),
            comment.authored_span()?,
            comment.text()?
        );
    }
    for diagnostic in syntax.diagnostics() {
        append!(
            facts,
            "diagnostic {:?}: {}\n",
            diagnostic.severity(),
            diagnostic.message()
        );
        append!(
            facts,
            "  code={:?} help={:?} note={:?} url={:?}\n",
            diagnostic.code(),
            diagnostic.help(),
            diagnostic.note(),
            diagnostic.url()
        );
        for label in diagnostic.labels() {
            append!(
                facts,
                "  label @{:?} primary={} {:?}\n",
                label.authored_span()?,
                label.primary(),
                label.message()
            );
        }
    }
    let printed = String::from(syntax.source().text());
    drop(syntax);
    Ok(Inspected { printed, facts })
}

fn statement_kind(statement: &Statement<'_>) -> &'static str {
    match statement {
        Statement::BlockStatement(_) => "BlockStatement",
        Statement::BreakStatement(_) => "BreakStatement",
        Statement::ContinueStatement(_) => "ContinueStatement",
        Statement::DebuggerStatement(_) => "DebuggerStatement",
        Statement::DoWhileStatement(_) => "DoWhileStatement",
        Statement::EmptyStatement(_) => "EmptyStatement",
        Statement::ExpressionStatement(_) => "ExpressionStatement",
        Statement::ForInStatement(_) => "ForInStatement",
        Statement::ForOfStatement(_) => "ForOfStatement",
        Statement::ForStatement(_) => "ForStatement",
        Statement::IfStatement(_) => "IfStatement",
        Statement::LabeledStatement(_) => "LabeledStatement",
        Statement::ReturnStatement(_) => "ReturnStatement",
        Statement::SwitchStatement(_) => "SwitchStatement",
        Statement::ThrowStatement(_) => "ThrowStatement",
        Statement::TryStatement(_) => "TryStatement",
        Statement::WhileStatement(_) => "WhileStatement",
        Statement::WithStatement(_) => "WithStatement",
        Statement::VariableDeclaration(_) => "VariableDeclaration",
        Statement::FunctionDeclaration(_) => "FunctionDeclaration",
        Statement::ClassDeclaration(_) => "ClassDeclaration",
        Statement::TSTypeAliasDeclaration(_) => "TSTypeAliasDeclaration",
        Statement::TSInterfaceDeclaration(_) => "TSInterfaceDeclaration",
        Statement::TSEnumDeclaration(_) => "TSEnumDeclaration",
        Statement::TSModuleDeclaration(_) => "TSModuleDeclaration",
        Statement::TSGlobalDeclaration(_) => "TSGlobalDeclaration",
        Statement::TSImportEqualsDeclaration(_) => "TSImportEqualsDeclaration",
        Statement::ImportDeclaration(_) => "ImportDeclaration",
        Statement::ExportAllDeclaration(_) => "ExportAllDeclaration",
        Statement::ExportDefaultDeclaration(_) => "ExportDefaultDeclaration",
        Statement::ExportNamedDeclaration(_) => "ExportNamedDeclaration",
        Statement::TSExportAssignment(_) => "TSExportAssignment",
        Statement::TSNamespaceExportDeclaration(_) => "TSNamespaceExportDeclaration",
    }
}

#[cfg(test)]
mod tests {
    use super::inspect;
    use vize_l1::embed::Lang;
    use vize_l1::embed::syntax::{ProgramGoal, ProgramOptions};

    #[test]
    fn explicit_languages_inspect_real_statement_kinds_and_preserve_every_byte() {
        for (lang, jsx, input, kind) in [
            (
                Lang::Js,
                false,
                "/* α */ import { ref } from 'vue';\r\nexport const x = ref(0);",
                "ImportDeclaration",
            ),
            (
                Lang::Ts,
                false,
                "/* β */ interface Props { value: number }\nexport const x: Props = { value: 1 };",
                "TSInterfaceDeclaration",
            ),
            (
                Lang::Js,
                true,
                "const node = <section>{values.map(value => <span>{value}</span>)}</section>;",
                "VariableDeclaration",
            ),
            (
                Lang::Ts,
                true,
                "type Props = { value: number }; export const View = ({ value }: Props) => <span>{value}</span>;",
                "TSTypeAliasDeclaration",
            ),
        ] {
            let observed = inspect(
                input,
                ProgramOptions {
                    lang,
                    jsx,
                    goal: ProgramGoal::Module,
                },
            )
            .unwrap();
            assert_eq!(observed.printed.as_str(), input);
            assert!(observed.facts.contains("hole=None"));
            assert!(observed.facts.contains(kind));
            assert!(!observed.facts.contains("diagnostic "));
        }
    }

    #[test]
    fn language_and_syntax_failures_remain_visible_without_losing_authored_bytes() {
        for (lang, goal, input) in [
            (
                Lang::Js,
                ProgramGoal::Module,
                "interface Props { value: number }",
            ),
            (
                Lang::Ts,
                ProgramGoal::Module,
                "export const node = <span>{value}</span>;",
            ),
            (Lang::Ts, ProgramGoal::Module, "f<>/((\nd=//#__PURE__0"),
        ] {
            let observed = inspect(
                input,
                ProgramOptions {
                    lang,
                    jsx: false,
                    goal,
                },
            )
            .unwrap();
            assert_eq!(observed.printed.as_str(), input);
            assert!(
                observed.facts.contains("hole=Some(Syntax)"),
                "input={input:?} facts={:?}",
                observed.facts
            );
            assert!(observed.facts.contains("diagnostic Error:"));
            assert!(observed.facts.contains("label @"));
            assert!(!observed.facts.contains("statement "));
        }
    }

    #[test]
    fn malformed_unicode_comments_and_every_utf8_prefix_are_observed_without_rewrite() {
        let input = "/* α */ const value: number = ;\r\n// β";
        let options = ProgramOptions {
            lang: Lang::Ts,
            jsx: true,
            goal: ProgramGoal::Script,
        };
        for cut in (0..=input.len()).filter(|&cut| input.is_char_boundary(cut)) {
            let authored = input.get(..cut).unwrap();
            let observed = inspect(authored, options).unwrap();
            assert_eq!(observed.printed.as_str(), authored);
            assert!(observed.facts.starts_with("script: tsx script; hole="));
        }
        let observed = inspect(input, options).unwrap();
        assert!(observed.facts.contains("/* α */"));
        assert!(observed.facts.contains("diagnostic Error:"));
        let valid = inspect("/* α */ const value: number = 1;\r\n// β", options).unwrap();
        assert!(valid.facts.contains("/* α */"));
        assert!(valid.facts.contains("// β"));
        assert!(valid.facts.contains("script: tsx script; hole=None"));
    }
}
