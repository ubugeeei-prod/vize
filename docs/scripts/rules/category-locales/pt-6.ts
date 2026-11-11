export const portugueseRules6: Record<string, readonly [string, string, string]> = {
  "type/await-thenable": [
    "Proibir await de valores que não são thenable.",
    "plainNumber devolve um número, portanto o await espera um valor que não é thenable.",
    "Promise.resolve(1) é thenable, portanto o await espera uma Promise real.",
  ],
  "type/no-array-delete": [
    "Proibir delete de elementos em valores semelhantes a array.",
    "delete numbers[0] remove um elemento e deixa um buraco em vez de reindexar o array.",
    "splice(0, 1) remove o primeiro elemento e mantém o array denso.",
  ],
  "type/no-base-to-string": [
    "Proibir a conversão em string de valores que recaem em Object.prototype.toString().",
    "plain é um objeto simples, então toString() recai em Object.prototype.toString().",
    "label já é uma string, então toString() usa a conversão de string.",
  ],
  "type/no-confusing-void-expression": [
    "Proibir expressões do tipo void em posições de valor confusas.",
    "logVoid() devolve void, mas o inicializador trata essa chamada como um valor.",
    "Chamar logVoid() como instrução usa o resultado void apenas pelo seu efeito.",
  ],
  "type/no-deprecated": [
    "Proibir o uso de declarações obsoletas.",
    "legacy está marcado com @deprecated, e a chamada ainda usa essa declaração.",
    "current não tem marca de obsolescência, então a chamada usa uma declaração suportada.",
  ],
  "type/no-duplicate-type-constituents": [
    "Proibir constituintes duplicados em uniões ou interseções.",
    "string aparece duas vezes na mesma união, então o segundo constituinte não acrescenta nada.",
    "O alias mantém um único constituinte string.",
  ],
  "type/no-for-in-array": [
    "Proibir for-in sobre valores semelhantes a array.",
    "for-in percorre as chaves de um array em vez dos valores dos elementos.",
    "for-of percorre os elementos numéricos do array.",
  ],
  "type/no-generated-empty-object-type": [
    "Proibir tipos de objeto vazios gerados por operações de tipo.",
    "Omit remove a única chave, então o alias resulta num tipo de objeto vazio.",
    "Omit remove a e ainda deixa b, então o alias não é um tipo de objeto vazio.",
  ],
  "type/no-implied-eval": [
    "Proibir APIs de execução dinâmica baseadas em string.",
    "setTimeout recebe uma string, que é executada como código dinâmico.",
    "setTimeout recebe uma função, então nenhuma string é avaliada.",
  ],
  "type/no-meaningless-void-operator": [
    "Proibir o operador void em expressões que já são void.",
    "logVoid() já é void, então o operador void não muda o resultado.",
    "getNumber() devolve uma Promise, então void descarta esse thenable de propósito.",
  ],
  "type/no-misused-promises": [
    "Proibir Promises em lugares que não foram feitos para tratá-las.",
    "forEach espera um callback void, mas o callback async devolve uma Promise.",
    "O callback não é async, então devolve void como forEach espera.",
  ],
  "type/no-misused-spread": [
    "Proibir espalhar valores em posições incompatíveis.",
    "Espalhar uma string num array coloca um valor numa posição incompatível.",
    "Espalhar um array noutro array mantém o tipo dos elementos compatível.",
  ],
  "type/no-mixed-enums": [
    "Proibir a mistura de membros string e numéricos num enum.",
    "O enum mistura um membro numérico com um membro string.",
    "Os dois membros são numéricos, então o enum tem um único domínio de valores.",
  ],
  "type/no-redundant-type-constituents": [
    "Proibir membros de uniões e interseções que não acrescentam ou substituem informação de tipo.",
    "unknown já cobre string, então o constituinte string não restringe a união.",
    "string e number são ambos constituintes úteis da união.",
  ],
  "type/no-unnecessary-boolean-literal-compare": [
    "Proibir comparações desnecessárias com literais booleanos.",
    "flag já é boolean, então compará-lo com true não acrescenta uma verificação.",
    "Devolver flag usa o valor booleano diretamente.",
  ],
  "type/no-unnecessary-condition": [
    "Proibir condições que são sempre verdadeiras, sempre falsas, never ou sempre nullish.",
    "value é string, então nunca é nullish e o fallback de ?? nunca executa.",
    "value pode ser undefined, então ?? fornece uma string quando o valor está ausente.",
  ],
  "type/no-unnecessary-template-expression": [
    "Proibir template literals que envolvem uma única expressão sem necessidade.",
    "O template contém apenas label, então não constrói uma string nova.",
    "Atribuir label usa a string sem um invólucro de template.",
  ],
  "type/no-unnecessary-type-arguments": [
    "Proibir argumentos de tipo iguais ao valor padrão.",
    "O argumento explícito number é igual ao parâmetro de tipo padrão.",
    "Chamar identity(1) deixa o parâmetro de tipo padrão valer.",
  ],
  "type/no-unnecessary-type-assertion": [
    "Proibir asserções de tipo que não mudam o tipo da expressão.",
    "known já é number, então afirmar number não muda o seu tipo.",
    "Usar known sem asserção mantém o tipo number anotado.",
  ],
  "type/no-unnecessary-type-conversion": [
    "Proibir conversões que não mudam o tipo nem o valor da expressão.",
    "label já é string, então String(label) não muda o seu tipo nem o seu valor.",
    "count é um número, então String(count) converte-o numa string.",
  ],
  "type/no-unsafe-argument": [
    "Proibir chamar uma função com um valor do tipo `any`.",
    "anyValue é any e é passado a um parâmetro que espera number.",
    "A chamada passa o número 1 ao parâmetro number.",
  ],
  "type/no-unsafe-assignment": [
    "Proibir atribuir valores any a alvos mais específicos.",
    "anyValue é any e é atribuído a uma ligação anotada como number.",
    "A ligação é inicializada com o número 1.",
  ],
  "type/no-unsafe-call": [
    "Proibir chamar valores tipados como any.",
    "anyValue tem tipo any, então chamá-lo não verifica o destino da chamada.",
    "plainNumber é uma função, então a chamada tem um destino conhecido.",
  ],
  "type/no-unsafe-enum-assignment": [
    "Proibir atribuir um valor que não é do enum a um enum.",
    "A ligação é anotada como Color, mas é inicializada com o número 1.",
    "Color.Red é um membro do enum Color.",
  ],
  "type/no-unsafe-enum-comparison": [
    "Proibir comparar um valor de enum com um valor de outro domínio.",
    "value é number, então compará-lo com o membro Color.Red cruza domínios de valor.",
    "value é Color, então a comparação permanece dentro do enum Color.",
  ],
  "type/no-unsafe-member-access": [
    "Proibir acesso a membros de valores tipados como any.",
    "anyValue é any, então ler member é um acesso a propriedade sem verificação.",
    "box tem uma propriedade member conhecida, então o acesso é verificado.",
  ],
  "type/no-unsafe-return": [
    "Proibir devolver valores any a partir de funções.",
    "A função promete number, mas devolve anyValue, que é any.",
    "A função devolve o número 1, que corresponde ao tipo de retorno anotado.",
  ],
  "type/no-unsafe-unary-minus": [
    "Proibir a negação unária em valores que não são number nem bigint.",
    "label é string, então o menos unário não nega um number nem um bigint.",
    "count é um número, então o menos unário nega esse número.",
  ],
  "type/only-throw-error": [
    "Exigir que os valores lançados sejam semelhantes a Error.",
    "O valor lançado é uma string, que não é um Error.",
    "new Error cria um Error para o mesmo texto de falha.",
  ],
  "type/prefer-promise-reject-errors": [
    "Exigir que o motivo de rejeição de uma Promise seja semelhante a Error.",
    "Promise.reject recebe uma string, que não é um motivo Error.",
    "Promise.reject recebe um Error para o mesmo texto de falha.",
  ],
  "type/prefer-reduce-type-parameter": [
    "Preferir o parâmetro de tipo de Array#reduce a uma asserção do valor inicial.",
    "O valor inicial é afirmado como number[] em vez de usar o parâmetro de tipo de reduce.",
    "reduce<number[]> declara o tipo do acumulador sem afirmar o array inicial.",
  ],
  "type/prefer-return-this-type": [
    "Exigir `this` quando o método devolve apenas o tipo `this`.",
    "setValue devolve this, mas o tipo de retorno declarado é o nome da classe Builder.",
    "O tipo de retorno this corresponde ao valor que o método devolve.",
  ],
  "type/related-getter-setter-pairs": [
    "Exigir que getters e setters emparelhados usem tipos compatíveis.",
    "O getter devolve string enquanto o setter aceita number, então o par é incompatível.",
    "O getter e o setter usam ambos string.",
  ],
  "type/require-await": [
    "Proibir funções async que não contêm uma expressão await.",
    "noAwait é async, mas o corpo nunca faz await de um valor.",
    "withAwait faz await de Promise.resolve(1) antes de o devolver.",
  ],
  "type/restrict-plus-operands": [
    "Exigir que os operandos de + sejam explicitamente compatíveis.",
    "label é string e numbers é um array, então os dois operandos de + não são primitivos compatíveis.",
    "label e extra são ambos string, então os operandos de + coincidem.",
  ],
  "type/restrict-template-expressions": [
    "Exigir que as expressões de um template literal sejam compatíveis com string.",
    "O template interpola um array, que não é uma expressão compatível com string.",
    "O template interpola label, que é uma string.",
  ],
  "type/return-await": [
    "Exigir o uso consistente de `return await` conforme o contexto e a configuração.",
    "Dentro de try, devolver getNumber() sem await perde a rejeição nesse catch.",
    "return await getNumber() mantém a rejeição dentro do mesmo try e catch.",
  ],
  "type/unbound-method": [
    "Proibir referências a métodos sem o seu recetor (`this`).",
    "counter.increment extrai o método sem o recetor, então this perde-se quando ele é chamado.",
    "counter.increment() chama o método em counter, então this permanece ligado.",
  ],
  "type/use-unknown-in-catch-callback-variable": [
    "Exigir que a variável do callback de catch de uma Promise tenha uma anotação unknown explícita.",
    "O callback de catch nomeia error sem uma anotação unknown explícita.",
    "O callback de catch anota error como unknown.",
  ],
};
