use super::{collect_plugin_injection_stubs, render_nuxt_injection_context_stub};
use vize_carton::FxHashSet;

#[test]
fn scans_src_app_plugins_for_nuxt2_injections() {
    let project_root =
        std::env::temp_dir().join(format!("vize-nuxt-src-app-plugins-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&project_root);
    let plugin_dir = project_root.join("src/app/plugins");
    std::fs::create_dir_all(&plugin_dir).unwrap();
    std::fs::write(
        plugin_dir.join("logger.ts"),
        r#"export default (_context, inject) => {
  inject("logger", {
    info(message) {
      return message.length;
    },
  });
};
"#,
    )
    .unwrap();

    let mut stubs = Vec::new();
    let mut seen_names = FxHashSet::default();
    collect_plugin_injection_stubs(&project_root, &mut stubs, &mut seen_names);

    assert!(
        stubs
            .iter()
            .any(|stub| stub.contains("$logger: __VizeNuxtInjection<'$logger'>;")),
        "expected UseContextReturn injection augmentation from src/app/plugins:\n{stubs:#?}"
    );
    assert!(
        stubs
            .iter()
            .any(|stub| stub.contains("declare const $logger")),
        "expected global injection stub from src/app/plugins:\n{stubs:#?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn renders_use_context_injection_augmentations() {
    let stub = render_nuxt_injection_context_stub(&["logger".into()]);

    assert!(stub.contains("$logger: __VizeNuxtInjection<'$logger'>;"));
    assert!(stub.contains("interface Context extends __VizeNuxtInjectedProperties"));
    assert!(stub.contains("interface UseContextReturn extends __VizeNuxtInjectedProperties"));
}
