//! Generic message fan-out for compiler reports and tool integrations.
//!
//! `Telegraph` owns a set of emitters and delivers each message, or batch of
//! messages, to every emitter. Domain crates provide the message type and
//! emitter implementations; Carton only provides the routing primitive.

use std::boxed::Box as StdBox;
use std::vec::Vec as StdVec;

/// An output endpoint that can render or deliver messages.
pub trait Emitter<Message>: Send + Sync {
    /// Output produced by this emitter.
    type Output;

    /// Emit one message.
    fn emit(&self, message: &Message) -> Self::Output;

    /// Emit a batch of messages as one output.
    fn emit_all(&self, messages: &[Message]) -> Self::Output;

    /// Stable emitter name for diagnostics, logs, and tests.
    fn name(&self) -> &'static str;
}

impl<Message, Output, T> Emitter<Message> for StdBox<T>
where
    T: Emitter<Message, Output = Output> + ?Sized,
{
    type Output = Output;

    fn emit(&self, message: &Message) -> Self::Output {
        self.as_ref().emit(message)
    }

    fn emit_all(&self, messages: &[Message]) -> Self::Output {
        self.as_ref().emit_all(messages)
    }

    fn name(&self) -> &'static str {
        self.as_ref().name()
    }
}

/// Routes messages to every registered emitter.
pub struct Telegraph<Message, Output> {
    emitters: StdVec<StdBox<dyn Emitter<Message, Output = Output>>>,
}

impl<Message, Output> Telegraph<Message, Output> {
    /// Create a telegraph with no emitters.
    pub fn new() -> Self {
        Self {
            emitters: StdVec::new(),
        }
    }

    /// Add an emitter.
    pub fn add_emitter<E>(&mut self, emitter: E)
    where
        E: Emitter<Message, Output = Output> + 'static,
    {
        self.emitters.push(StdBox::new(emitter));
    }

    /// Add an already boxed emitter.
    pub fn add_boxed_emitter(&mut self, emitter: StdBox<dyn Emitter<Message, Output = Output>>) {
        self.emitters.push(emitter);
    }

    /// Number of registered emitters.
    pub fn len(&self) -> usize {
        self.emitters.len()
    }

    /// Whether no emitters are registered.
    pub fn is_empty(&self) -> bool {
        self.emitters.is_empty()
    }

    /// Transmit one message through all emitters.
    pub fn transmit(&self, message: &Message) -> StdVec<Output> {
        self.emitters
            .iter()
            .map(|emitter| emitter.emit(message))
            .collect()
    }

    /// Transmit multiple messages through all emitters.
    pub fn transmit_all(&self, messages: &[Message]) -> StdVec<Output> {
        self.emitters
            .iter()
            .map(|emitter| emitter.emit_all(messages))
            .collect()
    }
}

impl<Message, Output> Default for Telegraph<Message, Output> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{Emitter, Telegraph};
    use std::string::String;

    #[derive(Debug, Clone)]
    struct Message(&'static str);

    struct PrefixEmitter(&'static str);

    impl Emitter<Message> for PrefixEmitter {
        type Output = String;

        fn emit(&self, message: &Message) -> Self::Output {
            let mut output = String::from(self.0);
            output.push_str(message.0);
            output
        }

        fn emit_all(&self, messages: &[Message]) -> Self::Output {
            let mut output = String::from(self.0);
            for message in messages {
                if !output.ends_with(':') {
                    output.push(',');
                }
                output.push_str(message.0);
            }
            output
        }

        fn name(&self) -> &'static str {
            self.0
        }
    }

    #[test]
    fn transmits_one_message_to_all_emitters() {
        let mut telegraph = Telegraph::new();
        telegraph.add_emitter(PrefixEmitter("a:"));
        telegraph.add_emitter(PrefixEmitter("b:"));

        let outputs = telegraph.transmit(&Message("ping"));

        assert_eq!(outputs, ["a:ping", "b:ping"]);
    }

    #[test]
    fn transmits_batches_to_all_emitters() {
        let mut telegraph = Telegraph::new();
        telegraph.add_emitter(PrefixEmitter("batch:"));

        let outputs = telegraph.transmit_all(&[Message("one"), Message("two")]);

        assert_eq!(outputs, ["batch:one,two"]);
    }
}
