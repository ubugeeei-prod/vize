//! Layout NAPI bindings.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::cell::RefCell;

use crate::layout::{FlexStyle, LayoutEngine};

use super::types::{FlexStyleNapi, LayoutResultNapi};

thread_local! {
    // Taffy's tree is not Send, so keep the native layout state scoped to the
    // thread running the NAPI calls instead of placing it behind a shared static.
    static LAYOUT: RefCell<Option<LayoutEngine>> = const { RefCell::new(None) };
}

fn with_layout_mut<T>(f: impl FnOnce(&mut LayoutEngine) -> Result<T>) -> Result<T> {
    LAYOUT.with(|layout| {
        let mut guard = layout.try_borrow_mut().map_err(|e| {
            Error::new(
                Status::GenericFailure,
                format!("Layout borrow error: {}", e),
            )
        })?;
        let engine = guard
            .as_mut()
            .ok_or_else(|| Error::new(Status::GenericFailure, "Layout not initialized"))?;

        f(engine)
    })
}

fn with_layout<T>(f: impl FnOnce(&LayoutEngine) -> Result<T>) -> Result<T> {
    LAYOUT.with(|layout| {
        let guard = layout.try_borrow().map_err(|e| {
            Error::new(
                Status::GenericFailure,
                format!("Layout borrow error: {}", e),
            )
        })?;
        let engine = guard
            .as_ref()
            .ok_or_else(|| Error::new(Status::GenericFailure, "Layout not initialized"))?;

        f(engine)
    })
}

/// Initialize layout engine.
#[napi(js_name = "initLayout")]
#[allow(clippy::disallowed_macros)]
pub fn init_layout() -> Result<()> {
    LAYOUT.with(|layout| {
        *layout.try_borrow_mut().map_err(|e| {
            Error::new(
                Status::GenericFailure,
                format!("Layout borrow error: {}", e),
            )
        })? = Some(LayoutEngine::new());
        Ok(())
    })
}

/// Create a new layout node.
#[napi(js_name = "createLayoutNode")]
#[allow(clippy::disallowed_macros)]
pub fn create_layout_node(style: Option<FlexStyleNapi>) -> Result<i64> {
    with_layout_mut(|engine| {
        let flex_style = style.map(convert_flex_style).unwrap_or_default();
        let id = engine.new_node(&flex_style);
        Ok(id as i64)
    })
}

/// Create a new leaf layout node with measured size.
#[napi(js_name = "createLayoutLeaf")]
#[allow(clippy::disallowed_macros)]
pub fn create_layout_leaf(width: f64, height: f64, style: Option<FlexStyleNapi>) -> Result<i64> {
    with_layout_mut(|engine| {
        let flex_style = style.map(convert_flex_style).unwrap_or_default();
        let id = engine.new_leaf(&flex_style, width as f32, height as f32);
        Ok(id as i64)
    })
}

/// Set layout root node.
#[napi(js_name = "setLayoutRoot")]
#[allow(clippy::disallowed_macros)]
pub fn set_layout_root(id: i64) -> Result<()> {
    with_layout_mut(|engine| {
        engine.set_root(id as u64);
        Ok(())
    })
}

/// Add child to parent node.
#[napi(js_name = "addLayoutChild")]
#[allow(clippy::disallowed_macros)]
pub fn add_layout_child(parent: i64, child: i64) -> Result<()> {
    with_layout_mut(|engine| {
        engine.add_child(parent as u64, child as u64);
        Ok(())
    })
}

/// Remove child from parent node.
#[napi(js_name = "removeLayoutChild")]
#[allow(clippy::disallowed_macros)]
pub fn remove_layout_child(parent: i64, child: i64) -> Result<()> {
    with_layout_mut(|engine| {
        engine.remove_child(parent as u64, child as u64);
        Ok(())
    })
}

/// Update node style.
#[napi(js_name = "setLayoutStyle")]
#[allow(clippy::disallowed_macros)]
pub fn set_layout_style(id: i64, style: FlexStyleNapi) -> Result<()> {
    with_layout_mut(|engine| {
        let flex_style = convert_flex_style(style);
        engine.set_style(id as u64, &flex_style);
        Ok(())
    })
}

/// Remove a node.
#[napi(js_name = "removeLayoutNode")]
#[allow(clippy::disallowed_macros)]
pub fn remove_layout_node(id: i64) -> Result<()> {
    with_layout_mut(|engine| {
        engine.remove(id as u64);
        Ok(())
    })
}

/// Compute layout.
#[napi(js_name = "computeLayout")]
#[allow(clippy::disallowed_macros)]
pub fn compute_layout(width: i32, height: i32) -> Result<()> {
    with_layout_mut(|engine| {
        engine.compute(width as f32, height as f32);
        Ok(())
    })
}

/// Get layout result for a node.
#[napi(js_name = "getLayout")]
#[allow(clippy::disallowed_macros)]
pub fn get_layout(id: i64) -> Result<Option<LayoutResultNapi>> {
    with_layout(|engine| {
        Ok(engine.layout(id as u64).map(|rect| LayoutResultNapi {
            id,
            x: rect.x as i32,
            y: rect.y as i32,
            width: rect.width as i32,
            height: rect.height as i32,
        }))
    })
}

/// Get all layout results.
#[napi(js_name = "getAllLayouts")]
#[allow(clippy::disallowed_macros)]
pub fn get_all_layouts() -> Result<Vec<LayoutResultNapi>> {
    with_layout(|engine| {
        let results: Vec<_> = engine
            .layouts()
            .iter()
            .map(|(&id, &rect)| LayoutResultNapi {
                id: id as i64,
                x: rect.x as i32,
                y: rect.y as i32,
                width: rect.width as i32,
                height: rect.height as i32,
            })
            .collect();

        Ok(results)
    })
}

/// Clear layout engine.
#[napi(js_name = "clearLayout")]
#[allow(clippy::disallowed_macros)]
pub fn clear_layout() -> Result<()> {
    LAYOUT.with(|layout| {
        if let Some(ref mut engine) = *layout.try_borrow_mut().map_err(|e| {
            Error::new(
                Status::GenericFailure,
                format!("Layout borrow error: {}", e),
            )
        })? {
            engine.clear();
        }

        Ok(())
    })
}

/// Convert FlexStyleNapi to FlexStyle.
fn convert_flex_style(style: FlexStyleNapi) -> FlexStyle {
    use crate::layout::*;

    let mut result = FlexStyle::default();

    if let Some(dir) = style.flex_direction {
        result.flex_direction = match dir.as_str() {
            "column" => FlexDirection::Column,
            "row-reverse" => FlexDirection::RowReverse,
            "column-reverse" => FlexDirection::ColumnReverse,
            _ => FlexDirection::Row,
        };
    }

    if let Some(wrap) = style.flex_wrap {
        result.flex_wrap = match wrap.as_str() {
            "wrap" => FlexWrap::Wrap,
            "wrap-reverse" => FlexWrap::WrapReverse,
            _ => FlexWrap::NoWrap,
        };
    }

    if let Some(jc) = style.justify_content {
        result.justify_content = match jc.as_str() {
            "flex-end" | "end" => JustifyContent::FlexEnd,
            "center" => JustifyContent::Center,
            "space-between" => JustifyContent::SpaceBetween,
            "space-around" => JustifyContent::SpaceAround,
            "space-evenly" => JustifyContent::SpaceEvenly,
            _ => JustifyContent::FlexStart,
        };
    }

    if let Some(ai) = style.align_items {
        result.align_items = match ai.as_str() {
            "flex-start" | "start" => AlignItems::FlexStart,
            "flex-end" | "end" => AlignItems::FlexEnd,
            "center" => AlignItems::Center,
            "baseline" => AlignItems::Baseline,
            _ => AlignItems::Stretch,
        };
    }

    if let Some(grow) = style.flex_grow {
        result.flex_grow = grow as f32;
    }

    if let Some(shrink) = style.flex_shrink {
        result.flex_shrink = shrink as f32;
    }

    if let Some(width) = style.width {
        result.width = parse_dimension(&width);
    }

    if let Some(height) = style.height {
        result.height = parse_dimension(&height);
    }

    if let Some(p) = style.padding {
        result.padding = Edges::all(p as f32);
    }

    if let Some(m) = style.margin {
        result.margin = Edges::all(m as f32);
    }

    if let Some(g) = style.gap {
        result.gap = Gap::all(g as f32);
    }

    result
}

/// Parse dimension string.
fn parse_dimension(s: &str) -> crate::layout::Dimension {
    use crate::layout::Dimension;

    if s == "auto" {
        return Dimension::Auto;
    }

    if let Some(pct) = s.strip_suffix('%') {
        if let Ok(v) = pct.parse::<f32>() {
            return Dimension::Percent(v);
        }
    }

    if let Ok(v) = s.parse::<f32>() {
        return Dimension::Points(v);
    }

    Dimension::Auto
}
