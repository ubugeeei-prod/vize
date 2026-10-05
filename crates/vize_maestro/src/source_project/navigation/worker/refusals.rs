//! Sticky original-worker refusals answer owned queued requests.
use super::{Command, Control, NavigationRefusal, Receiver};

pub(in crate::source_project::navigation) fn refused(
    receiver: Receiver<Command>,
    control: &Control,
    refusal: NavigationRefusal,
) {
    while !control.retired() {
        let Ok(command) = receiver.recv() else { break };
        if control.retired() {
            break;
        }
        match command {
            Command::ModuleLinks(request) => request.refuse(refusal.clone()),
            Command::LinkedEditing(request) => request.refuse(refusal.clone()),
            Command::Highlights(request) => {
                if !request.reply.is_canceled() {
                    let _ = request.reply.send(Err(refusal.clone()));
                }
            }
            Command::TemplateDefinition(_, reply) => {
                if !reply.is_canceled() {
                    let _ = reply.send(Err(refusal.clone()));
                }
            }
            #[cfg(test)]
            Command::SelectedInspect(_, reply) => {
                let _ = reply.send(Err(refusal.clone()));
            }
            Command::Definition(_, reply) => {
                if !reply.is_canceled() {
                    let _ = reply.send(Err(refusal.clone()));
                }
            }
            Command::References(_, _, reply) => {
                if !reply.is_canceled() {
                    let _ = reply.send(Err(refusal.clone()));
                }
            }
            Command::Stop => break,
            #[cfg(test)]
            Command::Inspect(_, reply) => {
                let _ = reply.send(Err(refusal.clone()));
            }
            #[cfg(test)]
            Command::Pause(entered, resumed) => {
                let _ = entered.send(());
                let _ = resumed.recv();
            }
            #[cfg(test)]
            Command::Panic => panic!("actual native worker unwind"),
        }
    }
}
