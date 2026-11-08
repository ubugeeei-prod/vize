/// Resolve local type literals the same way vue-eslint-parser's merged SFC
/// program does. Imported types stay unknown, but a type/interface declared in
/// either script block retains the authored member span for the diagnostic.
struct TypeEventResolver<'a, 'b> {
    own: &'a Program<'a>,
    sibling: Option<&'b Program<'b>>,
    sibling_offset: Option<usize>,
}
impl TypeEventResolver<'_, '_> {
    fn collect(
        &self,
        ty: &TSType<'_>,
        block: TypeBlock,
        seen: &mut Vec<(TypeBlock, Span)>,
        events: &mut Vec<DeclaredEvent>,
    ) {
        match ty {
            TSType::TSUnionType(union) => {
                for ty in &union.types {
                    self.collect(ty, block, seen, events);
                }
            }
            TSType::TSIntersectionType(intersection) => {
                for ty in &intersection.types {
                    self.collect(ty, block, seen, events);
                }
            }
            TSType::TSParenthesizedType(parenthesized) => {
                self.collect(&parenthesized.type_annotation, block, seen, events);
            }
            TSType::TSTypeLiteral(literal) => {
                self.collect_members(&literal.members, block, events);
            }
            TSType::TSFunctionType(function) if function.this_param.is_none() => {
                collect_event_from_parameters(
                    &function.params,
                    function.span,
                    self.origin(block),
                    events,
                );
            }
            TSType::TSTypeReference(reference) => {
                let TSTypeName::IdentifierReference(identifier) = &reference.type_name else {
                    return;
                };
                let Some((declaration, declaration_block)) =
                    self.declaration(identifier.name.as_str(), block)
                else {
                    return;
                };
                let span = match declaration {
                    LocalType::Interface(interface) => interface.span,
                    LocalType::Alias(alias) => alias.span,
                };
                let key = (declaration_block, span);
                if seen.contains(&key) {
                    return;
                }
                seen.push(key);
                match declaration {
                    LocalType::Interface(interface) => {
                        self.collect_members(&interface.body.body, declaration_block, events);
                    }
                    LocalType::Alias(alias) => {
                        self.collect(&alias.type_annotation, declaration_block, seen, events);
                    }
                }
                seen.pop();
            }
            _ => {}
        }
    }

    fn collect_members(
        &self,
        members: &[TSSignature<'_>],
        block: TypeBlock,
        events: &mut Vec<DeclaredEvent>,
    ) {
        for member in members {
            collect_event_from_signature(member, self.origin(block), events);
        }
    }

    fn origin(&self, block: TypeBlock) -> Option<usize> {
        match block {
            TypeBlock::Own => None,
            TypeBlock::Sibling => self.sibling_offset,
        }
    }

    fn declaration(&self, name: &str, block: TypeBlock) -> Option<(LocalType<'_>, TypeBlock)> {
        match block {
            TypeBlock::Own => top_level_type(self.own, name)
                .map(|declaration| (declaration, TypeBlock::Own))
                .or_else(|| {
                    top_level_type(self.sibling?, name)
                        .map(|declaration| (declaration, TypeBlock::Sibling))
                }),
            TypeBlock::Sibling => self
                .sibling
                .and_then(|program| top_level_type(program, name))
                .map(|declaration| (declaration, TypeBlock::Sibling))
                .or_else(|| {
                    top_level_type(self.own, name).map(|declaration| (declaration, TypeBlock::Own))
                }),
        }
    }
}

fn top_level_type<'a>(program: &'a Program<'a>, name: &str) -> Option<LocalType<'a>> {
    let mut found = None;
    for statement in &program.body {
        let declaration = match statement {
            Statement::TSInterfaceDeclaration(interface) => LocalType::Interface(interface),
            Statement::TSTypeAliasDeclaration(alias) => LocalType::Alias(alias),
            Statement::ExportNamedDeclaration(export) => match &export.declaration {
                Some(Declaration::TSInterfaceDeclaration(interface)) => {
                    LocalType::Interface(interface)
                }
                Some(Declaration::TSTypeAliasDeclaration(alias)) => LocalType::Alias(alias),
                _ => continue,
            },
            _ => continue,
        };
        let declared_name = match declaration {
            LocalType::Interface(interface) => interface.id.name.as_str(),
            LocalType::Alias(alias) => alias.id.name.as_str(),
        };
        if declared_name == name {
            if found.is_some() {
                return None;
            }
            found = Some(declaration);
        }
    }
    found
}

fn collect_event_from_parameters(
    params: &oxc_ast::ast::FormalParameters<'_>,
    span: Span,
    origin_offset: Option<usize>,
    events: &mut Vec<DeclaredEvent>,
) {
    if let Some(first) = params.items.first()
        && let Some(annotation) = &first.type_annotation
    {
        collect_event_names_from_type(&annotation.type_annotation, span, origin_offset, events);
    }
}

fn collect_event_names_from_type(
    ty: &TSType<'_>,
    span: Span,
    origin_offset: Option<usize>,
    events: &mut Vec<DeclaredEvent>,
) {
    match ty {
        TSType::TSLiteralType(literal_type) => {
            if let TSLiteral::StringLiteral(string) = &literal_type.literal {
                events.push(DeclaredEvent {
                    name: CompactString::new(string.value.as_str()),
                    span,
                    origin_offset,
                });
            }
        }
        TSType::TSUnionType(union) => {
            for ty in &union.types {
                collect_event_names_from_type(ty, span, origin_offset, events);
            }
        }
        TSType::TSParenthesizedType(parenthesized) => collect_event_names_from_type(
            &parenthesized.type_annotation,
            span,
            origin_offset,
            events,
        ),
        _ => {}
    }
}
/// Extract an event name from a type-literal member: a property signature
/// (`change: [...]`) or a call signature (`(e: 'change', ...): void`).
fn collect_event_from_signature(
    member: &TSSignature<'_>,
    origin_offset: Option<usize>,
    events: &mut Vec<DeclaredEvent>,
) {
    match member {
        TSSignature::TSPropertySignature(property) => {
            if let Some(name) = property_key_name(&property.key) {
                events.push(DeclaredEvent {
                    name: CompactString::new(name),
                    span: property.span,
                    origin_offset,
                });
            }
        }
        TSSignature::TSMethodSignature(method) => {
            if let Some(name) = property_key_name(&method.key) {
                events.push(DeclaredEvent {
                    name: CompactString::new(name),
                    span: method.span,
                    origin_offset,
                });
            }
        }
        TSSignature::TSCallSignatureDeclaration(call_signature) => {
            collect_event_from_parameters(
                &call_signature.params,
                call_signature.span,
                origin_offset,
                events,
            );
        }
        _ => {}
    }
}
