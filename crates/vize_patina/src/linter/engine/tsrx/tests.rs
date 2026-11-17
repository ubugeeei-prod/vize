use crate::linter::{LintResult, Linter};

fn lint_tsrx(linter: &Linter, source: &str, filename: &str) -> LintResult {
    linter.lint_tsrx(source, filename)
}

fn rows(result: &LintResult) -> Vec<(&str, u32, u32)> {
    result
        .diagnostics
        .iter()
        .map(|diagnostic| (diagnostic.rule_name, diagnostic.start, diagnostic.end))
        .collect()
}

#[test]
fn regions_merge_in_source_order_with_one_report_per_rule_and_range() {
    // The `<li>` region is the `.map()` loop's region in the `<ul>` region,
    // so the loop rules see it.
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
            ("vue/require-v-for-key", 81, 145),
            ("vue/no-multi-spaces", 105, 107),
            ("a11y/alt-text", 120, 138),
            ("vue/no-multi-spaces", 194, 196),
        ]
    );
    assert_eq!((result.error_count, result.warning_count), (1, 4));
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
fn value_and_regions_are_linted_inside_the_enclosing_region() {
    // `{n && …}` with a non-boolean `n` is a `ui.and` in the `<ul>` region.
    let source = "export function Avatar({ n, src }: { n: number; src: string }) @{
  <ul>{n && <li><img  src={src} /></li>}{n && <li>{n}</li>}</ul>
}
";
    let result = lint_tsrx(&Linter::new(), source, "Avatar.tsrx");
    assert_eq!(
        rows(&result),
        [("a11y/alt-text", 82, 100), ("vue/no-multi-spaces", 86, 88)]
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
fn a_keyed_host_loop_lints_its_items_and_its_empty_region() {
    // The head's `key` keys every item; `@empty` renders outside the list.
    let source = "export function List({ items }: { items: string[] }) @{
  <ul  class=\"list\">
    @for (const item of items; key item) {
      <li  title={item}><img src={item} /></li>
    } @empty {
      <li  class=\"none\"><img src=\"none.png\" /></li>
    }
  </ul>
}
";
    let result = lint_tsrx(&Linter::new(), source, "List.tsrx");
    let at = |text: &str| u32::try_from(source.find(text).unwrap()).unwrap();
    assert_eq!(
        rows(&result),
        [
            ("vue/no-multi-spaces", 61, 63),
            ("vue/no-multi-spaces", 129, 131),
            ("a11y/alt-text", 144, 162),
            ("vue/no-multi-spaces", 192, 194),
            ("a11y/alt-text", 207, 229),
        ]
    );
    assert_eq!(at("<ul  ") + 3, 61);
    assert_eq!(at("<li  title") + 3, 129);
    assert_eq!(at("<li  class") + 3, 192);
    assert_eq!(at("<img src=\"none.png\""), 207);
}

#[test]
fn an_unkeyed_host_loop_needs_a_key_on_its_items() {
    let source = "export function List({ items }: { items: string[] }) @{
  <ul>
    @for (const item of items) {
      <li>{item}</li>
    }
  </ul>
}
";
    let result = lint_tsrx(&Linter::new(), source, "List.tsrx");
    // Reported at the loop, as for a JSX `.map()` list.
    let start = u32::try_from(source.find("@for").unwrap()).unwrap();
    let end = u32::try_from(source.find("\n  </ul>").unwrap()).unwrap();
    assert_eq!(rows(&result), [("vue/require-v-for-key", start, end)]);
    assert_eq!((start, end), (67, 123));
}
