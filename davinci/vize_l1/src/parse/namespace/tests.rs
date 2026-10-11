//! The deferred owner must retain the complete previous eager event stream.

use vize_l0::{Allocator, Vec};

use super::NamespaceSink;
use crate::event::{Event, Recorder};
use crate::markup::{Component, LexOptions, Lexer};
use crate::parse::SurfaceError;

fn record<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    eager: bool,
) -> (Vec<'a, Event>, Vec<'a, SurfaceError>) {
    let mut events = Vec::new_in(&allocator);
    let mut errors = Vec::new_in(&allocator);
    let recorder = Recorder {
        events: &mut events,
        errors: &mut errors,
    };
    let mut sink = NamespaceSink::new(allocator, source, recorder);
    sink.live = eager;
    Lexer::<Component, _>::new(source, sink, LexOptions::default()).run();
    (events, errors)
}

#[test]
fn deferred_title_scope_keeps_every_eager_event_and_error() {
    let sources = [
        "<div><span>before</span></div><svg><title><span>one</span></title></svg>",
        "<div><svg><title><span>one</span></title></svg><svg><title><b>two</b></title></svg></div>",
        "<svg><TITLE><span>upper</span></TITLE><title><b>later</b></title></svg>",
        "<svg><foreignObject><title><span>html</span></title></foreignObject><title><b>svg</b></title></svg>",
        "<div><svg></div><title><span>html</span></title>",
        "<button><span><button></button></button><svg><title><b>svg</b></title></svg>",
        "<a><a></a></a><svg><title><b>svg</b></title></svg>",
        "<svg><path/><title><span>svg</span></title></svg><title><b>html</b></title>",
        "<svg><title/><title><span>svg</span></title></svg>",
        "<math><mtext><svg><title><b>svg</b></title></svg></mtext></math>",
        "<div :[broken><svg><title><b>svg</b></title></svg></div>",
        "<svg><title",
        "<div><span>no title</span><button>one</button></div>",
    ];
    for source in sources {
        let allocator = Allocator::new();
        let (eager_events, eager_errors) = record(&allocator, source, true);
        let (deferred_events, deferred_errors) = record(&allocator, source, false);
        assert_eq!(
            deferred_events, eager_events,
            "whole event stream: {source}"
        );
        assert!(
            deferred_errors
                .iter()
                .map(|error| (error.code, error.offset))
                .eq(eager_errors.iter().map(|error| (error.code, error.offset))),
            "all exact diagnostics: {source}"
        );
        let (tree, _) = crate::parse(&allocator, source);
        assert_eq!(crate::check_fidelity(&tree), Ok(()), "{source}");
    }
}
