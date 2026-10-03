//! Owned requests are answered while the actual original selected File lives.

use super::{Command, Control, NavigationRefusal, SelectedNavigation};
use std::sync::mpsc::Receiver;

pub(super) fn run(
    query: &SelectedNavigation<'_, '_>,
    receiver: Receiver<Command>,
    control: &Control,
) {
    while !control.retired() {
        let Ok(command) = receiver.recv() else { break };
        if control.retired() {
            break;
        }
        match command {
            Command::TemplateDefinition(position, reply) => {
                if !reply.is_canceled() {
                    #[cfg(test)]
                    control
                        .queries
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let _ = reply.send(query.definitions(position));
                }
            }
            Command::References(position, include, reply) => {
                if !reply.is_canceled() {
                    #[cfg(test)]
                    control
                        .queries
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let _ = reply.send(query.references(position, include));
                }
            }
            Command::Definition(_, reply) => {
                if !reply.is_canceled() {
                    let _ = reply.send(Err(NavigationRefusal::Language));
                }
            }
            Command::Stop => break,
            #[cfg(test)]
            Command::SelectedInspect(position, reply) => {
                if !reply.is_canceled() {
                    let result = query.symbol(position).map(|symbol| {
                        let mut original = query.original.clone();
                        original.handler_body = symbol.and_then(|symbol| match symbol {
                            vize_l2::file::TemplateSymbolRef::HandlerLocal(local) => {
                                local.handler().resolution().map(|resolution| {
                                    core::ptr::from_ref(resolution.input().body()) as usize
                                })
                            }
                            vize_l2::file::TemplateSymbolRef::File(_) => None,
                        });
                        original
                    });
                    let _ = reply.send(result);
                }
            }
            #[cfg(test)]
            Command::Inspect(_, reply) => {
                let _ = reply.send(Err(NavigationRefusal::Language));
            }
            #[cfg(test)]
            Command::Pause(entered, resumed) => {
                let _ = entered.send(());
                let _ = resumed.recv();
            }
            #[cfg(test)]
            Command::Panic => panic!("actual selected SFC worker unwind"),
        }
    }
}
