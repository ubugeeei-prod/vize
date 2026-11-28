use windows_sys::Win32::System::Console::{STD_INPUT_HANDLE, STD_OUTPUT_HANDLE};

use super::{console_mode, set_console_mode, std_handle, write_file};

#[test]
fn standard_output_handle_accepts_an_empty_synchronous_write() {
    let handle = std_handle(STD_OUTPUT_HANDLE).unwrap();
    assert_eq!(write_file(handle, b"").unwrap(), 0);
}

#[test]
fn console_mode_round_trips_or_reports_a_non_console_handle() {
    for selector in [STD_INPUT_HANDLE, STD_OUTPUT_HANDLE] {
        let Ok(handle) = std_handle(selector) else {
            continue;
        };
        match console_mode(handle) {
            Ok(mode) => {
                set_console_mode(handle, mode).unwrap();
                assert_eq!(console_mode(handle).unwrap(), mode);
            }
            // Test runners usually attach pipes, which are not console handles.
            Err(error) => assert!(error.raw_os_error().is_some()),
        }
    }
}
