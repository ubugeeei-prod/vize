use crate::virtual_ts::generated_names as names;
use vize_carton::{String, append};
use vize_l0::ts;

/// Runtime prop source used to publish an Options API component's `Props`.
pub(crate) struct OptionsApiPropsSource {
    direct: Option<DirectOptionsApiPropsSource>,
    captures_default: bool,
}

enum DirectOptionsApiPropsSource {
    /// Runtime values that must remain in setup scope.
    DeferredObject(String),
    /// Array-form props, which carry names but no runtime types.
    Names(Vec<String>),
}

impl OptionsApiPropsSource {
    pub(crate) fn deferred_object(source: String) -> Self {
        Self {
            direct: Some(DirectOptionsApiPropsSource::DeferredObject(source)),
            captures_default: false,
        }
    }

    pub(crate) fn names(names: Vec<String>) -> Self {
        Self {
            direct: Some(DirectOptionsApiPropsSource::Names(names)),
            captures_default: false,
        }
    }

    pub(crate) fn default_export() -> Self {
        Self {
            direct: None,
            captures_default: true,
        }
    }

    pub(crate) fn with_default_export(mut self) -> Self {
        self.captures_default = true;
        self
    }

    pub(crate) fn deferred_object_source(&self) -> Option<&str> {
        match self.direct.as_ref() {
            Some(DirectOptionsApiPropsSource::DeferredObject(source)) => Some(source.as_str()),
            _ => None,
        }
    }

    pub(crate) fn captures_default(&self) -> bool {
        self.captures_default
    }
}

/// Emit the public prop shape without moving runtime values into type position.
pub(super) fn emit_options_api_props_type(
    ts: &mut crate::virtual_ts::TextBuf,
    generic_decl: &str,
    source: &OptionsApiPropsSource,
    type_name: &'static str,
    exported: bool,
) {
    if source.captures_default {
        append!(
            *ts,
            "type {}<T> = $IsAny<T> extends true ? {{}} : T extends abstract new (...args: any[]) => {{ $props: infer P }} ? P : {{}};\n",
            names::declared("$DefaultProps")
        );
    }
    let head = if exported {
        names::export_type(type_name)
    } else {
        names::local_type(type_name)
    };
    let declared = match source.direct.as_ref() {
        // A `default` makes the prop optional for the parent even beside
        // `required: true` (Vue's `$props` is `Partial<Defaults> & …`), except
        // on a plain `Boolean` prop, whose `required: true` wins; this is the
        // `$props` `vue-tsc` checks the parent against.
        Some(DirectOptionsApiPropsSource::DeferredObject(_)) => {
            ts.push_str(
            "type $OptionsPropRequired<T> = T extends { required: true } ? T extends { default: any } ? T extends { type: BooleanConstructor } ? true : false : true : false;\ntype $OptionsPropShape<T extends Record<string, any>> = { [K in keyof T as $OptionsPropRequired<T[K]> extends true ? K : never]-?: __RuntimePropCtor<T[K]>; } & { [K in keyof T as $OptionsPropRequired<T[K]> extends true ? never : K]?: __RuntimePropCtor<T[K]>; };\n",
            );
            false
        }
        Some(DirectOptionsApiPropsSource::Names(names)) => {
            if exported {
                ts! { &mut *ts => #head #generic_decl #(" = ") }
            } else {
                ts! { &mut *ts => #head #(" = ") }
            }
            ts! { &mut *ts =>
                {
                    #("\n")
                    #{
                        for name in names {
                            ts! { &mut *__ts => #("  \"") #name #("\"?: unknown;\n") }
                        }
                    }
                }
            }
            append_default_props(ts, source);
            ts.push_str(";\n");
            true
        }
        None => {
            if exported {
                append!(*ts, "{head}{generic_decl} = ");
            } else {
                append!(*ts, "{head} = ");
            }
            push_default_props(ts);
            ts.push_str(";\n");
            true
        }
    };
    if exported && declared {
        names::reexport(ts, type_name);
    }
}

fn push_default_props(ts: &mut crate::virtual_ts::TextBuf) {
    append!(
        *ts,
        "{}<{}[\"__default__\"]>",
        names::declared("$DefaultProps"),
        names::setup_return("")
    );
}

pub(crate) fn append_default_props(
    ts: &mut crate::virtual_ts::TextBuf,
    source: &OptionsApiPropsSource,
) {
    if source.captures_default {
        ts.push_str(" & ");
        push_default_props(ts);
    }
}
