use vize_carton::source_anchor::SourceAnchor;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct ModuleSpan {
    pub start: u32,
    pub end: u32,
}

impl ModuleSpan {
    pub const fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ModuleLanguage {
    JavaScript,
    TypeScript,
    Jsx,
    Tsx,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ModuleDiagnostic {
    pub message: Box<str>,
    pub span: ModuleSpan,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ModuleImport {
    pub specifier: Box<str>,
    pub locals: Vec<Box<str>>,
    pub bindings: Vec<ModuleImportBinding>,
    pub dynamic: bool,
    pub type_only: bool,
    pub span: ModuleSpan,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ModuleImportBindingKind {
    Default,
    Named,
    Namespace,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ModuleImportBinding {
    pub imported: Option<Box<str>>,
    pub local: Box<str>,
    pub kind: ModuleImportBindingKind,
    pub type_only: bool,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ModuleExport {
    pub source: Option<Box<str>>,
    pub names: Vec<Box<str>>,
    pub default: bool,
    pub type_only: bool,
    pub span: ModuleSpan,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ModuleDeclaration {
    pub name: Box<str>,
    pub span: ModuleSpan,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ModuleReference {
    pub name: Box<str>,
    pub span: ModuleSpan,
    pub resolved_declaration: Option<usize>,
    pub read: bool,
    pub write: bool,
    pub type_only: bool,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ModuleBindingKind {
    Const,
    Let,
    Var,
    Other,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ModulePattern {
    Identifier(Box<str>),
    Path(Vec<Box<str>>),
    Object(Vec<ModuleObjectBinding>),
    Array(Vec<Option<ModulePattern>>),
    Rest(Box<ModulePattern>),
    Assignment {
        binding: Box<ModulePattern>,
        default: Box<ModuleExpression>,
    },
    Unknown {
        text: Box<str>,
        span: ModuleSpan,
    },
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ModuleObjectBinding {
    pub key: Box<str>,
    pub value: ModulePattern,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ModuleLiteralKind {
    String,
    Boolean,
    Number,
    BigInt,
    Null,
    Template,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ModuleExpression {
    pub kind: ModuleExpressionKind,
    pub span: ModuleSpan,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ModuleExpressionKind {
    Identifier(Box<str>),
    Path(Vec<Box<str>>),
    Literal {
        kind: ModuleLiteralKind,
        text: Box<str>,
        value: Option<Box<str>>,
    },
    Call {
        callee: Box<ModuleExpression>,
        arguments: Vec<ModuleExpression>,
        type_arguments: Option<Box<str>>,
    },
    Object {
        properties: Vec<ModuleObjectProperty>,
    },
    Array(Vec<Option<ModuleExpression>>),
    Function {
        async_: bool,
        parameters: Vec<ModulePattern>,
    },
    Await(Box<ModuleExpression>),
    Spread(Box<ModuleExpression>),
    Unknown(Box<str>),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ModuleObjectProperty {
    pub key: Option<Box<str>>,
    pub value: ModuleExpression,
    pub spread: bool,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ModuleOperation {
    pub kind: ModuleOperationKind,
    pub span: ModuleSpan,
    pub function: Option<usize>,
    pub top_level: bool,
    pub after_await: bool,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ModuleOperationKind {
    Binding {
        kind: ModuleBindingKind,
        pattern: ModulePattern,
        initializer: Option<ModuleExpression>,
    },
    Assignment {
        target: ModulePattern,
        value: ModuleExpression,
    },
    Call(ModuleExpression),
    Return(Option<ModuleExpression>),
    Await(ModuleExpression),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ModuleFunction {
    pub id: usize,
    pub parent: Option<usize>,
    pub name: Option<Box<str>>,
    pub async_: bool,
    pub parameters: Vec<ModulePattern>,
    pub span: ModuleSpan,
    pub references: Vec<Box<str>>,
    pub local_bindings: Vec<Box<str>>,
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct ModuleOperations {
    pub operations: Vec<ModuleOperation>,
    pub functions: Vec<ModuleFunction>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ModuleInstructionKind {
    Operation,
    Condition,
    Iteration,
    Return,
    Throw,
    Break,
    Continue,
    Unreachable,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ModuleInstruction {
    pub kind: ModuleInstructionKind,
    pub span: Option<ModuleSpan>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ModuleBlock {
    pub instructions: Vec<ModuleInstruction>,
    pub span: Option<ModuleSpan>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ModuleEdgeKind {
    Normal,
    TrueBranch,
    FalseBranch,
    LoopBack,
    Return,
    Break,
    Continue,
    Exception,
    Function,
    Unreachable,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ModuleEdge {
    pub from: usize,
    pub to: usize,
    pub kind: ModuleEdgeKind,
    pub span: Option<ModuleSpan>,
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct ModuleCfg {
    pub entry: usize,
    pub blocks: Vec<ModuleBlock>,
    pub edges: Vec<ModuleEdge>,
}

#[derive(Debug, Clone)]
pub struct ModuleSyntax {
    pub name: Box<str>,
    pub source: Box<str>,
    pub language: ModuleLanguage,
    pub base_offset: u32,
    pub source_anchor: Option<SourceAnchor>,
    pub diagnostics: Vec<ModuleDiagnostic>,
    pub imports: Vec<ModuleImport>,
    pub exports: Vec<ModuleExport>,
    pub declarations: Vec<ModuleDeclaration>,
    pub references: Vec<ModuleReference>,
    pub operations: ModuleOperations,
    pub cfg: ModuleCfg,
}

#[derive(Debug, Clone, Default)]
pub struct ModuleDocument {
    pub modules: Vec<ModuleSyntax>,
}

impl ModuleDocument {
    pub fn from_module(module: ModuleSyntax) -> Self {
        Self {
            modules: vec![module],
        }
    }
}
