//! MIR type definitions
//!
//! This module defines the core data structures for the MIR (Middle Intermediate Representation).

use std::collections::HashMap;
use front_end::types::Type as FrontEndType;

/// A unique identifier for a basic block
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockId(pub usize);

/// A unique identifier for a variable
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VarId(pub usize);

/// A single MIR instruction
#[derive(Debug, Clone)]
pub enum Instruction {
    /// Assign a value to a variable
    Assign {
        target: VarId,
        source: Operand,
    },
    
    /// Binary operation (e.g., add, subtract)
    BinaryOp {
        target: VarId,
        left: Operand,
        op: BinaryOperation,
        right: Operand,
    },
    
    /// Call a function
    Call {
        target: Option<VarId>,
        function: String,
        arguments: Vec<Operand>,
    },
    
    /// Return from a function
    Return(Option<Operand>),
    
    /// Jump to another block
    Jump(BlockId),
    
    /// Conditional jump
    Branch {
        condition: Operand,
        true_block: BlockId,
        false_block: BlockId,
    },
    
    /// No operation (placeholder)
    Nop,
}

/// Binary operations
#[derive(Debug, Clone, Copy)]
pub enum BinaryOperation {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
    Equal,
    NotEqual,
    LessThan,
    LessThanEqual,
    GreaterThan,
    GreaterThanEqual,
    And,
    Or,
}

/// An operand to an instruction
#[derive(Debug, Clone)]
pub enum Operand {
    /// A variable reference
    Variable(VarId),
    
    /// A constant value
    Constant(Constant),
}

/// A constant value
#[derive(Debug, Clone)]
pub enum Constant {
    /// An integer constant
    Integer(i64),
    
    /// A boolean constant
    Boolean(bool),
    
    /// A string constant
    String(String),
}

/// A basic block in the MIR
#[derive(Debug, Clone)]
pub struct BasicBlock {
    /// The block's ID
    pub id: BlockId,
    
    /// The instructions in this block
    pub instructions: Vec<Instruction>,
}

/// A function definition in the MIR
#[derive(Debug, Clone)]
pub struct MirFunction {
    /// The function's name
    pub name: String,
    
    /// The function's parameters
    pub parameters: Vec<(VarId, FrontEndType)>,
    
    /// The function's return type
    pub return_type: Option<FrontEndType>,
    
    /// The function's basic blocks
    pub blocks: Vec<BasicBlock>,
    
    /// The entry block ID
    pub entry_block: BlockId,
    
    /// Variable information
    pub variables: HashMap<VarId, MirVariable>,
}

/// A variable in the MIR
#[derive(Debug, Clone)]
pub struct MirVariable {
    /// The variable's ID
    pub id: VarId,
    
    /// The variable's name (for debugging)
    pub name: String,
    
    /// The variable's type
    pub typ: FrontEndType,
}

/// A complete MIR program
#[derive(Debug, Clone)]
pub struct MirProgram {
    /// Global variables
    pub globals: HashMap<String, MirVariable>,
    
    /// Functions defined in the program
    pub functions: HashMap<String, MirFunction>,
    
    /// The next available variable ID
    pub next_var_id: usize,
    
    /// The next available block ID
    pub next_block_id: usize,
}

impl MirProgram {
    /// Create a new, empty MIR program
    pub fn new() -> Self {
        Self {
            globals: HashMap::new(),
            functions: HashMap::new(),
            next_var_id: 0,
            next_block_id: 0,
        }
    }
    
    /// Generate a new variable ID
    pub fn new_var_id(&mut self) -> VarId {
        let id = self.next_var_id;
        self.next_var_id += 1;
        VarId(id)
    }
    
    /// Generate a new block ID
    pub fn new_block_id(&mut self) -> BlockId {
        let id = self.next_block_id;
        self.next_block_id += 1;
        BlockId(id)
    }
}
