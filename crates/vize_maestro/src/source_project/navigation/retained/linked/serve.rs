//! Cancellation is checked before and during original lexical query traversal.
use super::super::super::worker::linked::Request;
use super::{Command, Control, NavigationRefusal, query::TemplateNames};
use std::sync::mpsc::Receiver;

pub(super) fn run(query: &TemplateNames<'_, '_>, receiver: Receiver<Command>, control: &Control) {
    while !control.retired() {
        let Ok(command) = receiver.recv() else {
            break;
        };
        if control.retired() {
            break;
        }
        match command {
            Command::LinkedEditing(Request::Ranges(position, reply)) => {
                if !reply.is_canceled() {
                    #[cfg(test)]
                    control
                        .queries
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let result =
                        query.ranges(position, || control.retired() || reply.is_canceled());
                    let _ = reply.send(result);
                }
            }
            #[cfg(test)]
            Command::LinkedEditing(Request::Inspect(reply)) => {
                let _ = reply.send(Ok(query.original.clone()));
            }
            Command::Highlights(request) => {
                let _ = request.reply.send(Err(NavigationRefusal::Language));
            }
            Command::TemplateDefinition(_, reply) | Command::References(_, _, reply) => {
                let _ = reply.send(Err(NavigationRefusal::Language));
            }
            Command::Definition(_, reply) => {
                let _ = reply.send(Err(NavigationRefusal::Language));
            }
            Command::Stop => break,
            #[cfg(test)]
            Command::SelectedInspect(_, reply) => {
                let _ = reply.send(Err(NavigationRefusal::Language));
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
            Command::Panic => panic!("actual native names worker unwind"),
        }
    }
}
