use core::fmt::{self, Write};

use rustpython_parser::{Parse, ast};
use vize_l0::String;

const PREC_TERNARY: u8 = 1;
const PREC_OR: u8 = 2;
const PREC_AND: u8 = 3;
const PREC_COMPARE: u8 = 4;
const PREC_ADD: u8 = 5;
const PREC_MUL: u8 = 6;
const PREC_UNARY: u8 = 7;
const PREC_POWER: u8 = 8;
const PREC_PRIMARY: u8 = 9;

pub(super) fn parse(source: &str) -> Result<ast::Expr, ()> {
    ast::Expr::parse(source.trim(), "<template expression>").map_err(|_| ())
}

pub(super) fn expression(expr: &ast::Expr) -> Result<String, ()> {
    let mut out = String::default();
    write_expression(expr, &mut out).map_err(|_| ())?;
    Ok(out)
}

pub(super) fn write_expression(expr: &ast::Expr, out: &mut dyn Write) -> Result<(), fmt::Error> {
    emit(expr, out, 0).map_err(|()| fmt::Error)
}

fn emit(expr: &ast::Expr, out: &mut dyn Write, parent: u8) -> Result<(), ()> {
    let precedence = precedence(expr)?;
    let wrapped = precedence < parent;
    if wrapped {
        out.write_char('(').map_err(|_| ())?;
    }
    match expr {
        ast::Expr::Name(node) => out.write_str(node.id.as_str()).map_err(|_| ())?,
        ast::Expr::Constant(node) => emit_constant(&node.value, out)?,
        ast::Expr::BoolOp(node) => {
            let operator = match node.op {
                ast::BoolOp::And => " && ",
                ast::BoolOp::Or => " || ",
            };
            emit_joined(&node.values, operator, out, precedence)?;
        }
        ast::Expr::BinOp(node) => {
            emit(&node.left, out, precedence)?;
            let operator = binary_operator(node.op).ok_or(())?;
            write!(out, " {operator} ").map_err(|_| ())?;
            emit(&node.right, out, precedence.saturating_add(1))?;
        }
        ast::Expr::UnaryOp(node) => {
            let operator = match node.op {
                ast::UnaryOp::Not => "!",
                ast::UnaryOp::UAdd => "+",
                ast::UnaryOp::USub => "-",
                ast::UnaryOp::Invert => return Err(()),
            };
            out.write_str(operator).map_err(|_| ())?;
            emit(&node.operand, out, precedence)?;
        }
        ast::Expr::IfExp(node) => {
            emit(&node.test, out, PREC_OR)?;
            out.write_str(" ? ").map_err(|_| ())?;
            emit(&node.body, out, PREC_TERNARY)?;
            out.write_str(" : ").map_err(|_| ())?;
            emit(&node.orelse, out, PREC_TERNARY)?;
        }
        ast::Expr::Compare(node) if node.ops.len() == 1 && node.comparators.len() == 1 => {
            emit(&node.left, out, precedence)?;
            let operator = node.ops.first().and_then(compare_operator).ok_or(())?;
            write!(out, " {operator} ").map_err(|_| ())?;
            emit(node.comparators.first().ok_or(())?, out, precedence)?;
        }
        ast::Expr::Call(node) => emit_call(node, out)?,
        ast::Expr::Attribute(node) => {
            emit(&node.value, out, PREC_PRIMARY)?;
            write!(out, ".{}", attribute_name(node.attr.as_str())).map_err(|_| ())?;
        }
        ast::Expr::Subscript(node) => {
            emit(&node.value, out, PREC_PRIMARY)?;
            out.write_char('[').map_err(|_| ())?;
            emit(&node.slice, out, 0)?;
            out.write_char(']').map_err(|_| ())?;
        }
        ast::Expr::List(node) => emit_sequence(&node.elts, '[', ']', out)?,
        ast::Expr::Tuple(node) => emit_sequence(&node.elts, '[', ']', out)?,
        ast::Expr::JoinedStr(node) => emit_joined_string(&node.values, out)?,
        _ => return Err(()),
    }
    if wrapped {
        out.write_char(')').map_err(|_| ())?;
    }
    Ok(())
}

fn emit_call(node: &ast::ExprCall, out: &mut dyn Write) -> Result<(), ()> {
    if !node.keywords.is_empty() {
        return Err(());
    }
    if let ast::Expr::Name(name) = node.func.as_ref()
        && name.id.as_str() == "len"
        && node.args.len() == 1
    {
        emit(node.args.first().ok_or(())?, out, PREC_PRIMARY)?;
        return out.write_str(".length").map_err(|_| ());
    }
    emit(&node.func, out, PREC_PRIMARY)?;
    out.write_char('(').map_err(|_| ())?;
    emit_arguments(&node.args, out)?;
    out.write_char(')').map_err(|_| ())
}

fn emit_arguments(values: &[ast::Expr], out: &mut dyn Write) -> Result<(), ()> {
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            out.write_str(", ").map_err(|_| ())?;
        }
        emit(value, out, 0)?;
    }
    Ok(())
}

fn emit_sequence(
    values: &[ast::Expr],
    open: char,
    close: char,
    out: &mut dyn Write,
) -> Result<(), ()> {
    out.write_char(open).map_err(|_| ())?;
    emit_arguments(values, out)?;
    out.write_char(close).map_err(|_| ())
}

fn emit_joined(
    values: &[ast::Expr],
    separator: &str,
    out: &mut dyn Write,
    prec: u8,
) -> Result<(), ()> {
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            out.write_str(separator).map_err(|_| ())?;
        }
        emit(value, out, prec)?;
    }
    Ok(())
}

fn emit_joined_string(values: &[ast::Expr], out: &mut dyn Write) -> Result<(), ()> {
    out.write_char('`').map_err(|_| ())?;
    for value in values {
        match value {
            ast::Expr::Constant(node) => match &node.value {
                ast::Constant::Str(text) => out
                    .write_str(&text.replace('`', "\\`").replace("${", "\\${"))
                    .map_err(|_| ())?,
                _ => return Err(()),
            },
            ast::Expr::FormattedValue(node)
                if node.conversion == ast::ConversionFlag::None && node.format_spec.is_none() =>
            {
                out.write_str("${").map_err(|_| ())?;
                emit(&node.value, out, 0)?;
                out.write_char('}').map_err(|_| ())?;
            }
            _ => return Err(()),
        }
    }
    out.write_char('`').map_err(|_| ())
}

fn emit_constant(value: &ast::Constant, out: &mut dyn Write) -> Result<(), ()> {
    match value {
        ast::Constant::None => out.write_str("null").map_err(|_| ()),
        ast::Constant::Bool(value) => write!(out, "{value}").map_err(|_| ()),
        ast::Constant::Str(value) => {
            let quoted = serde_json::to_string(value).map_err(|_| ())?;
            out.write_str(&quoted).map_err(|_| ())
        }
        ast::Constant::Int(value) => write!(out, "{value}").map_err(|_| ()),
        ast::Constant::Float(value) => write!(out, "{value}").map_err(|_| ()),
        _ => Err(()),
    }
}

fn precedence(expr: &ast::Expr) -> Result<u8, ()> {
    Ok(match expr {
        ast::Expr::IfExp(_) => PREC_TERNARY,
        ast::Expr::BoolOp(node) => match node.op {
            ast::BoolOp::Or => PREC_OR,
            ast::BoolOp::And => PREC_AND,
        },
        ast::Expr::Compare(_) => PREC_COMPARE,
        ast::Expr::BinOp(node) => match node.op {
            ast::Operator::Add | ast::Operator::Sub => PREC_ADD,
            ast::Operator::Mult | ast::Operator::Div | ast::Operator::Mod => PREC_MUL,
            ast::Operator::Pow => PREC_POWER,
            _ => return Err(()),
        },
        ast::Expr::UnaryOp(_) => PREC_UNARY,
        ast::Expr::Name(_)
        | ast::Expr::Constant(_)
        | ast::Expr::Call(_)
        | ast::Expr::Attribute(_)
        | ast::Expr::Subscript(_)
        | ast::Expr::List(_)
        | ast::Expr::Tuple(_)
        | ast::Expr::JoinedStr(_) => PREC_PRIMARY,
        _ => return Err(()),
    })
}

fn binary_operator(operator: ast::Operator) -> Option<&'static str> {
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

fn compare_operator(operator: &ast::CmpOp) -> Option<&'static str> {
    Some(match operator {
        ast::CmpOp::Eq => "===",
        ast::CmpOp::NotEq => "!==",
        ast::CmpOp::Lt => "<",
        ast::CmpOp::LtE => "<=",
        ast::CmpOp::Gt => ">",
        ast::CmpOp::GtE => ">=",
        ast::CmpOp::Is => "===",
        ast::CmpOp::IsNot => "!==",
        ast::CmpOp::In | ast::CmpOp::NotIn => return None,
    })
}

fn attribute_name(name: &str) -> &str {
    match name {
        "upper" => "toUpperCase",
        "lower" => "toLowerCase",
        "append" => "push",
        _ => name,
    }
}

pub(super) fn is_supported(expr: &ast::Expr) -> bool {
    expression(expr).is_ok()
}

pub(super) fn enumerate_bindings(expr: &ast::Expr, each: &mut dyn FnMut(&str)) {
    match expr {
        ast::Expr::Name(node) if !matches!(node.id.as_str(), "len" | "True" | "False" | "None") => {
            each(node.id.as_str());
        }
        ast::Expr::BoolOp(node) => visit_many(&node.values, each),
        ast::Expr::BinOp(node) => {
            enumerate_bindings(&node.left, each);
            enumerate_bindings(&node.right, each);
        }
        ast::Expr::UnaryOp(node) => enumerate_bindings(&node.operand, each),
        ast::Expr::IfExp(node) => {
            enumerate_bindings(&node.test, each);
            enumerate_bindings(&node.body, each);
            enumerate_bindings(&node.orelse, each);
        }
        ast::Expr::Compare(node) => {
            enumerate_bindings(&node.left, each);
            visit_many(&node.comparators, each);
        }
        ast::Expr::Call(node) => {
            if !matches!(node.func.as_ref(), ast::Expr::Name(name) if name.id.as_str() == "len") {
                enumerate_bindings(&node.func, each);
            }
            visit_many(&node.args, each);
            for keyword in &node.keywords {
                enumerate_bindings(&keyword.value, each);
            }
        }
        ast::Expr::Attribute(node) => enumerate_bindings(&node.value, each),
        ast::Expr::Subscript(node) => {
            enumerate_bindings(&node.value, each);
            enumerate_bindings(&node.slice, each);
        }
        ast::Expr::List(node) => visit_many(&node.elts, each),
        ast::Expr::Tuple(node) => visit_many(&node.elts, each),
        ast::Expr::JoinedStr(node) => visit_many(&node.values, each),
        ast::Expr::FormattedValue(node) => enumerate_bindings(&node.value, each),
        _ => {}
    }
}

fn visit_many(values: &[ast::Expr], each: &mut dyn FnMut(&str)) {
    for value in values {
        enumerate_bindings(value, each);
    }
}

pub(super) fn is_constant(expr: &ast::Expr) -> bool {
    match expr {
        ast::Expr::Constant(_) => true,
        ast::Expr::List(node) => node.elts.iter().all(is_constant),
        ast::Expr::Tuple(node) => node.elts.iter().all(is_constant),
        ast::Expr::UnaryOp(node) => is_constant(&node.operand),
        ast::Expr::BinOp(node) => is_constant(&node.left) && is_constant(&node.right),
        ast::Expr::BoolOp(node) => node.values.iter().all(is_constant),
        ast::Expr::Compare(node) => {
            is_constant(&node.left) && node.comparators.iter().all(is_constant)
        }
        ast::Expr::IfExp(node) => {
            is_constant(&node.test) && is_constant(&node.body) && is_constant(&node.orelse)
        }
        ast::Expr::JoinedStr(node) => node.values.iter().all(is_constant),
        ast::Expr::FormattedValue(node) => is_constant(&node.value),
        _ => false,
    }
}
