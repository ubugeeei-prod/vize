//! Event steps on a model control: the step's DOM writes, then the
//! directive listener that turns them into a model assignment.

use super::json::{Json, get, get_arr, get_bool, get_str};
use super::model::{Control, Kind, Live, loose_equal, loose_to_number, trim_js};

pub(super) fn transform(control: &Control, text: &str) -> Result<Json, String> {
    let text = if control.trim {
        trim_js(text)?
    } else {
        text.into()
    };
    match control.number {
        true => loose_to_number(&text),
        false => Ok(Json::String(text)),
    }
}

/// The `value`, `checked` and `selectedValues` fields of an event step.
pub(super) fn apply_fields(control: &Control, live: &mut Live, step: &Json) -> Result<(), String> {
    if let Ok(value) = get(step, "value") {
        let value = get_str(value)?;
        match control.kind {
            Kind::Text => live.value = value.into(),
            Kind::Select => {
                if control.multiple {
                    return Err("multiple selects take selectedValues".into());
                }
                let first = control.options.iter().position(|o| o == value);
                live.selected = (0..control.options.len())
                    .map(|i| first == Some(i))
                    .collect();
            }
            _ => return Err("value steps apply to text inputs and single selects".into()),
        }
    }
    if let Ok(checked) = get(step, "checked") {
        if !matches!(control.kind, Kind::Checkbox | Kind::Radio) {
            return Err("checked applies to toggles".into());
        }
        live.checked = get_bool(checked)?;
    }
    if let Ok(values) = get(step, "selectedValues") {
        if !(control.kind == Kind::Select && control.multiple) {
            return Err("selectedValues needs a multiple select".into());
        }
        let values = get_arr(values)?
            .iter()
            .map(get_str)
            .collect::<Result<Vec<_>, _>>()?;
        live.selected = control
            .options
            .iter()
            .map(|o| values.contains(&o.as_str()))
            .collect();
    }
    Ok(())
}

/// The directive's listener for `name`: the model value it assigns, if any.
pub(super) fn dispatch(
    control: &Control,
    live: &mut Live,
    name: &str,
) -> Result<Option<Json>, String> {
    let mut assigned = None;
    match (control.kind, name) {
        (Kind::Text, "input") => {
            if !control.lazy && !live.composing {
                assigned = Some(transform(control, &live.value)?);
            }
        }
        (Kind::Text, "change") => {
            if control.lazy {
                assigned = Some(transform(control, &live.value)?);
            }
            if control.trim {
                live.value = trim_js(&live.value)?;
            }
            if !control.lazy && live.composing {
                live.composing = false;
                assigned = Some(transform(control, &live.value)?);
            }
        }
        (Kind::Text, "compositionstart") => {
            if !control.lazy {
                live.composing = true;
            }
        }
        (Kind::Text, "compositionend") => {
            if !control.lazy && live.composing {
                live.composing = false;
                assigned = Some(transform(control, &live.value)?);
            }
        }
        (Kind::Checkbox, "change") => match &live.last_model {
            Json::Array(items) => {
                // `looseIndexOf` then `splice(index, 1)`: only the first loose match leaves.
                let target = Json::String(control.dom_value.clone());
                let mut index = None;
                for (i, item) in items.iter().enumerate() {
                    let equal = loose_equal(item, &target)?;
                    if index.is_none() && equal {
                        index = Some(i);
                    }
                }
                match (live.checked, index) {
                    (true, None) => {
                        let mut items = items.clone();
                        items.push(target);
                        assigned = Some(Json::Array(items));
                    }
                    (false, Some(i)) => {
                        let mut items = items.clone();
                        items.remove(i);
                        assigned = Some(Json::Array(items));
                    }
                    _ => {}
                }
            }
            _ => assigned = Some(Json::Bool(live.checked)),
        },
        (Kind::Radio, "change") => assigned = Some(Json::String(control.dom_value.clone())),
        (Kind::Select, "change") => {
            let chosen = (control.options.iter().zip(&live.selected))
                .filter(|(_, selected)| **selected)
                .map(|(value, _)| match control.number {
                    true => loose_to_number(value),
                    false => Ok(Json::String(value.clone())),
                })
                .collect::<Result<Vec<_>, _>>()?;
            assigned = Some(match (control.multiple, chosen.as_slice()) {
                (true, _) => Json::Array(chosen),
                (false, [value]) => value.clone(),
                _ => return Err("single select change without one selection".into()),
            });
        }
        _ => return Err(format!("unsupported {name} event on a model control")),
    }
    Ok(assigned)
}
