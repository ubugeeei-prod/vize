//! Template names that the component declares as props never read globals.

use super::tests::lint_sfc_with_only_this_rule;

fn reported(script: &str) -> Vec<String> {
    let source = format!(
        "<script setup lang=\"ts\">\n{script}\n</script>\n<template>\n  <Popover :open=\"open\">{{{{ close }}}}</Popover>\n</template>\n"
    );
    lint_sfc_with_only_this_rule(&source)
        .into_iter()
        .map(|message| message.to_string())
        .collect()
}

fn names(script: &str) -> Vec<&'static str> {
    let messages = reported(script);
    ["open", "close"]
        .into_iter()
        .filter(|name| {
            messages
                .iter()
                .any(|message| message.contains(&format!("'{name}'")))
        })
        .collect()
}

#[test]
fn every_syntactic_prop_declaration_shadows_the_global() {
    let cases = [
        // withDefaults keys over a type the file cannot see.
        "withDefaults(defineProps<Props>(), { open: undefined, close: '' })",
        // Pick literal keys, in the argument and through local heritage.
        "import type { Root } from 'reka-ui'\ndefineProps<Pick<Root, 'open' | \"close\">>()",
        "import type { Root } from 'reka-ui'\ninterface Props extends Pick<Root, 'open'>, Pick<Root, 'close'> {}\ndefineProps<Props>()",
        // Runtime declarations and destructured names.
        "defineProps({ open: Boolean, close: String })",
        "defineProps(['open', 'close'])",
        "const { open, close: closeLabel = '' } = defineProps<{ open?: boolean }>()",
    ];
    for script in cases {
        assert!(names(script).is_empty(), "{script}: {:?}", reported(script));
    }
}

#[test]
fn an_unresolved_imported_props_type_may_declare_any_template_name() {
    for script in [
        "import type { ErrorDialogData } from '#shared/types'\ndefineProps<ErrorDialogData>()",
        "import type { PopoverRootProps } from 'reka-ui'\ninterface Props extends Pick<PopoverRootProps, 'modal'> {}\nwithDefaults(defineProps<Props>(), {})",
    ] {
        assert!(names(script).is_empty(), "{script}: {:?}", reported(script));
    }
}

#[test]
fn complete_local_props_still_leave_free_globals_reported() {
    assert_eq!(
        names("defineProps<{ label?: string }>()"),
        ["open", "close"]
    );
    assert_eq!(
        names("const close = () => {}\ndefineProps<{ open?: boolean }>()"),
        Vec::<&str>::new()
    );
    assert_eq!(names("defineProps<{ open?: boolean }>()"), ["close"]);
}
