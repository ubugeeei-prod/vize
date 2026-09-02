---
title: Contributions
---

<!-- Generated translation; source: contributing.md -->

# Contributions

Merci d’avoir aidé à rendre Vize plus affûté. Le projet en est à sa phase **de Real World Testing** et
en route vers la version alpha v1, donc de petits changements ciblés avec une vérification claire sont les plus faciles à examiner. Si vous êtes
ici pour rapporter des résultats plutôt que pour ouvrir une PR, commencez par le guide
[Testing & Feedback](./guide/testing.md).

## Mise en place

Utilisez le runtime de développement Node.js de `package.json#devEngines.runtime` et la version Rust
de `rust-toolchain.toml`. L’espace de travail déclare une version minimale Rust prise en charge (MSRV) de `1.95.0` dans `Cargo.toml`
(`[workspace.package].rust-version`; les contributions doivent être compilées sous cette version.

Le shell Nix par défaut contient la chaîne d’outils locale reproductible. Le support de Blacksmith Testbox est
optionnel et se trouve dans un shell séparé avec la ligne de ligne de ligne Blacksmith, `rsync`et GitHub épinglée :

```sh
nix develop             # local development
nix develop .#testbox   # hosted Testbox workflows
```

Installez les dépendances à partir de la racine de l’espace de travail :

```sh
vp install --frozen-lockfile --prefer-offline
```

Si `vp` n’est pas encore disponible, installez- [Vite+](https://viteplus.dev/guide/install) d’abord.

## Échecs courants

Faites la vérification la plus étroite qui couvre votre changement, puis élargissez lorsque vous touchez le comportement partagé.

```sh
vp check <changed-files>
node --test tests/tooling/<test-file>.test.ts
cargo fmt --all -- --check
cargo test -p <crate>
```

Avant d’ouvrir une PR qui modifie les outils partagés, l’automatisation des releases, les liaisons natives ou le comportement
du compilateur, exécutez localement la tâche de workspace pertinente depuis CI lorsque cela est possible.

Les workflows de build, test et lint root sont locaux par défaut et ne nécessitent pas d’identifiants hébergés :

```sh
vp run --workspace-root build
vp run --workspace-root test
vp run --workspace-root lint
```

Dans le shell de développement Nix, `vp build`, `vp test`et `vp lint` sont des abrégations pour ces tâches
d’espace de travail.

Pour la parité d’inversion-CI Linux à commande unique, entrez dans le shell dédié Testbox. Le `nix develop` shell par défaut
omet intentionnellement Blacksmith et n’a pas besoin de son artefact ou de ses identifiants hébergés :

```sh
nix develop .#testbox
```

Ensuite, lance le cycle de vie protégé ci-dessous. Il efface tout ancien identifiant de boîte avant le réchauffement, saute les tâches à distance si
authentification, push ou échauffement échoue, et tente toujours d’arrêter une boîte réchauffée avec succès, même
lorsqu’une tâche échoue :

```sh
run_testbox_checks() {
  unset BLACKSMITH_TESTBOX_ID testbox_output
  "$VIZE_BLACKSMITH_BIN" auth login || return
  git push --set-upstream origin "$(git branch --show-current)" || return

  if testbox_output="$(vp run --workspace-root testbox:warmup)"; then
    BLACKSMITH_TESTBOX_ID="$(printf '%s\n' "$testbox_output" | tail -n1)"
  else
    warmup_status=$?
    unset testbox_output
    return "$warmup_status"
  fi
  if [ -z "$BLACKSMITH_TESTBOX_ID" ]; then
    printf '%s\n' "Testbox warmup returned no box id." >&2
    unset BLACKSMITH_TESTBOX_ID testbox_output
    return 1
  fi
  export BLACKSMITH_TESTBOX_ID

  if vp run --workspace-root build:testbox &&
    vp run --workspace-root test:testbox &&
    vp run --workspace-root lint:testbox; then
    testbox_status=0
  else
    testbox_status=$?
  fi
  if vp run --workspace-root testbox:stop; then
    stop_status=0
  else
    stop_status=$?
  fi
  unset BLACKSMITH_TESTBOX_ID testbox_output

  if [ "$testbox_status" -ne 0 ]; then
    return "$testbox_status"
  fi
  return "$stop_status"
}
run_testbox_checks
```

Pour les modifications de tâches de Blacksmith Testbox, validez également la forme du workflow avec
`node --test tests/tooling/github-workflows.test.ts`.

## Discipline du changement du processeur de langage

Vize suit la pratique des projets compilateur de rustc, TypeScript, TypeScript-Go et Flow : classifie le
changement, ajoute le plus petit élément significatif, examine la sortie générée sous forme de contrat, puis élargit à
parité, performance ou portes de sortie lorsque la surface touchée en a besoin. Voir
[Language Engineering Practices](./architecture/language-engineering-practices.md) pour la matrice complète de
.

Utilisez l’une de ces classes de changement dans les PR lorsque cela est applicable :

- Parseur ou AST
- Compilateur et codegen
- Analyse sémantique, lint et analyse croisée
- Virtual TypeScript et vérification de type
- Formateur et LSP
- Emballage à l’exécution, version ou documentation

Pour les changements orientés vers le langage, incluez le différentiel de fixture ou instantané qui prouve le comportement. Pour
rafrafraîchissement instantané, expliquez pourquoi la nouvelle sortie est correcte et évitez le churn de base large, sauf si la
RP concerne spécifiquement cette famille de sortie.

Lorsqu’un dysmatch du compilateur commence à partir d’une reproduction externe ou d’un fichier projet local, utilisez le
[Compiler Inspector](./guide/compiler-inspector.md) de terrain d’école pour inspecter la sortie officielle Vue, la sortie Vize,
Virtual TS, VIR, et le graphe cross-file. Ajoutez le permalink de l’inspecteur au corps de relations publiques, puis obtenez le
luminaire minimisé ou instantané complet qui transforme le résultat en contrat examiné. Les lots locaux peuvent
être emballés avec `vize inspector <file-or-glob>`, et le transfert d’agent peut utiliser
`vize inspector --format agent`.

## Demandes de tirage

- Utilisez les commits conventionnels pour les messages de commit et les titres PR, tels que
  `fix(vite-plugin): surface SFC compile errors`.
  - Gardez les RP concentrés sur un seul changement de comportement ou un changement de documentation/gouvernance.
- Inclure des commandes de vérification dans l’organisme de relations publiques.
- Ne rafraîchissez pas de grandes lignes de base snapshot à moins que la PR ne porte spécifiquement sur ces sorties.
- N’incluez pas de secrets, de jetons de registre, de détails de vulnérabilité privée ou de chemins locaux de la machine dans
  rapports, engagements ou relations publiques.

## Demandes de correction

Utilisez le modèle de rapport de correction pour les régressions, plantages, diagnostics incorrects, problèmes d’installation
paquets et échecs de version. Utilisez le modèle de demande de fonctionnalité pour les nouvelles intégrations, les modifications d’API, les
ou les améliorations du flux de travail. Une reproduction minimale — idéalement un lien d’inspecteur de terrain de jeu — rend un rapport de
beaucoup plus rapide à appliquer.

Les rapports de sécurité devraient suivre
[`SECURITY.md`](https://github.com/ubugeeei-prod/vize/blob/main/SECURITY.md) plutôt que les modèles publics
correctifs.

## Code de conduite et de gouvernance

En participant, vous acceptez de respecter les
[Contributor Covenant v2.1](https://www.contributor-covenant.org/version/2/1/code_of_conduct/). Le modèle de gouvernance
et le processus décisionnel sont documentés dans
[`GOVERNANCE.md`](https://github.com/ubugeeei-prod/vize/blob/main/GOVERNANCE.md). Pour obtenir de l’aide
le bon canal, voir [`SUPPORT.md`](https://github.com/ubugeeei-prod/vize/blob/main/SUPPORT.md).
