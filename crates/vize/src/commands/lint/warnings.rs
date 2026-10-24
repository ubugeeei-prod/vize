pub(super) fn exit_if_over_max(total_warnings: usize, max: Option<usize>) {
    if let Some(max) = max
        && total_warnings > max
    {
        eprintln!("\nToo many warnings ({} > max {})", total_warnings, max);
        std::process::exit(1);
    }
}
