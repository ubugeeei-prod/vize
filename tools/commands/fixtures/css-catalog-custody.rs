#!/usr/bin/env rust-script
//! ```cargo
//! [package]
//! edition = "2024"
//! ```

//! Offline semantic export of the actual compiled static CSS catalog.
#[path = "../../../crates/vize_maestro/src/ide/style"]
mod fixture {
    mod data;

    use std::io::{self, Write};

    fn count(out: &mut impl Write, value: usize) -> io::Result<()> {
        out.write_all(&(value as u64).to_le_bytes())
    }

    fn string(out: &mut impl Write, value: &str) -> io::Result<()> {
        count(out, value.len())?;
        out.write_all(value.as_bytes())
    }

    fn optional(out: &mut impl Write, value: Option<&str>) -> io::Result<()> {
        out.write_all(&[u8::from(value.is_some())])?;
        if let Some(value) = value {
            string(out, value)?;
        }
        Ok(())
    }

    fn strings(out: &mut impl Write, values: &[&str]) -> io::Result<()> {
        count(out, values.len())?;
        for value in values {
            string(out, value)?;
        }
        Ok(())
    }

    fn baseline(out: &mut impl Write, value: Option<data::CssBaseline>) -> io::Result<()> {
        out.write_all(&[u8::from(value.is_some())])?;
        if let Some(value) = value {
            string(out, value.status)?;
            optional(out, value.low_date)?;
            optional(out, value.high_date)?;
        }
        Ok(())
    }

    fn value(out: &mut impl Write, value: &data::CssValue) -> io::Result<()> {
        string(out, value.name)?;
        optional(out, value.description)?;
        strings(out, value.browsers)?;
        baseline(out, value.baseline)
    }

    fn entry(out: &mut impl Write, entry: &data::CssEntry) -> io::Result<()> {
        string(out, entry.name)?;
        optional(out, entry.description)?;
        optional(out, entry.syntax)?;
        count(out, entry.references.len())?;
        for reference in entry.references {
            string(out, reference.name)?;
            string(out, reference.url)?;
        }
        count(out, entry.values.len())?;
        for item in entry.values {
            value(out, item)?;
        }
        optional(out, entry.at_rule)?;
        optional(out, entry.status)?;
        strings(out, entry.restrictions)?;
        strings(out, entry.browsers)?;
        baseline(out, entry.baseline)?;
        out.write_all(&[u8::from(entry.relevance.is_some())])?;
        if let Some(relevance) = entry.relevance {
            out.write_all(&relevance.to_le_bytes())?;
        }
        count(out, entry.descriptors.len())?;
        for descriptor in entry.descriptors {
            self::entry(out, descriptor)?;
        }
        optional(out, entry.entry_type)
    }

    type Lookup = fn(&str) -> Option<&'static data::CssEntry>;

    fn family(
        out: &mut impl Write,
        name: &str,
        entries: &[&'static data::CssEntry],
        lookup: Lookup,
    ) -> io::Result<()> {
        string(out, name)?;
        count(out, entries.len())?;
        assert!(entries.windows(2).all(|rows| rows[0].name < rows[1].name));
        for row in entries {
            assert!(std::ptr::eq(lookup(row.name).unwrap(), *row));
            assert!(std::ptr::eq(
                lookup(&row.name.to_ascii_uppercase()).unwrap(),
                *row
            ));
            entry(out, row)?;
        }
        assert!(lookup("not-an-authored-css-name").is_none());
        Ok(())
    }

    pub(super) fn run() -> io::Result<()> {
        let mut out = io::BufWriter::new(io::stdout().lock());
        out.write_all(b"VIZECSS1")?;
        family(&mut out, "properties", data::properties(), data::property)?;
        family(&mut out, "atDirectives", data::at_rules(), data::at_rule)?;
        family(
            &mut out,
            "pseudoClasses",
            data::pseudo_classes(),
            data::pseudo_class,
        )?;
        family(
            &mut out,
            "pseudoElements",
            data::pseudo_elements(),
            data::pseudo_element,
        )?;
        string(&mut out, "colors")?;
        count(&mut out, data::colors().len())?;
        assert!(
            data::colors().windows(2).all(|rows| {
                rows[0].name.to_ascii_lowercase() < rows[1].name.to_ascii_lowercase()
            })
        );
        for row in data::colors() {
            assert!(std::ptr::eq(data::color(row.name).unwrap(), row));
            assert!(std::ptr::eq(
                data::color(&row.name.to_ascii_uppercase()).unwrap(),
                row
            ));
            value(&mut out, row)?;
        }
        assert!(data::color("not-an-authored-color").is_none());
        out.flush()
    }
}

fn main() -> std::io::Result<()> {
    fixture::run()
}
