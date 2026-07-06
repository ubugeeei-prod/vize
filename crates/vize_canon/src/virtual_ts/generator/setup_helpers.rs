//! Setup-scope compiler macro helper emission.
//!
//! Generic SFCs need a narrower `defineProps<T>()` boolean-prop model than the
//! shared helper can express safely. The shared conditional boolean-key helper
//! is intentionally left in place for non-generic SFCs, while this module uses
//! the parsed OXC type AST to pass only concrete local boolean keys for generic
//! setup scopes.

use vize_carton::{String, append};

use crate::virtual_ts::helpers::{VUE_SETUP_HELPERS, VUE_SETUP_HELPERS_HOISTED};

mod boolean_keys;

use boolean_keys::collect_define_props_boolean_keys;

pub(super) fn emit_setup_helpers(
    ts: &mut String,
    script_content: Option<&str>,
    generic_param: Option<&str>,
    hoist_shared_preamble: bool,
) {
    if generic_param.is_none() {
        ts.push_str(if hoist_shared_preamble {
            VUE_SETUP_HELPERS_HOISTED
        } else {
            VUE_SETUP_HELPERS
        });
        return;
    }

    let Some(boolean_keys) = script_content.and_then(collect_define_props_boolean_keys) else {
        ts.push_str(if hoist_shared_preamble {
            VUE_SETUP_HELPERS_HOISTED
        } else {
            VUE_SETUP_HELPERS
        });
        return;
    };
    emit_define_props_boolean_keys_type(ts, &boolean_keys);
    if hoist_shared_preamble {
        emit_hoisted_setup_helpers(ts);
    } else {
        emit_embedded_setup_helpers(ts);
    }
}

fn emit_define_props_boolean_keys_type(ts: &mut String, keys: &[String]) {
    if keys.is_empty() {
        ts.push_str("  type __VizeDefinePropsBooleanKeys<_T> = never;\n");
        return;
    }

    ts.push_str("  type __VizeDefinePropsBooleanKeys<_T> =\n");
    for (index, key) in keys.iter().enumerate() {
        let separator = if index == 0 { "    " } else { "  | " };
        let mut key_literal = String::default();
        push_ts_string_literal(&mut key_literal, key.as_str());
        append!(
            *ts,
            "{separator}(_T extends {{ {key_literal}?: boolean | undefined }} ? {key_literal} : never)\n"
        );
    }
    ts.push_str("  ;\n");
}

fn emit_hoisted_setup_helpers(ts: &mut String) {
    ts.push_str(
        r#"  // Compiler macros (setup-scope only; signatures hoisted to the shared helpers file)
  const defineProps = __vize_defineProps as {
    <_T = unknown>(): __DefineProps<_T, Extract<__VizeDefinePropsBooleanKeys<_T>, keyof _T>>;
    <const _T extends readonly string[]>(_props: _T): { [K in _T[number]]?: any };
    <const _T extends Record<string, any>>(_props: _T): __RuntimePropShape<_T>;
  };
  const defineEmits = __vize_defineEmits;
  const defineExpose = __vize_defineExpose;
  const defineModel = __vize_defineModel;
  const defineSlots = __vize_defineSlots;
  const withDefaults = __vize_withDefaults;
  const useTemplateRef = __vize_useTemplateRef;
  // Mark compiler macros as used
  void defineProps; void defineEmits; void defineExpose; void defineModel; void defineSlots; void withDefaults; void useTemplateRef;"#,
    );
}

fn emit_embedded_setup_helpers(ts: &mut String) {
    ts.push_str(
        r#"  // Compiler macros (only valid in setup scope, not global)
  function defineProps<_T = unknown>(): __DefineProps<_T, Extract<__VizeDefinePropsBooleanKeys<_T>, keyof _T>>;
  function defineProps<const _T extends readonly string[]>(_props: _T): { [K in _T[number]]?: any };
  function defineProps<const _T extends Record<string, any>>(_props: _T): __RuntimePropShape<_T>;
  function defineProps(_props?: any) { void _props; return undefined as any; }
  function defineEmits<_T = unknown>(): __EmitFn<_T>;
  function defineEmits<const _T extends readonly string[]>(_events: _T): (event: _T[number], ...args: any[]) => void;
  function defineEmits<const _T extends Record<string, any>>(_events: _T): __EmitFn<_T>;
  function defineEmits(_events?: any) { void _events; return (() => {}) as any; }
  function defineExpose<_T = unknown>(_exposed?: _T): void { void _exposed; }
  function defineModel<_T = unknown>(): __Ref<_T | undefined>;
  function defineModel<_T = unknown>(_options: any): __Ref<_T>;
  function defineModel<_T = unknown>(_name: string, _options?: any): __Ref<_T>;
  function defineModel(_name_or_options?: any, _options?: any) { void _name_or_options; void _options; return undefined as any; }
  function defineSlots<_T = unknown>(): _T { return undefined as unknown as _T; }
  function withDefaults<_T, _D extends __WithDefaultsArgs<_T>>(_props: _T, _defaults: _D): __WithDefaultsResult<_T, _D>; function withDefaults<_T, _D extends Record<string, any>>(_props: _T, _defaults: _D): __WithDefaultsResult<_T, _D>; function withDefaults(_props: any, _defaults: any) { void _props; void _defaults; return undefined as any; }
  function useTemplateRef<_T = any>(_key: string): __ShallowRef<_T | null> { void _key; return undefined as unknown as __ShallowRef<_T | null>; }
  // Mark compiler macros as used
  void defineProps; void defineEmits; void defineExpose; void defineModel; void defineSlots; void withDefaults; void useTemplateRef;"#,
    );
}

fn push_ts_string_literal(out: &mut String, value: &str) {
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(ch),
        }
    }
    out.push('"');
}
