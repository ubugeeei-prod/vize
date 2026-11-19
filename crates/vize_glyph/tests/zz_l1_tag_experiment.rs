//! Temporary experiment: compare scanner and L1 tag routes over a corpus.
#![allow(clippy::all, clippy::pedantic, clippy::restriction)]
use vize_glyph::{EndOfLine, FormatOptions, experiment_set_l1_tags, format_sfc, format_template};

fn run(src: &str, sfc: bool, opts: &FormatOptions) -> std::string::String {
    if sfc {
        match format_sfc(src, opts) {
            Ok(r) => format!("OK{}", r.code),
            Err(e) => format!("ERR{e:?}"),
        }
    } else {
        match format_template(src, opts) {
            Ok(r) => format!("OK{r}"),
            Err(e) => format!("ERR{e:?}"),
        }
    }
}

#[test]
fn corpus() {
    let Ok(list) = std::env::var("GLYPH_CORPUS") else {
        return;
    };
    let list = std::fs::read_to_string(list).unwrap();
    let mut opt_sets = vec![FormatOptions::default()];
    let mut o = FormatOptions::default();
    o.sort_attributes = true;
    opt_sets.push(o);
    let mut o = FormatOptions::default();
    o.end_of_line = EndOfLine::Cr;
    opt_sets.push(o);
    let mut o = FormatOptions::default();
    o.canonical_directive_layout = true;
    opt_sets.push(o);
    let (mut n, mut bad) = (0, 0);
    for path in list.lines() {
        let Ok(src) = std::fs::read_to_string(path) else {
            continue;
        };
        let variants = [src.clone(), src.replace('\n', "\r")];
        for (vi, src) in variants.iter().enumerate() {
            for sfc in [true, false] {
                for (oi, opts) in opt_sets.iter().enumerate() {
                    n += 1;
                    experiment_set_l1_tags(false);
                    let a = std::panic::catch_unwind(|| run(src, sfc, opts))
                        .unwrap_or_else(|_| "PANIC".into());
                    experiment_set_l1_tags(true);
                    let b = std::panic::catch_unwind(|| run(src, sfc, opts))
                        .unwrap_or_else(|_| "PANIC".into());
                    if a != b {
                        bad += 1;
                        if bad <= 60 {
                            eprintln!("DIFF {path} cr={vi} sfc={sfc} opts={oi}");
                            let al: Vec<_> = a.split(['\n', '\r']).collect();
                            let bl: Vec<_> = b.split(['\n', '\r']).collect();
                            for i in 0..al.len().max(bl.len()) {
                                if al.get(i) != bl.get(i) {
                                    eprintln!("  base: {:?}\n  l1:   {:?}", al.get(i), bl.get(i));
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    eprintln!("runs={n} diffs={bad}");
}
