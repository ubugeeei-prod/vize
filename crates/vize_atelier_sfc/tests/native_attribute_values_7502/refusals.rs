use super::{Test, check, inputs::FILENAME};
use serde_json::{Value, json};
use vize_atelier_sfc::NativeSelectedSfcDomOptions;
use vize_l0::{String, ToCompactString, cstr};

fn field<'a>(value: &'a Value, name: &str) -> Test<&'a Value> {
    value.get(name).ok_or_else(|| cstr!("missing {name}"))
}

fn array<'a>(value: &'a Value, name: &str) -> Test<&'a Vec<Value>> {
    field(value, name)?
        .as_array()
        .ok_or_else(|| cstr!("{name} is not an array"))
}

fn text<'a>(value: &'a Value, name: &str) -> Test<&'a str> {
    field(value, name)?
        .as_str()
        .ok_or_else(|| cstr!("{name} is not a string"))
}

pub fn validate(packet: &Value) -> Test {
    let rows = array(packet, "rows")?;
    check(
        rows.len() == 84,
        "all fourteen original inputs, three targets and two modes",
    )?;
    check(
        field(packet, "summary")?
            == &json!({
                "fixtures":14,"outcomes":84,"positive":72,"lowerRefusals":12
            }),
        "complete positive/refusal outcome counts",
    )?;
    for pair in rows.chunks_exact(2) {
        let recorded = pair.first().ok_or("Recorded row")?;
        let plain = pair.get(1).ok_or("NoLinks row")?;
        for key in ["id", "target", "nativeSource", "observation", "l3"] {
            check(
                field(recorded, key)? == field(plain, key)?,
                "Recorded/NoLinks original custody equality",
            )?;
        }
        check(
            field(recorded, "sourceMap")? == &json!(true)
                && field(plain, "sourceMap")? == &json!(false)
                && text(recorded, "linkMode")? == "Recorded"
                && text(plain, "linkMode")? == "NoLinks",
            "actual link modes",
        )?;
        check(
            field(field(recorded, "result")?, "code")? == field(field(plain, "result")?, "code")?,
            "Recorded/NoLinks complete code equality",
        )?;
    }
    for row in rows {
        let source = text(row, "nativeSource")?;
        let observation = field(row, "observation")?;
        check(
            field(observation, "descriptorSameSource")? == &json!(true),
            "whole original descriptor custody",
        )?;
        check(
            text(observation, "descriptorOptionsDebug")?
                == cstr!("{:?}", NativeSelectedSfcDomOptions::default().descriptor).as_str(),
            "unchanged native descriptor defaults",
        )?;
        check(
            array(observation, "descriptorIssues")?.is_empty()
                && array(observation, "descriptorErrors")?.is_empty(),
            "clean original descriptor",
        )?;
        let selected = field(observation, "selected")?;
        check(
            selected.is_object()
                && field(selected, "sameRootSource")? == &json!(true)
                && text(selected, "blockSource")? == text(row, "source")?,
            "actual original selected template",
        )?;
        let file = field(observation, "file")?;
        check(
            file.is_object()
                && field(file, "sameSource")? == &json!(true)
                && text(file, "source")? == source,
            "actual original File source custody",
        )?;
        let result = field(row, "result")?;
        let l3 = field(row, "l3")?;
        if text(row, "disposition")? == "positive" {
            positive(row, observation, file, result, l3, source)?;
        } else {
            lower_refusal(observation, file, result, l3)?;
        }
    }
    Ok(())
}

fn positive(
    row: &Value,
    observation: &Value,
    file: &Value,
    result: &Value,
    l3: &Value,
    source: &str,
) -> Test {
    check(
        field(observation, "admitted")? == &json!(true)
            && array(observation, "sourceIssues")?.is_empty()
            && field(file, "complete")? == &json!(true),
        "complete original lower admission",
    )?;
    check(
        array(file, "issues")?.is_empty()
            && array(file, "templateIssues")?.is_empty()
            && field(file, "templateInterruption")?.is_null()
            && array(file, "interruptedPrograms")?.is_empty(),
        "complete original File diagnostics",
    )?;
    check(
        text(l3, "state")? == "complete"
            && field(l3, "publicError")?.is_null()
            && field(l3, "ownerSame")? == &json!(true)
            && field(l3, "fileSame")? == &json!(true)
            && field(l3, "valueCount")? == &json!(1)
            && field(l3, "visitError")?.is_null(),
        "genuine original L3 value receipt",
    )?;
    let values = array(l3, "values")?;
    check(values.len() == 1, "one genuine title value")?;
    for value in values {
        check(
            field(value, "sameFile")? == &json!(true)
                && field(value, "sameObservation")? == &json!(true)
                && field(value, "sameDecodedValue")? == &json!(true),
            "same original File/value allocation",
        )?;
        let observed = field(value, "observation")?;
        check(
            field(observed, "authoredRootSame")? == &json!(true)
                && text(observed, "authoredRoot")? == source
                && field(observed, "valueSpan")? == field(observed, "sourceSpan")?,
            "whole original value source",
        )?;
        check(
            slice(source, field(observed, "valueSpan")?)? == text(observed, "raw")?,
            "original authored value bytes",
        )?;
        check(
            field(field(value, "attribute")?, "value")? == field(observed, "decoded")?,
            "actual canonical decoded value",
        )?;
    }
    check(
        text(result, "classification")? == "complete-original-sfc-module"
            && field(result, "publicError")?.is_null()
            && field(result, "mapError")?.is_null(),
        "complete public native module",
    )?;
    let code = text(result, "code")?;
    let links = array(result, "links")?;
    if field(row, "sourceMap")? == &json!(true) {
        let map_text = text(result, "mapText")?;
        let map: Value =
            serde_json::from_str(map_text).map_err(|error| error.to_compact_string())?;
        check(
            &map == field(result, "map")?
                && field(&map, "version")? == &json!(3)
                && field(&map, "sources")? == &json!([FILENAME])
                && field(&map, "sourcesContent")? == &json!([source]),
            "complete raw whole-source map",
        )?;
    } else {
        check(
            field(result, "mapText")?.is_null()
                && field(result, "map")?.is_null()
                && links.is_empty(),
            "genuine NoLinks output",
        )?;
    }
    for link in links {
        slice(source, field(link, "authored")?)?;
        slice(code, field(link, "generated")?)?;
    }
    Ok(())
}

fn lower_refusal(observation: &Value, file: &Value, result: &Value, l3: &Value) -> Test {
    let issues = array(observation, "sourceIssues")?;
    check(
        field(observation, "admitted")? == &json!(false)
            && field(file, "complete")? == &json!(false)
            && issues.len() == 1
            && issues
                .first()
                .and_then(|issue| issue.get("templateIssueKind"))
                == Some(&json!("UnsupportedChild")),
        "original class/entity-text typed lower refusal",
    )?;
    check(
        text(l3, "state")? == "lower-refusal" && array(l3, "values")?.is_empty(),
        "incomplete File grants no L3 receipt",
    )?;
    check(
        text(result, "classification")? == "lower-refusal"
            && field(result, "publicError")?.is_string()
            && field(result, "code")?.is_null()
            && field(result, "mapText")?.is_null()
            && field(result, "map")?.is_null()
            && array(result, "links")?.is_empty(),
        "typed refusal returns no partial output",
    )
}

fn slice<'a>(source: &'a str, range: &Value) -> Test<&'a str> {
    let start = field(range, "start")?.as_u64().ok_or("range start")?;
    let end = field(range, "end")?.as_u64().ok_or("range end")?;
    source
        .get(
            usize::try_from(start).map_err(|error| error.to_compact_string())?
                ..usize::try_from(end).map_err(|error| error.to_compact_string())?,
        )
        .ok_or_else(|| String::from("original UTF-8 byte range"))
}
