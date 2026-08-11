use vize_fresco::{Backend, Event, Key, KeyEvent, KeyModifiers, terminal::TerminalOptions};

use super::{capabilities, report, sources};
use crate::commands::doctor::tui::{model::DoctorTuiModel, run_loop};

const TEST_TERMINAL_OPTIONS: TerminalOptions = TerminalOptions {
    raw_mode: false,
    alternate_screen: true,
    mouse_capture: false,
    bracketed_paste: true,
    hide_cursor: true,
};

#[test]
fn every_interactive_exit_renders_then_restores_owned_terminal_modes() {
    let exit_events = [
        Event::Key(KeyEvent::char('q')),
        Event::Key(KeyEvent::key(Key::Esc)),
        Event::Key(KeyEvent::new(
            Key::Char('c'),
            KeyModifiers {
                ctrl: true,
                ..KeyModifiers::NONE
            },
        )),
    ];

    for exit_event in exit_events {
        let report = report();
        let sources = sources();
        let mut capabilities = capabilities(100, 20, true);
        let mut backend = Backend::with_writer(100, 20, Vec::new());
        backend.init_with_options(TEST_TERMINAL_OPTIONS).unwrap();
        backend.clear().unwrap();
        backend.cursor_mut().hide();
        let mut model = DoctorTuiModel::new(&report, backend.width(), backend.height());
        let mut reads = 0;

        run_loop(
            &mut backend,
            &mut model,
            &sources,
            std::path::Path::new("."),
            &mut capabilities,
            || {
                reads += 1;
                if reads == 1 {
                    Ok(exit_event.clone())
                } else {
                    panic!("exit must not request another input event")
                }
            },
        )
        .unwrap();
        backend.restore().unwrap();

        assert_eq!(reads, 1);
        let output = backend.writer();
        assert!(contains_bytes(output, b"\x1b[?1049h"));
        assert!(contains_bytes(output, b"\x1b[?2004h"));
        assert!(contains_bytes(output, b"\x1b[?25l"));
        assert!(output.ends_with(b"\x1b[?2004l\x1b[?1049l\x1b[?25h"));
    }
}

fn contains_bytes(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}
