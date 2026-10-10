# Public configuration CI repair custody

Issue: [#8371](https://github.com/ubugeeei-prod/vize/issues/8371).
Public layer: [#8398](https://github.com/ubugeeei-prod/vize/pull/8398),
observed head `c404b022cfc67df1c2c4049156df6e4af106e80d`.

[Check run 38043239322](https://github.com/ubugeeei-prod/vize/actions/runs/38043239322)
failed the source-length ratchet on the library initializer tests and English
Musea guide. Move the five complete initializer test bodies into a child module;
keep their inputs and oracles unchanged. The moved source block has SHA-256
`9d1a8b99a5b142aa715855ba3a17bd06d1e5bc21797344f31f425507bbabbb5c`.

Retain every original Musea guide heading and complete code fence, including the
supported dedicated-config example. Move the new public Vite recipe unchanged
to the [configuration reference](../../content/guide/configuration-reference.md#musea-shared-configuration)
and retain a guide link. The moved complete fence has SHA-256
`34a202856a18c1193c9619a18bb1900fa0d6bacdcebb96c9f527d68b0e9e859d`.
The reader layer must preserve this section together with its historical recipes
and direct the English Musea whole-scope fixture to the reference page.

[Musea run 38043240938](https://github.com/ubugeeei-prod/vize/actions/runs/38043240938)
failed before rendering because the workspace `vize/config` export was unbuilt.
Use the existing CLI package build after the native binding and before the Vite
and Musea packages. Keep browser inputs, oracles, runtime settings, dependency
manifests, and publishers unchanged. Actual rendering qualification requires
successful source Actions on the repaired head; the failed run is not proof.
