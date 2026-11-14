use core::fmt::Write;

use poc_support::Error;
use poc_support::vue::{Binding, Bindings};
use rustpython_parser::ast::Ranged;
use rustpython_parser::{Parse, ast};
use vize_l0::{Span, String};

use super::{emit, unsupported};

/// Parse a Python module with RustPython, then lower the supported setup DSL.
pub(crate) fn script(source: &str, base: u32) -> Result<(String, String, Bindings), Error> {
    let suite = ast::Suite::parse(source, "<script setup lang=python>").map_err(|error| {
        let offset = u32::from(error.offset);
        Error::new(
            "Python parse error",
            Span::new(base + offset, base + offset.saturating_add(1)),
        )
    })?;
    let mut lowered = LoweredScript::default();
    let mut cursor = 0usize;
    for statement in &suite {
        let start = offset(statement.range().start());
        lowered.append_comments(source.get(cursor..start).unwrap_or_default());
        lowered.statement(statement, source, base)?;
        cursor = offset(statement.range().end());
    }
    lowered.append_comments(source.get(cursor..).unwrap_or_default());
    Ok((
        lowered.imports,
        String::from(lowered.body.trim_matches('\n')),
        lowered.bindings,
    ))
}

#[derive(Default)]
struct LoweredScript {
    imports: String,
    body: String,
    bindings: Bindings,
}

impl LoweredScript {
    fn statement(&mut self, statement: &ast::Stmt, source: &str, base: u32) -> Result<(), Error> {
        match statement {
            ast::Stmt::ImportFrom(node)
                if node.module.as_ref().is_some_and(|module| module == "vue")
                    && node.level.as_ref().is_none_or(|level| level.to_u32() == 0) =>
            {
                self.vue_import(node, source, base)
            }
            ast::Stmt::Expr(node) if define_props(&node.value).is_some() => {
                self.props(define_props(&node.value).unwrap_or_default(), source, base)
            }
            ast::Stmt::Assign(node) if node.targets.len() == 1 => {
                self.top_level_assignment(node, source, base)
            }
            ast::Stmt::FunctionDef(node) => self.function(node, source, base),
            _ => Err(unsupported(source, base, statement.range())),
        }
    }

    fn vue_import(
        &mut self,
        node: &ast::StmtImportFrom,
        source: &str,
        base: u32,
    ) -> Result<(), Error> {
        self.imports.push_str("import { ");
        for (index, alias) in node.names.iter().enumerate() {
            if alias.name.as_str() == "*" {
                return Err(unsupported(source, base, alias.range));
            }
            if index > 0 {
                self.imports.push_str(", ");
            }
            self.imports.push_str(alias.name.as_str());
            if let Some(asname) = &alias.asname {
                write!(self.imports, " as {asname}")
                    .map_err(|_| unsupported(source, base, node.range))?;
            }
        }
        self.imports.push_str(" } from \"vue\";\n");
        Ok(())
    }

    fn props(&mut self, values: &[ast::Expr], source: &str, base: u32) -> Result<(), Error> {
        for value in values {
            let ast::Expr::Constant(constant) = value else {
                return Err(unsupported(source, base, value.range()));
            };
            let ast::Constant::Str(name) = &constant.value else {
                return Err(unsupported(source, base, value.range()));
            };
            self.bindings.declare(name, Binding::Prop);
        }
        Ok(())
    }

    fn top_level_assignment(
        &mut self,
        node: &ast::StmtAssign,
        source: &str,
        base: u32,
    ) -> Result<(), Error> {
        let Some(ast::Expr::Name(target)) = node.targets.first() else {
            return Err(unsupported(source, base, node.range));
        };
        let value = emit::expression(&node.value)
            .map_err(|()| unsupported(source, base, node.value.range()))?;
        let kind = if matches!(node.value.as_ref(), ast::Expr::Call(call) if matches!(call.func.as_ref(), ast::Expr::Name(name) if name.id.as_str() == "ref"))
        {
            Binding::Ref
        } else {
            Binding::Plain
        };
        self.bindings.declare(target.id.as_str(), kind);
        writeln!(self.body, "const {} = {value};", target.id)
            .map_err(|_| unsupported(source, base, node.range))
    }

    fn function(
        &mut self,
        node: &ast::StmtFunctionDef,
        source: &str,
        base: u32,
    ) -> Result<(), Error> {
        if !node.decorator_list.is_empty()
            || node.returns.is_some()
            || !node.type_params.is_empty()
            || node.args.vararg.is_some()
            || node.args.kwarg.is_some()
            || !node.args.kwonlyargs.is_empty()
        {
            return Err(unsupported(source, base, node.range));
        }
        write!(self.body, "function {}(", node.name)
            .map_err(|_| unsupported(source, base, node.range))?;
        let mut first = true;
        for argument in node.args.posonlyargs.iter().chain(&node.args.args) {
            if argument.default.is_some() || argument.def.annotation.is_some() {
                return Err(unsupported(source, base, argument.def.range));
            }
            if !first {
                self.body.push_str(", ");
            }
            first = false;
            self.body.push_str(argument.def.arg.as_str());
        }
        self.body.push_str(") {\n");
        for statement in &node.body {
            lower_function_statement(statement, &mut self.body, source, base)?;
        }
        self.body.push_str("}\n");
        self.bindings.declare(node.name.as_str(), Binding::Plain);
        Ok(())
    }

    fn append_comments(&mut self, gap: &str) {
        if gap.contains("\n\n") && !self.body.ends_with("\n\n") {
            self.body.push('\n');
        }
        for line in gap.lines() {
            if let Some(comment) = line.trim().strip_prefix('#') {
                writeln!(self.body, "//{comment}").ok();
            }
        }
    }
}

fn lower_function_statement(
    statement: &ast::Stmt,
    body: &mut String,
    source: &str,
    base: u32,
) -> Result<(), Error> {
    match statement {
        ast::Stmt::Return(node) => {
            let value = node
                .value
                .as_deref()
                .ok_or_else(|| unsupported(source, base, node.range))?;
            let value =
                emit::expression(value).map_err(|()| unsupported(source, base, value.range()))?;
            writeln!(body, "  return {value};").map_err(|_| unsupported(source, base, node.range))
        }
        ast::Stmt::Expr(node) => {
            let value = emit::expression(&node.value)
                .map_err(|()| unsupported(source, base, node.value.range()))?;
            writeln!(body, "  {value};").map_err(|_| unsupported(source, base, node.range))
        }
        ast::Stmt::Assign(node) if node.targets.len() == 1 => {
            let target = node
                .targets
                .first()
                .ok_or_else(|| unsupported(source, base, node.range))?;
            let target =
                emit::expression(target).map_err(|()| unsupported(source, base, target.range()))?;
            let value = emit::expression(&node.value)
                .map_err(|()| unsupported(source, base, node.value.range()))?;
            writeln!(body, "  {target} = {value};")
                .map_err(|_| unsupported(source, base, node.range))
        }
        ast::Stmt::AugAssign(node) => {
            let target = emit::expression(&node.target)
                .map_err(|()| unsupported(source, base, node.target.range()))?;
            let value = emit::expression(&node.value)
                .map_err(|()| unsupported(source, base, node.value.range()))?;
            let operator = assignment_operator(node.op)
                .ok_or_else(|| unsupported(source, base, node.range))?;
            writeln!(body, "  {target} {operator}= {value};")
                .map_err(|_| unsupported(source, base, node.range))
        }
        _ => Err(unsupported(source, base, statement.range())),
    }
}

fn assignment_operator(operator: ast::Operator) -> Option<&'static str> {
    Some(match operator {
        ast::Operator::Add => "+",
        ast::Operator::Sub => "-",
        ast::Operator::Mult => "*",
        ast::Operator::Div => "/",
        ast::Operator::Mod => "%",
        ast::Operator::Pow => "**",
        _ => return None,
    })
}

fn define_props(expression: &ast::Expr) -> Option<&[ast::Expr]> {
    let ast::Expr::Call(call) = expression else {
        return None;
    };
    if !call.keywords.is_empty() || call.args.len() != 1 {
        return None;
    }
    let ast::Expr::Name(function) = call.func.as_ref() else {
        return None;
    };
    let ast::Expr::List(list) = call.args.first()? else {
        return None;
    };
    (function.id.as_str() == "define_props").then_some(list.elts.as_slice())
}

fn offset(value: ast::text_size::TextSize) -> usize {
    u32::from(value) as usize
}
