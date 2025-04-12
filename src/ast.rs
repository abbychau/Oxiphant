/// Location in source code
#[derive(Debug, Clone, PartialEq)]
pub struct Location {
    pub file: String,
    pub line: usize,
    pub column: usize,
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.file, self.line, self.column)
    }
}

/// PHP variable types
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Integer,
    Float,
    String,
    Boolean,
    Array,
    Null,
    Mixed, // For variables that could be any type (PHP is dynamically typed)
}

/// Binary operators
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    // Arithmetic
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,

    // Comparison
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,

    // Logical
    LogicalAnd,
    LogicalOr,

    // Assignment
    Assign,

    // String concatenation
    Concat,

    // Array access
    ArrayAccess,
}

/// Unary operators
#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Negate,
    LogicalNot,
}

/// AST nodes
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    // Program
    Program(Vec<Node>),

    // Statements
    ExpressionStmt(Box<Node>),
    BlockStmt(Vec<Node>, Location),
    IfStmt {
        condition: Box<Node>,
        then_branch: Box<Node>,
        else_branch: Option<Box<Node>>,
        location: Location,
    },
    WhileStmt {
        condition: Box<Node>,
        body: Box<Node>,
        location: Location,
    },
    ForStmt {
        init: Option<Box<Node>>,
        condition: Option<Box<Node>>,
        increment: Option<Box<Node>>,
        body: Box<Node>,
        location: Location,
    },
    ForeachStmt {
        array: Box<Node>,
        value_var: String,
        key_var: Option<String>,
        body: Box<Node>,
        location: Location,
    },
    ReturnStmt(Option<Box<Node>>, Location),
    EchoStmt(Vec<Node>, Location),

    // Declarations
    VarDecl {
        name: String,
        initializer: Option<Box<Node>>,
        location: Location,
    },
    FunctionDecl {
        name: String,
        params: Vec<(String, Option<Type>)>,
        body: Box<Node>,
        location: Location,
    },

    // Expressions
    BinaryExpr {
        op: BinaryOp,
        left: Box<Node>,
        right: Box<Node>,
        location: Location,
    },
    UnaryExpr {
        op: UnaryOp,
        expr: Box<Node>,
        location: Location,
    },
    Variable(String, Location),
    FunctionCall {
        name: String,
        args: Vec<Node>,
        location: Location,
    },

    // Literals
    IntLiteral(i64, Location),
    FloatLiteral(f64, Location),
    StringLiteral(String, Location),
    BooleanLiteral(bool, Location),
    NullLiteral(Location),
    ArrayLiteral(Vec<(Option<Node>, Node)>, Location), // (key, value) pairs
}
