fn check_options_api<'a>(
    program: &'a Program<'a>,
    source: &str,
    offset: usize,
    sfc: SfcScriptContext<'_>,
    result: &mut ScriptLintResult,
) {
    let built = SemanticBuilder::new().with_build_nodes(true).build(program);
    let objects = VueObjects::collect(program, source, &built.semantic, sfc.is_sfc);
    let mut template_used = FxHashSet::default();
    if let Some(template) = sfc.template_source {
        collect_template_emitted_events(template, "", &mut template_used);
    }
    let mut visitor = OptionsUnusedCollector {
        semantic: &built.semantic,
        objects: &objects,
        frames: Vec::new(),
        template_used: &template_used,
        offset,
        result,
    };
    visitor.visit_program(program);
}
struct OptionsFrame {
    events: Vec<DeclaredEvent>,
    used: FxHashSet<CompactString>,
    emit: Option<SymbolId>,
    context: Option<SymbolId>,
    escaped: bool,
}

struct OptionsUnusedCollector<'o, 'r, 'a> {
    semantic: &'o Semantic<'a>,
    objects: &'o VueObjects,
    frames: Vec<OptionsFrame>,
    template_used: &'o FxHashSet<CompactString>,
    offset: usize,
    result: &'r mut ScriptLintResult,
}

impl<'a> Visit<'a> for OptionsUnusedCollector<'_, '_, 'a> {
    fn visit_object_expression(&mut self, object: &ObjectExpression<'a>) {
        let events = self
            .objects
            .kind(object)
            .and_then(|_| collect_options_declared_events(object));
        let pushed = events.is_some();
        if let Some(events) = events {
            let (emit, context) = setup_context(object);
            self.frames.push(OptionsFrame {
                events,
                used: self.template_used.clone(),
                emit,
                context,
                escaped: false,
            });
        }
        walk_object_expression(self, object);
        if pushed && let Some(frame) = self.frames.pop() {
            for event in frame.events {
                if !frame.escaped && !frame.used.contains(event.name.as_str()) {
                    report(&event.name, event.span, self.offset, self.result);
                }
            }
        }
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if self.dispatches(&call.callee)
            && let Some(Argument::StringLiteral(event)) = call.arguments.first()
            && let Some(frame) = self.frames.last_mut()
        {
            frame.used.insert(CompactString::new(event.value.as_str()));
        }
        let escaped = self.frames.last().is_some_and(|frame| {
            [frame.emit, frame.context]
                .into_iter()
                .flatten()
                .any(|symbol| {
                    call.arguments
                        .iter()
                        .any(|argument| self.argument_references_symbol(argument, symbol))
                })
        });
        if escaped && let Some(frame) = self.frames.last_mut() {
            frame.escaped = true;
        }
        walk_call_expression(self, call);
    }
}

impl<'a> OptionsUnusedCollector<'_, '_, 'a> {
    fn dispatches(&self, callee: &Expression<'a>) -> bool {
        let Some(frame) = self.frames.last() else {
            return false;
        };
        match callee.without_parentheses() {
            Expression::Identifier(identifier) => {
                frame.emit.is_some() && self.symbol(identifier) == frame.emit
            }
            expression => {
                let Some(member) = expression.as_member_expression() else {
                    return false;
                };
                match member.static_property_name() {
                    Some("emit") => {
                        frame.context.is_some()
                            && matches!(
                                member.object().without_parentheses(),
                                Expression::Identifier(identifier)
                                    if self.symbol(identifier) == frame.context
                            )
                    }
                    Some("$emit") => matches!(
                        member.object().without_parentheses(),
                        Expression::ThisExpression(_)
                    ),
                    _ => false,
                }
            }
        }
    }

    fn symbol(&self, identifier: &IdentifierReference<'a>) -> Option<SymbolId> {
        self.semantic
            .scoping()
            .get_reference(identifier.reference_id.get()?)
            .symbol_id()
    }

    fn argument_references_symbol(&self, argument: &Argument<'a>, symbol: SymbolId) -> bool {
        match argument {
            Argument::ObjectExpression(object) => object.properties.iter().any(|property| {
                matches!(property, ObjectPropertyKind::ObjectProperty(property)
                    if self.expression_is_symbol(&property.value, symbol))
            }),
            Argument::ArrayExpression(array) => array.elements.iter().any(|element| {
                element
                    .as_expression()
                    .is_some_and(|expression| self.expression_is_symbol(expression, symbol))
            }),
            _ => argument
                .as_expression()
                .is_some_and(|expression| self.expression_is_symbol(expression, symbol)),
        }
    }

    fn expression_is_symbol(&self, expression: &Expression<'a>, symbol: SymbolId) -> bool {
        match expression.without_parentheses() {
            Expression::Identifier(identifier) => self.symbol(identifier) == Some(symbol),
            expression => expression.as_member_expression().is_some_and(|member| {
                matches!(member.object().without_parentheses(), Expression::Identifier(identifier)
                    if self.symbol(identifier) == Some(symbol))
            }),
        }
    }
}

fn collect_options_declared_events(object: &ObjectExpression<'_>) -> Option<Vec<DeclaredEvent>> {
    let value = object.properties.iter().find_map(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return None;
        };
        (!property.computed && property_key_name(&property.key) == Some("emits"))
            .then_some(&property.value)
    })?;
    let mut events = Vec::new();
    match value.without_parentheses() {
        Expression::ArrayExpression(array) => {
            for element in &array.elements {
                let ArrayExpressionElement::StringLiteral(literal) = element else {
                    // Keep independently known literal declarations even when
                    // the array also contains a dynamic event name. The
                    // unknown entry cannot make a different literal event used.
                    continue;
                };
                events.push(DeclaredEvent {
                    name: CompactString::new(literal.value.as_str()),
                    span: literal.span,
                    origin_offset: None,
                });
            }
        }
        Expression::ObjectExpression(object) => {
            for property in &object.properties {
                let ObjectPropertyKind::ObjectProperty(property) = property else {
                    continue;
                };
                if property.computed {
                    continue;
                }
                let Some(name) = property_key_name(&property.key) else {
                    continue;
                };
                events.push(DeclaredEvent {
                    name: CompactString::new(name),
                    span: property.key.span(),
                    origin_offset: None,
                });
            }
        }
        _ => return None,
    }
    Some(events)
}
