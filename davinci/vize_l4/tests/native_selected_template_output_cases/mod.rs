//! Original admission and an independent Source Map v3 position decoder.

use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
};
use vize_l1::{
    SurfaceChild, SurfaceParseOptions,
    container::{Vue, vue::DescriptorOptions},
    markup::NativeTemplateComponent,
};
use vize_l2::lang::js::{NativeInterpolationInput, NativeTemplateFile, NativeTemplateOwner};
use vize_l4::targets::dom::NativeTemplateDomOutput;

pub(super) fn check(condition: bool) -> Result<(), &'static str> {
    if condition {
        Ok(())
    } else {
        Err("required condition")
    }
}

pub(super) fn equal<T: PartialEq>(actual: T, expected: T) -> Result<(), &'static str> {
    check(actual == expected)
}

pub(super) fn completed<'a>(
    arena: &'a Allocator,
    source: &'a str,
) -> Result<NativeTemplateFile<'a>, &'static str> {
    let observation = Vue.observe_descriptor(
        arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: SurfaceParseOptions::default(),
        },
    );
    let selected = NativeTemplateComponent::parse_in(
        arena,
        observation.admitted().map_err(|_| "original descriptor")?,
    )
    .map_err(|_| "original selected Component")?
    .ok_or("selected template")?;
    let mut owner = NativeTemplateOwner::new(selected).map_err(|_| "original File begin")?;
    {
        let mut walk = owner.begin().map_err(|_| "root begin")?;
        let selected = walk.selected();
        for child in selected.children() {
            if matches!(child.surface(), SurfaceChild::Interpolation(_)) {
                let input = NativeInterpolationInput::from_operand(
                    selected
                        .observe_interpolation_expression(child.reborrow())
                        .map_err(|_| "original stock expression")?,
                );
                walk.root_interpolation(child, input)
                    .map_err(|_| "whole input receiver")?;
            } else {
                walk.child(child).map_err(|_| "ordinary original child")?;
            }
        }
        walk.complete().map_err(|_| "normal root end")?;
    }
    Ok(core::hint::black_box(owner.finish()))
}

// Decode serialized segments without using the production map builder or its
// line table. Char iteration handles all JavaScript terminators and UTF-16.
pub(super) fn map_positions(
    output: &NativeTemplateDomOutput<'_, '_>,
    filename: &str,
) -> Result<(), &'static str> {
    let map: serde_json::Value =
        serde_json::from_str(&output.source_map(filename)).map_err(|_| "complete source map")?;
    equal(&map["version"], &serde_json::json!(3))?;
    equal(&map["file"], &serde_json::json!(filename))?;
    equal(&map["sources"], &serde_json::json!([filename]))?;
    equal(
        &map["sourcesContent"],
        &serde_json::json!([output.source_block().root_source()]),
    )?;
    let names = map["names"].as_array().ok_or("map names")?;
    let mappings = map["mappings"].as_str().ok_or("map segments")?;
    let mut expected = Vec::new();
    let mut links: Vec<_> = output.links().iter().filter(|link| link.segment).collect();
    links.sort_by_key(|link| link.generated.start);
    for link in links {
        let (generated_line, generated_column) = position(output.code(), link.generated.start)?;
        let (source_line, source_column) =
            position(output.source_block().root_source(), link.authored.start)?;
        let name = link.name.as_ref().map(|name| name.as_str().to_owned());
        expected.push((
            generated_line,
            generated_column,
            source_line,
            source_column,
            name,
        ));
    }
    let mut actual = Vec::new();
    let (mut source_index, mut source_line, mut source_column, mut name_index) = (0, 0, 0, 0);
    for (generated_line, line) in mappings.split(';').enumerate() {
        let mut generated_column = 0;
        for segment in line.split(',').filter(|segment| !segment.is_empty()) {
            let fields = vlq(segment)?;
            check(fields.len() == 4 || fields.len() == 5)?;
            generated_column += fields[0];
            source_index += fields[1];
            source_line += fields[2];
            source_column += fields[3];
            equal(source_index, 0)?;
            check(generated_column >= 0 && source_line >= 0 && source_column >= 0)?;
            let name = if fields.len() == 5 {
                name_index += fields[4];
                Some(
                    names
                        .get(usize::try_from(name_index).map_err(|_| "name index")?)
                        .and_then(serde_json::Value::as_str)
                        .ok_or("named segment")?
                        .to_owned(),
                )
            } else {
                None
            };
            actual.push((
                generated_line as u32,
                generated_column as u32,
                source_line as u32,
                source_column as u32,
                name,
            ));
        }
    }
    equal(actual, expected)
}

fn position(text: &str, byte: u32) -> Result<(u32, u32), &'static str> {
    let prefix = text.get(..byte as usize).ok_or("real UTF-8 offset")?;
    let (mut line, mut column) = (0, 0);
    let mut chars = prefix.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '\r' => {
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
                line += 1;
                column = 0;
            }
            '\n' | '\u{2028}' | '\u{2029}' => {
                line += 1;
                column = 0;
            }
            _ => column += ch.len_utf16() as u32,
        }
    }
    Ok((line, column))
}

fn vlq(segment: &str) -> Result<Vec<i64>, &'static str> {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut fields = Vec::new();
    let (mut value, mut shift) = (0_u64, 0);
    for byte in segment.bytes() {
        let digit = ALPHABET
            .iter()
            .position(|&candidate| candidate == byte)
            .ok_or("base64 VLQ digit")? as u64;
        check(shift < 60)?;
        value |= (digit & 31) << shift;
        if digit & 32 == 0 {
            let magnitude = (value >> 1) as i64;
            fields.push(if value & 1 == 0 {
                magnitude
            } else {
                -magnitude
            });
            value = 0;
            shift = 0;
        } else {
            shift += 5;
        }
    }
    equal(shift, 0)?;
    Ok(fields)
}
