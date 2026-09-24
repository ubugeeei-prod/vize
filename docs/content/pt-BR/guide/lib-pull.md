---
title: Distribuição de código-fonte (vize lib)
---

<!-- Generated translation; source: guide/lib-pull.md -->

# Distribuição de código-fonte (`vize lib`)

`vize lib` copia componentes de `@vizejs/ui` e composables de `@vizejs/composable` para o seu projeto
**como código-fonte**, no estilo do shadcn/ui. Os arquivos copiados são seus: edite-os à vontade, e o Vize
registra de qual versão de pacote cada arquivo veio para que as atualizações continuem seguras.

```bash
vpx vize lib pull rating
```

Isso grava a família `rating` e tudo o que ela importa (`controllable-state`, `id`, ...) em
`src/components/vize/` e registra o pull em `vize-lib.lock.json`.

## De onde vêm os fontes

Todo tarball publicado de `@vizejs/ui` e `@vizejs/composable` contém um registro versionado:

```text
node_modules/@vizejs/ui/
  registry/
    registry.json          # itens, arquivos, digests sha256, dependências
    files/families/...     # fontes .vue / .ts / .css brutos (sem testes)
```

`vize lib` resolve um registro nesta ordem:

1. `--registry <path>`: um `registry.json`, seu diretório ou um diretório de pacote descompactado. Repita a
   flag para passar os registros ui e composable. A descoberta automática é então desativada.
2. O pacote instalado no projeto (`node_modules/@vizejs/ui/registry/registry.json`, procurado nos diretórios
   pais como na resolução do Node).
3. `npm pack @vizejs/<pkg>@<version>` em um diretório temporário seguido de `tar -xzf`, quando a requisição fixa
   uma versão diferente da instalada ou quando o pacote não está instalado (então `@latest`). Use `--offline`
   para proibir esta etapa.

Nenhum servidor de registro nem cliente HTTP extra é usado: o registro é exatamente o tarball que o npm já
serve, então cada versão é imutável e reproduzível.

## Comandos

| Comando                                  | O que faz                                                                            |
| ---------------------------------------- | ------------------------------------------------------------------------------------ |
| `vize lib init [--dry-run]`              | Detecta o layout do projeto e grava a seção `lib` da configuração.                   |
| `vize lib list [--kind ui\|composable]`  | Lista os itens disponíveis.                                                          |
| `vize lib search <words>`                | Busca por nomes, títulos, descrições e aliases.                                      |
| `vize lib info <name>`                   | Mostra arquivos, dependências do registro, peers npm e a versão do pacote.           |
| `vize lib pull <item>... [--dir <dir>]`  | Copia itens e suas dependências do registro; `--dry-run`, `--overwrite`.             |
| `vize lib add <item>...`                 | Alias de `pull` compatível com o shadcn (`-p/--path`, `-o/--overwrite`, `-y/--yes`). |
| `vize lib status`                        | Compara os arquivos copiados com o lockfile e o registro instalado.                  |
| `vize lib diff <name> [--to <version>]`  | Diff unificado da sua cópia local para uma versão do registro.                       |
| `vize lib update [<name>...] [--to <v>]` | Aplica mudanças upstream sem sobrescrever edições locais; `--dry-run`, `--force`.    |
| `vize lib remove <name>...`              | Remove itens e dependências que nada mais usa; `--dry-run`, `--force`.               |
| `vize lib outdated`                      | Compara as versões do lock com os registros instalado e mais recente.                |

Todos os comandos aceitam `--json` para saída legível por máquina e `--root <dir>` para operar em outro projeto.

### Nomeando itens

Itens são referenciados pelo nome canônico (`rating`), por alias (`star rating`, `useToggle`), com um prefixo
de tipo quando o nome existe nos dois pacotes (`ui:locale`, `composable:locale`) e com uma versão exata do pacote
(`rating@0.427.0`, `composable:use-toggle@0.427.0`).

### Diretórios de destino

Os arquivos copiados mantêm o layout do registro (`families/form/rating/rating.vue`,
`foundations/id/deterministic-id.ts`, ...) sob um diretório por tipo, então os imports relativos entre itens
continuam funcionando sem reescrita. O diretório é escolhido por:

1. `--dir <dir>` (deve ficar dentro do projeto),
2. a seção `lib` do `vize.config.*`,
3. o padrão do registro: `src/components/vize` (ui) e `src/composables/vize` (composable).

```ts
// vize.config.ts
import { defineConfig } from "vize";

export default defineConfig({
  lib: {
    uiDir: "src/ui/vendor",
    composableDir: "src/composables/vendor",
    // dir: "src/vendor",        // fallback comum aos dois tipos
    // lockfile: "vize-lib.lock.json",
  },
});
```

Depois que um tipo foi copiado para um diretório, os próximos pulls desse tipo o reutilizam; um `--dir`
conflitante é rejeitado em vez de dividir o grafo de dependências.

Os fontes copiados importam apenas caminhos relativos e pacotes npm como `vue`. `pull` informa qualquer
dependência npm que o seu `package.json` ainda não declara; ele nunca instala pacotes por você.

## Primeiros passos: `init`

```bash
vize lib init --dry-run   # mostra o layout detectado e a mudança na configuração
vize lib init             # grava
```

`init` detecta o diretório de fontes (`src/`, ou `app/` em projetos Nuxt 4) e o TypeScript, e grava
`lib.uiDir` / `lib.composableDir`. Ele cria `vize.config.json` quando não há configuração, acrescenta a seção
`lib` a um `vize.config.json` existente sem mexer no resto do arquivo, e imprime um trecho para
`vize.config.ts` / `.pkl` em vez de editar código. Uma seção `lib` existente é mantida, a menos que `--force`
seja usado. Se o `tsconfig.json` não tiver `allowImportingTsExtensions`, `init` avisa: os fontes copiados
importam arquivos vizinhos como `./x.ts`.

## Verificando atualizações: `outdated`

`vize lib outdated` lista cada item copiado cujo registro difere do lockfile:

| Coluna    | Significado                                                                           |
| --------- | ------------------------------------------------------------------------------------- |
| `current` | Versão registrada em `vize-lib.lock.json`.                                            |
| `wanted`  | Versão do registro que `update` usaria (pacote instalado, senão o mais recente).      |
| `latest`  | Versão mais recente publicada no npm (`npm view`; ignorada com `--offline`).          |
| `state`   | `update-available`, `newer-release`, `removed-upstream`, `unknown` (ou `up-to-date`). |

`update-available` significa que o `contentHash` do item mudou; uma nova versão que não altera os arquivos do
item o mantém `up-to-date`. `--json` inclui todos os itens.

## Registros de terceiros

Qualquer pacote ou site pode publicar um registro no mesmo formato e expô-lo sob um namespace:

```json
{
  "lib": {
    "registries": {
      "@acme": "npm:@acme/vue-kit",
      "@design": { "source": "https://design.example.com/r/registry.json", "dir": "src/design" },
      "@local": { "source": "./registry", "dir": "src/local" }
    }
  }
}
```

```bash
vize lib pull @acme/data-table @design/button@2.1.0
vize lib list --kind @acme
```

- `npm:<package>[@range]` usa o `registry/registry.json` do pacote instalado, senão `npm pack`.
- `https://…/registry.json` é obtido com `curl`, e cada arquivo é baixado sob demanda de `files/<path>` ao lado dele (apenas https).
- Qualquer outro valor é um caminho relativo ao arquivo de configuração: um `registry.json`, seu diretório ou um diretório de pacote.

Registros de terceiros são validados com o mesmo JSON Schema dos oficiais (campos desconhecidos, digests
malformados, papéis ou dependências desconhecidos e fechamento de dependências incompleto são rejeitados), e cada
byte baixado é verificado com SHA-256 antes de ser gravado. Os itens de um namespace vão para o seu `dir` (senão o
`defaultTargetDirectory` do registro), ficam no lock sob a chave `@namespace` e nunca sobrescrevem um arquivo que
pertence a outro item copiado.

## Versionamento e atualizações seguras

`vize-lib.lock.json` (faça commit dele) registra, por item, o pacote e a versão exata de origem, o
`contentHash` do registro, se você o pediu ou se ele veio como dependência, e o SHA-256 de cada arquivo no momento
do pull. Esses digests são a base de merge de uma comparação de três vias entre **o seu arquivo**, **o arquivo
como copiado** e **o arquivo do novo registro**:

| Seu arquivo vs. copiado | Registro vs. copiado | `update` / `pull` faz                             |
| ----------------------- | -------------------- | ------------------------------------------------- |
| inalterado              | inalterado           | nada (`unchanged`)                                |
| inalterado              | alterado             | substitui (`update`)                              |
| editado                 | inalterado           | mantém sua edição (`keep-local`)                  |
| editado                 | alterado             | recusa (`conflict`) sem `--force` / `--overwrite` |
| inalterado              | removido upstream    | remove (`delete`)                                 |
| editado                 | removido upstream    | recusa (`conflict-delete`) sem `--force`          |
| ausente                 | qualquer             | restaura (`create`)                               |
| existe, fora do lock    | qualquer             | recusa (`conflict`) sem `--overwrite`             |

Nada é gravado enquanto houver conflito sem resolução, então uma atualização recusada deixa os arquivos e o
lockfile intactos. Use `vize lib diff <name> --to <version>` para revisar a mudança upstream, faça o merge
manualmente e então rode `update --force`.

Uma atualização típica:

```bash
pnpm add @vizejs/ui@latest       # ou: vize lib update --to 0.428.0
vize lib status                  # quais itens têm atualizações / edições locais
vize lib update --dry-run        # pré-visualiza as ações de arquivo
vize lib update                  # aplica; conflitos são listados, se houver
```

`remove` apaga um item e todas as dependências copiadas só para ele. Ele recusa enquanto outro item copiado
ainda importa o alvo e mantém arquivos editados localmente, a menos que `--force` seja usado.

## Formato do registro

O documento do registro é descrito por
[`vize-lib-registry.schema.json`](https://github.com/ubugeeei-prod/vize/blob/main/npm/cli/schemas/vize-lib-registry.schema.json)
e o lockfile por
[`vize-lib-lock.schema.json`](https://github.com/ubugeeei-prod/vize/blob/main/npm/cli/schemas/vize-lib-lock.schema.json).
Ambos são publicados no pacote npm `vize` em `schemas/`.

- `registryDependencies` é o fechamento transitivo completo calculado a partir do grafo de imports relativos
  no build; fundações compartilhadas são itens separados, então copiar dois componentes que precisam de `id`
  copia `id` uma única vez.
- Cada arquivo traz seu `sha256`; `vize lib` verifica cada byte copiado contra ele.
- `contentHash` só muda quando os arquivos de um item mudam, então `status` só informa "update available" para
  itens cujos fontes realmente diferem entre versões.
