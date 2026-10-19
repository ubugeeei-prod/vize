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

    const LEXED_HOLE: &str =
        "  code=OxcCode { scope: None, number: None } help=None note=None url=None\n";

    #[test]
    fn explicit_languages_inspect_real_statement_kinds_and_preserve_every_byte() {
        for (lang, jsx, input, facts) in [
            (
                Lang::Js,
                false,
                "/* α */ import { ref } from 'vue';\r\nexport const x = ref(0);",
                "script: js module; hole=None\n\
                 statement ImportDeclaration @Span { start: 9, end: 35 }\n\
                 statement ExportNamedDeclaration @Span { start: 37, end: 61 }\n\
                 comment SingleLineBlock @Span { start: 0, end: 8 } \"/* α */\"\n",
            ),
            (
                Lang::Ts,
                false,
                "/* β */ interface Props { value: number }\nexport const x: Props = { value: 1 };",
                "script: ts module; hole=None\n\
                 statement TSInterfaceDeclaration @Span { start: 9, end: 42 }\n\
                 statement ExportNamedDeclaration @Span { start: 43, end: 80 }\n\
                 comment SingleLineBlock @Span { start: 0, end: 8 } \"/* β */\"\n",
            ),
            (
                Lang::Js,
                true,
                "const node = <section>{values.map(value => <span>{value}</span>)}</section>;",
                "script: jsx module; hole=None\n\
                 statement VariableDeclaration @Span { start: 0, end: 76 }\n",
            ),
            (
                Lang::Ts,
                true,
                "type Props = { value: number }; export const View = ({ value }: Props) => <span>{value}</span>;",
                "script: tsx module; hole=None\n\
                 statement TSTypeAliasDeclaration @Span { start: 0, end: 31 }\n\
                 statement ExportNamedDeclaration @Span { start: 32, end: 95 }\n",
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
            assert_eq!(observed.facts.as_str(), facts);
        }
    }

    #[test]
    fn language_and_syntax_failures_remain_visible_without_losing_authored_bytes() {
        let regex = vize_l0::cstr!(
            "script: ts module; hole=Some(Syntax)\n\
             diagnostic Error: Unterminated regular expression\n{LEXED_HOLE}\
             \x20 label @Span {{ start: 34, end: 41 }} primary=false None\n\
             diagnostic Error: Unexpected token\n{LEXED_HOLE}\
             \x20 label @Span {{ start: 34, end: 41 }} primary=false None\n"
        );
        let generic = vize_l0::cstr!(
            "script: ts module; hole=Some(Syntax)\n\
             comment Line @Span {{ start: 9, end: 21 }} \"//#__PURE__0\"\n\
             diagnostic Error: Type argument list cannot be empty.\n\
             \x20 code=OxcCode {{ scope: Some(\"TS\"), number: Some(\"1099\") }} help=None note=None url=None\n\
             \x20 label @Span {{ start: 1, end: 3 }} primary=false None\n\
             diagnostic Error: Unexpected token\n{LEXED_HOLE}\
             \x20 label @Span {{ start: 21, end: 21 }} primary=false None\n"
        );
        for (lang, goal, input, facts) in [
            (
                Lang::Js,
                ProgramGoal::Module,
                "interface Props { value: number }",
                "script: js module; hole=Some(Syntax)\n\
                 diagnostic Error: Expected a semicolon or an implicit semicolon after a statement, but found none\n\
                 \x20 code=OxcCode { scope: None, number: None } help=Some(\"Try inserting a semicolon here\") note=None url=None\n\
                 \x20 label @Span { start: 9, end: 9 } primary=false None\n",
            ),
            (
                Lang::Ts,
                ProgramGoal::Module,
                "export const node = <span>{value}</span>;",
                regex.as_str(),
            ),
            (
                Lang::Ts,
                ProgramGoal::Module,
                "f<>/((\nd=//#__PURE__0",
                generic.as_str(),
            ),
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
            assert_eq!(observed.facts.as_str(), facts, "input={input:?}");
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
            // Only the empty prefix and the complete leading comment parse clean.
            let hole = if cut == 0 || (8..=13).contains(&cut) {
                "None"
            } else {
                "Some(Syntax)"
            };
            let header = vize_l0::cstr!("script: tsx script; hole={hole}");
            assert_eq!(
                observed.facts.lines().next(),
                Some(header.as_str()),
                "cut={cut}"
            );
        }

        let observed = inspect(input, options).unwrap();
        assert_eq!(
            observed.facts.as_str(),
            "script: tsx script; hole=Some(Syntax)\n\
             comment SingleLineBlock @Span { start: 0, end: 8 } \"/* α */\"\n\
             diagnostic Error: Unexpected token\n\
             \x20 code=OxcCode { scope: None, number: None } help=None note=None url=None\n\
             \x20 label @Span { start: 31, end: 32 } primary=false None\n"
        );
        let valid = inspect("/* α */ const value: number = 1;\r\n// β", options).unwrap();
        assert_eq!(
            valid.facts.as_str(),
            "script: tsx script; hole=None\n\
             statement VariableDeclaration @Span { start: 9, end: 33 }\n\
             comment SingleLineBlock @Span { start: 0, end: 8 } \"/* α */\"\n\
             comment Line @Span { start: 35, end: 40 } \"// β\"\n"
        );
    }
}
