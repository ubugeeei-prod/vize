//! Relocate declarations using the same authored tree as the other module readers.
use super::{
    Declaration, FxHashSet, GetSpan, NamespaceHoistPlan, TSModuleDeclarationName, collect_captures,
    expando_targets, is_ambient_declaration, merge_partner_name, top_level_declaration,
};
use oxc_ast::ast::Program;

impl NamespaceHoistPlan {
    pub(in super::super) fn from_authored_program(program: &Program<'_>, script: &str) -> Self {
        let mut spans = Vec::new();
        let mut hoisted = FxHashSet::default();
        {
            let expandos = expando_targets(&program.body);
            // A function with an assigned property stays beside its assignments,
            // unless a class, enum or namespace gives it a module-scope partner.
            let mut partners = FxHashSet::default();
            // Canon publishes all namespace ranges first, then their merge
            // partners and other callable or nominal declarations in source order.
            for (decl, span) in program.body.iter().filter_map(top_level_declaration) {
                if matches!(decl, Declaration::TSModuleDeclaration(_)) {
                    spans.push((span.start, span.end));
                }
                if expandos.is_empty() {
                    continue;
                }
                let name = match decl {
                    Declaration::TSModuleDeclaration(module) => match &module.id {
                        TSModuleDeclarationName::Identifier(id) => id.name.as_str(),
                        _ => continue,
                    },
                    Declaration::ClassDeclaration(class) => {
                        let Some(id) = &class.id else { continue };
                        id.name.as_str()
                    }
                    Declaration::TSEnumDeclaration(enumeration) => enumeration.id.name.as_str(),
                    _ => continue,
                };
                if expandos.contains(name) {
                    partners.insert(name);
                }
            }
            for (decl, span) in program.body.iter().filter_map(top_level_declaration) {
                let Some(name) = merge_partner_name(decl) else {
                    continue;
                };
                if matches!(decl, Declaration::FunctionDeclaration(_))
                    && expandos.contains(name.as_str())
                    && !partners.contains(name.as_str())
                {
                    continue;
                }
                hoisted.insert(name);
                spans.push((span.start, span.end));
            }
        }
        if spans.is_empty() {
            return Self::default();
        }

        let mut captured = collect_captures(&program.body, &spans, &hoisted);
        if !captured.is_empty() {
            let mut comments = None;
            for statement in program.body.iter().filter(|s| is_ambient_declaration(s)) {
                let span = statement.span();
                let comments = comments.get_or_insert_with(|| {
                    program.comments.iter().map(|c| c.span).collect::<Vec<_>>()
                });
                if super::super::super::script_blocks::relocatable_ambient_line(
                    script, span, comments,
                ) {
                    captured.retain(|c| {
                        c.source_range.start < span.start as usize
                            || c.source_range.end > span.end as usize
                    });
                }
            }
        }
        for (start, _) in &mut spans {
            if let Some(comment) = program.comments.iter().find(|comment| {
                comment.is_jsdoc() && comment.attached_to == *start && {
                    let end = comment.span.start as usize;
                    script
                        .get(super::super::line_start_at(script, end)..end)
                        .is_some_and(|line| line.trim().is_empty())
                }
            }) {
                *start = comment.span.start;
            }
        }
        Self {
            spans: super::super::include_leading_ts_directive_comments(script, spans),
            hoisted,
            captured,
        }
    }
}
