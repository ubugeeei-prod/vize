---
title: Filosofia
---

<!-- Generated translation; source: philosophy.md -->

# Filosofia

> **⚠️ Trabalho em andamento:** O Vize está em desenvolvimento ativo e ainda não está pronto para uso em produção. Os princípios de design abaixo descrevem a visão e a direção do projeto.

Vize é mais do que um compilador — é uma declaração de design sobre como Vue.js ferramentas devem funcionar.

## Por Que A Vize Existe

O ecossistema JavaScript há muito tempo depende de ferramentas baseadas em JavaScript para compilar, lint, formatar e verificar tipos de código JavaScript. Isso cria um gargalo fundamental: as ferramentas que processam seu código estão sujeitas às mesmas limitações de tempo de execução que o código que processam — pausas na coleta de lixo, execução em thread única e overhead dinâmico de despacho.

Vize adota uma abordagem diferente. Ao reescrever toda a Vue.js cadeia de ferramentas no Rust, eliminamos essas restrições no nível da arquitetura. O resultado não é uma melhoria incremental — é uma mudança categórica no que é possível.

## Princípios de Design

### 1. Cadeia de Ferramentas Unificada

O desenvolvimento tradicional de Vue.js requer montar uma constelação de ferramentas separadas: um compilador (`@vue/compiler-sfc`), um linter (eslint + eslint-plugin-vue), um formatador (mais bonito), um verificador de tipos (vue-tsc) e um explorador de componentes (Storybook). Cada ferramenta tem seu próprio parser, sua própria representação AST e seu próprio formato de configuração.

Vize unifica todos esses em um único binário. Um analisador sintético. Um AST. Uma superfície de configuração. Isso elimina passagens redundantes de análise, reduz a complexidade da configuração e garante que todas as ferramentas compartilhem uma compreensão consistente do seu código.

```
@vue/compiler-sfc  +  eslint-plugin-vue  +  prettier  +  vue-tsc  +  Storybook
                              ↓
                            vize
```

### 2. Desempenho como Característica

Velocidade não é algo agradável — é um pré-requisito para a experiência do desenvolvedor. Quando a compilação leva segundos, os desenvolvedores perdem o fluxo. Quando o linting leva minutos, os desenvolvedores o desativam. Quando a verificação de tipos demora demais, os desenvolvedores pulam.

O Vize foi projetado para que toda ferramenta rode rápido o suficiente para ser usada interativamente:

- **Compilação**: 15.000 arquivos SFC em 498ms (multithreaded)
- **Formatação**: quase instantânea, mesmo em grandes bases de código
- **Linting**: Feedback em tempo real através do LSP
- **Verificação de tipo**: Análise incremental sem sobrecarga V8

Isso é alcançado por meio das abstrações de custo zero, alocação de arenas e multithreading nativo com Rayon do Rust.

### 3. Compatibilidade Drop-in

O Vize não pede para você reescrever seu código ou alterar seu fluxo de trabalho. O plugin Vite é um substituto direto para `@vitejs/plugin-vue`. Seus componentes existentes do Vue, `<script setup>`, estilos com mira e HMR funcionam sem modificações.

Esse princípio se estende ao ecossistema mais amplo. O plugin Vite da Vize é compatível com o Nuxt, e o LSP se integra ao VS Code por meio de protocolos padrão. Adotar o Vize deve parecer como atualizar seu motor, não reconstruir seu carro.

### 4. Arte como Arquitetura

Cada caixa Vize leva o nome de um conceito das artes visuais — pintura, escultura e curadoria de museus. Isso não é mera fantasia. A convenção de nomes codifica uma filosofia: **código é um meio criativo**, e as ferramentas que o moldam devem refletir a arte envolvida.

| Caixa        | Origem da Arte                             | Função                                             |
| ------------ | ------------------------------------------ | -------------------------------------------------- |
| **Caixa**    | Estojo de portfólio do artista             | Utilidades compartilhadas — a caixa de ferramentas |
| **Relevo**   | Projeção escultórica da superfície         | AST — a superfície estruturada do código           |
| **Armadura** | Esqueleto sustentando uma escultura        | Analisador — a estrutura estrutural                |
| **Croquis**  | Esboço gestual rápido                      | Análise semântica — capturando a essência          |
| **Atelier**  | Oficina de artistas                        | Compilador — onde ocorre a transformação           |
| **Vitrine**  | Vitrine de vidro                           | Encadernações — expor a obra                       |
| **Canon**    | Padrão das proporções ideais               | Verificador de tipos — garantindo a correção       |
| **Pátina**   | Superfície envelhecida indicando qualidade | Linter — polimento da superfície                   |
| **Glifo**    | Símbolo ou forma de letra entalhada        | Forformatação — moldando o texto                   |
| **Maestro**  | Maestro maestro                            | LSP — orquestrando a experiência                   |
| **Musea**    | Plural de museu                            | Galeria componente — exposição da obra             |
| **Afresco**  | Técnica de pintura mural                   | Estrutura TUI — pintar o terminal                  |

Esse sistema de nomenclatura tem um propósito prático: torna a hierarquia das caixas intuitiva. Quando você vê `vize_atelier_dom`, entende imediatamente que é um _workshop_ que produz _saída VDOM_. Quando você vê `vize_patina`, sabe que ele _aprimora_ seu código.

#### A Analogia da Escultura

A analogia mais profunda é entre compilação de software e escultura. Considere como um escultor trabalha:

1. **Armadura** — O escultor começa construindo uma armadura: um esqueleto de arame que define a estrutura básica. No Vize, o analisador (`vize_armature`) constrói a estrutura estrutural (AST) a partir do texto bruto.

2. **Relevo** — O escultor constrói a superfície sobre a armadura, criando um _relevo_ — uma superfície estruturada que se projeta de um plano plano. No Vize, o AST (`vize_relief`) dá uma forma estruturada e tridimensional ao que originalmente era texto plano.

3. **Croquis** — Antes de se comprometer com uma escultura final, o artista faz esboços rápidos (_croquis_) para entender o caráter essencial do sujeito. No Vize, análise semântica (`vize_croquis`) é uma passagem rápida que captura o significado do código — quais variáveis são vinculadas, quais expressões são válidas — sem se comprometer com um destino de compilação.

4. **Atelier** — O escultor se desloca para o _ateliê_ (oficina) para criar a peça final. Múltiplos ateliês podem produzir diferentes versões do mesmo tema. No Vize, os backends de compilação (`vize_atelier_dom`, `vize_atelier_vapor`, `vize_atelier_ssr`) são oficinas diferentes que produzem diferentes versões (VDOM, Vapor, SSR) do mesmo AST analisado.

5. **Vitrine** — A obra finalizada é colocada em uma _vitrine_ (vitrine de vidro) para que outros possam observá-la. No Vize, as ligações (`vize_vitrine`) são uma camada transparente que permite aos consumidores JavaScript acessar a saída compilada.

6. **Musea** — Por fim, as obras são exibidas em um _museu_ para apreciação e estudo. No Vize, a galeria de componentes (`vize_musea`) é onde os componentes são exibidos, explorados e documentados.

#### A Analogia dos Artesanatos de Qualidade

As caixas restantes seguem uma analogia de artesanato:

- **Cânone** (verificador de tipos) — Na escultura clássica, o _cânone_ era um padrão de proporções humanas ideais. Polykleitos escreveu o _Kanon_ definindo razões matemáticas para a figura perfeita. No Vize, o verificador de tipos impõe as "proporções ideais" do seu código — os tipos devem estar corretos, os adereços devem corresponder, as emissões devem se conformar.

- **Pátina** (linter) — Uma _pátina_ é o acabamento superficial que se desenvolve em materiais envelhecidos, indicando qualidade e cuidado. Uma escultura de bronze com uma rica pátina foi bem conservada. No Vize, o linter examina a superfície do seu código, identificando problemas que afetam sua qualidade.

- **Glifo** (formatador) — Um _glifo_ é um símbolo ou forma de letra entalhada — pense nas formas de letra precisas e consistentes em uma fonte. Cada glifo tem proporções e espaçamentos exatos. No Vize, o formatador garante que seu código tenha proporções consistentes e precisas.

- **Maestro** (LSP) — Um _maestro_ é o maestro que orquestra um conjunto em uma apresentação unificada. No Vize, o servidor LSP orquestra todas as funcionalidades da linguagem (completão, diagnóstico, formatação, navegação) em uma experiência unificada de editor.

- **Afresco** (TUI) — Um _afresco_ é uma técnica de pintura em que pigmento é aplicado sobre o reboco úmido, tornando-se parte da própria parede. No Vize, o framework TUI "pinta" diretamente na superfície do terminal.

### 5. Pensamento Vapor-First

O Vue 3.6 introduz o modo Vapor — uma estratégia de compilação que gera código reativo detalhado sem o DOM virtual. O Vize foi projetado com o modo Vapor como um alvo de compilação de primeira classe desde o primeiro dia.

Embora `@vue/compiler-sfc` adicionado suporte ao Vapor de forma incremental, o `vize_atelier_vapor` da Vize foi construído junto com `vize_atelier_dom` desde o início. Isso significa que a infraestrutura de compilação compartilhada (`vize_atelier_core`) é projetada para atender igualmente bem ambos os modos de saída.

### 6. Soberania do Desenvolvedor

Vize é uma cadeia **de ferramentas independente** . Não é controlada pela equipe central Vue.js e não reivindica ser a forma "oficial" de construir aplicações Vue. Isso é intencional.

Ao permanecer independente, a Vize pode:

- Experimente estratégias de compilação sem o ônus da compatibilidade retroativa
- Avance mais rápido do que um projeto oficial sujeito a processos de governança
- Servir como campo de testes para ideias que podem eventualmente influenciar a cadeia oficial de ferramentas
- Ofereça uma alternativa para desenvolvedores que desejam o máximo desempenho

Ao mesmo tempo, a Vize acompanha de perto a especificação oficial Vue.js. O objetivo é a compatibilidade, não a fragmentação.

### 7. Ficar sobre os ombros da oxidação

Vize não existe isoladamente. Faz parte de um movimento mais amplo para reescrever ferramentas JavaScript em linguagens de sistema — o que a comunidade chama de "oxidação". Vize abraça e integra esse ecossistema:

- **OXC** — Vize usa o [Oxidation Compiler](https://oxc.rs/) (oxc) para análise sintática em JavaScript e TypeScript. O OXC fornece a análise AST de alto desempenho de JS/TS que alimenta `vize_croquis` (análise semântica) e `vize_atelier_core` (geração de código). Em vez de reimplementar um parser JS, o Vize delega para a implementação testada do OXC.
- **oxlint** — Vize foi projetado pensando [oxlint](https://oxc.rs/docs/guide/usage/linter) . Embora `vize_patina` lide com linting de templates específicos do Vue, a história mais ampla do linting em JavaScript é melhor servida pelo motor de regras nativo do oxlint para Rust. As duas ferramentas são complementares, não competidoras.
- **Corsa** — A camada nativa de execução TypeScript da Vize, construída em torno de [`corsa-bind`](https://github.com/ubugeeei/corsa-bind), representa a direção que a Vize está seguindo para a verificação de tipos JavaScript/TypeScript sem que tudo passe por um compilador hospedado em JavaScript. `vize_canon` utiliza essa pilha para diagnósticos nativos, continuando a fornecer análise de tipos de modelos específicos para o Vue.
- **LightningCSS** — O Vize utiliza [LightningCSS](https://lightningcss.dev/) para análise e transformação de CSS dentro de `vize_atelier_sfc`, aproveitando seu processamento CSS nativo de Rust para estilos com escopo.

Ainda existem muitos desafios não resolvidos nesse espaço — interoperabilidade entre ferramentas com AST, análise incremental entre fronteiras de linguagens e consistência na integração com editores. O Vize busca ser um campo de testes para soluções para esses problemas dentro do ecossistema Vue.js, contribuindo para o movimento mais amplo de oxidação.

### 8. Colaboração com Vite+ e OXC

[Vite+](https://viteplus.dev/) e [OXC](https://oxc.rs) são cadeias de ferramentas **independentes de framework** — elas oferecem capacidades gerais de agrupamento, análise sintática, linting e formatação JS/TS/CSS que funcionam em qualquer framework. Vize é **específico para Vue** e foi projetado para **integrar-se a** essas ferramentas do ecossistema, em vez de competir com elas.

O Vize depende diretamente do OXC para análise em JavaScript/TypeScript e do LightningCSS para processamento de CSS dentro dos SFCs do Vue. O linter (pátina) e o formatador (glifo) do Vize lidam com preocupações específicas do Vue (diretivas modelo, estrutura SFC, convenções de componentes) que estão fora do escopo de ferramentas agnósticas ao framework. Uma integração mais profunda com o OXC está planejada — por exemplo, delegando `<script>` linting/formatação de blocos ao OXC enquanto o Vize cuida das camadas de coordenação de `<template>` e SFC específicas do Vue. O plugin Vite da Vize (`@vizejs/vite-plugin`) é construído sobre o Vite e projetado para ser um substituto direto para `@vitejs/plugin-vue`, abraçando totalmente o ecossistema Vite.

Como autor de Vize, eu ([@ubugeeei](https://github.com/ubugeeei)) quero deixar claro: **não tenho intenção adversária em relação a nenhum desses projetos.** Estou totalmente aberto à colaboração e acredito que os melhores resultados vêm de ferramentas que se complementam. Se houver mudanças necessárias de qualquer lado para possibilitar uma melhor integração, estou pronto para trabalhar juntos para que isso aconteça.

## O nome

- _Vize\*\* (_/viːz/\*) deriva de três palavras:

* **Vizir** — um conselheiro ou conselheiro sábio
* **Visor** — algo que ajuda a enxergar claramente
* **Orientador** — um guia que ajuda você a tomar decisões melhores

Juntos, descrevem uma ferramenta que _enxerga através do seu código_ e _te aconselha com sabedoria_. A pronúncia rima com "brisa" — rápida, natural e refrescante.
