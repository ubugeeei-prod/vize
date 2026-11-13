type Locale = "zh-CN" | "pt-BR" | "fr";
type Packet = readonly [purpose: string, bad: string, good: string];

export const configuredPackets: Record<Locale, Record<string, Packet>> = {
  "zh-CN": {
    "vue/max-props": [
      "限制 Vue 组件的 props 数量。",
      "组件声明了三个 props，超过配置的上限。",
      "组件只声明一个 prop。",
    ],
    "vue/max-template-depth": [
      "限制模板的嵌套深度。",
      "span 的嵌套深度超过配置的上限。",
      "模板只有一层元素，深度没有超过配置的上限。",
    ],
    "vue/no-restricted-block": [
      "禁止使用配置的块。",
      "SFC 包含被禁止的 i18n 块。",
      "SFC 只保留 template 块。",
    ],
    "vue/no-restricted-call-after-await": [
      "禁止在 await 之后调用受限制的方法。",
      "setup 在 await 之后调用 onMounted。",
      "setup 在 await 之前注册 onMounted。",
    ],
    "vue/no-restricted-class": [
      "禁止使用配置的 class。",
      "元素使用了被禁止的 legacy class。",
      "元素使用允许的 class。",
    ],
    "vue/no-restricted-component-names": [
      "禁止使用配置的组件名。",
      "组件名是被禁止的 RestrictedShowcase。",
      "组件名是允许的 Widget。",
    ],
    "vue/no-restricted-component-options": [
      "禁止使用配置的组件选项。",
      "组件设置了被禁止的 inheritAttrs。",
      "组件只设置 name。",
    ],
    "vue/no-restricted-custom-event": [
      "禁止使用配置的自定义事件。",
      "模板通过 $emit 发出被禁止的 legacyEvent。",
      "模板发出允许的 save 事件。",
    ],
    "vue/no-restricted-html-elements": [
      "禁止使用配置的元素。",
      "模板使用了被禁止的 marquee。",
      "模板使用允许的 p 元素。",
    ],
    "vue/no-restricted-props": [
      "禁止使用配置的 props。",
      "组件声明了被禁止的 third prop。",
      "组件声明允许的 first prop。",
    ],
    "vue/no-restricted-static-attribute": [
      "禁止使用配置的静态属性。",
      "按钮带有被禁止的 data-legacy 属性。",
      "按钮只保留 type。",
    ],
    "vue/no-restricted-v-on": [
      "禁止使用配置的 v-on 参数。",
      "按钮监听被禁止的 legacy-click。",
      "按钮监听 click。",
    ],
    "vue/no-unsupported-features": [
      "禁止在指定的 Vue.js 版本上使用尚未支持的语法。",
      "defineModel 需要比配置版本更新的 Vue。",
      "组件不使用受版本限制的宏。",
    ],
    "vue/prefer-single-event-payload": [
      "强制自定义事件只传递单个负载参数。",
      "move 事件声明并发出两个负载值，内联监听器只能收到 x。",
      "move 事件传递一个对象，所有监听器都能收到两个坐标。",
    ],
  },
  "pt-BR": {
    "vue/max-props": [
      "Limita o número de props de um componente Vue.",
      "O componente declara três props, acima do máximo configurado.",
      "O componente declara uma prop.",
    ],
    "vue/max-template-depth": [
      "Limita a profundidade do template.",
      "O span está aninhado além do máximo configurado.",
      "O template tem um único elemento.",
    ],
    "vue/no-restricted-block": [
      "Proíbe o bloco configurado.",
      "O SFC contém o bloco i18n proibido.",
      "O SFC mantém apenas o bloco template.",
    ],
    "vue/no-restricted-call-after-await": [
      "Proíbe chamar um método restrito depois de await.",
      "O setup chama onMounted depois de await.",
      "O setup registra onMounted sem await.",
    ],
    "vue/no-restricted-class": [
      "Proíbe a class configurada.",
      "O elemento usa a class proibida legacy.",
      "O elemento usa uma class permitida.",
    ],
    "vue/no-restricted-component-names": [
      "Proíbe o nome de componente configurado.",
      "O nome do componente é o proibido RestrictedShowcase.",
      "O nome do componente é Widget.",
    ],
    "vue/no-restricted-component-options": [
      "Proíbe a opção de componente configurada.",
      "O componente define inheritAttrs, que está proibido.",
      "O componente define apenas name.",
    ],
    "vue/no-restricted-custom-event": [
      "Proíbe o evento personalizado configurado.",
      "O template emite o evento proibido legacyEvent.",
      "O template emite o evento save.",
    ],
    "vue/no-restricted-html-elements": [
      "Proíbe o elemento configurado.",
      "O template usa o marquee proibido.",
      "O template usa p.",
    ],
    "vue/no-restricted-props": [
      "Proíbe a prop configurada.",
      "O componente declara a prop proibida third.",
      "O componente declara a prop first.",
    ],
    "vue/no-restricted-static-attribute": [
      "Proíbe o atributo estático configurado.",
      "O botão tem o atributo proibido data-legacy.",
      "O botão mantém apenas type.",
    ],
    "vue/no-restricted-v-on": [
      "Proíbe o argumento de v-on configurado.",
      "O botão escuta o evento proibido legacy-click.",
      "O botão escuta click.",
    ],
    "vue/no-unsupported-features": [
      "Proíbe sintaxe ainda não suportada na versão configurada do Vue.js.",
      "defineModel exige um Vue mais novo do que a versão configurada.",
      "O componente não usa macros limitadas por versão.",
    ],
    "vue/prefer-single-event-payload": [
      "Exige um único argumento de payload ao emitir eventos personalizados.",
      "O evento move declara e emite dois valores, e um listener inline só recebe x.",
      "O evento move envia um objeto, e todo listener recebe as duas coordenadas.",
    ],
  },
  fr: {
    "vue/max-props": [
      "Limite le nombre de props d'un composant Vue.",
      "Le composant déclare trois props, au-delà du maximum configuré.",
      "Le composant déclare une prop.",
    ],
    "vue/max-template-depth": [
      "Limite la profondeur du template.",
      "Le span est imbriqué au-delà du maximum configuré.",
      "Le template n'a qu'un élément.",
    ],
    "vue/no-restricted-block": [
      "Interdit le bloc configuré.",
      "Le SFC contient le bloc i18n interdit.",
      "Le SFC ne garde que le bloc template.",
    ],
    "vue/no-restricted-call-after-await": [
      "Interdit d'appeler une méthode restreinte après await.",
      "setup appelle onMounted après await.",
      "setup enregistre onMounted sans await.",
    ],
    "vue/no-restricted-class": [
      "Interdit la class configurée.",
      "L'élément utilise la class interdite legacy.",
      "L'élément utilise une class autorisée.",
    ],
    "vue/no-restricted-component-names": [
      "Interdit le nom de composant configuré.",
      "Le nom du composant est le nom interdit RestrictedShowcase.",
      "Le nom du composant est Widget.",
    ],
    "vue/no-restricted-component-options": [
      "Interdit l'option de composant configurée.",
      "Le composant définit inheritAttrs, qui est interdit.",
      "Le composant ne définit que name.",
    ],
    "vue/no-restricted-custom-event": [
      "Interdit l'événement personnalisé configuré.",
      "Le template émet l'événement interdit legacyEvent.",
      "Le template émet l'événement save.",
    ],
    "vue/no-restricted-html-elements": [
      "Interdit l'élément configuré.",
      "Le template utilise le marquee interdit.",
      "Le template utilise p.",
    ],
    "vue/no-restricted-props": [
      "Interdit la prop configurée.",
      "Le composant déclare la prop interdite third.",
      "Le composant déclare la prop first.",
    ],
    "vue/no-restricted-static-attribute": [
      "Interdit l'attribut statique configuré.",
      "Le bouton porte l'attribut interdit data-legacy.",
      "Le bouton ne garde que type.",
    ],
    "vue/no-restricted-v-on": [
      "Interdit l'argument v-on configuré.",
      "Le bouton écoute l'événement interdit legacy-click.",
      "Le bouton écoute click.",
    ],
    "vue/no-unsupported-features": [
      "Interdit une syntaxe encore non prise en charge sur la version Vue.js configurée.",
      "defineModel exige un Vue plus récent que la version configurée.",
      "Le composant n'utilise pas de macro limitée par version.",
    ],
    "vue/prefer-single-event-payload": [
      "Impose un seul argument de charge utile pour les événements personnalisés.",
      "L'événement move déclare et émet deux valeurs, et un écouteur en ligne ne reçoit que x.",
      "L'événement move transmet un objet, et chaque écouteur reçoit les deux coordonnées.",
    ],
  },
};
