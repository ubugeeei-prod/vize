export const chineseRules6: Record<string, readonly [string, string, string]> = {
  "type/await-thenable": [
    "禁止 await 非 thenable 的值。",
    "plainNumber 返回数字，因此 await 等待的不是 thenable 值。",
    "Promise.resolve(1) 是 thenable，await 等待的是真正的 Promise。",
  ],
  "type/no-array-delete": [
    "禁止对类数组值使用 delete 删除元素。",
    "delete numbers[0] 删除数组元素并留下空洞，而不会重新排列其余元素。",
    "splice(0, 1) 删除第一个元素，并保持数组连续。",
  ],
  "type/no-base-to-string": [
    "禁止把会回退到 Object.prototype.toString() 的值转换成字符串。",
    "plain 是普通对象，因此 toString() 会回退到 Object.prototype.toString()。",
    "label 已经是字符串，因此 toString() 使用字符串自身的转换。",
  ],
  "type/no-confusing-void-expression": [
    "禁止在容易混淆的值位置使用 void 类型的表达式。",
    "logVoid() 返回 void，但初始化语句把这次调用当作一个值。",
    "把 logVoid() 写成语句后，void 结果只用于其副作用。",
  ],
  "type/no-deprecated": [
    "禁止使用已弃用的声明。",
    "legacy 标有 @deprecated，调用仍然使用这个声明。",
    "current 没有弃用标记，因此调用使用的是仍受支持的声明。",
  ],
  "type/no-duplicate-type-constituents": [
    "禁止联合类型或交叉类型中的重复成员。",
    "string 在同一个联合类型中出现两次，第二个成员没有增加任何信息。",
    "类型别名只保留一个 string 成员。",
  ],
  "type/no-for-in-array": [
    "禁止对类数组值使用 for-in 遍历。",
    "for-in 遍历的是数组的键，而不是元素值。",
    "for-of 遍历数组中的数值元素。",
  ],
  "type/no-generated-empty-object-type": [
    "禁止由类型运算生成的空对象类型。",
    "Omit 去掉了唯一的键，因此别名会变成空对象类型。",
    "Omit 去掉 a 后仍保留 b，因此别名不是空对象类型。",
  ],
  "type/no-implied-eval": [
    "禁止基于字符串的动态代码执行 API。",
    "setTimeout 接收字符串，这段字符串会作为动态代码执行。",
    "setTimeout 接收函数，因此不会求值字符串。",
  ],
  "type/no-meaningless-void-operator": [
    "禁止对已经是 void 的表达式使用 void 运算符。",
    "logVoid() 已经是 void，因此 void 运算符不会改变结果。",
    "getNumber() 返回 Promise，因此 void 是有意丢弃这个 thenable。",
  ],
  "type/no-misused-promises": [
    "禁止在并非为处理 Promise 而设计的位置使用 Promise。",
    "forEach 期望回调返回 void，但 async 回调返回 Promise。",
    "回调不是 async，因此它按 forEach 的期望返回 void。",
  ],
  "type/no-misused-spread": [
    "禁止把值展开到不兼容的位置。",
    "把字符串展开到数组中，是把值放进不兼容的位置。",
    "把数组展开到数组中，元素类型保持兼容。",
  ],
  "type/no-mixed-enums": [
    "禁止在枚举中混用字符串成员和数值成员。",
    "枚举同时包含数值成员和字符串成员。",
    "两个成员都是数值，因此枚举只有一种值域。",
  ],
  "type/no-redundant-type-constituents": [
    "禁止联合类型和交叉类型中不起作用或覆盖类型信息的成员。",
    "unknown 已经包含 string，因此 string 成员不会收窄这个联合类型。",
    "string 和 number 都是这个联合类型中有意义的成员。",
  ],
  "type/no-unnecessary-boolean-literal-compare": [
    "禁止与布尔字面量进行不必要的比较。",
    "flag 已经是布尔值，因此与 true 比较不会增加检查。",
    "直接返回 flag 就是在使用这个布尔值。",
  ],
  "type/no-unnecessary-condition": [
    "禁止始终为真、始终为假、never 或始终为 nullish 的条件。",
    "value 是字符串，因此它不会是 nullish，?? 的后备值不会执行。",
    "value 可能是 undefined，因此缺失时 ?? 会补上字符串。",
  ],
  "type/no-unnecessary-template-expression": [
    "禁止用不必要的模板字符串只包裹一个表达式。",
    "模板里只有 label，因此它并没有拼接出新的字符串。",
    "直接把 label 赋给变量，不需要再用模板包裹。",
  ],
  "type/no-unnecessary-type-arguments": [
    "禁止与默认值相同的类型参数。",
    "显式写出的 number 与类型参数的默认值相同。",
    "调用 identity(1) 时会使用类型参数的默认值。",
  ],
  "type/no-unnecessary-type-assertion": [
    "禁止不改变表达式类型的类型断言。",
    "known 已经是 number，因此再断言为 number 不会改变它的类型。",
    "直接使用 known，保留已注明的 number 类型。",
  ],
  "type/no-unnecessary-type-conversion": [
    "禁止不改变表达式类型或值的转换。",
    "label 已经是字符串，因此 String(label) 不会改变它的类型或值。",
    "count 是数字，因此 String(count) 会把它转换成字符串。",
  ],
  "type/no-unnecessary-type-parameters": [
    "禁止在签名中只使用一次的类型参数。",
    "T 在 logValue 的签名中只出现一次，因此没有关联任何类型。",
    "把 value 声明为 unknown，无需类型参数即可接受相同的参数。",
  ],
  "type/no-unsafe-argument": [
    "禁止用 `any` 类型的值调用函数。",
    "anyValue 的类型是 any，它被传给了期望 number 的参数。",
    "调用把数字 1 传给 number 参数。",
  ],
  "type/no-unsafe-assignment": [
    "禁止把 any 类型的值赋给更具体的目标。",
    "anyValue 的类型是 any，它被赋给了注明为 number 的绑定。",
    "这个绑定用数字 1 初始化。",
  ],
  "type/no-unsafe-call": [
    "禁止调用类型为 any 的值。",
    "anyValue 的类型是 any，因此调用它时无法检查被调用者。",
    "plainNumber 是函数，因此调用目标是明确的。",
  ],
  "type/no-unsafe-enum-assignment": [
    "禁止把非枚举值赋给枚举。",
    "绑定注明为 Color，却用数字 1 初始化。",
    "Color.Red 是 Color 枚举的成员。",
  ],
  "type/no-unsafe-enum-comparison": [
    "禁止把枚举值与另一枚举域的值比较。",
    "value 是数字，因此把它和枚举成员 Color.Red 比较跨越了值域。",
    "value 是 Color，因此比较停留在 Color 枚举之内。",
  ],
  "type/no-unsafe-member-access": [
    "禁止访问类型为 any 的值的成员。",
    "anyValue 的类型是 any，因此读取 member 是未经检查的属性访问。",
    "box 具有已知的 member 属性，因此这次访问会受到检查。",
  ],
  "type/no-unsafe-return": [
    "禁止从函数返回 any 类型的值。",
    "函数声明返回 number，但返回的 anyValue 是 any。",
    "函数返回数字 1，与注明的返回类型一致。",
  ],
  "type/no-unsafe-unary-minus": [
    "禁止对非 number 且非 bigint 的值使用一元负号。",
    "label 是字符串，因此一元负号并没有作用在 number 或 bigint 上。",
    "count 是数字，因此一元负号否定的是这个数字。",
  ],
  "type/only-throw-error": [
    "要求抛出的值是 Error 一类的值。",
    "抛出的值是字符串，不是 Error。",
    "new Error 为同一段失败文本创建一个 Error。",
  ],
  "type/prefer-promise-reject-errors": [
    "要求 Promise 的拒绝原因是 Error 一类的值。",
    "Promise.reject 接收字符串，这不是 Error 原因。",
    "Promise.reject 接收的是同一段失败文本对应的 Error。",
  ],
  "type/prefer-reduce-type-parameter": [
    "推荐使用 Array#reduce 的类型参数，而不是对初始值做断言。",
    "初始值被断言为 number[]，而没有使用 reduce 的类型参数。",
    "reduce<number[]> 直接声明累加器类型，不再断言初始数组。",
  ],
  "type/prefer-return-this-type": [
    "要求在只返回 `this` 类型时使用 `this`。",
    "setValue 返回 this，但声明的返回类型是类名 Builder。",
    "返回类型 this 与方法实际返回的值一致。",
  ],
  "type/related-getter-setter-pairs": [
    "要求成对的 getter 和 setter 使用兼容的类型。",
    "getter 返回 string，setter 却接受 number，因此这一对类型不兼容。",
    "getter 和 setter 都使用 string。",
  ],
  "type/require-await": [
    "禁止不包含 await 表达式的 async 函数。",
    "noAwait 是 async，但它的函数体从未 await 任何值。",
    "withAwait 在返回之前 await 了 Promise.resolve(1)。",
  ],
  "type/restrict-plus-operands": [
    "要求加号的操作数明确兼容。",
    "label 是字符串，numbers 是数组，因此加号两边不是兼容的原始值。",
    "label 和 extra 都是字符串，因此加号两边的类型一致。",
  ],
  "type/restrict-template-expressions": [
    "要求模板字符串中的表达式与字符串兼容。",
    "模板插入了数组，这不是与字符串兼容的表达式。",
    "模板插入的 label 是字符串。",
  ],
  "type/return-await": [
    "要求根据上下文和配置一致地使用 `return await`。",
    "在 try 中不 await 就返回 getNumber()，这个 catch 接收不到拒绝原因。",
    "return await getNumber() 会把拒绝留在同一个 try 和 catch 中。",
  ],
  "type/unbound-method": [
    "禁止在没有接收者（`this`）的情况下引用方法。",
    "counter.increment 取下了方法而没有带上接收者，之后调用时 this 会丢失。",
    "counter.increment() 在 counter 上调用方法，因此 this 仍然绑定。",
  ],
  "type/use-unknown-in-catch-callback-variable": [
    "要求 Promise 的 catch 回调参数使用显式的 unknown 注解。",
    "catch 回调给出了 error，却没有显式的 unknown 注解。",
    "catch 回调把 error 注明为 unknown。",
  ],
};
