use super::inspect;
use vize_l1::embed::Lang;
use vize_l1::embed::syntax::{ProgramGoal, ProgramOptions};

#[test]
fn explicit_languages_inspect_real_statement_kinds_and_preserve_every_byte() {
    for (lang, jsx, input, kind) in [
        (
            Lang::Js,
            false,
            "/* α */ import { ref } from 'vue';\r\nexport const x = ref(0);",
            "ImportDeclaration",
        ),
        (
            Lang::Ts,
            false,
            "/* β */ interface Props { value: number }\nexport const x: Props = { value: 1 };",
            "TSInterfaceDeclaration",
        ),
        (
            Lang::Js,
            true,
            "const node = <section>{values.map(value => <span>{value}</span>)}</section>;",
            "VariableDeclaration",
        ),
        (
            Lang::Ts,
            true,
            "type Props = { value: number }; export const View = ({ value }: Props) => <span>{value}</span>;",
            "TSTypeAliasDeclaration",
        ),
    ] {
        let observed = inspect(
            input,
            ProgramOptions {
                lang,
                jsx,
                goal: ProgramGoal::Module,
            },
        )
        .unwrap();
        assert_eq!(observed.printed.as_str(), input);
        assert!(observed.facts.contains("hole=None"));
        assert!(observed.facts.contains(kind));
        assert!(!observed.facts.contains("diagnostic "));
    }
}

#[test]
fn language_and_syntax_failures_remain_visible_without_losing_authored_bytes() {
    for (lang, goal, input) in [
        (
            Lang::Js,
            ProgramGoal::Module,
            "interface Props { value: number }",
        ),
        (
            Lang::Ts,
            ProgramGoal::Module,
            "export const node = <span>{value}</span>;",
        ),
        (Lang::Ts, ProgramGoal::Module, "f<>/((\nd=//#__PURE__0"),
    ] {
        let observed = inspect(
            input,
            ProgramOptions {
                lang,
                jsx: false,
                goal,
            },
        )
        .unwrap();
        assert_eq!(observed.printed.as_str(), input);
        assert!(
            observed.facts.contains("hole=Some(Syntax)"),
            "input={input:?} facts={:?}",
            observed.facts
        );
        assert!(observed.facts.contains("diagnostic Error:"));
        assert!(observed.facts.contains("label @"));
        assert!(!observed.facts.contains("statement "));
    }
}

#[test]
fn malformed_unicode_comments_and_every_utf8_prefix_are_observed_without_rewrite() {
    let input = "/* α */ const value: number = ;\r\n// β";
    let options = ProgramOptions {
        lang: Lang::Ts,
        jsx: true,
        goal: ProgramGoal::Script,
    };
    for cut in (0..=input.len()).filter(|&cut| input.is_char_boundary(cut)) {
        let authored = input.get(..cut).unwrap();
        let observed = inspect(authored, options).unwrap();
        assert_eq!(observed.printed.as_str(), authored);
        assert!(observed.facts.starts_with("script: tsx script; hole="));
    }
    let observed = inspect(input, options).unwrap();
    assert!(observed.facts.contains("/* α */"));
    assert!(observed.facts.contains("diagnostic Error:"));
    let valid = inspect("/* α */ const value: number = 1;\r\n// β", options).unwrap();
    assert!(valid.facts.contains("/* α */"));
    assert!(valid.facts.contains("// β"));
    assert!(valid.facts.contains("script: tsx script; hole=None"));
}
