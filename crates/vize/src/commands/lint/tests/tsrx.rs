use super::super::tsrx::lint_tsrx;
use vize_patina::{LintResult, Linter};

fn rows(result: &LintResult) -> Vec<(&str, u32, u32)> {
    result
        .diagnostics
        .iter()
        .map(|diagnostic| (diagnostic.rule_name, diagnostic.start, diagnostic.end))
        .collect()
}

#[test]
fn regions_merge_in_source_order_with_one_report_per_rule_and_range() {
    // The `<li>` region is nested in the `<ul>` region's expression child.
    let source = "export function List({ items }: { items: string[] }) @{
  <ul  class=\"list\">
    {items.map((item) => <li  title={item}><img src={item} /></li>)}
  </ul>
}

export function Footer() @{
  <footer  class=\"footer\">done</footer>
}
";
    let result = lint_tsrx(&Linter::new(), source, "List.tsrx");
    assert_eq!(
        rows(&result),
        [
            ("vue/no-multi-spaces", 61, 63),
            ("vue/no-multi-spaces", 105, 107),
            ("a11y/alt-text", 120, 138),
            ("vue/no-multi-spaces", 194, 196),
        ]
    );
    assert_eq!((result.error_count, result.warning_count), (0, 4));
}

#[test]
fn conditional_arms_are_linted_as_branches_of_the_enclosing_region() {
    let source = "export function Avatar({ ok, src }: { ok: boolean; src: string }) @{
  <p>{ok ? <img  src={src} /> : null}</p>
}
";
    let result = lint_tsrx(&Linter::new(), source, "Avatar.tsrx");
    assert_eq!(
        rows(&result),
        [("a11y/alt-text", 80, 98), ("vue/no-multi-spaces", 84, 86)]
    );
}

#[test]
fn a_module_with_an_unsupported_construct_reports_every_one_and_lints_no_region() {
    let source = "export function A() @{
  <div  class=\"a\">
    @try {
      <p>a</p>
    } @catch (error) {
      <p>b</p>
    }
  </div>
}
";
    let result = lint_tsrx(&Linter::new(), source, "A.tsrx");
    assert_eq!(rows(&result), [("parser/tsrx", 46, 111)]);
    assert_eq!(
        result.diagnostics[0].message,
        "Try is not supported in TSRX modules yet, so this module's markup is not linted"
    );
    assert_eq!((result.error_count, result.warning_count), (1, 0));
}

#[test]
fn a_module_with_host_loops_reports_each_and_lints_no_region() {
    let source = "export function List({ items }: { items: string[] }) @{
  <ul  class=\"list\">
    @for (const item of items; key item) {
      <li  title={item}><img src={item} /></li>
    } @empty {
      <li>none</li>
    }
  </ul>
}
";
    let result = lint_tsrx(&Linter::new(), source, "List.tsrx");
    assert_eq!(rows(&result), [("parser/tsrx", 81, 208)]);
    assert_eq!(
        result.diagnostics[0].message,
        "@for is not linted in TSRX modules yet, so this module's markup is not linted"
    );
    assert_eq!((result.error_count, result.warning_count), (1, 0));
}
