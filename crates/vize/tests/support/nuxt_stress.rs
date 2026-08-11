pub(crate) fn required_iterations() -> usize {
    let raw = std::env::var_os("VIZE_NUXT_CONFIG_ITERATIONS");
    if std::env::var_os("CI").is_some() {
        assert!(
            raw.is_some(),
            "CI must set VIZE_NUXT_CONFIG_ITERATIONS for the Nuxt config oracles"
        );
    }
    let iterations = raw
        .map(|raw| {
            let raw = raw.to_string_lossy();
            raw.parse::<usize>()
                .unwrap_or_else(|_| panic!("VIZE_NUXT_CONFIG_ITERATIONS must be an integer: {raw}"))
        })
        .unwrap_or(1);
    assert!(
        iterations > 0,
        "Nuxt config isolation iterations must be positive"
    );
    if std::env::var_os("CI").is_some() {
        assert!(
            iterations >= 100,
            "CI must run at least 100 Nuxt config oracle iterations"
        );
    }
    iterations
}
