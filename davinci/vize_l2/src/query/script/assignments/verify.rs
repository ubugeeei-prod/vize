use super::{
    Assignment, BindingPattern, Expression, GetSpan, LiteralAssignments, Primitive, Row,
    SourceBlock, TSType, VariableDeclarationKind, Verdict, absolute,
};
use oxc_ast::ast::{Declaration, Statement, VariableDeclarator};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssignmentError {
    Missing,
    Extra,
    Outside,
    Source,
    Type,
    Verdict,
}

/// Checked against a complete enumeration of the original statement slots.
/// Borrowing both operands binds the evidence to that immutable program.
pub struct CheckedAssignments<'r, 'p, 'a> {
    rows: &'r [Row],
    original: &'p vize_l1::embed::authored_program::AuthoredProgram<'a>,
}
impl<'r, 'p, 'a> CheckedAssignments<'r, 'p, 'a> {
    pub fn original(&self) -> &'p vize_l1::embed::authored_program::AuthoredProgram<'a> {
        self.original
    }
    pub fn source(&self) -> SourceBlock<'a> {
        self.original.block()
    }
    pub fn rows(&self) -> &'r [Row] {
        self.rows
    }
    pub fn assignments(&self) -> impl Iterator<Item = &'r Assignment> {
        self.rows.iter().filter_map(|r| match r {
            Row::Assignment(a) => Some(a),
            Row::Outside(_) => None,
        })
    }
    pub fn is_complete(&self) -> bool {
        self.rows.iter().all(|r| matches!(r, Row::Assignment(_)))
    }
}

impl<'p, 'a> LiteralAssignments<'p, 'a> {
    /// The verifier reads the original keyword spelling and literal AST kinds.
    /// A proof cannot hide a supported initializer as an outside row, attribute
    /// an initializer to a different binding, or invent a mismatch.
    pub fn check<'r>(
        &self,
        rows: &'r [Row],
    ) -> Result<CheckedAssignments<'r, 'p, 'a>, AssignmentError> {
        let source = self.original.block();
        let mut claims = rows.iter();
        // Enumerate the original AST independently of the evidence producer.
        // In particular a missed statement cannot silently disappear from a
        // proof that Patina relies on to account for every possible write.
        for stmt in &self.original.parsed().program.body {
            let (variables, exported) = match stmt {
                Statement::EmptyStatement(_) => continue,
                Statement::VariableDeclaration(v) if !v.declare => (Some(v), false),
                Statement::ExportNamedDeclaration(e) if !e.export_kind.is_type() => {
                    match e.declaration.as_ref() {
                        Some(Declaration::VariableDeclaration(v)) if !v.declare => (Some(v), true),
                        _ => (None, false),
                    }
                }
                _ => (None, false),
            };
            if let Some(list) = variables {
                for decl in &list.declarations {
                    check_variable(
                        source,
                        decl,
                        exported,
                        claims.next().ok_or(AssignmentError::Missing)?,
                    )?;
                }
            } else {
                outside(
                    source,
                    stmt.span(),
                    claims.next().ok_or(AssignmentError::Missing)?,
                )?;
            }
        }
        if claims.next().is_some() {
            return Err(AssignmentError::Extra);
        }
        Ok(CheckedAssignments {
            rows,
            original: self.original,
        })
    }
}

fn outside(
    source: SourceBlock<'_>,
    span: oxc_span::Span,
    claim: &Row,
) -> Result<(), AssignmentError> {
    if claim == &Row::Outside(absolute(source, span)) {
        Ok(())
    } else {
        Err(AssignmentError::Outside)
    }
}

fn check_variable(
    source: SourceBlock<'_>,
    decl: &VariableDeclarator<'_>,
    exported: bool,
    claim: &Row,
) -> Result<(), AssignmentError> {
    let (BindingPattern::BindingIdentifier(id), Some(ty), Some(expr)) =
        (&decl.id, &decl.type_annotation, &decl.init)
    else {
        return outside(source, decl.span, claim);
    };
    let annotation = &ty.type_annotation;
    if decl.definite
        || matches!(
            decl.kind,
            VariableDeclarationKind::Using | VariableDeclarationKind::AwaitUsing
        )
        || !matches!(
            absolute(source, annotation.span()).slice(source.root_source()),
            "string" | "boolean" | "number"
        )
        || !matches!(
            annotation,
            TSType::TSStringKeyword(_) | TSType::TSBooleanKeyword(_) | TSType::TSNumberKeyword(_)
        )
        || !matches!(
            expr,
            Expression::StringLiteral(_)
                | Expression::NumericLiteral(_)
                | Expression::BooleanLiteral(_)
        )
    {
        return outside(source, decl.span, claim);
    }
    let Row::Assignment(row) = claim else {
        return Err(AssignmentError::Outside);
    };
    if row.kind != decl.kind
        || row.exported != exported
        || row.binding != absolute(source, id.span)
        || row.annotation != absolute(source, annotation.span())
        || row.initializer != absolute(source, expr.span())
    {
        return Err(AssignmentError::Source);
    }
    let word = source
        .root_source()
        .get(row.annotation.start as usize..row.annotation.end as usize);
    if word != Some(row.expected.keyword())
        || !matches!(
            (expr, row.actual),
            (Expression::StringLiteral(_), Primitive::String)
                | (Expression::BooleanLiteral(_), Primitive::Boolean)
                | (Expression::NumericLiteral(_), Primitive::Number)
        )
    {
        return Err(AssignmentError::Type);
    }
    let same = matches!(
        (annotation, expr),
        (TSType::TSBooleanKeyword(_), Expression::BooleanLiteral(_))
            | (TSType::TSNumberKeyword(_), Expression::NumericLiteral(_))
            | (TSType::TSStringKeyword(_), Expression::StringLiteral(_))
    );
    if same != (row.verdict == Verdict::Assignable) {
        return Err(AssignmentError::Verdict);
    }
    Ok(())
}
