//! Deep enum-flow relations used by `no-unsafe-enum-assignment`.
//!
//! The lint listener's compact type facts settle direct enum assignments.
//! Structural members, contextual types, resolved signatures and computed
//! enum keys need paired checker types, so they are resolved here in the same
//! expression batch as the other relation rules.
mod computed;
mod deep;
mod types;

pub(in crate::linter) use types::{
    EnumCallArgument, EnumCallAsk, EnumCheckAsk, EnumMismatch, EnumReceiver, HeritageSource,
};

use super::checker::{CALL, CONSTRUCT, Checker};
use super::{Located, Span};
use computed::computed_receivers;
use corsa::api::{SignatureResponse, TypeResponse};
use deep::{enum_names, mismatch, number_index, reference_arguments};

pub(super) async fn check(
    checker: &mut Checker<'_>,
    located: &Located,
    ask: &EnumCheckAsk,
    translate: &dyn Fn(Span) -> Span,
) -> Option<EnumMismatch> {
    match ask {
        EnumCheckAsk::Pair {
            sender,
            receiver,
            safe_operands,
        } => {
            let sender = located.ty(translate(*sender))?.clone();
            let receiver = match receiver {
                EnumReceiver::TypeAt(span) => located.ty(translate(*span)).cloned(),
                EnumReceiver::Contextual(span) => {
                    let handle = located.handle(translate(*span))?;
                    checker.contextual(handle).await
                }
            }?;
            let flow = Flow {
                located,
                translate,
                safe_operands,
            };
            flow.unsafe_to(checker, &sender, &receiver)
                .await
                .then_some(())?;
            result(checker, &receiver).await
        }
        EnumCheckAsk::Return {
            function,
            returned,
            receiver,
            is_async,
            safe_operands,
        } => {
            let mut sender = located.ty(translate(*returned))?.clone();
            let explicit = receiver
                .and_then(|receiver| located.ty(translate(receiver)))
                .cloned();
            let contextual = match located.handle(translate(*returned)) {
                Some(handle) => checker.contextual(handle).await,
                None => None,
            };
            let from_context = contextual.is_some();
            let mut receiver = match contextual.or(explicit) {
                Some(receiver) => receiver,
                None => {
                    let own = located.ty(translate(*function))?.clone();
                    let signature = checker.signatures(&own, CALL).await.into_iter().next()?;
                    checker.return_type(&signature).await?
                }
            };
            if !from_context && *is_async {
                receiver = awaited(checker, &receiver).await;
                sender = awaited(checker, &sender).await;
            }
            let flow = Flow {
                located,
                translate,
                safe_operands,
            };
            flow.unsafe_to(checker, &sender, &receiver)
                .await
                .then_some(())?;
            result(checker, &receiver).await
        }
        EnumCheckAsk::Heritage {
            sender,
            member,
            safe_operands,
            sources,
        } => {
            let sender = located.ty(translate(*sender))?.clone();
            let mut receivers = Vec::new();
            for source in sources {
                let span = translate(source.span);
                let ty = if source.base_types {
                    located.ty(span)
                } else {
                    located.type_node(span)
                };
                let Some(ty) = ty.cloned() else {
                    continue;
                };
                let types = if source.base_types {
                    let constructors = checker.signatures(&ty, CONSTRUCT).await;
                    let instance = match constructors.first() {
                        Some(signature) => checker
                            .return_type(signature)
                            .await
                            .unwrap_or_else(|| ty.clone()),
                        None => ty.clone(),
                    };
                    checker
                        .session()
                        .client()
                        .get_base_types(
                            checker.session().snapshot().handle.clone(),
                            checker.session().project_handle(),
                            instance.id,
                        )
                        .await
                        .unwrap_or_default()
                } else {
                    vec![ty]
                };
                for ty in types {
                    let Some(property) = checker.property(&ty, member.as_str()).await else {
                        continue;
                    };
                    if let Some(ty) = checker.type_of_symbol(&property).await {
                        receivers.push(ty);
                    }
                }
            }
            if receivers.is_empty() {
                return None;
            }
            let flow = Flow {
                located,
                translate,
                safe_operands,
            };
            for receiver in &receivers {
                if !flow.unsafe_to(checker, &sender, receiver).await {
                    return None;
                }
            }
            results(checker, &receivers).await
        }
        EnumCheckAsk::Computed { object, property } => {
            let object = located.ty(translate(*object))?.clone();
            let sender = located.ty(translate(*property))?.clone();
            computed_receivers(checker, &object, &sender).await
        }
    }
}

pub(super) async fn call(
    checker: &mut Checker<'_>,
    located: &Located,
    ask: &EnumCallAsk,
    translate: &dyn Fn(Span) -> Span,
) -> Vec<Option<EnumMismatch>> {
    let mut replies = vec![None; ask.arguments.len()];
    let Some(handle) = located.handle(translate(ask.call)).cloned() else {
        return replies;
    };
    let signature = checker.resolved_signature(&handle).await;
    let Some(signature) = signature else {
        return replies;
    };
    let mut position = usize::from(ask.tagged_template);
    let mut remaining = false;
    for (index, argument) in ask.arguments.iter().enumerate() {
        if argument.literal {
            position = position.saturating_add(1);
            continue;
        }
        if !argument.spread {
            let Some(sender) = located.ty(translate(argument.expression)).cloned() else {
                position = position.saturating_add(1);
                continue;
            };
            let receiver = argument_receiver(
                checker, located, argument, translate, &signature, position, remaining,
            )
            .await;
            let flow = Flow {
                located,
                translate,
                safe_operands: &argument.safe_operands,
            };
            if let Some(receiver) = receiver
                && flow.unsafe_to(checker, &sender, &receiver).await
            {
                let answer = result(checker, &receiver).await;
                if let Some(reply) = replies.get_mut(index) {
                    *reply = answer;
                }
            }
            position = position.saturating_add(1);
            continue;
        }
        let Some(spread) = located.ty(translate(argument.expression)).cloned() else {
            continue;
        };
        let (tuple, target) = checker.tuple(&spread).await;
        if tuple {
            let elements = reference_arguments(checker, &spread).await;
            let mut receivers = Vec::new();
            for element in &elements {
                let receiver = parameter_type(checker, &signature, position, remaining).await;
                if let Some(receiver) = receiver
                    && mismatch(checker, element, &receiver).await
                {
                    receivers.push(receiver);
                }
                position = position.saturating_add(1);
            }
            if !receivers.is_empty() {
                let answer = results(checker, &receivers).await;
                if let Some(reply) = replies.get_mut(index) {
                    *reply = answer;
                }
            }
            // `spreadType.target.combinedFlags & ElementFlags.Variable`: a
            // rest or variadic element takes every remaining argument.
            if target
                .as_ref()
                .map_or(&spread.element_flags, |target| &target.element_flags)
                .iter()
                .any(|flag| flag & ((1 << 2) | (1 << 3)) != 0)
            {
                remaining = true;
            }
            continue;
        }
        let sender = spread_value(checker, located, argument, translate)
            .await
            .unwrap_or(spread);
        let receiver = argument_receiver(
            checker, located, argument, translate, &signature, position, remaining,
        )
        .await;
        if let Some(receiver) = receiver
            && mismatch(checker, &sender, &receiver).await
        {
            let answer = result(checker, &receiver).await;
            if let Some(reply) = replies.get_mut(index) {
                *reply = answer;
            }
        }
        position = position.saturating_add(1);
    }
    replies
}

async fn parameter_type(
    checker: &mut Checker<'_>,
    signature: &SignatureResponse,
    position: usize,
    remaining: bool,
) -> Option<TypeResponse> {
    if remaining && let Some(receiver) = variadic_receiver(checker, signature).await {
        return Some(receiver);
    }
    checker
        .parameter_type(&signature.id, i32::try_from(position).unwrap_or(i32::MAX))
        .await
}

async fn variadic_receiver(
    checker: &mut Checker<'_>,
    signature: &SignatureResponse,
) -> Option<TypeResponse> {
    if signature.flags & 1 == 0 {
        return None;
    }
    let index = signature.parameters.len().checked_sub(1)?;
    checker
        .parameter_type(&signature.id, i32::try_from(index).ok()?)
        .await
}

async fn argument_receiver(
    checker: &mut Checker<'_>,
    located: &Located,
    argument: &super::EnumCallArgument,
    translate: &dyn Fn(Span) -> Span,
    signature: &SignatureResponse,
    position: usize,
    remaining: bool,
) -> Option<TypeResponse> {
    if let Some(parameter) = parameter_type(checker, signature, position, remaining).await {
        return Some(parameter);
    }
    let handle = located.handle(translate(argument.node))?;
    checker.contextual(handle).await
}

async fn spread_value(
    checker: &mut Checker<'_>,
    located: &Located,
    argument: &super::EnumCallArgument,
    translate: &dyn Fn(Span) -> Span,
) -> Option<TypeResponse> {
    if let Some(node) = located.ty(translate(argument.node)).cloned()
        && located
            .ty(translate(argument.expression))
            .is_none_or(|expression| expression.id != node.id)
    {
        return Some(node);
    }
    let expression = located.ty(translate(argument.expression))?.clone();
    let arguments = reference_arguments(checker, &expression).await;
    if arguments.len() == 1 {
        return arguments.into_iter().next();
    }
    number_index(checker, &expression).await
}

async fn awaited(checker: &mut Checker<'_>, ty: &TypeResponse) -> TypeResponse {
    reference_arguments(checker, ty)
        .await
        .into_iter()
        .next()
        .unwrap_or_else(|| ty.clone())
}

/// One sender expression and the operands that make it a bit flag.
struct Flow<'a> {
    located: &'a Located,
    translate: &'a dyn Fn(Span) -> Span,
    /// Primitive leaves of a bitwise expression (empty for any other).
    safe_operands: &'a [Span],
}

impl Flow<'_> {
    /// typescript-eslint `isUnsafeAssignment`: an enum mismatch that is not a
    /// bit flag of the receiver's own members (`Flags.A | Flags.B`), in every
    /// position the rule checks.
    async fn unsafe_to(
        &self,
        checker: &mut Checker<'_>,
        sender: &TypeResponse,
        receiver: &TypeResponse,
    ) -> bool {
        if !mismatch(checker, sender, receiver).await {
            return false;
        }
        if self.safe_operands.is_empty() {
            return true;
        }
        for operand in self.safe_operands {
            let Some(operand) = self.located.ty((self.translate)(*operand)).cloned() else {
                return true;
            };
            if mismatch(checker, &operand, receiver).await {
                return true;
            }
        }
        false
    }
}

async fn result(checker: &mut Checker<'_>, receiver: &TypeResponse) -> Option<EnumMismatch> {
    results(checker, std::slice::from_ref(receiver)).await
}

pub(super) async fn results(
    checker: &mut Checker<'_>,
    receivers: &[TypeResponse],
) -> Option<EnumMismatch> {
    let mut names = Vec::new();
    for receiver in receivers {
        names.extend(enum_names(checker, receiver).await);
    }
    names.sort_unstable();
    names.dedup();
    (!names.is_empty()).then_some(EnumMismatch { names })
}
