//! Exact whole-buffer witnesses across indentation chunk boundaries.

#[cfg(test)]
mod indentation_trace {
    use super::super::Buf;
    use vize_l0::{String, cstr};

    #[test]
    fn newline_keeps_every_byte_at_chunk_boundaries() {
        for depth in [0, 1, 15, 16, 17, 60, 64] {
            let mut buf = Buf::new(false);
            buf.indent = depth;
            buf.push("前置き");
            buf.newline();
            buf.push("後置き");
            let expected = cstr!("前置き\n{}後置き", "  ".repeat(depth as usize));
            assert_eq!(buf.code, expected, "depth {depth}");
            assert_eq!(buf.indent, depth);
        }
    }

    #[test]
    fn indentation_transitions_preserve_whole_buffer_and_helper_state() {
        let mut buf = Buf::new(true);
        buf.use_to_display_string();
        let helpers = buf.used;
        let mut expected = String::from("start");
        buf.push("start");
        for depth in [0, 1, 15, 16, 17, 60, 64] {
            while buf.indent < depth {
                buf.indent();
            }
            while buf.indent > depth {
                buf.deindent();
            }
            buf.newline();
            buf.push("end");
            expected.push_str(cstr!("\n{}end", "  ".repeat(depth as usize)).as_str());
        }
        for _ in 0..65 {
            buf.deindent();
        }
        buf.newline();
        expected.push('\n');
        assert_eq!(buf.code, expected);
        assert_eq!((buf.track_visits, buf.indent, buf.used), (true, 0, helpers));
        let [helper] = buf.used_order.as_slice() else {
            panic!("indentation must keep the one registered helper");
        };
        assert_eq!(helper.alias(), "_toDisplayString");
    }
}
