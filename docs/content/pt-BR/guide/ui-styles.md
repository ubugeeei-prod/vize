---
title: Estilos de UI
---

<!-- Generated translation; source: guide/ui-styles.md -->

# Estilos de UI

Os componentes de `@vizejs/ui` funcionam sem uma folha de estilos visual. Para usar os estilos opcionais do Vize, importe a base, uma paleta e os componentes que sua página usa:

```ts
import "@vizejs/ui/base.css";
import "@vizejs/ui/theme-preset-paper.css";
import "@vizejs/ui/component-button.css";
import "@vizejs/ui/component-input.css";
import "@vizejs/ui/component-textarea.css";
import "@vizejs/ui/component-checkbox.css";
import "@vizejs/ui/component-switch.css";
import "@vizejs/ui/component-dialog.css";
import "@vizejs/ui/component-card.css";
import "@vizejs/ui/component-badge.css";
import "@vizejs/ui/component-alert.css";
import "@vizejs/ui/component-tooltip.css";
// Add these only when their low-level behavior is used without the JS entry:
// import "@vizejs/ui/component-progress-bar.css";
// import "@vizejs/ui/component-scroll-area.css";
// import "@vizejs/ui/motion.css";
```

```vue
<template>
  <main data-vize-theme="paper">
    <Button>Save changes</Button>
  </main>
</template>
```

`base.css` fornece tokens semânticos, densidade e a política de forced-colors; ele é um alias para a exportação existente `theme.css`. Ele não redefine elementos da aplicação nem altera componentes do Vize sem estilo. Os arquivos de componente são assets CSS simples e não alteram a API de JavaScript ou Vue. Importar um componente isoladamente não adiciona as regras visuais de nenhum outro componente.

Input, Textarea, Checkbox e Switch têm, cada um, sua própria exportação CSS. Os campos de texto mantêm o comportamento nativo de edição e redimensionamento; o Checkbox distingue marcado de misto, enquanto o Switch move seu indicador quando o estado muda. As pequenas transições de estado param sob `prefers-reduced-motion`. O foco permanece visível para usuários de teclado, e o modo forced-colors mantém as marcas nativas das caixas de seleção. Os arquivos de componente redefinem suas sobrescritas locais em cada fronteira de tema, de modo que um formulário Paper aninhado dentro de uma página Signal mantém a tipografia e as proporções do Paper.

Card, Badge, Alert e Tooltip também têm, cada um, um arquivo visual separado. O Card usa seus hooks `variant`, `density` e `tone`; o Badge distingue rótulos, contagens e texto de status; o Alert segue sua variante de live region sem adicionar um botão de fechar. O Tooltip mantém visível o foco de teclado do gatilho e só inicia sua breve entrada depois que sua posição flutuante é medida. Superfícies estáticas de Card não são animadas. As mudanças de tom do Badge e as entradas de Alert/Tooltip param sob `prefers-reduced-motion`; o modo forced-colors restaura as bordas do sistema. Esses estilos são redefinidos em fronteiras de tema aninhadas, incluindo hosts de Shadow DOM.

As receitas existentes de estrutura e movimento de ProgressBar e ScrollArea também são publicadas como arquivos independentes somente de CSS. Suas entradas JavaScript já carregam a folha de estilos agregada legada para o comportamento necessário. Importe o arquivo independente ao usar os hooks CSS sem a entrada JavaScript, ou ao controlar as folhas de estilos explicitamente; evite importar os dois caminhos na mesma página.

| Preset    | Característica                                                        |
| --------- | --------------------------------------------------------------------- |
| `paper`   | Papel quente, tinta, bordas finas e controles quadrados               |
| `signal`  | Superfícies densas de grafite com bordas nítidas e pouca profundidade |
| `atelier` | Neutros silenciosos de estúdio com um único destaque contido          |

Para oferecer um seletor de estilos, substitua a importação do preset único acima pelos presets que você deseja oferecer. Cada folha de estilos é ativada apenas em seu próprio escopo:

```ts
import "@vizejs/ui/theme-preset-paper.css";
import "@vizejs/ui/theme-preset-signal.css";
import "@vizejs/ui/theme-preset-atelier.css";

document.documentElement.dataset.vizeTheme = "signal";
```

Os presets existentes `midnight`, `play`, `high-contrast` e `headless` continuam disponíveis. Defina `data-vize-theme` em `<html>` quando um diálogo ou tooltip for teleportado para o body do documento, para que o conteúdo flutuante herde a mesma paleta da página. Para uma preferência armazenada, `@vizejs/ui/theme-scope` fornece um script de inicialização executado antes da pintura. As importações antigas de `theme.css`, `theme-preset-*.css` e `style.css` continuam funcionando; evite importar `style.css` junto com `base.css`, pois ele já contém a base e todos os presets legados.

O Button tem um feedback curto de hover, pressionamento e foco. Com `dialog.css`, o Dialog anima sua chegada e uma saída de 200ms. Fechar libera imediatamente a contenção de foco, o inert externo e o bloqueio de rolagem; o painel em saída permanece inert e oculto das tecnologias assistivas até o fim de sua animação. O Dialog headless ainda é desmontado imediatamente, assim como o Dialog estilizado quando `prefers-reduced-motion: reduce` corresponde. Ambos os arquivos visuais mantêm bordas claras no modo forced-colors. O CSS da aplicação fora das cascade layers do Vize pode sobrescrever qualquer regra.
