//! Pretty printer for HIR
//!
//! This module provides functionality to print HIR in a human-readable format.

use crate::hir::types::*;
use std::fmt::Write;

/// Pretty-print a HIR program to a string
pub fn pretty_print(program: &HirProgram) -> String {
    let mut printer = HirPrinter::new();
    printer.print_program(program)
}

/// Helper struct for pretty-printing HIR
struct HirPrinter {
    /// The output buffer
    output: String,
    /// Current indentation level
    indent: usize,
}

impl HirPrinter {
    /// Create a new HIR printer
    fn new() -> Self {
        Self {
            output: String::new(),
            indent: 0,
        }
    }
    
    /// Print a HIR program
    fn print_program(&mut self, program: &HirProgram) -> String {
        writeln!(self.output, "HIR Program with {} statements", program.statements.len()).unwrap();
        
        for stmt in &program.statements {
            self.print_statement(stmt);
        }
        
        // Print type information
        writeln!(self.output, "\nType Information:").unwrap();
        writeln!(self.output, "  Variables: {} entries", program.type_info.variables.len()).unwrap();
        for (name, typ) in &program.type_info.variables {
            writeln!(self.output, "    {}: {:?}", name, typ).unwrap();
        }
        
        writeln!(self.output, "  Functions: {} entries", program.type_info.functions.len()).unwrap();
        for (name, return_type) in &program.type_info.functions {
            writeln!(self.output, "    {}() -> {:?}", name, return_type).unwrap();
        }
        
        self.output.clone()
    }
    
    /// Print a statement with proper indentation
    fn print_statement(&mut self, stmt: &HirStatement) {
        self.print_indent();
        
        match stmt {
            HirStatement::Declaration(var) => {
                        // Print permissions
                        let perms: Vec<String> = var.permissions.iter()
                            .map(|p| format!("{:?}", p).to_lowercase())
                            .collect();
                
                        writeln!(self.output, "var {} : {:?} [{}]", 
                            var.name, var.typ, perms.join(", ")).unwrap();
                
                        if let Some(init) = &var.initializer {
                            self.indent += 1;
                            self.print_indent();
                            write!(self.output, "= ").unwrap();
                            self.print_expression(init);
                            writeln!(self.output).unwrap();
                            self.indent -= 1;
                        }
                    },
            HirStatement::Assignment(assign) => {
                        write!(self.output, "{} = ", assign.target).unwrap();
                        self.print_expression(&assign.value);
                        writeln!(self.output).unwrap();
                    },
            HirStatement::Function(func) => {
                        // Function header
                        write!(self.output, "fn {}(", func.name).unwrap();
                
                        for (i, param) in func.parameters.iter().enumerate() {
                            if i > 0 { write!(self.output, ", ").unwrap(); }
                    
                            let perms: Vec<String> = param.permissions.iter()
                                .map(|p| format!("{:?}", p).to_lowercase())
                                .collect();
                    
                            write!(self.output, "{}: {:?} [{}]", 
                                param.name, param.typ, perms.join(", ")).unwrap();
                        }
                
                        if let Some(ret_type) = &func.return_type {
                            write!(self.output, ") -> {:?}", ret_type).unwrap();
                        } else {
                            write!(self.output, ")").unwrap();
                        }
                
                        writeln!(self.output, " {{").unwrap();
                
                        // Function body
                        self.indent += 1;
                        for body_stmt in &func.body {
                            self.print_statement(body_stmt);
                        }
                        self.indent -= 1;
                
                        self.print_indent();
                        writeln!(self.output, "}}").unwrap();
                    },
            HirStatement::Return(expr_opt) => {
                        write!(self.output, "return").unwrap();
                        if let Some(expr) = expr_opt {
                            write!(self.output, " ").unwrap();
                            self.print_expression(expr);
                        }
                        writeln!(self.output).unwrap();
                    },
            HirStatement::Print(expr) => {
                        write!(self.output, "print ").unwrap();
                        self.print_expression(expr);
                        writeln!(self.output).unwrap();
                    },
            HirStatement::Expression(expr) => {
                        self.print_expression(expr);
                        writeln!(self.output).unwrap();
                    },
            HirStatement::Block(statements) => {
                        writeln!(self.output, "{{").unwrap();
                        self.indent += 1;
                        for stmt in statements {
                            self.print_statement(stmt);
                        }
                        self.indent -= 1;
                        self.print_indent();
                        writeln!(self.output, "}}").unwrap();
                    },
HirStatement::If { condition, then_branch, else_branch } => todo!(),
            HirStatement::While { condition, body } => todo!(),
        }
    }
    
    /// Print an expression
    fn print_expression(&mut self, expr: &HirExpression) {
        match expr {
            HirExpression::Integer(val, _) => {
                        write!(self.output, "{}", val).unwrap();
                    },
            HirExpression::Boolean(val) => {
                        write!(self.output, "{}", val).unwrap();
                    },
            HirExpression::String(val) => {
                        write!(self.output, "\"{}\"", val).unwrap();
                    },
            HirExpression::Variable(name, typ, _) => {
                        write!(self.output, "{}: {:?}", name, typ).unwrap();
                    },
            HirExpression::Binary { left, operator, right, result_type } => {
                        write!(self.output, "(").unwrap();
                        self.print_expression(left);
                        write!(self.output, " {:?} ", operator).unwrap();
                        self.print_expression(right);
                        write!(self.output, "): {:?}", result_type).unwrap();
                    },
            HirExpression::Call { function, arguments, result_type } => {
                        write!(self.output, "{}(", function).unwrap();
                        for (i, arg) in arguments.iter().enumerate() {
                            if i > 0 { write!(self.output, ", ").unwrap(); }
                            self.print_expression(arg);
                        }
                        write!(self.output, "): {:?}", result_type).unwrap();
                    },
            HirExpression::Cast { expr, target_type } => {
                        write!(self.output, "cast<").unwrap();
                        write!(self.output, "{:?}>", target_type).unwrap();
                        write!(self.output, "(").unwrap();
                        self.print_expression(expr);
                        write!(self.output, ")").unwrap();
                    },
            HirExpression::Peak(expr) => {
                        write!(self.output, "peak ").unwrap();
                        self.print_expression(expr);
                    },
            HirExpression::Clone(expr) => {
                        write!(self.output, "clone ").unwrap();
                        self.print_expression(expr);
                    },
HirExpression::Conditional { condition, then_expr, else_expr, result_type } => todo!(),
        }
    }
    
    /// Print the current indentation
    fn print_indent(&mut self) {
        for _ in 0..self.indent {
            write!(self.output, "  ").unwrap();
        }
    }
}
