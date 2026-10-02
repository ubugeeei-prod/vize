use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use std::io::{self, Write};

fn main() {
    let mut inputs = vec![
        "f<>/((\nd=//#__PURE__0".to_owned(),
        "/*#__PURE__*/:".to_owned(),
        "/* @__PURE__ */:".to_owned(),
        "/*#__PURE__*/:;".to_owned(),
        "export const X = /* @__PURE__ */ foo(/* comment */);".to_owned(),
        "/*#__PURE__*/ foo + /*#__PURE__*/ bar()".to_owned(),
        "/*#__PURE__*/ foo();".to_owned(),
        "/*#__PURE__*/ new Foo();".to_owned(),
        "// comment\nexport const answer: number = 42;".to_owned(),
    ];
    for prefix in ["f<>/((\nd=", "f<a>/((\nd=", "f<>/`\nd=", "(", "a:", ""] {
        for annotation in ["#__PURE__", "@__PURE__", "#__NO_SIDE_EFFECTS__", "plain"] {
            for tail in ["0", ":", ":;", "\n", "\nfoo();", "\n/* other */\nfoo();"] {
                inputs.push(format!("{prefix}//{annotation}{tail}"));
                inputs.push(format!("{prefix}/*{annotation}*/{tail}"));
            }
        }
    }
    let full = inputs.clone();
    for input in full {
        for cut in 0..input.len() {
            if input.is_char_boundary(cut) {
                inputs.push(input[..cut].to_owned());
            }
        }
    }
    inputs.sort();
    inputs.dedup();
    let profiles = [
        SourceType::mjs(),
        SourceType::cjs(),
        SourceType::jsx(),
        SourceType::jsx().with_script(true),
        SourceType::ts(),
        SourceType::ts().with_script(true),
        SourceType::tsx(),
        SourceType::tsx().with_script(true),
    ];
    let one = std::env::args().nth(1).map(|s| s.parse::<usize>().unwrap());
    let mut count = 0;
    for (case, input) in inputs.iter().enumerate() {
        for (profile, source_type) in profiles.into_iter().enumerate() {
            if one.is_some_and(|one| one != count) {
                count += 1;
                continue;
            }
            println!("CASE {count} input {case} profile {profile}: {input:?}");
            io::stdout().flush().unwrap();
            let allocator = Allocator::default();
            let parsed = Parser::new(&allocator, input, source_type).parse();
            println!("AST {:?}", parsed.program);
            println!(
                "DIAGNOSTICS {:?} PANICKED {} FLOW {}",
                parsed.diagnostics, parsed.panicked, parsed.is_flow_language
            );
            drop(parsed);
            count += 1;
        }
    }
    eprintln!("Observed {count} admitted profile/case combinations");
}
