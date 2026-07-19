---
title: VS Code
---

<!-- Generated translation; source: integrations/vscode.md -->

# Intégration VS Code

> **⚠️ Travaux en cours :** Le support de l’éditeur de Vize est encore expérimental.

> **Important :** Pour le support quotidien de l’éditeur Vue, continuez à utiliser les outils officiels de langage Vue
> (`vuejs/language-tools`) pour l’instant. Vize est conçu pour une évaluation par adhésion progressive.

Le dépôt contient deux extensions expérimentales de VS Code :

- **Vize** — Support linguistique Vue soutenu par `vize lsp`
- **Vize Art** — surlignage syntaxique pour les fichiers Musea `*.art.vue`

Installez-les depuis le VS Code Marketplace :

```bash
code --install-extension ubugeeei.vize
code --install-extension vize.vize-art
```

Installez les deux si vous voulez `*.art.vue` recevoir le survol de Vize, la complétion, la prise en charge de la définition et
référence en plus de la surlignure de syntaxe.

## Vize Extension

L’extension Vize commence `vize lsp` et peut choisir dans des bundles de capacités spécifiques.
Lorsque vous ouvrez un fichier Vue avec l’extension toujours désactivée, ou sans capacités activées, l’extension propose désormais une configuration d’espace de travail recommandée en un clic afin que le survol, le saut et les diagnostics ne restent pas silencieusement désactivés.
Cette configuration écrit `vize.enable`, `vize.lint.enable`, `vize.typecheck.enable`et `vize.editor.enable` pour l’espace de travail actuel.
Si vous ne définissez manuellement que `vize.enable: true`, Vize utilise aussi les diagnostics recommandés et
profil éditeur au lieu de lancer un serveur de langue vide.
L’élément de la barre d’état Vize s’ouvre `Vize: Show Status`, ce qui vous donne le sélecteur de profil, le sélecteur
le binaire, l’action de redémarrage, les paramètres et les journaux d’un seul endroit.

### Point de départ recommandé

```json
{
  "vize.enable": true,
  "vize.lint.enable": true,
  "vize.typecheck.enable": false,
  "vize.editor.enable": false,
  "vize.formatting.enable": false
}
```

Cela permet d’abord de faire des diagnostics de peluches tout en laissant la navigation, la complétion et la mise en forme à vos outils Vue
existants.

### Contextes courants

| Cadre                        | Objectif                                                                 |
| ---------------------------- | ------------------------------------------------------------------------ |
| `vize.enable`                | Activez l’extension et le serveur de langage                             |
| `vize.serverPath`            | Écraser le chemin exécutable `vize`                                      |
| `vize.lint.enable`           | Activer le diagnostic des peluches                                       |
| `vize.typecheck.enable`      | Activez les diagnostics sensibles au type et les fonctionnalités backend |
| `vize.editor.enable`         | Activez le pack d’assistance à l’éditeur                                 |
| `vize.completion.enable`     | Activer la complétion                                                    |
| `vize.formatting.enable`     | Activer la mise en forme des documents                                   |
| `vize.definition.enable`     | Activer la définition directe                                            |
| `vize.references.enable`     | Activer les références                                                   |
| `vize.hover.enable`          | Activer le survol                                                        |
| `vize.codeActions.enable`    | Activez les solutions rapides pour les peluches                          |
| `vize.semanticTokens.enable` | Activer les jetons sémantiques                                           |
| `vize.trace.server`          | Communication LSP de trace                                               |

### Commandes utiles

| Commandement                              | Objectif                                                                     |
| ----------------------------------------- | ---------------------------------------------------------------------------- |
| `Vize: Show Status`                       | Ouvrir le hub d’action d’état et de configuration                            |
| `Vize: Enable Recommended Profile`        | Activez la peluche, la vérification de la police et l’assistance à l’éditeur |
| `Vize: Enable Lint-Only Profile`          | Activez les diagnostics tout en gardant d’autres outils en usage             |
| `Vize: Select Language Server Executable` | Définir `vize.serverPath` depuis un sélecteur de fichiers                    |
| `Vize: Disable Language Server`           | Arrêtez Vize pour la configuration cible actuelle                            |
| `Vize: Restart Language Server`           | Redémarrer le serveur de langage                                             |
| `Vize: Show Output Channel`               | Afficher les extensions et les journaux LSP                                  |

### Ce que l’extension utilise

```text
VS Code
  ↕ Language Server Protocol
vize lsp (vize_maestro)
  → vize_armature
  → vize_croquis
  → vize_patina
  → vize_canon
  → vize_glyph
```

### Installation depuis Source ou VSIX

Installez `vp` une fois depuis le [Vite+ install guide](https://viteplus.dev/guide/install), puis :

```bash
git clone https://github.com/ubugeeei-prod/vize.git
cd vize
cd editors/vscode
vp install -- --ignore-workspace
vp pack
vp exec vsce package --no-dependencies --out dist/vize.vsix
code --install-extension dist/vize.vsix
```

## Extension artistique Vize

`Vize Art` fournit la surlignance de syntaxe pour les fichiers `*.art.vue` Musea.
Son identifiant d’extension Marketplace est `vize.vize-art`.

Il reconnaît :

- `<art>` blocs de métadonnées
- `<variant>` blocs
- Sections standard de Vue `<template>`, `<script>`et `<style>`

## Autres rédacteurs

`vize lsp` suit le protocole Language Server et peut être utilisé par des éditeurs tels que Neovim, Helix,
Zed et Emacs.

Exemple de configuration Neovim :

```lua
require("lspconfig").vize.setup({
  cmd = { "vize", "lsp" },
  filetypes = { "vue" },
  init_options = {
    lint = true,
    typecheck = true,
    editor = true,
  },
})
```

`editor = true` est la façon la plus simple de tester le survol, la complétion, le saut, les références et les symboles
ensemble. Quand un autre serveur TypeScript comme tsgo possède les diagnostics de projet, gardez
`typecheck = false` et activez uniquement les capacités spécifiques à Vue que vous souhaitez évaluer.
