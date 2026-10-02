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
mod tests;
