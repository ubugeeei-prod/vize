//! Parser for the committed `[l3-dump-v2]` graph pages, written from the page
//! format alone.

use super::program::{EdgeKind, EffectScope, Op, OpKind, Phase, Program, Region, Span, StateEdge};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Section {
    None,
    Root,
    Regions,
    Ops,
    Edges,
    Effects,
}

fn fail<T>(line: usize, message: &str) -> Result<T, String> {
    Err(format!("line {line}: {message}"))
}

pub(super) fn field<'t>(line: usize, name: &str, token: &'t str) -> Result<&'t str, String> {
    match token
        .strip_prefix(name)
        .and_then(|rest| rest.strip_prefix('='))
    {
        Some(value) => Ok(value),
        None => fail(line, &format!("expected {name}=..., got {token}")),
    }
}

pub(super) fn number(line: usize, name: &str, text: &str) -> Result<u32, String> {
    match text.bytes().all(|b| b.is_ascii_digit()) {
        true => text
            .parse()
            .or_else(|_| fail(line, &format!("invalid {name} integer {text}"))),
        false => fail(line, &format!("invalid {name} integer {text}")),
    }
}

fn optional(line: usize, name: &str, text: &str) -> Result<Option<u32>, String> {
    match text {
        "-" => Ok(None),
        _ => number(line, name, text).map(Some),
    }
}

fn span(line: usize, text: &str) -> Result<Span, String> {
    match text.split(':').collect::<Vec<_>>().as_slice() {
        [start, stop] => Ok(Span {
            start: number(line, "span.start", start)?,
            stop: number(line, "span.stop", stop)?,
        }),
        _ => fail(line, &format!("invalid span {text}")),
    }
}

fn arity<'t, const N: usize>(line: usize, tokens: &[&'t str]) -> Result<[&'t str; N], String> {
    match <[&str; N]>::try_from(tokens) {
        Ok(tokens) => Ok(tokens),
        Err(_) => fail(line, &format!("expected {N} fields, got {}", tokens.len())),
    }
}

fn region(line: usize, tokens: &[&str]) -> Result<Region, String> {
    let [id, parent, owner, at] = arity(line, tokens)?;
    Ok(Region {
        id: number(line, "id", field(line, "id", id)?)?,
        parent: optional(line, "parent", field(line, "parent", parent)?)?,
        owner: optional(line, "owner", field(line, "owner", owner)?)?,
        span: span(line, field(line, "span", at)?)?,
    })
}

fn op(line: usize, tokens: &[&str]) -> Result<Op, String> {
    let [id, kind, region, effect, at] = arity(line, tokens)?;
    let id = number(line, "id", field(line, "id", id)?)?;
    let raw = field(line, "kind", kind)?;
    let Some(kind) = OpKind::parse(raw) else {
        return fail(line, &format!("unknown op kind {raw}"));
    };
    Ok(Op {
        id,
        kind,
        region: number(line, "region", field(line, "region", region)?)?,
        effect: optional(line, "effect", field(line, "effect", effect)?)?,
        span: span(line, field(line, "span", at)?)?,
    })
}

fn edge(line: usize, tokens: &[&str]) -> Result<StateEdge, String> {
    let [source, target, kind, effect] = arity(line, tokens)?;
    let source = number(line, "from", field(line, "from", source)?)?;
    let target = number(line, "to", field(line, "to", target)?)?;
    let raw = field(line, "kind", kind)?;
    let Some(kind) = EdgeKind::parse(raw) else {
        return fail(line, &format!("unknown edge kind {raw}"));
    };
    Ok(StateEdge {
        source,
        target,
        kind,
        effect: optional(line, "effect", field(line, "effect", effect)?)?,
    })
}

fn effect(line: usize, tokens: &[&str]) -> Result<EffectScope, String> {
    let [id, owner, region, at] = arity(line, tokens)?;
    Ok(EffectScope {
        id: number(line, "id", field(line, "id", id)?)?,
        owner: number(line, "owner", field(line, "owner", owner)?)?,
        region: number(line, "region", field(line, "region", region)?)?,
        span: span(line, field(line, "span", at)?)?,
    })
}

fn record(line: usize, text: &str, section: Section, program: &mut Program) -> Result<(), String> {
    let tokens: Vec<&str> = text.split(' ').collect();
    match section {
        Section::None => return fail(line, "record before [l3-dump-v2]"),
        Section::Root => {
            let raw = field(line, "phase", text)?;
            let Some(phase) = Phase::parse(raw) else {
                return fail(line, &format!("unknown phase {raw}"));
            };
            program.phase = phase;
        }
        Section::Regions => program.regions.push(region(line, &tokens)?),
        Section::Ops => program.ops.push(op(line, &tokens)?),
        Section::Edges => program.edges.push(edge(line, &tokens)?),
        Section::Effects => program.effects.push(effect(line, &tokens)?),
    }
    Ok(())
}

/// Parse one graph page. Section headers may repeat; records append in order.
pub fn parse_program(text: &str) -> Result<Program, String> {
    let mut program = Program::empty();
    let mut section = Section::None;
    for (index, raw) in text.split('\n').enumerate() {
        let line = index + 1;
        match raw.trim_ascii() {
            "" => {}
            "[l3-dump-v2]" => section = Section::Root,
            "[l3-dump-v2.regions]" => section = Section::Regions,
            "[l3-dump-v2.ops]" => section = Section::Ops,
            "[l3-dump-v2.edges]" => section = Section::Edges,
            "[l3-dump-v2.effects]" => section = Section::Effects,
            text => record(line, text, section, &mut program)?,
        }
    }
    Ok(program)
}
