//! Names and `defineEmits<T>()` types, resolved as upstream's merged program
//! sees them.
//!
//! vue-eslint-parser lints `<script>` and `<script setup>` as one program, so
//! a setup reference may resolve to a sibling-block declaration. Here each block
//! is its own parse: a reference unresolved in the linted block is looked up by
//! name among the sibling's top-level declarations. Sibling nodes carry no
//! references into this block's [`Semantic`], so they resolve by name only.
//!
//! Types follow upstream's `flattenTypeNodes` and `extractRuntimeEmits`.

use oxc_ast::{
    AstKind,
    ast::{
        BindingPattern, Declaration, Expression, FormalParameters, IdentifierReference, Program,
        PropertyKey, Statement, TSInterfaceDeclaration, TSLiteral, TSSignature, TSType,
        TSTypeAliasDeclaration, TSTypeName, VariableDeclarationKind,
    },
};
use oxc_semantic::Semantic;
use oxc_span::Span;
use oxc_syntax::symbol::SymbolId;
use vize_l0::{CompactString, ToCompactString};

use super::declared::{Collect, Emits, define_value};

/// Which parse a node belongs to.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Block {
    Own,
    Sibling,
}

pub(super) struct Resolver<'s, 'a, 'c> {
    pub(super) semantic: &'s Semantic<'a>,
    pub(super) program: &'a Program<'a>,
    pub(super) sibling: Option<&'c Program<'c>>,
}

impl Resolver<'_, '_, '_> {
    /// Upstream's `getObjectOrArray`: a literal, or a `const` bound to one.
    pub(super) fn object_or_array(
        &self,
        expression: &Expression<'_>,
        block: Block,
        depth: u8,
    ) -> Option<Emits> {
        let Expression::Identifier(identifier) = expression.without_parentheses() else {
            return define_value(expression);
        };
        if depth > 8 {
            return None;
        }
        if block == Block::Own
            && let Some(symbol) = self.symbol(identifier)
        {
            if !self
                .semantic
                .scoping()
                .symbol_redeclarations(symbol)
                .is_empty()
            {
                return None;
            }
            let AstKind::VariableDeclarator(declarator) =
                self.semantic.symbol_declaration(symbol).kind()
            else {
                return None;
            };
            if declarator.kind != VariableDeclarationKind::Const
                || !matches!(declarator.id, BindingPattern::BindingIdentifier(_))
            {
                return None;
            }
            return self.object_or_array(declarator.init.as_ref()?, Block::Own, depth + 1);
        }
        let name = identifier.name.as_str();
        if let Some(init) = self
            .sibling
            .and_then(|sibling| top_level_const(sibling, name))
        {
            return self.object_or_array(init, Block::Sibling, depth + 1);
        }
        match block {
            Block::Sibling => {
                let init = top_level_const(self.program, name)?;
                self.object_or_array(init, Block::Own, depth + 1)
            }
            Block::Own => None,
        }
    }

    /// The events a `defineEmits<T>()` type declares.
    pub(super) fn type_emits(&self, ty: &TSType<'_>) -> Emits {
        let mut collect = Collect::default();
        self.collect_type(ty, Block::Own, &mut collect, &mut Vec::new());
        collect.finish()
    }

    fn collect_type(
        &self,
        ty: &TSType<'_>,
        block: Block,
        collect: &mut Collect,
        aliases: &mut Vec<(bool, Span)>,
    ) {
        match ty {
            TSType::TSUnionType(union) => {
                for ty in &union.types {
                    self.collect_type(ty, block, collect, aliases);
                }
            }
            TSType::TSIntersectionType(intersection) => {
                for ty in &intersection.types {
                    self.collect_type(ty, block, collect, aliases);
                }
            }
            TSType::TSParenthesizedType(paren) => {
                self.collect_type(&paren.type_annotation, block, collect, aliases);
            }
            TSType::TSTypeLiteral(literal) => collect_members(&literal.members, collect),
            TSType::TSFunctionType(function) => {
                if function.this_param.is_none() {
                    event_names(&function.params, collect);
                }
            }
            TSType::TSTypeReference(reference) => match &reference.type_name {
                TSTypeName::IdentifierReference(name) => {
                    self.collect_reference(name, block, collect, aliases);
                }
                _ => collect.unknown = true,
            },
            _ => collect.unknown = true,
        }
    }

    fn collect_reference(
        &self,
        name: &IdentifierReference<'_>,
        block: Block,
        collect: &mut Collect,
        aliases: &mut Vec<(bool, Span)>,
    ) {
        if block == Block::Own
            && let Some(symbol) = self.symbol(name)
        {
            if !self
                .semantic
                .scoping()
                .symbol_redeclarations(symbol)
                .is_empty()
            {
                collect.unknown = true;
                return;
            }
            match self.semantic.symbol_declaration(symbol).kind() {
                AstKind::TSInterfaceDeclaration(interface) => {
                    collect_members(&interface.body.body, collect);
                }
                AstKind::TSTypeAliasDeclaration(alias) => {
                    self.collect_alias(alias, Block::Own, collect, aliases);
                }
                _ => collect.unknown = true,
            }
            return;
        }
        let name = name.name.as_str();
        if let Some(sibling) = self.sibling
            && let Some(declaration) = top_level_type(sibling, name)
        {
            self.collect_declaration(declaration, Block::Sibling, collect, aliases);
            return;
        }
        match (block, top_level_type(self.program, name)) {
            (Block::Sibling, Some(declaration)) => {
                self.collect_declaration(declaration, Block::Own, collect, aliases);
            }
            _ => collect.unknown = true,
        }
    }

    fn collect_declaration(
        &self,
        declaration: TypeDeclaration<'_, '_>,
        block: Block,
        collect: &mut Collect,
        aliases: &mut Vec<(bool, Span)>,
    ) {
        match declaration {
            TypeDeclaration::Interface(interface) => collect_members(&interface.body.body, collect),
            TypeDeclaration::Alias(alias) => self.collect_alias(alias, block, collect, aliases),
        }
    }

    fn collect_alias(
        &self,
        alias: &TSTypeAliasDeclaration<'_>,
        block: Block,
        collect: &mut Collect,
        aliases: &mut Vec<(bool, Span)>,
    ) {
        let key = (block == Block::Sibling, alias.span);
        if aliases.contains(&key) {
            collect.unknown = true;
            return;
        }
        aliases.push(key);
        self.collect_type(&alias.type_annotation, block, collect, aliases);
    }

    fn symbol(&self, identifier: &IdentifierReference<'_>) -> Option<SymbolId> {
        let scoping = self.semantic.scoping();
        scoping
            .get_reference(identifier.reference_id.get()?)
            .symbol_id()
    }
}

#[derive(Clone, Copy)]
enum TypeDeclaration<'b, 'x> {
    Interface(&'b TSInterfaceDeclaration<'x>),
    Alias(&'b TSTypeAliasDeclaration<'x>),
}

/// The only top-level interface or type alias named `name`.
fn top_level_type<'b, 'x>(program: &'b Program<'x>, name: &str) -> Option<TypeDeclaration<'b, 'x>> {
    let mut found = None;
    for statement in &program.body {
        let declaration = match statement {
            Statement::TSInterfaceDeclaration(interface) => TypeDeclaration::Interface(interface),
            Statement::TSTypeAliasDeclaration(alias) => TypeDeclaration::Alias(alias),
            Statement::ExportNamedDeclaration(export) => match &export.declaration {
                Some(Declaration::TSInterfaceDeclaration(interface)) => {
                    TypeDeclaration::Interface(interface)
                }
                Some(Declaration::TSTypeAliasDeclaration(alias)) => TypeDeclaration::Alias(alias),
                _ => continue,
            },
            _ => continue,
        };
        let declared = match declaration {
            TypeDeclaration::Interface(interface) => interface.id.name.as_str(),
            TypeDeclaration::Alias(alias) => alias.id.name.as_str(),
        };
        if declared == name {
            if found.is_some() {
                return None;
            }
            found = Some(declaration);
        }
    }
    found
}

/// The initializer of a top-level `const name = ...`.
fn top_level_const<'b, 'x>(program: &'b Program<'x>, name: &str) -> Option<&'b Expression<'x>> {
    program.body.iter().find_map(|statement| {
        let declaration = match statement {
            Statement::VariableDeclaration(declaration) => declaration,
            Statement::ExportNamedDeclaration(export) => match &export.declaration {
                Some(Declaration::VariableDeclaration(declaration)) => declaration,
                _ => return None,
            },
            _ => return None,
        };
        if declaration.kind != VariableDeclarationKind::Const {
            return None;
        }
        declaration
            .declarations
            .iter()
            .find_map(|declarator| match &declarator.id {
                BindingPattern::BindingIdentifier(id) if id.name == name => {
                    declarator.init.as_ref()
                }
                _ => None,
            })
    })
}

fn collect_members(members: &[TSSignature<'_>], collect: &mut Collect) {
    for member in members {
        match member {
            TSSignature::TSCallSignatureDeclaration(signature) => {
                if signature.this_param.is_none() {
                    event_names(&signature.params, collect);
                }
            }
            TSSignature::TSPropertySignature(property) => collect.push(type_key(&property.key)),
            TSSignature::TSMethodSignature(method) => collect.push(type_key(&method.key)),
            _ => {}
        }
    }
}

/// A type member key: upstream accepts any `Identifier` (even computed) or
/// `Literal` key.
fn type_key(key: &PropertyKey<'_>) -> Option<CompactString> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.to_compact_string()),
        PropertyKey::Identifier(identifier) => Some(identifier.name.to_compact_string()),
        PropertyKey::StringLiteral(literal) => Some(literal.value.to_compact_string()),
        PropertyKey::NumericLiteral(literal) => Some(literal.value.to_compact_string()),
        PropertyKey::BooleanLiteral(literal) => Some(literal.value.to_compact_string()),
        PropertyKey::BigIntLiteral(literal) => Some(literal.value.to_compact_string()),
        PropertyKey::NullLiteral(_) => Some(CompactString::new("null")),
        _ => None,
    }
}

/// Upstream's `extractEventNames`: literal types of the first parameter. Any
/// other parameter type declares nothing, and is not unknown.
fn event_names(params: &FormalParameters<'_>, collect: &mut Collect) {
    let Some(first) = params.items.first() else {
        return;
    };
    if !matches!(first.pattern, BindingPattern::BindingIdentifier(_)) {
        return;
    }
    let Some(annotation) = first.type_annotation.as_ref() else {
        return;
    };
    let mut ty = &annotation.type_annotation;
    while let TSType::TSParenthesizedType(paren) = ty {
        ty = &paren.type_annotation;
    }
    let types = match ty {
        TSType::TSUnionType(union) => union.types.iter().collect(),
        ty => vec![ty],
    };
    for ty in types {
        if let Some(name) = literal_type(ty) {
            collect.push(Some(name));
        }
    }
}

fn literal_type(ty: &TSType<'_>) -> Option<CompactString> {
    let TSType::TSLiteralType(literal) = ty else {
        return None;
    };
    match &literal.literal {
        TSLiteral::StringLiteral(literal) => Some(literal.value.to_compact_string()),
        TSLiteral::NumericLiteral(literal) => Some(literal.value.to_compact_string()),
        TSLiteral::BooleanLiteral(literal) => Some(literal.value.to_compact_string()),
        TSLiteral::BigIntLiteral(literal) => Some(literal.value.to_compact_string()),
        _ => None,
    }
}
