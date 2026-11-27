use oxc_allocator::Allocator;
use oxc_ast::ast::{
    BindingPattern, Declaration, ExportNamedDeclaration, Expression, ModuleExportName, PropertyKey,
    Statement, TSGlobalDeclaration, TSInterfaceDeclaration, TSModuleBlock, TSModuleDeclaration,
    TSModuleDeclarationBody, TSSignature, TSTypeAnnotation, VariableDeclaration,
};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_l0::{FxHashMap, String, ToCompactString};

#[derive(Default)]
struct InterfaceInfo {
    members: Vec<(String, String)>,
    extends: Vec<String>,
}

/// What one walk over the `declare global` blocks of a declaration finds.
#[derive(Debug, Default)]
pub(super) struct DeclaredGlobals {
    /// `const name: Type` members as `(name, type text)`.
    pub(super) values: Vec<(String, String)>,
    pub(super) type_reexports: Vec<GlobalTypeReexport>,
}

/// One specifier of `export type { local as exported } from 'specifier'`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct GlobalTypeReexport {
    pub(super) local: String,
    pub(super) exported: String,
    pub(super) specifier: String,
}

pub(super) fn parse_interface_members_content(
    content: &str,
    interface_name: &str,
) -> Vec<(String, String)> {
    let interfaces = collect_interfaces(content);
    let Some(interface) = interfaces.get(normalize_interface_name(interface_name)) else {
        return Vec::new();
    };
    interface.members.clone()
}

pub(super) fn parse_global_component_members_content(content: &str) -> Vec<(String, String)> {
    let interfaces = collect_interfaces(content);
    let mut members = Vec::new();
    let mut visited = Vec::new();
    collect_interface_members_recursive(
        "GlobalComponents",
        &interfaces,
        &mut visited,
        &mut members,
    );
    members
}

pub(super) fn parse_declared_globals_content(content: &str) -> DeclaredGlobals {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, content, SourceType::d_ts()).parse();
    if ret.panicked {
        return DeclaredGlobals::default();
    }

    let mut globals = DeclaredGlobals::default();
    for statement in &ret.program.body {
        collect_globals_from_statement(statement, content, &mut globals);
    }
    globals
}

fn collect_interfaces(content: &str) -> FxHashMap<String, InterfaceInfo> {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, content, SourceType::d_ts()).parse();
    if ret.panicked {
        return FxHashMap::default();
    }

    let mut interfaces = FxHashMap::default();
    for statement in &ret.program.body {
        collect_interfaces_from_statement(statement, content, &mut interfaces);
    }
    interfaces
}

fn collect_interfaces_from_statement(
    statement: &Statement,
    content: &str,
    interfaces: &mut FxHashMap<String, InterfaceInfo>,
) {
    match statement {
        Statement::TSInterfaceDeclaration(interface) => {
            collect_interface(interface, content, interfaces);
        }
        Statement::TSModuleDeclaration(module) => {
            collect_interfaces_from_module(module, content, interfaces);
        }
        Statement::TSGlobalDeclaration(global) => {
            collect_interfaces_from_block(&global.body, content, interfaces);
        }
        Statement::ExportNamedDeclaration(export) => {
            if let Some(declaration) = &export.declaration {
                collect_interfaces_from_declaration(declaration, content, interfaces);
            }
        }
        _ => {}
    }
}

fn collect_interfaces_from_declaration(
    declaration: &Declaration,
    content: &str,
    interfaces: &mut FxHashMap<String, InterfaceInfo>,
) {
    match declaration {
        Declaration::TSInterfaceDeclaration(interface) => {
            collect_interface(interface, content, interfaces);
        }
        Declaration::TSModuleDeclaration(module) => {
            collect_interfaces_from_module(module, content, interfaces);
        }
        Declaration::TSGlobalDeclaration(global) => {
            collect_interfaces_from_block(&global.body, content, interfaces);
        }
        _ => {}
    }
}

fn collect_interfaces_from_module(
    module: &TSModuleDeclaration,
    content: &str,
    interfaces: &mut FxHashMap<String, InterfaceInfo>,
) {
    let Some(body) = &module.body else {
        return;
    };
    match body {
        TSModuleDeclarationBody::TSModuleDeclaration(module) => {
            collect_interfaces_from_module(module, content, interfaces);
        }
        TSModuleDeclarationBody::TSModuleBlock(block) => {
            collect_interfaces_from_block(block, content, interfaces);
        }
    }
}

fn collect_interfaces_from_block(
    block: &TSModuleBlock,
    content: &str,
    interfaces: &mut FxHashMap<String, InterfaceInfo>,
) {
    for statement in &block.body {
        collect_interfaces_from_statement(statement, content, interfaces);
    }
}

fn collect_interface(
    interface: &TSInterfaceDeclaration,
    content: &str,
    interfaces: &mut FxHashMap<String, InterfaceInfo>,
) {
    let info = interfaces
        .entry(interface.id.name.as_str().to_compact_string())
        .or_default();
    info.members.extend(
        interface
            .body
            .body
            .iter()
            .filter_map(|signature| interface_member(signature, content)),
    );
    for extended in interface.extends.iter().filter_map(interface_heritage_name) {
        if !info.extends.iter().any(|name| name.as_str() == extended) {
            info.extends.push(extended.to_compact_string());
        }
    }
}

fn collect_interface_members_recursive(
    name: &str,
    interfaces: &FxHashMap<String, InterfaceInfo>,
    visited: &mut Vec<String>,
    members: &mut Vec<(String, String)>,
) {
    if visited.iter().any(|visited| visited.as_str() == name) {
        return;
    }
    visited.push(name.to_compact_string());

    let Some(interface) = interfaces.get(name) else {
        return;
    };
    for member in &interface.members {
        if !members
            .iter()
            .any(|(existing, _)| existing.as_str() == member.0.as_str())
        {
            members.push(member.clone());
        }
    }
    for extended in &interface.extends {
        collect_interface_members_recursive(extended.as_str(), interfaces, visited, members);
    }
}

fn interface_member(signature: &TSSignature, content: &str) -> Option<(String, String)> {
    let TSSignature::TSPropertySignature(property) = signature else {
        return None;
    };
    let name = property_key_name(&property.key, property.computed)?;
    let type_annotation = property.type_annotation.as_ref()?;
    Some((
        name.to_compact_string(),
        type_annotation_text(type_annotation, content)?,
    ))
}

fn interface_heritage_name<'a>(
    heritage: &'a oxc_ast::ast::TSInterfaceHeritage<'a>,
) -> Option<&'a str> {
    match &heritage.expression {
        Expression::Identifier(identifier) => Some(identifier.name.as_str()),
        _ => None,
    }
}

fn collect_globals_from_statement(
    statement: &Statement,
    content: &str,
    globals: &mut DeclaredGlobals,
) {
    match statement {
        Statement::TSGlobalDeclaration(global) => {
            collect_globals_from_global(global, content, globals);
        }
        Statement::TSModuleDeclaration(module) => {
            collect_globals_from_module(module, content, globals);
        }
        Statement::ExportNamedDeclaration(export) => {
            if let Some(Declaration::TSGlobalDeclaration(global)) = &export.declaration {
                collect_globals_from_global(global, content, globals);
            }
        }
        _ => {}
    }
}

fn collect_globals_from_module(
    module: &TSModuleDeclaration,
    content: &str,
    globals: &mut DeclaredGlobals,
) {
    let Some(body) = &module.body else {
        return;
    };
    match body {
        TSModuleDeclarationBody::TSModuleDeclaration(module) => {
            collect_globals_from_module(module, content, globals);
        }
        TSModuleDeclarationBody::TSModuleBlock(block) => {
            for statement in &block.body {
                collect_globals_from_statement(statement, content, globals);
            }
        }
    }
}

fn collect_globals_from_global(
    global: &TSGlobalDeclaration,
    content: &str,
    globals: &mut DeclaredGlobals,
) {
    for statement in &global.body.body {
        match statement {
            Statement::VariableDeclaration(declaration) => {
                collect_variable_values(declaration, content, &mut globals.values);
            }
            Statement::ExportNamedDeclaration(export) => {
                if let Some(Declaration::VariableDeclaration(declaration)) = &export.declaration {
                    collect_variable_values(declaration, content, &mut globals.values);
                }
                collect_type_reexports(export, &mut globals.type_reexports);
            }
            _ => {}
        }
    }
}

/// A global type cannot be declared as an alias of `import('m').T` without
/// restating its type parameters, so generators re-export it from inside the
/// global augmentation instead. Value re-exports are left out: a global value
/// is always spelled as a `const`.
fn collect_type_reexports(
    export: &ExportNamedDeclaration,
    reexports: &mut Vec<GlobalTypeReexport>,
) {
    let Some(source) = &export.source else {
        return;
    };
    for specifier in &export.specifiers {
        if !export.export_kind.is_type() && !specifier.export_kind.is_type() {
            continue;
        }
        let (Some(local), Some(exported)) = (
            exported_identifier(&specifier.local),
            exported_identifier(&specifier.exported),
        ) else {
            continue;
        };
        reexports.push(GlobalTypeReexport {
            local: local.to_compact_string(),
            exported: exported.to_compact_string(),
            specifier: source.value.as_str().to_compact_string(),
        });
    }
}

/// String-literal export names cannot name a global type, so only identifiers
/// are kept.
fn exported_identifier<'a>(name: &'a ModuleExportName<'a>) -> Option<&'a str> {
    match name {
        ModuleExportName::IdentifierName(identifier) => Some(identifier.name.as_str()),
        ModuleExportName::IdentifierReference(identifier) => Some(identifier.name.as_str()),
        ModuleExportName::StringLiteral(_) => None,
    }
}

fn collect_variable_values(
    declaration: &VariableDeclaration,
    content: &str,
    values: &mut Vec<(String, String)>,
) {
    for declarator in &declaration.declarations {
        let BindingPattern::BindingIdentifier(identifier) = &declarator.id else {
            continue;
        };
        let Some(type_annotation) = &declarator.type_annotation else {
            continue;
        };
        let Some(type_annotation) = type_annotation_text(type_annotation, content) else {
            continue;
        };
        values.push((
            identifier.name.as_str().to_compact_string(),
            type_annotation,
        ));
    }
}

fn property_key_name<'a>(key: &'a PropertyKey, computed: bool) -> Option<&'a str> {
    if computed {
        return None;
    }
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.as_str()),
        PropertyKey::StringLiteral(literal) => Some(literal.value.as_str()),
        _ => None,
    }
}

fn type_annotation_text(annotation: &TSTypeAnnotation, content: &str) -> Option<String> {
    let start = annotation.span.start as usize;
    let end = annotation.span.end as usize;
    let raw = content.get(start..end)?.trim();
    let raw = raw.strip_prefix(':').unwrap_or(raw).trim();
    Some(raw.trim_end_matches(';').trim().to_compact_string())
}

fn normalize_interface_name(interface_name: &str) -> &str {
    interface_name
        .trim()
        .strip_prefix("interface ")
        .unwrap_or(interface_name)
        .trim()
}
