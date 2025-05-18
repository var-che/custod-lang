use std::collections::HashMap;
use crate::types::{Type, Permission, PermissionedType};
use crate::ast::{Expression, Statement};
use crate::symbol_table::{SymbolTable, Span, Symbol, SymbolKind};

/// Represents a type variable used during type inference
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeVar(usize);

/// Represents either a concrete type or a type variable
#[derive(Debug, Clone, PartialEq)]
pub enum InferenceType {
    Concrete(Type),
    Variable(TypeVar),
}

/// The type environment tracks type variables and constraints
pub struct TypeEnvironment {
    /// Mapping from type variables to their resolved types (if known)
    substitutions: HashMap<TypeVar, InferenceType>,
    /// Counter to generate unique type variables
    next_var_id: usize,
}

/// Provides methods for type unification and inference
pub struct TypeInferer<'a> {
    /// The type environment for this inference session
    env: TypeEnvironment,
    /// Reference to the symbol table for looking up variables
    symbol_table: &'a mut SymbolTable,
    /// Type errors found during inference
    errors: Vec<String>,
}

impl TypeEnvironment {
    pub fn new() -> Self {
        Self {
            substitutions: HashMap::new(),
            next_var_id: 0,
        }
    }
    
    /// Create a new type variable
    pub fn fresh_var(&mut self) -> TypeVar {
        let var = TypeVar(self.next_var_id);
        self.next_var_id += 1;
        var
    }
    
    /// Apply substitutions to resolve a type to its most concrete form
    pub fn resolve(&self, t: &InferenceType) -> InferenceType {
        match t {
            InferenceType::Variable(var) => {
                if let Some(target) = self.substitutions.get(var) {
                    // Recursively resolve to handle chains of substitutions
                    self.resolve(target)
                } else {
                    // No substitution found, return the original variable
                    t.clone()
                }
            },
            _ => t.clone(),
        }
    }
    
    /// Add a substitution from a type variable to another type
    pub fn add_substitution(&mut self, var: TypeVar, target: InferenceType) {
        // Ensure we don't create a cycle by resolving the target first
        let resolved_target = self.resolve(&target);
        self.substitutions.insert(var, resolved_target);
    }
}

impl<'a> TypeInferer<'a> {
    pub fn new(symbol_table: &'a mut SymbolTable) -> Self {
        Self {
            env: TypeEnvironment::new(),
            symbol_table,
            errors: Vec::new(),
        }
    }
    
    /// Get any errors found during type inference
    pub fn get_errors(&self) -> &[String] {
        &self.errors
    }
    
    /// Unify two types, updating the type environment
    pub fn unify(&mut self, t1: InferenceType, t2: InferenceType, span: Span) -> Result<(), String> {
        let t1 = self.env.resolve(&t1);
        let t2 = self.env.resolve(&t2);
        
        match (t1, t2) {
            // If both are concrete types, they must be equal
            (InferenceType::Concrete(c1), InferenceType::Concrete(c2)) => {
                if c1 == c2 {
                    Ok(())
                } else {
                    Err(format!("Type mismatch: {:?} is incompatible with {:?}", c1, c2))
                }
            },
            
            // If one is a type variable, bind it to the other type
            (InferenceType::Variable(var), other) => {
                self.env.add_substitution(var, other);
                Ok(())
            },
            
            (other, InferenceType::Variable(var)) => {
                self.env.add_substitution(var, other);
                Ok(())
            },
        }
    }
    
    /// Infer the type of an expression
    pub fn infer_expression(&mut self, expr: &Expression, span: Span) -> InferenceType {
        match expr {
            Expression::Number(_) => InferenceType::Concrete(Type::Int),
            
            Expression::Variable(name) => {
                // Look up the variable in the symbol table
                if let Some(symbol) = self.symbol_table.resolve(name, span.clone()) {
                    InferenceType::Concrete(symbol.typ.base_type.clone())
                } else {
                    // Symbol not found - the symbol table will have already recorded an error
                    // Return a placeholder type to continue analysis
                    InferenceType::Concrete(Type::Int)
                }
            },
            
            Expression::Binary { left, operator, right } => {
                // Infer types of both operands
                let left_type = self.infer_expression(left, span.clone());
                let right_type = self.infer_expression(right, span.clone());
                
                // Unify the operand types
                if let Err(err) = self.unify(left_type.clone(), right_type.clone(), span.clone()) {
                    self.errors.push(format!("In binary expression: {}", err));
                }
                
                // Handle specific operator types
                match operator {
                    // Comparison operators always return Bool
                    crate::token::TokenType::Greater | 
                    crate::token::TokenType::GreaterEqual |
                    crate::token::TokenType::Less |
                    crate::token::TokenType::LessEqual |
                    crate::token::TokenType::EqualEqual |
                    crate::token::TokenType::BangEqual => InferenceType::Concrete(Type::Bool),
                    
                    // Arithmetic operators return the same type as their operands
                    _ => left_type,
                }
            },
            
            Expression::Call { function, arguments } => {
                // Function calls are complex - we'd need to look up the function signature
                // For now, just use a placeholder type
                // In a full implementation, we would:
                // 1. Look up the function in a function table
                // 2. Check argument types against parameter types
                // 3. Return the function's return type
                
                // Placeholder implementation
                for arg in arguments {
                    let _ = self.infer_expression(arg, span.clone());
                }
                
                // For demo purposes, assume all functions return Int
                // In a real implementation, we would look up the function signature
                InferenceType::Concrete(Type::Int)
            },
            
            Expression::Peak(expr) => {
                // Peak returns the same type as its operand but with read permission
                self.infer_expression(expr, span)
            },
            
            Expression::Clone(expr) => {
                // Clone returns the same type as its operand
                self.infer_expression(expr, span)
            },
        }
    }
    
    /// Infer the return type of a function based on its body
    pub fn infer_function_return_type(&mut self, body: &[Statement], span: Span) -> Option<Type> {
        // Look for return statements
        for stmt in body {
            match stmt {
                Statement::Return(expr) => {
                    let expr_type = self.infer_expression(expr, span.clone());
                    match self.env.resolve(&expr_type) {
                        InferenceType::Concrete(t) => return Some(t),
                        _ => {} // Continue looking for more concrete returns
                    }
                },
                Statement::Block(inner_statements) => {
                    // Recursively check blocks
                    if let Some(ret_type) = self.infer_function_return_type(inner_statements, span.clone()) {
                        return Some(ret_type);
                    }
                },
                _ => {}
            }
        }
        
        // If we reach here without finding a return, check the last statement
        // for an implicit return
        if let Some(last) = body.last() {
            match last {
                Statement::Expression(expr) => {
                    let expr_type = self.infer_expression(expr, span);
                    match self.env.resolve(&expr_type) {
                        InferenceType::Concrete(t) => return Some(t),
                        _ => {}
                    }
                },
                _ => {}
            }
        }
        
        // If we can't find any returns, default to Unit
        Some(Type::Unit)
    }
    
    /// Process a statement for type inference
    pub fn infer_statement(&mut self, stmt: &Statement, span: Span) -> Result<(), String> {
        match stmt {
            Statement::Declaration { name, typ, initializer } => {
                if let Some(expr) = initializer {
                    let expr_type = self.infer_expression(expr, span.clone());
                    
                    // If the declaration has an explicit type, unify with the expression type
                    let decl_type = InferenceType::Concrete(typ.base_type.clone());
                    if let Err(err) = self.unify(decl_type, expr_type, span) {
                        self.errors.push(format!("In declaration of '{}': {}", name, err));
                    }
                }
                Ok(())
            },
            
            Statement::Assignment { target, value, target_type } => {
                let expr_type = self.infer_expression(value, span.clone());
                let target_concrete_type = InferenceType::Concrete(target_type.base_type.clone());
                
                if let Err(err) = self.unify(target_concrete_type, expr_type, span) {
                    self.errors.push(format!("In assignment to '{}': {}", target, err));
                }
                Ok(())
            },
            
            Statement::Expression(expr) => {
                let _ = self.infer_expression(expr, span);
                Ok(())
            },
            
            Statement::Print(expr) => {
                let _ = self.infer_expression(expr, span);
                Ok(())
            },
            
            Statement::Return(expr) => {
                // For returns, we would ideally check against the function's declared return type
                // This would require more context than we currently have
                let _ = self.infer_expression(expr, span);
                Ok(())
            },
            
            Statement::Block(statements) => {
                for stmt in statements {
                    self.infer_statement(stmt, span.clone())?;
                }
                Ok(())
            },
            
            Statement::Function { params, body, return_type, .. } => {
                // First check parameters
                for (name, param_type) in params {
                    // Record parameter types in environment
                    // (This would be more complex in a real implementation)
                }
                
                // Process function body
                for stmt in body {
                    self.infer_statement(stmt, span.clone())?;
                }
                
                // If return type isn't explicitly specified, try to infer it
                if return_type.is_none() {
                    if let Some(inferred_return) = self.infer_function_return_type(body, span) {
                        println!("Inferred return type: {:?}", inferred_return);
                        // We would use this inferred return type for type checking
                    }
                }
                
                Ok(())
            },
            
            Statement::Actor { state, methods, behaviors, .. } => {
                // Process actor components
                for stmt in state {
                    self.infer_statement(stmt, span.clone())?;
                }
                
                for method in methods {
                    self.infer_statement(method, span.clone())?;
                }
                
                for behavior in behaviors {
                    self.infer_statement(behavior, span.clone())?;
                }
                
                Ok(())
            },
            
            Statement::AtomicBlock(statements) => {
                for stmt in statements {
                    self.infer_statement(stmt, span.clone())?;
                }
                Ok(())
            },
        }
    }
    
    /// Infer types for a whole program (list of statements)
    pub fn infer_program(&mut self, statements: &[Statement]) -> Result<(), Vec<String>> {
        for stmt in statements {
            // Use a default span if we don't have a better one
            let span = Span::point(0, 0);
            if let Err(err) = self.infer_statement(stmt, span) {
                self.errors.push(err);
            }
        }
        
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }
    
    /// Try to infer the type of a variable declaration without an explicit type
    pub fn infer_variable_declaration_type(&mut self, initializer: &Expression, span: Span) -> Type {
        let inferred = self.infer_expression(initializer, span);
        match self.env.resolve(&inferred) {
            InferenceType::Concrete(t) => t,
            InferenceType::Variable(_) => {
                // If we couldn't infer a concrete type, default to Int
                // In a real system, we might want to report an error here
                Type::Int
            }
        }
    }
}

// Type utilities for working with the AST
pub trait TypeInferenceExt {
    fn infer_type(&self, inferer: &mut TypeInferer) -> Type;
}

impl TypeInferenceExt for Expression {
    fn infer_type(&self, inferer: &mut TypeInferer) -> Type {
        let span = Span::point(0, 0); // Default span
        let inferred = inferer.infer_expression(self, span);
        match inferer.env.resolve(&inferred) {
            InferenceType::Concrete(t) => t,
            InferenceType::Variable(_) => Type::Int, // Default
        }
    }
}
