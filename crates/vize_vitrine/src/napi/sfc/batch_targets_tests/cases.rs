//! Cases where a target's emit changes the shared analysis, and where the
//! targets cannot share it at all.

use super::{assert_targets_compile_apart, case};

/// What one target's emit changes in the shared analysis, and what decides
/// whether the targets can share it at all.
#[test]
fn shared_analysis_cases_compile_as_they_do_apart() {
    let files = [
        case(
            "BothScripts",
            "<script lang=\"ts\">\nimport { helper } from './helper'\nexport interface Shared { id: string }\nexport default { name: 'Both' }\n</script>\n<script setup lang=\"ts\">\nconst props = defineProps<Shared>()\nconst label = helper(props.id)\n</script>\n<template><p>{{ label }}</p></template>\n",
        ),
        case(
            "EmitsByRender",
            "<script setup lang=\"ts\">\nconst emit = defineEmits<{ (event: 'save', id: number): void }>()\n</script>\n<template><button @click=\"emit('save', 1)\">save</button><i @click=\"$emit('save', 2)\">x</i></template>\n",
        ),
        case(
            "DestructureExpose",
            "<script setup lang=\"ts\">\nconst { count = 1, label } = defineProps<{ count?: number; label: string }>()\ndefineExpose({ double: () => count * 2, label: label as string })\n</script>\n<template><p>{{ count }} {{ label }}</p></template>\n",
        ),
        case(
            "DefaultsModel",
            "<script setup lang=\"ts\">\nconst props = withDefaults(defineProps<{ size?: number; tag?: string }>(), { size: 2 as number, tag: 'p' })\nconst model = defineModel<string>({ default: '' as string })\nconst [title, modifiers] = defineModel<string>('title')\n</script>\n<template><component :is=\"props.tag\"><input v-model=\"model\" /><input v-model=\"title\" />{{ props.size }} {{ modifiers }}</component></template>\n",
        ),
        case(
            "CssModules",
            "<script setup>\nconst active = true\n</script>\n<template><p :class=\"[$style.box, theme.dark]\">{{ active }}</p></template>\n<style module>\n.box { color: red }\n</style>\n<style module=\"theme\">\n.dark { color: black }\n</style>\n",
        ),
        case(
            "CssModuleShadowed",
            "<script setup>\nconst theme = { dark: 'local' }\n</script>\n<template><p :class=\"theme.dark\">x</p></template>\n<style module=\"theme\">\n.dark { color: black }\n</style>\n",
        ),
        case(
            "DemotionOrder",
            "<script setup>\nimport { reactive } from 'vue'\nconst first = reactive({ text: '' })\nconst second = reactive({ text: '' })\n</script>\n<template><input v-model=\"first\" /><input v-model=\"second\" /><div v-if=\"first\" v-for=\"item in [1]\" :key=\"item\">{{ item }}</div></template>\n<style scoped>\n:deep { color: red }\n</style>\n",
        ),
        case(
            "VaporSetup",
            "<script setup vapor>\nimport { ref } from 'vue'\nconst count = ref(0)\n</script>\n<template><button @click=\"count++\">{{ count }}</button></template>\n",
        ),
        case(
            "RootMatch",
            "<script setup>\nconst result = { kind: 'ok', data: 'ready' }\n</script>\n<template v-match=\"result\"><p v-when=\"{ kind: 'ok', const data }\">{{ data }}</p><i v-when=\"_\">other</i></template>\n",
        ),
        case(
            "MissingType",
            "<script setup lang=\"ts\">\nimport type { Missing } from './does-not-exist'\ndefineProps<Missing>()\n</script>\n<template><p>x</p></template>\n",
        ),
    ];
    let errors = assert_targets_compile_apart(&files);
    for (index, [client, ssr]) in errors.iter().enumerate() {
        // Root v-match is refused without its experiment; a missing type
        // module fails every target alike.
        let refused = matches!(index, 8 | 9);
        assert_eq!(!client.is_empty(), refused, "{index}: {client:?}");
        assert_eq!(client, ssr, "{index}");
    }
}
