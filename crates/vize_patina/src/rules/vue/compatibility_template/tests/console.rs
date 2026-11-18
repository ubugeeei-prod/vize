use super::super::NoConsole;
use super::reported;

#[test]
fn no_console_reports_the_console_object() {
    assert_eq!(
        reported(
            Box::new(NoConsole),
            concat!(
                "<button @error=\"console.error\" />",
                "<p>{{ console['log'](value) }}</p>",
                "<button @click=\"() => (console).warn(1)\" />",
                "<p>{{ logger.console }}</p>",
            ),
        ),
        ["console", "console", "console"]
    );
}

#[test]
fn no_literals_in_template_reports_the_literal_node() {
    assert_eq!(
        reported(
            Box::new(super::super::NoLiteralsInTemplate),
            concat!(
                "<Tab :on-shown=\"\n  () => {\n    focus()\n  }\n\" />",
                "<Tab :icon=\"({ tag: 'Shared' })\" />",
                "<Tab :items=\" [1, 2] \" />",
            ),
        ),
        ["() => {\n    focus()\n  }", "{ tag: 'Shared' }", "[1, 2]"]
    );
}
