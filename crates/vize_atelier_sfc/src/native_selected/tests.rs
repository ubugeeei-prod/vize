use super::{NativeSelectedSfcDomOptions, compile_native_selected_sfc_dom};
use serde_json::Value;
use vize_l0::{Allocator, String, cstr};
use vize_l2::op::{BindingOp, OnOp, Op, Region};

mod local;
mod refusal;

macro_rules! require {
    ($condition:expr, $($message:tt)+) => {
        if !$condition { return Err(cstr!($($message)+)); }
    };
}
fn text<'a>(value: &'a Value, key: &str) -> Result<&'a str, String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| cstr!("missing {key}"))
}
fn on<'o, 'a>(region: &'o Region<'a>) -> Option<&'o OnOp<'a>> {
    for op in &region.ops {
        if let Op::Element(element) = op {
            for binding in &element.bindings {
                if let BindingOp::On(on) = binding {
                    return Some(on);
                }
            }
            if let Some(on) = on(&element.children) {
                return Some(on);
            }
        }
    }
    None
}

#[test]
fn five_whole_original_click_components_and_maps_are_captured() -> Result<(), String> {
    let pack: Value = serde_json::from_str(include_str!(
        "../../tests/fixtures/native_selected_sfc_click_vue_3_5_35.json"
    ))
    .map_err(|error| cstr!("{error}"))?;
    let fixtures = pack
        .get("fixtures")
        .and_then(Value::as_array)
        .ok_or("fixtures")?;
    require!(fixtures.len() == 5, "five whole original sources");
    let mut rows = Vec::new();
    for fixture in fixtures {
        let id = text(fixture, "id")?;
        let source = text(fixture, "source")?;
        let arena = Allocator::default();
        let plain =
            compile_native_selected_sfc_dom(&arena, source, NativeSelectedSfcDomOptions::default());
        let mapped = compile_native_selected_sfc_dom(
            &arena,
            source,
            NativeSelectedSfcDomOptions {
                filename: "SelectedClick雪🌸.vue",
                source_map: true,
                ..NativeSelectedSfcDomOptions::default()
            },
        );
        let output = mapped.result().map_err(|error| cstr!("{id}: {error:?}"))?;
        let unlinked = plain
            .result()
            .map_err(|error| cstr!("{id}: plain {error:?}"))?;
        require!(
            unlinked.code() == output.code(),
            "{id}: NoLinks/Recorded whole module"
        );
        require!(
            unlinked.document().links().is_empty(),
            "{id}: NoLinks has no links"
        );
        require!(unlinked.source_map().is_none(), "{id}: optional map");
        require!(
            output.code() == text(fixture, "expectedCode")?,
            "{id}: complete independent reference assembly"
        );
        let map: Value = serde_json::from_str(output.source_map().ok_or("map")?)
            .map_err(|error| cstr!("{error}"))?;
        require!(
            map["sourcesContent"] == serde_json::json!([source]),
            "{id}: whole original map source"
        );
        require!(
            map["names"] == serde_json::json!(["$event"]),
            "{id}: original EventParameter name"
        );
        require!(
            map == fixture["nativeMap"],
            "{id}: complete source-built frozen map"
        );
        let observation = mapped.observation();
        require!(
            observation.issues().is_empty(),
            "{id}: source-admitted whole SFC"
        );
        let admitted = observation.admitted().ok_or("whole admitted view")?;
        let original = admitted.into_template_view();
        let file = original.file().ok_or("whole original File")?;
        require!(
            core::ptr::eq(file.artifact().source(), observation.descriptor().source()),
            "{id}: same original source"
        );
        require!(
            file.is_complete() && file.units().is_empty(),
            "{id}: complete scriptless File"
        );
        let handler = file
            .handler_for(on(file.artifact().root()).ok_or("actual On")?)
            .ok_or("actual File On join")?;
        let resolution = handler.resolution().ok_or("original resolution")?;
        let syntax = resolution.input().operand().syntax();
        require!(
            core::ptr::eq(syntax.source().authored_root(), source),
            "{id}: original handler source"
        );
        require!(
            syntax.diagnostics().count() == 0,
            "{id}: complete syntax diagnostics"
        );
        let span = syntax.source().span();
        let links: Vec<_> = output
            .document()
            .links()
            .iter()
            .filter(|link| link.authored.start >= span.start && link.authored.end <= span.end)
            .collect();
        require!(
            links
                .first()
                .is_some_and(|link| link.authored.start == span.start),
            "{id}: body start"
        );
        require!(
            links
                .last()
                .is_some_and(|link| link.authored.end == span.end),
            "{id}: body end"
        );
        require!(
            links
                .windows(2)
                .all(|pair| pair[0].authored.end == pair[1].authored.start),
            "{id}: whole contiguous body links"
        );
        let decoded: String = links
            .iter()
            .map(|link| link.generated.slice(output.code()))
            .collect();
        require!(
            decoded == syntax.source().text(),
            "{id}: whole unchanged decoded body"
        );
        if let Some(decode_map) = syntax.source().decode_map() {
            for segment in decode_map
                .segments()
                .iter()
                .filter(|segment| segment.authored().slice(source).starts_with('&'))
            {
                require!(
                    links.iter().any(|link| link.authored == segment.authored()
                        && link.name.is_none()
                        && link.generated.slice(output.code())
                            == segment.decoded().slice(syntax.source().text())),
                    "{id}: original entity atom"
                );
            }
        }
        for reference in resolution.references() {
            let authored = resolution
                .authored_span(reference.span)
                .map_err(|error| cstr!("{error:?}"))?;
            require!(
                links.iter().any(|link| link.authored == authored
                    && link.name.as_deref() == Some("$event")
                    && link.generated.slice(output.code()) == "$event"),
                "{id}: named original reference"
            );
        }
        for link in output.document().links() {
            require!(
                source
                    .get(link.authored.start as usize..link.authored.end as usize)
                    .is_some(),
                "{id}: valid authored UTF-8"
            );
            require!(
                output
                    .code()
                    .get(link.generated.start as usize..link.generated.end as usize)
                    .is_some(),
                "{id}: valid generated UTF-8"
            );
        }
        rows.push(
            serde_json::json!({"id":id,"source":source,"code":output.code(),"nativeMap":map}),
        );
    }
    if let Some(path) = std::env::var_os("VIZE_NATIVE_SELECTED_SFC_DOM_CAPTURE") {
        let capture = serde_json::json!({"schema":"vize.native-sfc.selected-click-capture","adapter":"vize_atelier_sfc::compile_native_selected_sfc_dom","fixtures":rows});
        std::fs::write(
            path,
            serde_json::to_vec_pretty(&capture).map_err(|error| cstr!("{error}"))?,
        )
        .map_err(|error| cstr!("{error}"))?;
    }
    Ok(())
}
