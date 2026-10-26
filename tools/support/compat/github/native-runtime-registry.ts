// Exact reviewed source/selector identities; later edits cannot reuse Issue 10694.
import { nativeRuntimeGuidanceRegistration } from "./native-runtime-guidance.ts";
type SourcePair = { path: string; before: string | null; after: string };
type Selector = { selector: string; source: string; consumesFixture: boolean };

export const nativeRuntimeRegistry: Record<string, { sources: SourcePair[]; tests: Selector[] }> = {
  lsp: nativeRuntimeGuidanceRegistration,
  typechecker: {
    sources: [
      {
        path: "crates/vize_carton/src/corsa_resolver.rs",
        before: "947bf22565f1948e3a2848dd90b7c1f3ec259bf293d6176941528291f0612717",
        after: "43d383c99bf861e08f18f2ce51bae235644c4dc96fb92308691d9b0c2f6ea2d9",
      },
      {
        path: "crates/vize_carton/src/corsa_resolver/tests.rs",
        before: "6c15b056384611da328d4dc4bc01dc6f3cbe665bec18bbabffdb747233aa4ee2",
        after: "3648f1a499135ec10c0668ddadd647fa8a827059f420813b4fe8a5015e409be2",
      },
      {
        path: "crates/vize_carton/src/corsa_resolver/tests/project_packages.rs",
        before: null,
        after: "5e07a6e0b21780b9e1beed2c0c681f8ae73850c928a807e3739d0dcd50acf516",
      },
      {
        path: "crates/vize_carton/src/corsa_resolver/tests/fixture_cases.rs",
        before: null,
        after: "f869b0a0acb0207661e6f6a403b91850f3241cf6e1b14fc6da326b20affdb83a",
      },
      {
        path: "crates/vize_canon/src/lsp_client/bootstrap.rs",
        before: "b0f9fa8f1a3b9eb8df8ea12f5141059cd195a60d393da7e671daf5a9010a4305",
        after: "7d3a41daa0d38b31c025340bf7c676ac10bc359cd45e5e18f9a1a34042a8c775",
      },
      {
        path: "crates/vize_canon/src/batch/error.rs",
        before: "495e53261574cb81f49590911c552f3b6f614d9b700e5fd6c18e4186ad156d14",
        after: "d511e1c749ea68eafc1c67d5612cd22c8144def02d4c4222abc6409670b98f6e",
      },
      {
        path: "crates/vize_canon/src/batch/snapshots/vize_canon__batch__error__tests__corsa_not_found_error_message.snap",
        before: "bef84760ee73db62358bf6b148fd029f37d5fc8dff61e5ac90c5a9ce09b7b0d1",
        after: "6c8c77c5d558f9e75c1e7950ae72bd6445fe810899d08a4c4c67c03a180584b3",
      },
      {
        path: "crates/vize_canon/src/batch/snapshots/vize_canon__batch__error__tests__corsa_not_found_no_pm.snap",
        before: "e40b76d36a5a64ecfcb4e49a6df5b753ff0d83ad013a57f0378b86a2356de5da",
        after: "de87619592c61e08e6406c25bfcdea334befdd33dfc0b62fa266aa0391a07ed2",
      },
      {
        path: "crates/vize_canon/src/batch/executor.rs",
        before: "81d91622c93dfbfec2bf806a7d25d1e901d76e1eb76d7ffee07279d2da9af349",
        after: "dcb7012d85f3c4803eacddd6f6d0315fd35f8c58c7a141f90b3d29aa2238e204",
      },
      {
        path: "crates/vize_canon/src/batch/executor/tests.rs",
        before: "138020f6b6f2aca5cabf28d457b1084b87cc3732b673538e57fcf45bbd7cc1d7",
        after: "00a3365fac379162010a0bf484acc9eb96e4818559c5d93fd2d083f70457f371",
      },
      {
        path: "crates/vize_canon/src/batch/executor/tests/incremental_fallback.rs",
        before: "dc06840ea2fc02f3f25ffb9c9e8b8f17ab639a20b8616643579104df4d9d34a2",
        after: "288475335b06e28f3127e83cdf1a8551ae648cd1ae4ca25ecf16d69e4825fc20",
      },
      {
        path: "crates/vize_carton/src/corsa_resolver/packages.rs",
        before: "d87a4b3ea808b4cf805dec9cc9f4b12aee919751b7294367a48f4ababef32452",
        after: "48bc5d78a036a239b775d8d51dc3f1ffafbd13d7cda90c490c42f2f3d3e5f0cd",
      },
      {
        path: "crates/vize_carton/src/corsa_resolver/normalize_tests.rs",
        before: "1c80643760d4de5765076cf8178b40654c302d52a88e49e73dc0e280f30c7436",
        after: "d51a53f8cd1de730c8a410ca363599d726e95e72dbd57667faea0a11d4965162",
      },
      {
        path: "crates/vize_carton/src/corsa_resolver/tests/package_runtime.rs",
        before: "1d6ec97de0e64a084bc4f95e0df4fc81cf4163fb893c1e6b214771bea48815ef",
        after: "86516dcc883e5743039b235bf845d13b575dfa9c66e0bfab9169b2fe6bfdba33",
      },
      {
        path: "crates/vize_carton/src/corsa_resolver/tests/provider_authority.rs",
        before: null,
        after: "729126fff584c3eeadc07779eed09ae5edf78e8d8548de18fc6da80381d0f05c",
      },
      {
        path: "crates/vize_canon/src/lsp_client/tests.rs",
        before: "6cdcce2ad44616ab339a1a8c3cbf2e0484af5dddc68486ef0bebbc79bdcbb280",
        after: "55bd45968d7b016ceadf328d0bc2b3650a26eb987dcbc71cad56e3e413bdde2e",
      },
    ],
    tests: [
      {
        selector:
          "corsa_resolver::tests::fixture_cases::runtime_selection_fixture_covers_declared_graphs_and_overrides",
        source: "crates/vize_carton/src/corsa_resolver/tests/fixture_cases.rs",
        consumesFixture: true,
      },
      {
        selector:
          "corsa_resolver::tests::project_packages::default_discovery_never_uses_ambient_runtime_or_package_manager",
        source: "crates/vize_carton/src/corsa_resolver/tests/project_packages.rs",
        consumesFixture: false,
      },
      {
        selector:
          "lsp_client::bootstrap::tests::runtime_selection_fixture_preserves_explicit_path_authority",
        source: "crates/vize_canon/src/lsp_client/bootstrap.rs",
        consumesFixture: true,
      },
      {
        selector:
          "batch::error::tests::explicit_missing_runtime_reports_the_full_path_without_install_advice",
        source: "crates/vize_canon/src/batch/error.rs",
        consumesFixture: false,
      },
      {
        selector:
          "batch::error::tests::every_package_manager_keeps_the_existing_typescript_dependency",
        source: "crates/vize_canon/src/batch/error.rs",
        consumesFixture: false,
      },
      {
        selector: "batch::executor::tests::cli_global_diagnostics_do_not_trigger_session_fallback",
        source: "crates/vize_canon/src/batch/executor/tests.rs",
        consumesFixture: false,
      },
      {
        selector:
          "batch::executor::tests::incremental_fallback::incremental_session_fallback_is_counted_per_check",
        source: "crates/vize_canon/src/batch/executor/tests/incremental_fallback.rs",
        consumesFixture: false,
      },
      {
        selector:
          "corsa_resolver::tests::provider_authority::explicit_alias_launcher_owns_config_and_all_environment_overrides",
        source: "crates/vize_carton/src/corsa_resolver/tests/provider_authority.rs",
        consumesFixture: false,
      },
      {
        selector:
          "corsa_resolver::tests::provider_authority::opaque_explicit_wrapper_is_not_replaced_by_another_provider",
        source: "crates/vize_carton/src/corsa_resolver/tests/provider_authority.rs",
        consumesFixture: false,
      },
      {
        selector:
          "corsa_resolver::tests::provider_authority::undeclared_canonical_package_does_not_supply_a_runtime",
        source: "crates/vize_carton/src/corsa_resolver/tests/provider_authority.rs",
        consumesFixture: false,
      },
      {
        selector:
          "corsa_resolver::tests::provider_authority::declared_alias_wins_over_an_undeclared_canonical_package",
        source: "crates/vize_carton/src/corsa_resolver/tests/provider_authority.rs",
        consumesFixture: false,
      },
      {
        selector:
          "corsa_resolver::tests::provider_authority::declared_canonical_and_alias_follow_their_nearest_hoisted_slot",
        source: "crates/vize_carton/src/corsa_resolver/tests/provider_authority.rs",
        consumesFixture: false,
      },
      {
        selector:
          "corsa_resolver::tests::provider_authority::declared_platform_follows_its_hoisted_slot",
        source: "crates/vize_carton/src/corsa_resolver/tests/provider_authority.rs",
        consumesFixture: false,
      },
      {
        selector:
          "corsa_resolver::tests::provider_authority::invalid_nearest_alias_is_not_rescued_by_an_ancestor_runtime",
        source: "crates/vize_carton/src/corsa_resolver/tests/provider_authority.rs",
        consumesFixture: false,
      },
      {
        selector:
          "corsa_resolver::tests::provider_authority::mismatched_nearest_platform_is_not_rescued_by_an_ancestor_runtime",
        source: "crates/vize_carton/src/corsa_resolver/tests/provider_authority.rs",
        consumesFixture: false,
      },
      {
        selector:
          "corsa_resolver::tests::provider_authority::every_implicit_cache_requires_the_developer_flag",
        source: "crates/vize_carton/src/corsa_resolver/tests/provider_authority.rs",
        consumesFixture: false,
      },
      {
        selector:
          "batch::executor::tests::keeps_explicit_opaque_wrapper_instead_of_unrelated_native_preview",
        source: "crates/vize_canon/src/batch/executor/tests.rs",
        consumesFixture: false,
      },
      {
        selector:
          "lsp_client::tests::keeps_explicit_opaque_wrapper_instead_of_unrelated_native_binary",
        source: "crates/vize_canon/src/lsp_client/tests.rs",
        consumesFixture: false,
      },
      {
        selector:
          "batch::error::tests::invalid_declared_runtime_names_its_package_without_install_advice",
        source: "crates/vize_canon/src/batch/error.rs",
        consumesFixture: false,
      },
      {
        selector: "batch::executor::tests::checks_with_cli_when_project_session_api_is_unavailable",
        source: "crates/vize_canon/src/batch/executor/tests.rs",
        consumesFixture: false,
      },
      {
        selector:
          "corsa_resolver::tests::project_packages::a_broken_declared_runtime_is_reported_separately_from_a_missing_one",
        source: "crates/vize_carton/src/corsa_resolver/tests/project_packages.rs",
        consumesFixture: false,
      },
      {
        selector:
          "corsa_resolver::tests::provider_authority::explicit_shim_follows_the_one_package_declaring_its_bin",
        source: "crates/vize_carton/src/corsa_resolver/tests/provider_authority.rs",
        consumesFixture: false,
      },
    ],
  },
  linter: {
    sources: [
      {
        path: "crates/vize_carton/src/corsa_resolver.rs",
        before: "947bf22565f1948e3a2848dd90b7c1f3ec259bf293d6176941528291f0612717",
        after: "43d383c99bf861e08f18f2ce51bae235644c4dc96fb92308691d9b0c2f6ea2d9",
      },
      {
        path: "crates/vize_carton/src/corsa_resolver/tests.rs",
        before: "6c15b056384611da328d4dc4bc01dc6f3cbe665bec18bbabffdb747233aa4ee2",
        after: "3648f1a499135ec10c0668ddadd647fa8a827059f420813b4fe8a5015e409be2",
      },
      {
        path: "crates/vize_carton/src/corsa_resolver/tests/project_packages.rs",
        before: null,
        after: "5e07a6e0b21780b9e1beed2c0c681f8ae73850c928a807e3739d0dcd50acf516",
      },
      {
        path: "crates/vize_carton/src/corsa_resolver/tests/fixture_cases.rs",
        before: null,
        after: "f869b0a0acb0207661e6f6a403b91850f3241cf6e1b14fc6da326b20affdb83a",
      },
      {
        path: "crates/vize_patina/src/linter/corsa_session/paths.rs",
        before: "520d83c671bf4965de96d35272e97e9b10fe4d98f2e4a96ae389a5934da17e2e",
        after: "34ad9e4bd4eaa321570e6959e5d708a2dbe4323615164a79d4a8c9bfd798f3b4",
      },
      {
        path: "crates/vize_patina/src/linter/corsa_session/paths/store.rs",
        before: "f6b53e7c54bb702a1ba69dc8e906c7f414c1bdcde68c20fe01964f38d1897fd6",
        after: "d8d091f2a510be377eb21c44cade1095367c5cbaaab1d6bb0b2c5c5508f5b841",
      },
      {
        path: "crates/vize_patina/src/linter/corsa_session/paths/store/runtime_tests.rs",
        before: null,
        after: "17c1842e901640ebdd3ccdad691acb1d73e139b27fe8e456105d471a48537967",
      },
      {
        path: "crates/vize_patina/src/linter/corsa_session/session.rs",
        before: "bf2aff6b0ed2aa044a39c48cb3280903d76e173da2af931f9416a454edbaeb24",
        after: "bf7e5173eb60de6a881dfb6fc2f549c1bcd153c1b3c7488d6f7c16eebfbcca5f",
      },
      {
        path: "crates/vize_patina/src/linter/native_type_aware.rs",
        before: "f1b118304bdcc17341379bc685f0c9e805972e519524aead474fa6e7d28eb7a7",
        after: "71cf576e7de8c3419f15af8ed6ed395fee6ddde77114f0e42068c5af130e64f5",
      },
      {
        path: "crates/vize_patina/src/linter/native_type_aware/plain_module_tests/packets.rs",
        before: "34e3d06a5cae777bc1315d5d4bbb94c8f44f1fc138ee3d7323a53762fcd5f5e7",
        after: "c72d860c16caa1fae043ddde528155a32f3de1ced1de02ad5a32fcb4b72f8c3f",
      },
      {
        path: "crates/vize_carton/src/corsa_resolver/packages.rs",
        before: "d87a4b3ea808b4cf805dec9cc9f4b12aee919751b7294367a48f4ababef32452",
        after: "48bc5d78a036a239b775d8d51dc3f1ffafbd13d7cda90c490c42f2f3d3e5f0cd",
      },
      {
        path: "crates/vize_carton/src/corsa_resolver/normalize_tests.rs",
        before: "1c80643760d4de5765076cf8178b40654c302d52a88e49e73dc0e280f30c7436",
        after: "d51a53f8cd1de730c8a410ca363599d726e95e72dbd57667faea0a11d4965162",
      },
      {
        path: "crates/vize_carton/src/corsa_resolver/tests/package_runtime.rs",
        before: "1d6ec97de0e64a084bc4f95e0df4fc81cf4163fb893c1e6b214771bea48815ef",
        after: "86516dcc883e5743039b235bf845d13b575dfa9c66e0bfab9169b2fe6bfdba33",
      },
      {
        path: "crates/vize_carton/src/corsa_resolver/tests/provider_authority.rs",
        before: null,
        after: "729126fff584c3eeadc07779eed09ae5edf78e8d8548de18fc6da80381d0f05c",
      },
    ],
    tests: [
      {
        selector:
          "corsa_resolver::tests::fixture_cases::runtime_selection_fixture_covers_declared_graphs_and_overrides",
        source: "crates/vize_carton/src/corsa_resolver/tests/fixture_cases.rs",
        consumesFixture: true,
      },
      {
        selector:
          "corsa_resolver::tests::project_packages::default_discovery_never_uses_ambient_runtime_or_package_manager",
        source: "crates/vize_carton/src/corsa_resolver/tests/project_packages.rs",
        consumesFixture: false,
      },
      {
        selector:
          "linter::corsa_session::paths::store::runtime_tests::runtime_selection_fixture_uses_selected_config_roots",
        source: "crates/vize_patina/src/linter/corsa_session/paths/store/runtime_tests.rs",
        consumesFixture: true,
      },
      {
        selector:
          "corsa_resolver::tests::provider_authority::explicit_alias_launcher_owns_config_and_all_environment_overrides",
        source: "crates/vize_carton/src/corsa_resolver/tests/provider_authority.rs",
        consumesFixture: false,
      },
      {
        selector:
          "corsa_resolver::tests::provider_authority::opaque_explicit_wrapper_is_not_replaced_by_another_provider",
        source: "crates/vize_carton/src/corsa_resolver/tests/provider_authority.rs",
        consumesFixture: false,
      },
      {
        selector:
          "corsa_resolver::tests::provider_authority::undeclared_canonical_package_does_not_supply_a_runtime",
        source: "crates/vize_carton/src/corsa_resolver/tests/provider_authority.rs",
        consumesFixture: false,
      },
      {
        selector:
          "corsa_resolver::tests::provider_authority::declared_alias_wins_over_an_undeclared_canonical_package",
        source: "crates/vize_carton/src/corsa_resolver/tests/provider_authority.rs",
        consumesFixture: false,
      },
      {
        selector:
          "corsa_resolver::tests::provider_authority::declared_canonical_and_alias_follow_their_nearest_hoisted_slot",
        source: "crates/vize_carton/src/corsa_resolver/tests/provider_authority.rs",
        consumesFixture: false,
      },
      {
        selector:
          "corsa_resolver::tests::provider_authority::declared_platform_follows_its_hoisted_slot",
        source: "crates/vize_carton/src/corsa_resolver/tests/provider_authority.rs",
        consumesFixture: false,
      },
      {
        selector:
          "corsa_resolver::tests::provider_authority::invalid_nearest_alias_is_not_rescued_by_an_ancestor_runtime",
        source: "crates/vize_carton/src/corsa_resolver/tests/provider_authority.rs",
        consumesFixture: false,
      },
      {
        selector:
          "corsa_resolver::tests::provider_authority::mismatched_nearest_platform_is_not_rescued_by_an_ancestor_runtime",
        source: "crates/vize_carton/src/corsa_resolver/tests/provider_authority.rs",
        consumesFixture: false,
      },
      {
        selector:
          "corsa_resolver::tests::provider_authority::every_implicit_cache_requires_the_developer_flag",
        source: "crates/vize_carton/src/corsa_resolver/tests/provider_authority.rs",
        consumesFixture: false,
      },
    ],
  },
};
