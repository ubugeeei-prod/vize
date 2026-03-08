//! Nuxt-specific auto-import and plugin injection helpers.

use std::{fs, path::Path};

use ignore::WalkBuilder;
use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Argument, Expression, ObjectExpression, ObjectPropertyKind, PropertyKey, Statement,
};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_canon::virtual_ts::VirtualTsOptions;
use vize_carton::{cstr, FxHashSet, String, ToCompactString};

use super::dts::{parse_declared_global_values, rewrite_relative_specifier};

pub(super) fn detect_nuxt_auto_imports(options: &mut VirtualTsOptions, cwd: &Path) {
    if !is_nuxt_project(cwd) {
        return;
    }

    let mut seen_names = FxHashSet::default();
    for stub in &options.auto_import_stubs {
        if let Some(name) = declared_name(stub) {
            seen_names.insert(name.to_compact_string());
        }
    }

    let mut collected = Vec::new();
    collect_generated_stubs(cwd, &mut collected, &mut seen_names);
    collect_plugin_injection_stubs(cwd, &mut collected, &mut seen_names);
    collect_fallback_stubs(&mut collected, &mut seen_names);

    options.auto_import_stubs.extend(collected);
}

fn is_nuxt_project(cwd: &Path) -> bool {
    cwd.join("nuxt.config.ts").exists()
        || cwd.join("nuxt.config.js").exists()
        || cwd.join("nuxt.config.mts").exists()
}

fn collect_generated_stubs(
    cwd: &Path,
    stubs: &mut Vec<String>,
    seen_names: &mut FxHashSet<String>,
) {
    let nuxt_types_dir = cwd.join(".nuxt/types");
    let mut found_typed_imports = false;

    if nuxt_types_dir.exists() {
        let walker = WalkBuilder::new(&nuxt_types_dir)
            .hidden(false)
            .standard_filters(false)
            .build();

        for entry in walker.flatten() {
            let path = entry.path();
            let is_dts = path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with(".d.ts"));
            if !path.is_file() || !is_dts {
                continue;
            }

            let file_name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");
            if file_name == "nitro-imports.d.ts" {
                continue;
            }
            if file_name == "imports.d.ts" {
                found_typed_imports = true;
            }

            if let Ok(values) = parse_declared_global_values(path) {
                for (name, type_annotation) in values {
                    push_declared_const(stubs, seen_names, &name, &type_annotation);
                }
            }
        }
    }

    if found_typed_imports {
        return;
    }

    let imports_path = cwd.join(".nuxt/imports.d.ts");
    if !imports_path.exists() {
        return;
    }

    if let Ok(content) = fs::read_to_string(&imports_path) {
        let base_dir = imports_path.parent().unwrap_or_else(|| Path::new("."));
        for line in content.lines() {
            let trimmed = line.trim();
            if !trimmed.starts_with("export {") {
                continue;
            }

            let Some((exports_part, from_part)) = trimmed
                .strip_prefix("export {")
                .and_then(|rest| rest.split_once("} from "))
            else {
                continue;
            };

            let module_specifier = parse_module_specifier(from_part);
            let Some(module_specifier) = module_specifier else {
                continue;
            };
            let module_specifier = rewrite_relative_specifier(module_specifier, base_dir);

            for export_part in exports_part.split(',') {
                let export_part = export_part.trim();
                if export_part.is_empty() {
                    continue;
                }

                let (local_name, exported_name) = parse_export_names(export_part);
                push_declared_const(
                    stubs,
                    seen_names,
                    exported_name,
                    &cstr!("typeof import('{module_specifier}')['{local_name}']"),
                );
            }
        }
    }
}

fn collect_plugin_injection_stubs(
    cwd: &Path,
    stubs: &mut Vec<String>,
    seen_names: &mut FxHashSet<String>,
) {
    let plugin_dirs = [cwd.join("app/plugins"), cwd.join("plugins")];
    let mut plugin_keys = Vec::new();

    for dir in plugin_dirs {
        if !dir.exists() {
            continue;
        }

        let walker = WalkBuilder::new(dir)
            .hidden(false)
            .standard_filters(false)
            .build();

        for entry in walker.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let Some(ext) = path.extension().and_then(|ext| ext.to_str()) else {
                continue;
            };
            if ext != "ts" && ext != "js" && ext != "mts" && ext != "cts" {
                continue;
            }

            if let Ok(source) = fs::read_to_string(path) {
                plugin_keys.extend(extract_plugin_provide_keys_from_source(&source));
            }
        }
    }

    plugin_keys.sort();
    plugin_keys.dedup();

    if plugin_keys.is_empty() {
        return;
    }

    stubs.push(
        "type __VizeNuxtInjection<K extends PropertyKey> = import('#app').NuxtApp extends Record<K, infer T> ? T : any;"
            .into(),
    );

    for key in plugin_keys {
        let injected_name = if key.starts_with('$') {
            key
        } else {
            cstr!("${key}")
        };
        push_stub(
            stubs,
            seen_names,
            cstr!("declare const {injected_name}: __VizeNuxtInjection<'{injected_name}'>;"),
        );
    }
}

fn push_declared_const(
    stubs: &mut Vec<String>,
    seen_names: &mut FxHashSet<String>,
    name: &str,
    type_annotation: &str,
) {
    push_stub(
        stubs,
        seen_names,
        cstr!("declare const {name}: {type_annotation};"),
    );
}

fn push_stub(stubs: &mut Vec<String>, seen_names: &mut FxHashSet<String>, stub: String) {
    let Some(name) = declared_name(&stub) else {
        stubs.push(stub);
        return;
    };
    if seen_names.insert(name.to_compact_string()) {
        stubs.push(stub);
    }
}

fn parse_module_specifier(from_part: &str) -> Option<&str> {
    let from_part = from_part.trim().trim_end_matches(';').trim();
    let quote = from_part.chars().next()?;
    if quote != '\'' && quote != '"' {
        return None;
    }
    let rest = &from_part[1..];
    let end = rest.find(quote)?;
    Some(&rest[..end])
}

fn parse_export_names(export_part: &str) -> (&str, &str) {
    if let Some((local_name, exported_name)) = export_part.split_once(" as ") {
        (local_name.trim(), exported_name.trim())
    } else {
        (export_part, export_part)
    }
}

fn declared_name(stub: &str) -> Option<&str> {
    for prefix in [
        "declare function ",
        "declare const ",
        "declare let ",
        "declare var ",
    ] {
        let Some(rest) = stub.strip_prefix(prefix) else {
            continue;
        };
        let end = rest
            .find(['<', '(', ':', '=', ';', ' '])
            .unwrap_or(rest.len());
        let name = rest[..end].trim();
        if !name.is_empty() {
            return Some(name);
        }
    }
    None
}

fn collect_fallback_stubs(stubs: &mut Vec<String>, seen_names: &mut FxHashSet<String>) {
    for stub in fallback_stub_strings() {
        push_stub(stubs, seen_names, stub);
    }
}

fn fallback_stub_strings() -> Vec<String> {
    vec![
        "declare function ref<T>(value: T): $Vue['Ref']<$Vue['UnwrapRef']<T>>;".into(),
        "declare function ref<T = any>(): $Vue['Ref']<T | undefined>;".into(),
        "declare function computed<T>(getter: () => T): $Vue['ComputedRef']<T>;".into(),
        "declare function computed<T>(options: { get: () => T; set: (value: T) => void }): $Vue['WritableComputedRef']<T>;".into(),
        "declare function reactive<T extends object>(target: T): $Vue['UnwrapNestedRefs']<T>;".into(),
        "declare function readonly<T extends object>(target: T): Readonly<T>;".into(),
        "declare function watch(source: any, cb: (...args: any[]) => any, options?: any): any;".into(),
        "declare function watchEffect(effect: () => void, options?: any): any;".into(),
        "declare function watchPostEffect(effect: () => void): any;".into(),
        "declare function watchSyncEffect(effect: () => void): any;".into(),
        "declare function onMounted(hook: () => any): void;".into(),
        "declare function onUnmounted(hook: () => any): void;".into(),
        "declare function onBeforeMount(hook: () => any): void;".into(),
        "declare function onBeforeUnmount(hook: () => any): void;".into(),
        "declare function onBeforeUpdate(hook: () => any): void;".into(),
        "declare function onUpdated(hook: () => any): void;".into(),
        "declare function onActivated(hook: () => any): void;".into(),
        "declare function onDeactivated(hook: () => any): void;".into(),
        "declare function onErrorCaptured(hook: (...args: any[]) => any): void;".into(),
        "declare function nextTick(fn?: () => void): Promise<void>;".into(),
        "declare function toRef<T extends object, K extends keyof T>(object: T, key: K): $Vue['Ref']<T[K]>;".into(),
        "declare function toRefs<T extends object>(object: T): { [K in keyof T]: $Vue['Ref']<T[K]> };".into(),
        "declare function unref<T>(ref: T | $Vue['Ref']<T>): T;".into(),
        "declare function isRef(value: any): value is $Vue['Ref'];".into(),
        "declare function shallowRef<T>(value: T): $Vue['ShallowRef']<T>;".into(),
        "declare function triggerRef(ref: $Vue['ShallowRef']): void;".into(),
        "declare function provide<T>(key: string | symbol, value: T): void;".into(),
        "declare function inject<T>(key: string | symbol, defaultValue?: T): T;".into(),
        "declare function defineAsyncComponent(source: any): any;".into(),
        "declare function h(type: any, ...args: any[]): any;".into(),
        "declare function useAttrs(): Record<string, unknown>;".into(),
        "declare function useSlots(): Record<string, (...args: any[]) => any>;".into(),
        "declare function toRaw<T>(observed: T): T;".into(),
        "declare function markRaw<T extends object>(value: T): T;".into(),
        "declare function effectScope(detached?: boolean): any;".into(),
        "declare function getCurrentScope(): any;".into(),
        "declare function onScopeDispose(fn: () => void): void;".into(),
        "declare function shallowReactive<T extends object>(target: T): T;".into(),
        "declare function shallowReadonly<T extends object>(target: T): Readonly<T>;".into(),
        "declare function customRef<T>(factory: any): $Vue['Ref']<T>;".into(),
        "declare function useRouter(): any;".into(),
        "declare function useRoute(name?: string): any;".into(),
        "declare function definePageMeta(meta: any): void;".into(),
        "declare function useSeoMeta(meta: any): void;".into(),
        "declare function useFetch<T = any>(url: string | (() => string), options?: any): any;".into(),
        "declare function useAsyncData<T = any>(handler: (...args: any[]) => T | Promise<T>, options?: any): any;".into(),
        "declare function useAsyncData<T = any>(key: string, handler: (...args: any[]) => T | Promise<T>, options?: any): any;".into(),
        "declare function useLazyFetch<T = any>(url: string | (() => string), options?: any): any;".into(),
        "declare function useLazyAsyncData<T = any>(handler: (...args: any[]) => T | Promise<T>, options?: any): any;".into(),
        "declare function useLazyAsyncData<T = any>(key: string, handler: (...args: any[]) => T | Promise<T>, options?: any): any;".into(),
        "declare function navigateTo(to: string | any, options?: any): any;".into(),
        "declare function createError(input: string | { statusCode?: number; statusMessage?: string; message?: string; data?: any; fatal?: boolean }): any;".into(),
        "declare function showError(error: any): any;".into(),
        "declare function clearError(options?: { redirect?: string }): Promise<void>;".into(),
        "declare function useNuxtApp(): any;".into(),
        "declare function useRuntimeConfig(): any;".into(),
        "declare function useAppConfig(): any;".into(),
        "declare function useState<T = any>(key: string, init?: () => T): $Vue['Ref']<T>;".into(),
        "declare function useCookie<T = any>(name: string, options?: any): $Vue['Ref']<T>;".into(),
        "declare function useHead(input: any): void;".into(),
        "declare function useRequestHeaders(headers?: string[]): Record<string, string>;".into(),
        "declare function useRequestURL(): URL;".into(),
        "declare function defineNuxtComponent(options: any): any;".into(),
        "declare function defineNuxtRouteMiddleware(middleware: any): any;".into(),
        "declare function useError(): any;".into(),
        "declare function abortNavigation(err?: any): any;".into(),
        "declare function addRouteMiddleware(name: string, middleware: any, options?: any): void;".into(),
        "declare function defineNuxtPlugin(plugin: any): any;".into(),
        "declare function setPageLayout(layout: string): void;".into(),
        "declare function setResponseStatus(code: number, message?: string): void;".into(),
        "declare function prerenderRoutes(routes: string | string[]): void;".into(),
        "declare function refreshNuxtData(keys?: string | string[]): Promise<void>;".into(),
        "declare function clearNuxtData(keys?: string | string[]): void;".into(),
        "declare function reloadNuxtApp(options?: any): void;".into(),
        "declare function callOnce(key: string, fn: () => any): Promise<void>;".into(),
        "declare function callOnce(fn: () => any): Promise<void>;".into(),
        "declare function onNuxtReady(callback: () => any): void;".into(),
        "declare function preloadComponents(components: string | string[]): Promise<void>;".into(),
        "declare function prefetchComponents(components: string | string[]): Promise<void>;".into(),
        "declare function useRequestEvent(): any;".into(),
        "declare function useRequestFetch(): typeof globalThis.fetch;".into(),
        "declare function useResponseHeaders(headers?: Record<string, string>): any;".into(),
    ]
}

fn extract_plugin_provide_keys_from_source(source: &str) -> Vec<String> {
    let allocator = Allocator::default();
    let source_type = SourceType::default()
        .with_module(true)
        .with_typescript(true);
    let ret = Parser::new(&allocator, source, source_type).parse();
    let mut keys = Vec::new();

    for statement in &ret.program.body {
        let Statement::ExportDefaultDeclaration(export) = statement else {
            continue;
        };
        let Some(call) = extract_call_expression_from_export(&export.declaration) else {
            continue;
        };
        let Expression::Identifier(callee) = &call.callee else {
            continue;
        };
        if callee.name.as_str() != "defineNuxtPlugin" {
            continue;
        }
        let Some(first_arg) = call.arguments.first() else {
            continue;
        };
        collect_plugin_keys_from_argument(first_arg, &mut keys);
    }

    keys
}

fn extract_call_expression_from_export<'a>(
    expr: &'a oxc_ast::ast::ExportDefaultDeclarationKind<'a>,
) -> Option<&'a oxc_ast::ast::CallExpression<'a>> {
    match expr {
        oxc_ast::ast::ExportDefaultDeclarationKind::CallExpression(call) => Some(call),
        oxc_ast::ast::ExportDefaultDeclarationKind::ParenthesizedExpression(paren) => {
            extract_call_expression(&paren.expression)
        }
        oxc_ast::ast::ExportDefaultDeclarationKind::TSAsExpression(ts_as) => {
            extract_call_expression(&ts_as.expression)
        }
        oxc_ast::ast::ExportDefaultDeclarationKind::TSSatisfiesExpression(ts_satisfies) => {
            extract_call_expression(&ts_satisfies.expression)
        }
        oxc_ast::ast::ExportDefaultDeclarationKind::TSNonNullExpression(ts_non_null) => {
            extract_call_expression(&ts_non_null.expression)
        }
        _ => None,
    }
}

fn extract_call_expression<'a>(
    expr: &'a Expression<'a>,
) -> Option<&'a oxc_ast::ast::CallExpression<'a>> {
    match expr {
        Expression::CallExpression(call) => Some(call),
        Expression::ParenthesizedExpression(paren) => extract_call_expression(&paren.expression),
        Expression::TSAsExpression(ts_as) => extract_call_expression(&ts_as.expression),
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            extract_call_expression(&ts_satisfies.expression)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            extract_call_expression(&ts_non_null.expression)
        }
        _ => None,
    }
}

fn collect_plugin_keys_from_argument(arg: &Argument<'_>, keys: &mut Vec<String>) {
    match arg {
        Argument::ObjectExpression(object) => collect_plugin_keys_from_object(object, keys),
        Argument::ArrowFunctionExpression(arrow) => {
            collect_plugin_keys_from_function_body(&arrow.body.statements, keys)
        }
        Argument::FunctionExpression(function) => {
            if let Some(body) = &function.body {
                collect_plugin_keys_from_function_body(&body.statements, keys);
            }
        }
        _ => {}
    }
}

fn collect_plugin_keys_from_function_body<'a>(
    statements: &oxc_allocator::Vec<'a, Statement<'a>>,
    keys: &mut Vec<String>,
) {
    for statement in statements {
        let Statement::ReturnStatement(ret) = statement else {
            continue;
        };
        let Some(argument) = &ret.argument else {
            continue;
        };
        let Some(object) = extract_object_expression(argument) else {
            continue;
        };
        collect_plugin_keys_from_object(object, keys);
    }
}

fn collect_plugin_keys_from_object(object: &ObjectExpression<'_>, keys: &mut Vec<String>) {
    if let Some(provide_object) =
        find_object_property(object, "provide").and_then(extract_object_expression)
    {
        collect_object_keys(provide_object, keys);
    }

    if let Some(setup_expression) = find_object_property(object, "setup") {
        match extract_expression(setup_expression) {
            Some(Expression::ArrowFunctionExpression(arrow)) => {
                collect_plugin_keys_from_function_body(&arrow.body.statements, keys);
            }
            Some(Expression::FunctionExpression(function)) => {
                if let Some(body) = &function.body {
                    collect_plugin_keys_from_function_body(&body.statements, keys);
                }
            }
            _ => {}
        }
    }
}

fn collect_object_keys(object: &ObjectExpression<'_>, keys: &mut Vec<String>) {
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        let Some(name) = static_property_name(&property.key) else {
            continue;
        };
        keys.push(name.to_compact_string());
    }
}

fn find_object_property<'a>(
    object: &'a ObjectExpression<'a>,
    name: &str,
) -> Option<&'a Expression<'a>> {
    object.properties.iter().find_map(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return None;
        };
        if static_property_name(&property.key) == Some(name) {
            Some(&property.value)
        } else {
            None
        }
    })
}

fn extract_object_expression<'a>(expr: &'a Expression<'a>) -> Option<&'a ObjectExpression<'a>> {
    match expr {
        Expression::ObjectExpression(object) => Some(object),
        Expression::ParenthesizedExpression(paren) => extract_object_expression(&paren.expression),
        Expression::TSAsExpression(ts_as) => extract_object_expression(&ts_as.expression),
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            extract_object_expression(&ts_satisfies.expression)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            extract_object_expression(&ts_non_null.expression)
        }
        _ => None,
    }
}

fn extract_expression<'a>(expr: &'a Expression<'a>) -> Option<&'a Expression<'a>> {
    match expr {
        Expression::ParenthesizedExpression(paren) => extract_expression(&paren.expression),
        Expression::TSAsExpression(ts_as) => extract_expression(&ts_as.expression),
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            extract_expression(&ts_satisfies.expression)
        }
        Expression::TSNonNullExpression(ts_non_null) => extract_expression(&ts_non_null.expression),
        _ => Some(expr),
    }
}

fn static_property_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.as_str()),
        PropertyKey::StringLiteral(literal) => Some(literal.value.as_str()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{
        declared_name, extract_plugin_provide_keys_from_source, parse_export_names,
        parse_module_specifier,
    };

    #[test]
    fn parses_module_export_lines() {
        assert_eq!(
            parse_module_specifier("'../../app/composables/users';"),
            Some("../../app/composables/users")
        );
        assert_eq!(parse_export_names("foo as bar"), ("foo", "bar"));
        assert_eq!(parse_export_names("foo"), ("foo", "foo"));
    }

    #[test]
    fn extracts_plugin_provide_keys_from_callback_plugin() {
        let source = r#"
export default defineNuxtPlugin(() => {
  return {
    provide: {
      scrollToTop: () => {},
      pageLifecycle: reactive({}),
    },
  }
})
"#;

        let keys = extract_plugin_provide_keys_from_source(source);
        assert_eq!(keys, vec!["scrollToTop", "pageLifecycle"]);
    }

    #[test]
    fn extracts_plugin_provide_keys_from_setup_plugin_object() {
        let source = r#"
export default defineNuxtPlugin({
  async setup() {
    return {
      provide: {
        masto,
      },
    }
  },
})
"#;

        let keys = extract_plugin_provide_keys_from_source(source);
        assert_eq!(keys, vec!["masto"]);
    }

    #[test]
    fn declared_name_supports_const_stubs() {
        assert_eq!(
            declared_name("declare const currentUser: any;"),
            Some("currentUser")
        );
    }

    #[test]
    fn relative_specifier_rewrite_matches_project_root_layout() {
        let rewritten = super::rewrite_relative_specifier(
            "../../app/composables/users",
            Path::new("/workspace/.nuxt/types"),
        );
        assert_eq!(rewritten.as_str(), "/workspace/app/composables/users");
    }
}
