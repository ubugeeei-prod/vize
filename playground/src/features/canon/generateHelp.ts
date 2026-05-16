/**
 * Generate help suggestions based on TypeScript error code and message.
 *
 * Maps common TypeScript error codes to human-friendly explanations
 * with Vue-specific context and fix suggestions.
 */
export function generateHelp(code: number, message: string): string | undefined {
  // Common TypeScript errors with helpful suggestions
  switch (code) {
    // ========== Name Resolution Errors ==========
    case 2304: {
      // Cannot find name 'X'
      const nameMatch = message.match(/Cannot find name '(\w+)'/);
      if (nameMatch) {
        const name = nameMatch[1];
        if (name.startsWith("$")) {
          return `**\`${name}\`** is a Vue template variable.\n\n**Why:** Template variables like \`$event\`, \`$refs\`, \`$slots\` are only available in \`<template>\`.\n\n**Fix:** Use it inside template, or access via script APIs:\n\`\`\`ts\n// In <script setup>\nconst slots = useSlots()\nconst attrs = useAttrs()\n\`\`\``;
        }
        if (
          [
            "ref",
            "reactive",
            "computed",
            "watch",
            "watchEffect",
            "onMounted",
            "onUnmounted",
            "toRef",
            "toRefs",
          ].includes(name)
        ) {
          return `**\`${name}\`** is a Vue Composition API function.\n\n**Fix:** Import from \`vue\`:\n\`\`\`ts\nimport { ${name} } from 'vue'\n\n// Usage example\nconst count = ref(0)\nconst doubled = computed(() => count.value * 2)\n\`\`\``;
        }
        return `**\`${name}\`** is not defined.\n\n**Possible causes:**\n- Not imported from a module\n- Not declared in this scope\n- Typo in the name\n\n**Fix options:**\n\`\`\`ts\n// 1. Import from module\nimport { ${name} } from './module'\n\n// 2. Declare locally\nconst ${name} = someValue\n\n// 3. Import from package\nimport { ${name} } from 'package-name'\n\`\`\``;
      }
      break;
    }
    case 2552: {
      // Cannot find name 'X'. Did you mean 'Y'?
      const meanMatch = message.match(/Did you mean '(\w+)'/);
      if (meanMatch) {
        return `**Typo detected.**\n\n**Suggestion:** Did you mean **\`${meanMatch[1]}\`**?\n\n**Fix:**\n\`\`\`ts\n// Change to:\n${meanMatch[1]}\n\`\`\``;
      }
      break;
    }

    // ========== Type Mismatch Errors ==========
    case 2322: {
      // Type 'X' is not assignable to type 'Y'
      const typeMatch = message.match(/Type '(.+?)' is not assignable to type '(.+?)'/);
      if (typeMatch) {
        const [, fromType, toType] = typeMatch;
        if (fromType === "string" && toType === "number") {
          return `**Type mismatch:** \`string\` cannot be assigned to \`number\`.\n\n**Why:** TypeScript requires explicit type conversion.\n\n**Fix options:**\n\`\`\`ts\n// parseInt for integers\nconst num = parseInt(str, 10)\n\n// parseFloat for decimals\nconst num = parseFloat(str)\n\n// Number constructor\nconst num = Number(str)\n\n// With fallback for NaN\nconst num = Number(str) || 0\n\n// Unary plus (shortest)\nconst num = +str\n\`\`\``;
        }
        if (fromType === "number" && toType === "string") {
          return `**Type mismatch:** \`number\` cannot be assigned to \`string\`.\n\n**Fix options:**\n\`\`\`ts\n// String constructor\nconst str = String(num)\n\n// toString method\nconst str = num.toString()\n\n// Template literal (recommended)\nconst str = \`\${num}\`\n\n// With formatting\nconst str = num.toFixed(2) // "123.45"\n\`\`\``;
        }
        return `**Type mismatch:** \`${fromType}\` cannot be assigned to \`${toType}\`.\n\n**Fix options:**\n\`\`\`ts\n// 1. Fix the value to match expected type\nconst value: ${toType} = correctValue\n\n// 2. Type assertion (if you're sure)\nconst value = someValue as ${toType}\n\n// 3. Update type definition to accept both\ntype MyType = ${fromType} | ${toType}\n\`\`\``;
      }
      return `**Type mismatch.** The value type doesn't match the expected type.\n\n**Fix:** Check the type definition and ensure the value matches.`;
    }
    case 2345: {
      // Argument type mismatch
      const argMatch = message.match(
        /Argument of type '(.+?)' is not assignable to parameter of type '(.+?)'/,
      );
      if (argMatch) {
        return `**Argument type mismatch.**\n\n**Expected:** \`${argMatch[2]}\`\n**Received:** \`${argMatch[1]}\`\n\n**Fix options:**\n\`\`\`ts\n// 1. Convert the argument\nfunc(convertedValue)\n\n// 2. Type assertion (if compatible)\nfunc(value as ${argMatch[2]})\n\n// 3. Update function to accept the type\nfunction func(param: ${argMatch[1]} | ${argMatch[2]}) { }\n\`\`\``;
      }
      return `**Argument type mismatch.** The argument doesn't match the function parameter type.\n\n**Fix:** Check the function signature and convert the argument if needed.`;
    }
    case 2349: // This expression is not callable
      return `**Expression is not callable.**\n\n**Why:** You're trying to call something that isn't a function.\n\n**Common causes:**\n- Value is \`undefined\` or \`null\`\n- It's an object or primitive, not a function\n- Property returns a value, not a method\n\n**Fix options:**\n\`\`\`ts\n// 1. Check if it's a function first\nif (typeof maybeFunc === 'function') {\n  maybeFunc()\n}\n\n// 2. Use optional chaining for optional calls\nmaybeFunc?.()\n\n// 3. Provide default function\nconst fn = maybeFunc ?? (() => {})\nfn()\n\`\`\``;

    // ========== Property Access Errors ==========
    case 2339: {
      // Property 'X' does not exist on type 'Y'
      const propMatch = message.match(/Property '(\w+)' does not exist on type '(.+?)'/);
      if (propMatch) {
        const [, prop, type] = propMatch;
        if (type.includes("Ref<")) {
          return `**Ref access error.**\n\n**Why:** \`Ref\` wraps the value in \`.value\` property.\n\n**Fix:** Access through \`.value\`:\n\`\`\`ts\n// Wrong\nmyRef.${prop}\n\n// Correct\nmyRef.value.${prop}\n\n// In template (auto-unwrapped)\n{{ myRef.${prop} }}\n\`\`\``;
        }
        return `**Property \`${prop}\` doesn't exist** on type \`${type}\`.\n\n**Possible causes:**\n- Typo in property name\n- Property not defined in type\n- Accessing wrong object\n\n**Fix options:**\n\`\`\`ts\n// 1. Check if property exists\nif ('${prop}' in obj) {\n  obj.${prop}\n}\n\n// 2. Optional chaining (returns undefined if missing)\nobj?.${prop}\n\n// 3. Extend the type definition\ninterface Extended extends Original {\n  ${prop}: SomeType\n}\n\n// 4. Index signature access\nobj['${prop}']\n\`\`\``;
      }
      break;
    }
    case 2551: {
      // Property 'X' does not exist. Did you mean 'Y'?
      const suggestMatch = message.match(/Did you mean '(\w+)'/);
      if (suggestMatch) {
        return `**Typo detected in property name.**\n\n**Suggestion:** Did you mean **\`${suggestMatch[1]}\`**?\n\n**Fix:**\n\`\`\`ts\n// Change to:\nobj.${suggestMatch[1]}\n\`\`\``;
      }
      break;
    }

    // ========== Null/Undefined Errors ==========
    case 2532: // Object is possibly 'undefined'
      return `**Value may be \`undefined\`.**\n\n**Why:** TypeScript detected this value could be \`undefined\` at runtime.\n\n**Fix options:**\n\`\`\`ts\n// 1. Optional chaining (safe access)\nobj?.property\nobj?.method()\n\n// 2. Nullish coalescing (provide default)\nconst value = obj ?? defaultValue\n\n// 3. Explicit undefined check\nif (obj !== undefined) {\n  obj.property // OK, obj is defined here\n}\n\n// 4. Non-null assertion (only if you're 100% sure)\nobj!.property // Tells TS "trust me, it's defined"\n\`\`\``;
    case 2531: // Object is possibly 'null'
      return `**Value may be \`null\`.**\n\n**Why:** TypeScript detected this value could be \`null\` at runtime.\n\n**Fix options:**\n\`\`\`ts\n// 1. Optional chaining\nobj?.property\n\n// 2. Nullish coalescing\nconst value = obj ?? defaultValue\n\n// 3. Explicit null check\nif (obj !== null) {\n  obj.property // OK\n}\n\n// 4. Combined check\nif (obj != null) { // checks both null and undefined\n  obj.property\n}\n\`\`\``;
    case 2533: // Object is possibly 'null' or 'undefined'
      return `**Value may be \`null\` or \`undefined\`.**\n\n**Fix:**\n\`\`\`ts\n// Optional chaining (recommended)\nobj?.property\nobj?.method?.()\n\n// With default value\nconst value = obj?.property ?? 'default'\n\n// Explicit check\nif (obj) {\n  obj.property // OK\n}\n\`\`\``;
    case 18048: // 'X' is possibly 'undefined'
      return `**Value may be \`undefined\`.**\n\n**Fix options:**\n\`\`\`ts\n// 1. Provide default value\nconst value = maybeUndefined ?? 'default'\n\n// 2. Initialize with value\nconst data = ref<string>('initial') // Not undefined\n\n// 3. Check before use\nif (value !== undefined) {\n  // use value safely\n}\n\n// 4. Array methods with fallback\nconst first = arr[0] ?? defaultItem\n\`\`\``;

    // ========== Type Unknown/Any Errors ==========
    case 2571: // Object is of type 'unknown'
      return `**Type is \`unknown\`.**\n\n**Why:** \`unknown\` is the type-safe version of \`any\`. You must narrow the type before using it.\n\n**Fix options:**\n\`\`\`ts\n// 1. typeof type guard\nif (typeof value === 'string') {\n  value.toUpperCase() // OK, value is string\n}\nif (typeof value === 'number') {\n  value.toFixed(2) // OK, value is number\n}\n\n// 2. instanceof check\nif (value instanceof Error) {\n  value.message // OK\n}\n\n// 3. Custom type guard\nfunction isUser(v: unknown): v is User {\n  return (\n    typeof v === 'object' &&\n    v !== null &&\n    'name' in v &&\n    'email' in v\n  )\n}\nif (isUser(value)) {\n  value.name // OK\n}\n\n// 4. Type assertion (less safe)\nconst user = value as User\n\`\`\``;
    case 7006: {
      // Parameter implicitly has an 'any' type
      const paramMatch = message.match(/Parameter '(\w+)'/);
      const paramName = paramMatch ? paramMatch[1] : "param";
      return `**Parameter \`${paramName}\` needs a type.**\n\n**Why:** TypeScript cannot infer the type and defaults to \`any\`.\n\n**Fix options:**\n\`\`\`ts\n// 1. Add explicit type annotation\nfunction example(${paramName}: string) {\n  return ${paramName}.toUpperCase()\n}\n\n// 2. Arrow function with type\nconst fn = (${paramName}: number) => ${paramName} * 2\n\n// 3. Default value (type inferred)\nfunction example(${paramName} = 'default') {\n  // ${paramName} is inferred as string\n}\n\n// 4. Object parameter with type\nfunction example({ ${paramName} }: { ${paramName}: string }) { }\n\`\`\``;
    }
    case 7031: // Binding element implicitly has an 'any' type
      return `**Destructured value needs a type.**\n\n**Why:** TypeScript cannot infer types in destructuring patterns.\n\n**Fix options:**\n\`\`\`ts\n// 1. Type the entire pattern\nconst { name, age }: { name: string; age: number } = obj\n\n// 2. Use an interface\ninterface Person {\n  name: string\n  age: number\n}\nconst { name, age }: Person = obj\n\n// 3. Function parameter destructuring\nfunction greet({ name, age }: Person) {\n  console.log(\`\${name} is \${age}\`)\n}\n\n// 4. With Vue defineProps\nconst { title, count } = defineProps<{\n  title: string\n  count: number\n}>()\n\`\`\``;
    case 7005: // Variable implicitly has an 'any' type
      return `**Variable type is implicitly \`any\`.**\n\n**Why:** TypeScript couldn't infer the type.\n\n**Fix options:**\n\`\`\`ts\n// 1. Add explicit type\nlet value: string\nlet items: number[]\nlet user: User | null = null\n\n// 2. Initialize with value (type inferred)\nlet value = 'hello' // string\nlet count = 0 // number\n\n// 3. Empty array with type\nconst items: string[] = []\nconst map: Map<string, number> = new Map()\n\n// 4. Generic type parameters\nconst ref = ref<User | null>(null)\n\`\`\``;

    // ========== Function Errors ==========
    case 2554: {
      // Expected X arguments, but got Y
      const argMatch = message.match(/Expected (\d+) arguments?, but got (\d+)/);
      if (argMatch) {
        const [, expected, got] = argMatch;
        const expectedNum = parseInt(expected);
        const gotNum = parseInt(got);

        // Vue event handler pattern: Expected 0 arguments, but got 1
        // This happens when @click="handler" passes $event but handler takes no args
        if (expectedNum === 0 && gotNum === 1) {
          return `**Event handler argument mismatch.**\n\n**Why:** In Vue templates, \`@click="handler"\` automatically passes the event object (\`$event\`) as the first argument. But your function expects 0 arguments.\n\n**Fix options:**\n\`\`\`ts\n// Option 1: Call the function explicitly (don't pass event)\n// @click="handler()"\n<button @click="handler()">Click</button>\n\n// Option 2: Use arrow function wrapper\n// @click="() => handler()"\n<button @click="() => handler()">Click</button>\n\n// Option 3: Accept the event parameter\nfunction handler(event?: Event) {\n  // event is optional, use if needed\n}\n// Then @click="handler" works\n\`\`\`\n\n**Note:** \`@click="handler"\` is equivalent to \`@click="handler($event)"\``;
        }

        // General case: wrong number of arguments
        return `**Wrong number of arguments.**\n\n**Expected:** ${expected} argument(s)\n**Provided:** ${got} argument(s)\n\n**Fix:**\n\`\`\`ts\n// Check the function signature\nfunction example(a: string, b: number, c?: boolean) {\n  // a, b are required\n  // c is optional\n}\n\n// Call with correct arguments\nexample('hello', 42) // OK\nexample('hello', 42, true) // OK\n\`\`\``;
      }
      break;
    }
    case 2555: {
      // Expected at least X arguments, but got Y
      const argMatch = message.match(/Expected at least (\d+) arguments?, but got (\d+)/);
      if (argMatch) {
        return `**Not enough arguments.**\n\n**Required:** at least ${argMatch[1]} argument(s)\n**Provided:** ${argMatch[2]} argument(s)\n\n**Fix:** Provide all required arguments:\n\`\`\`ts\n// Function with required and optional params\nfunction example(required1: string, required2: number, optional?: boolean) { }\n\n// Must provide at least required params\nexample('hello', 42) // OK\n\`\`\``;
      }
      return `**Not enough arguments.** Check the function signature for required parameters.`;
    }
    case 2556: // A spread argument must either have a tuple type
      return `**Spread argument type error.**\n\n**Why:** TypeScript needs to know the exact types when spreading.\n\n**Fix options:**\n\`\`\`ts\n// 1. Use tuple type\nconst args: [string, number] = ['hello', 42]\nfunc(...args) // OK\n\n// 2. Use 'as const' for literal types\nconst args = ['hello', 42] as const\nfunc(...args) // OK\n\n// 3. Type assertion\nconst args = ['hello', 42] as [string, number]\nfunc(...args)\n\n// 4. Rest parameters in function\nfunction func(...args: [string, number]) { }\n\`\`\``;

    // ========== Module/Import Errors ==========
    case 2307: {
      // Cannot find module 'X'
      const modMatch = message.match(/Cannot find module '([^']+)'/);
      const modName = modMatch ? modMatch[1] : "module";
      const pkgName = modName.startsWith(".") ? null : modName.split("/")[0];
      return `**Module not found:** \`${modName}\`\n\n**Fix options:**\n\`\`\`ts\n// 1. Install the package${pkgName ? `\n// npm install ${pkgName}` : ""}\n\n// 2. Install type definitions${pkgName ? `\n// npm install -D @types/${pkgName}` : ""}\n\n// 3. For local modules, check path\nimport { something } from './correct/path'\n\n// 4. Add module declaration\ndeclare module '${modName}' {\n  export const value: string\n}\n\`\`\``;
    }
    case 2306: // 'X' is not a module
      return `**File is not a module.**\n\n**Why:** This file doesn't have any exports.\n\n**Fix:** Add exports to the file:\n\`\`\`ts\n// Named exports\nexport const myValue = 'hello'\nexport function myFunc() { }\nexport interface MyType { }\n\n// Default export\nexport default MyComponent\n\n// Re-export from another module\nexport { something } from './other'\nexport * from './utils'\n\`\`\``;
    case 2614: {
      // Module 'X' has no exported member 'Y'
      const exportMatch = message.match(/has no exported member '(\w+)'/);
      if (exportMatch) {
        const name = exportMatch[1];
        return `**Export \`${name}\` not found** in the module.\n\n**Possible causes:**\n- Typo in import name\n- Using named import for default export\n- Export doesn't exist in this version\n\n**Fix options:**\n\`\`\`ts\n// 1. Check available exports\nimport { /* see available */ } from 'module'\n\n// 2. Maybe it's a default export?\nimport ${name} from 'module'\n\n// 3. Import all and access\nimport * as Module from 'module'\nModule.${name}\n\`\`\``;
      }
      return `**Export not found.** Check the module's available exports.`;
    }
    case 2792: // Cannot find module. Did you mean to set moduleResolution?
      return `**Module resolution configuration error.**\n\n**Fix:** Update \`tsconfig.json\`:\n\`\`\`ts\n// tsconfig.json\n{\n  "compilerOptions": {\n    // For Vite/modern bundlers\n    "moduleResolution": "bundler",\n    \n    // For Node.js ESM\n    "moduleResolution": "node16",\n    \n    // Legacy Node.js\n    "moduleResolution": "node"\n  }\n}\n\`\`\``;

    // ========== Vue Specific ==========
    case 2769: // No overload matches this call
      return `**No matching function signature.**\n\n**Why:** The arguments don't match any overload of this function.\n\n**For Vue components, check props:**\n\`\`\`ts\n// Define props with correct types\nconst props = defineProps<{\n  // Required prop\n  title: string\n  // Optional prop\n  count?: number\n  // Prop with default\n  enabled?: boolean\n}>()\n\n// Usage in parent\n<MyComponent\n  title="Hello"       // Required\n  :count="5"          // Optional number\n  :enabled="true"     // Optional boolean\n/>\n\`\`\`\n\n**For functions, check the signature:**\n\`\`\`ts\n// Multiple overloads\nfunction process(value: string): string\nfunction process(value: number): number\nfunction process(value: string | number) {\n  return value\n}\n\`\`\``;

    // ========== Misc Errors ==========
    case 1005: // ';' expected
      return `**Syntax error:** Semicolon \`;\` expected.\n\n**Common causes:**\n- Missing semicolon at end of statement\n- Unclosed bracket or parenthesis\n- Invalid syntax before this point\n\n**Fix:** Check the line above for syntax issues.`;
    case 1109: // Expression expected
      return `**Syntax error:** Expression expected.\n\n**Common causes:**\n- Incomplete statement\n- Extra comma or operator\n- Missing value in assignment\n\n**Fix:**\n\`\`\`ts\n// Wrong\nconst x = \nconst y = ,value\n\n// Correct\nconst x = value\nconst y = value\n\`\`\``;
    case 1128: // Declaration or statement expected
      return `**Syntax error:** Declaration or statement expected.\n\n**Common causes:**\n- Code outside of function/class body\n- Missing closing brace \`}\`\n- Invalid top-level code`;
    case 2365: // Operator cannot be applied
      return `**Invalid operator usage.**\n\n**Why:** This operator doesn't work with these types.\n\n**Fix:**\n\`\`\`ts\n// Wrong: comparing incompatible types\n'hello' > 5 // Error\n\n// Fix: convert to same type\nNumber('5') > 5 // OK\n'5'.localeCompare('10') // For string comparison\n\n// Wrong: arithmetic on non-numbers\n'a' + 1 // Results in 'a1' (concatenation)\n\n// Fix: ensure numeric operations\nNumber('5') + 1 // 6\n\`\`\``;
    case 2448: // Block-scoped variable already declared
      return `**Duplicate variable declaration.**\n\n**Why:** A variable with this name already exists in this scope.\n\n**Fix:**\n\`\`\`ts\n// Wrong: duplicate declaration\nconst value = 1\nconst value = 2 // Error!\n\n// Fix: use different names\nconst value = 1\nconst value2 = 2\n\n// Or reassign (with let)\nlet value = 1\nvalue = 2 // OK\n\`\`\``;
    case 2451: // Cannot redeclare block-scoped variable
      return `**Cannot redeclare variable.**\n\n**Why:** \`let\` and \`const\` create block-scoped variables that can't be redeclared.\n\n**Fix:**\n\`\`\`ts\n// Wrong\nlet value = 1\nlet value = 2 // Error!\n\n// Fix 1: Reassign instead\nlet value = 1\nvalue = 2 // OK with let\n\n// Fix 2: Use different scope\n{\n  const value = 1\n}\n{\n  const value = 2 // OK, different block\n}\n\n// Fix 3: Different variable name\nconst value1 = 1\nconst value2 = 2\n\`\`\``;
  }
  return undefined;
}
