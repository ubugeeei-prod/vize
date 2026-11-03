//! Native `v-model` reference semantics for P3-4 (`ui.model` in L3). A model
//! binding is a `set-prop` whose binding kind is `model`, with identical
//! `model-read`/`model-write` paths and `model-attribute` rows for the element
//! kind and the `lazy`/`trim`/`number` modifiers. Written from Vue's documented
//! directive contract, not from runtime code: text inputs with IME
//! composition, checkboxes (boolean and array membership), radios, and single
//! or multiple selects with static options. Live DOM state belongs to the
//! element and survives re-renders; directive hooks rewrite it only as the
//! contract says. Anything else fails closed.

use super::expression::{number, primitive_text, strict_equal};
use super::iteration::path;
use super::json::Json;
use super::program::{Op, OpKind, Program};
use super::values::{Operand, for_op, literal, one};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Text,
    Checkbox,
    Radio,
    Select,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Control {
    pub kind: Kind,
    pub path: String,
    pub lazy: bool,
    pub trim: bool,
    pub number: bool,
    pub multiple: bool,
    pub dom_value: String,
    pub options: Vec<String>,
}

/// DOM-owned state of one control. `selected` is per static option.
#[derive(Clone, Debug, PartialEq)]
pub struct Live {
    pub value: String,
    pub checked: bool,
    pub selected: Vec<bool>,
    pub composing: bool,
    pub last_model: Json,
}

impl Default for Live {
    fn default() -> Self {
        Self {
            value: String::new(),
            checked: false,
            selected: Vec::new(),
            composing: false,
            last_model: Json::Null,
        }
    }
}

pub fn model_row(rows: &[Operand], op: &Op) -> bool {
    op.kind == OpKind::SetProp
        && for_op(rows, op.id)
            .iter()
            .any(|row| row.role == "binding-kind" && row.kind == "literal" && row.text == "model")
}

pub fn static_attr<'r>(rows: &'r [Operand], id: u32, name: &str) -> Option<&'r Operand> {
    for_op(rows, id)
        .into_iter()
        .find(|row| row.role == "attribute" && row.name.as_deref() == Some(name))
}

fn option_values(program: &Program, rows: &[Operand], select: &Op) -> Result<Vec<String>, String> {
    let regions: Vec<u32> = (program.regions.iter())
        .filter(|r| r.owner == Some(select.id))
        .map(|r| r.id)
        .collect();
    let [region] = regions[..] else {
        return Err("select requires one option region".into());
    };
    let mut values: Vec<String> = Vec::new();
    for op in program.ops.iter().filter(|op| op.region == region) {
        if op.kind != OpKind::InsertNode || literal(one(rows, op.id, "tag")?)? != "option" {
            return Err("select children must be static options".into());
        }
        let Some(value) = static_attr(rows, op.id, "value") else {
            return Err("option requires a static value".into());
        };
        values.push(literal(value)?.into());
    }
    let distinct = values
        .iter()
        .enumerate()
        .all(|(i, v)| !values.iter().take(i).any(|w| w == v));
    if values.is_empty() || !distinct {
        return Err("select options must be distinct".into());
    }
    Ok(values)
}

/// Validate one model binding and extract its static control description.
pub fn control(program: &Program, rows: &[Operand], binding: &Op) -> Result<Control, String> {
    let operands = for_op(rows, binding.id);
    let roles = [
        "name",
        "model-read",
        "model-write",
        "model-attribute",
        "binding-kind",
    ];
    if operands
        .iter()
        .any(|row| !roles.contains(&row.role.as_str()))
    {
        return Err("unsupported model operand".into());
    }
    if one(rows, binding.id, "name")?.kind != "absent" {
        return Err("unsupported model argument".into());
    }
    let read = one(rows, binding.id, "model-read")?;
    let write = one(rows, binding.id, "model-write")?;
    if read.kind != "js" || write.kind != "js" || read.text != write.text || !path(&read.text) {
        return Err("model requires one assignable path".into());
    }
    let Some(target) =
        (program.ops.iter()).find(|op| Some(op.id) == read.target && op.kind == OpKind::InsertNode)
    else {
        return Err("model target must be an element".into());
    };
    if operands.iter().any(|row| row.target != Some(target.id)) {
        return Err("malformed model target".into());
    }
    let attrs: Vec<&&Operand> = operands
        .iter()
        .filter(|row| row.role == "model-attribute")
        .collect();
    let flag = |name: &str| {
        attrs
            .iter()
            .any(|row| row.name.as_deref() == Some(name) && row.kind == "absent")
    };
    let known = ["element-kind", "lazy", "trim", "number"];
    let names: Vec<Option<&str>> = attrs.iter().map(|row| row.name.as_deref()).collect();
    let distinct = names
        .iter()
        .enumerate()
        .all(|(i, n)| !names.iter().take(i).any(|m| m == n));
    if attrs
        .iter()
        .any(|row| !known.contains(&row.name.as_deref().unwrap_or_default()))
        || !distinct
    {
        return Err("unsupported model modifier".into());
    }
    let Some(element_kind) = attrs
        .iter()
        .find(|row| row.name.as_deref() == Some("element-kind"))
    else {
        return Err("missing model element kind".into());
    };
    let element_kind = literal(element_kind)?;
    let tag = literal(one(rows, target.id, "tag")?)?;
    if element_kind != tag {
        return Err("model element kind disagrees with its tag".into());
    }
    let type_attr = static_attr(rows, target.id, "type").map_or(Ok("text"), literal)?;
    let dom_value = static_attr(rows, target.id, "value").map_or(Ok("on"), literal)?;
    let kind = match (tag, type_attr) {
        ("input", "text") => Kind::Text,
        ("input", "checkbox") => Kind::Checkbox,
        ("input", "radio") => Kind::Radio,
        ("select", _) => Kind::Select,
        _ => return Err("unsupported model control".into()),
    };
    let modifiers: Vec<&str> = ["lazy", "trim", "number"]
        .into_iter()
        .filter(|m| flag(m))
        .collect();
    let select_ok = kind == Kind::Select && (modifiers.is_empty() || modifiers == ["number"]);
    if kind != Kind::Text && !select_ok && !modifiers.is_empty() {
        return Err("modifiers apply only to text and select controls".into());
    }
    if kind == Kind::Text && static_attr(rows, target.id, "value").is_some() {
        return Err("text models must not also carry a static value".into());
    }
    let multiple = static_attr(rows, target.id, "multiple").is_some_and(|row| row.kind == "absent");
    let options = match kind {
        Kind::Select => option_values(program, rows, target)?,
        _ => Vec::new(),
    };
    Ok(Control {
        kind,
        path: read.text.clone(),
        lazy: flag("lazy"),
        trim: flag("trim"),
        number: flag("number"),
        multiple,
        dom_value: dom_value.into(),
        options,
    })
}

fn js_whitespace(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\r')
}

/// Characters JavaScript treats as whitespace but this reference does not model.
fn exotic_space(c: char) -> bool {
    let n = u32::from(c);
    matches!(n, 0x0B | 0x0C | 0xA0 | 0x1680 | 0x2000..=0x200A)
        || [0x2028, 0x2029, 0x202F, 0x205F, 0x3000, 0xFEFF].contains(&n)
}

pub fn trim_js(text: &str) -> Result<String, String> {
    if text.chars().any(exotic_space) {
        return Err("unsupported whitespace in model value".into());
    }
    Ok(text.trim_matches(js_whitespace).into())
}

/// `looseToNumber`: `parseFloat`, keeping the string when it is `NaN`.
pub fn loose_to_number(text: &str) -> Result<Json, String> {
    let trimmed: Vec<char> = trim_js(text)?.chars().collect();
    let (negative, rest) = match trimmed.split_first() {
        Some(('-', rest)) => (true, rest),
        Some(('+', rest)) => (false, rest),
        _ => (false, trimmed.as_slice()),
    };
    if rest.starts_with(&"Infinity".chars().collect::<Vec<_>>()) {
        return Err("unsupported infinite model number".into());
    }
    let digits = rest.iter().take_while(|c| c.is_ascii_digit()).count();
    let (int_part, after) = rest.split_at(digits);
    let (fraction, after): (&[char], &[char]) = match after.split_first() {
        Some(('.', tail)) => tail.split_at(tail.iter().take_while(|c| c.is_ascii_digit()).count()),
        _ => (&[], after),
    };
    if int_part.is_empty() && fraction.is_empty() {
        return Ok(Json::String(text.into()));
    }
    if fraction.iter().any(|c| *c != '0') {
        return Err("unsupported fractional model number".into());
    }
    if let Some((c, tail)) = after.split_first()
        && matches!(c, 'e' | 'E')
        && tail
            .iter()
            .find(|d| !matches!(d, '+' | '-'))
            .is_some_and(char::is_ascii_digit)
    {
        return Err("unsupported exponent in model number".into());
    }
    let magnitude: String = int_part.iter().collect();
    let magnitude: i128 = match magnitude.is_empty() {
        true => 0,
        false => (magnitude.parse())
            .map_err(|_| String::from("integer result outside the safe range"))?,
    };
    number(if negative { -magnitude } else { magnitude })
}

/// `looseEqual` over primitives; structured values fail closed.
pub fn loose_equal(a: &Json, b: &Json) -> Result<bool, String> {
    Ok(primitive_text(a)? == primitive_text(b)?)
}

/// The DOM spelling of a text model value (`value == null ? '' : value`).
fn text_of(model: &Json) -> Result<String, String> {
    match model {
        Json::Null => Ok(String::new()),
        _ => primitive_text(model),
    }
}

fn checked_for(control: &Control, model: &Json) -> Result<bool, String> {
    let dom = Json::String(control.dom_value.clone());
    match (control.kind, model) {
        (Kind::Checkbox, Json::Array(items)) => {
            for item in items {
                if loose_equal(item, &dom)? {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        (Kind::Checkbox, Json::Bool(b)) => Ok(*b),
        (Kind::Checkbox, _) => Err("checkbox models must be booleans or arrays".into()),
        (Kind::Radio, _) => loose_equal(model, &dom),
        _ => Ok(false),
    }
}

fn selection(control: &Control, model: &Json) -> Result<Vec<bool>, String> {
    if control.multiple {
        let Json::Array(items) = model else {
            return Err("multiple selects require array models".into());
        };
        let texts = items
            .iter()
            .map(primitive_text)
            .collect::<Result<Vec<_>, _>>()?;
        return Ok(control.options.iter().map(|o| texts.contains(o)).collect());
    }
    let mut found = None;
    for (index, option) in control.options.iter().enumerate() {
        if found.is_none() && loose_equal(&Json::String(option.clone()), model)? {
            found = Some(index);
        }
    }
    Ok((0..control.options.len())
        .map(|i| found == Some(i))
        .collect())
}

/// Mounted hook: initialize DOM state from the model.
pub fn mount(control: &Control, model: &Json) -> Result<Live, String> {
    let live = Live {
        last_model: model.clone(),
        ..Live::default()
    };
    Ok(match control.kind {
        Kind::Text => Live {
            value: text_of(model)?,
            ..live
        },
        Kind::Checkbox | Kind::Radio => Live {
            checked: checked_for(control, model)?,
            ..live
        },
        Kind::Select => Live {
            selected: selection(control, model)?,
            ..live
        },
    })
}

/// Update hooks after a re-render; `assigning` suppresses select resync.
pub fn update(
    control: &Control,
    live: &Live,
    model: &Json,
    assigning: bool,
) -> Result<Live, String> {
    let previous = &live.last_model;
    let mut next = Live {
        last_model: model.clone(),
        ..live.clone()
    };
    match control.kind {
        Kind::Text => {
            if next.composing {
                return Ok(next);
            }
            let mut chars = next.value.chars();
            let leading_zero =
                chars.next() == Some('0') && chars.next().is_some_and(|c| c.is_ascii_digit());
            let current = match control.number && !leading_zero {
                true => loose_to_number(&next.value)?,
                false => Json::String(next.value.clone()),
            };
            let desired = match model {
                Json::Null => Json::String(String::new()),
                _ => model.clone(),
            };
            if strict_equal(&current, &desired)? {
                return Ok(next);
            }
            next.value = text_of(model)?;
        }
        Kind::Checkbox => next.checked = checked_for(control, model)?,
        Kind::Radio => {
            if strict_equal(previous, model)? {
                return Ok(next);
            }
            next.checked = checked_for(control, model)?;
        }
        Kind::Select => {
            if !assigning {
                next.selected = selection(control, model)?;
            }
        }
    }
    Ok(next)
}

pub fn display_value(control: &Control, live: &Live) -> String {
    match control.kind {
        Kind::Text => live.value.clone(),
        Kind::Checkbox | Kind::Radio => control.dom_value.clone(),
        Kind::Select => (control.options.iter().zip(&live.selected))
            .find(|(_, selected)| **selected)
            .map(|(option, _)| option.clone())
            .unwrap_or_default(),
    }
}
