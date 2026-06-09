//! Built-in and legacy Vue 2 component completions.

use tower_lsp::lsp_types::CompletionItem;

use crate::ide::completion::items;

/// Built-in Vue component completions.
pub(crate) fn builtin_component_completions() -> Vec<CompletionItem> {
    vec![
        items::component_item(
            "Transition",
            "Animate enter/leave",
            "<Transition name=\"$1\">\n\t$0\n</Transition>",
        ),
        items::component_item(
            "TransitionGroup",
            "Animate list",
            "<TransitionGroup name=\"$1\" tag=\"$2\">\n\t$0\n</TransitionGroup>",
        ),
        items::component_item(
            "KeepAlive",
            "Cache components",
            "<KeepAlive>\n\t$0\n</KeepAlive>",
        ),
        items::component_item(
            "Teleport",
            "Teleport content",
            "<Teleport to=\"$1\">\n\t$0\n</Teleport>",
        ),
        items::component_item(
            "Suspense",
            "Async dependencies",
            "<Suspense>\n\t<template #default>\n\t\t$0\n\t</template>\n\t<template #fallback>\n\t\tLoading...\n\t</template>\n</Suspense>",
        ),
        items::component_item("component", "Dynamic component", "<component :is=\"$1\" />"),
        items::component_item("slot", "Slot outlet", "<slot name=\"$1\">$0</slot>"),
        items::component_item(
            "template",
            "Template fragment",
            "<template #$1>\n\t$0\n</template>",
        ),
    ]
}

pub(crate) fn legacy_vue2_component_completions() -> Vec<CompletionItem> {
    vec![
        items::component_item(
            "NuxtLink",
            "Nuxt 2 route link",
            "<NuxtLink to=\"$1\">$0</NuxtLink>",
        ),
        items::component_item(
            "nuxt-link",
            "Nuxt 2 route link",
            "<nuxt-link to=\"$1\">$0</nuxt-link>",
        ),
        items::component_item("Nuxt", "Nuxt 2 page outlet", "<Nuxt />"),
        items::component_item("NuxtChild", "Nuxt 2 child route outlet", "<NuxtChild />"),
        items::component_item(
            "ClientOnly",
            "Client-only render",
            "<ClientOnly>$0</ClientOnly>",
        ),
        items::component_item(
            "client-only",
            "Client-only render",
            "<client-only>$0</client-only>",
        ),
        items::component_item("NoSsr", "Client-only render", "<NoSsr>$0</NoSsr>"),
    ]
}
