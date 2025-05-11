//! Permission Management for HIR
//!
//! Handles permission checking and validation to enforce the language's
//! permission-based safety rules.

use std::collections::{HashMap, HashSet};
use front_end::token::PermissionType;
use crate::hir::types::*;

#[derive(Debug, PartialEq, Clone)]
pub struct PermissionInfo {
    pub permissions: Vec<PermissionType>,  
    pub is_isolated: bool,
    pub is_sendable: bool,
}

impl PermissionInfo {
    pub fn new(permissions: Vec<PermissionType>) -> Self {
        // Check for invalid permission combinations
        let mut info = Self {
            permissions,
            is_isolated: true,
            is_sendable: true,
        };

        // Update isolation based on permissions
        if info.permissions.contains(&PermissionType::Reads) ||
           info.permissions.contains(&PermissionType::Writes) {
            info.is_isolated = false;
        }

        info
    }

    pub fn has_permission(&self, permission: &PermissionType) -> bool {
        self.permissions.contains(permission)
    }

    pub fn has_read_access(&self) -> bool {
        self.permissions.iter().any(|p| matches!(p, PermissionType::Read | PermissionType::Reads))
    }

    pub fn has_write_access(&self) -> bool {
        self.permissions.iter().any(|p| matches!(p, PermissionType::Write | PermissionType::Writes))
    }

    pub fn has_exclusive_permissions(&self) -> bool {
        // read,write means exclusive access - no one else can access
        self.permissions.contains(&PermissionType::Read) && 
        self.permissions.contains(&PermissionType::Write)
    }

    pub fn is_reads_write(&self) -> bool {
        // reads,write means others can read but only owner can write
        self.permissions.contains(&PermissionType::Reads) && 
        self.permissions.contains(&PermissionType::Write)
    }

    pub fn can_be_consumed(&self) -> bool {
        self.is_reads_write()
    }

    pub fn check_permission_combination(&self) -> Result<(), String> {
        if self.is_reads_write() {
            if self.permissions.contains(&PermissionType::Read) {
                return Err("Cannot combine 'reads write' with 'read'".to_string());
            }
            Ok(())
        } else if self.has_exclusive_permissions() {
            if self.permissions.contains(&PermissionType::Reads) || 
               self.permissions.contains(&PermissionType::Writes) {
                return Err("Cannot combine 'read write' with other permissions".to_string());
            }
            Ok(())
        } else {
            Ok(())
        }
    }
}

pub struct PermissionChecker {
    pub(crate) permissions: HashMap<String, PermissionInfo>,
    pub alias_table: AliasTable,
}

impl PermissionChecker {
    pub fn new() -> Self {
        Self {
            permissions: HashMap::new(),
            alias_table: AliasTable::new()
        }
    }

    pub fn check_program(program: &HirProgram) -> Result<(), String> {
        let mut checker = PermissionChecker::new();
        
        for statement in &program.statements {
            match statement {
                HirStatement::Actor(actor) => {
                    // Check actor state permissions
                    for var in &actor.state {
                        var.permissions.check_permission_combination()?;
                    }
                    
                    // Check method permissions
                    for method in &actor.methods {
                        for perm in &method.used_permissions {
                            perm.check_permission_combination()?;
                        }
                    }

                    // Check behavior permissions
                    for behavior in &actor.behaviors {
                        for stmt in &behavior.body {
                            checker.check_statement(stmt)?;
                        }
                    }
                }
                HirStatement::Declaration(var) => {
                    checker.check_declaration(var)?;
                }
                HirStatement::Assignment(assignment) => {
                    // Check if we have write permission to the target
                    checker.check_assignment(assignment)?;
                }
                HirStatement::Print(value) => {
                    // Check if we have read permission for the value being printed
                    checker.check_permissions(value)?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    pub fn check_statement(&mut self, stmt: &HirStatement) -> Result<(), String> {
        match stmt {
            HirStatement::AtomicBlock(stmts) => {
                for stmt in stmts {
                    self.check_statement(stmt)?;
                }
                Ok(())
            }
            HirStatement::Assignment(assign) => self.check_assignment(assign),
            HirStatement::Declaration(var) => self.check_declaration(var),
            HirStatement::Method(_) => Ok(()),
            HirStatement::Actor(_) => Ok(()),
            HirStatement::Print(value) => {
                // Check read permissions for printed values
                self.check_value_permissions(value)
            }
            HirStatement::ActorCall { .. } => Ok(()),
            HirStatement::Return(value) => {
                // Check permissions for return value
                self.check_value_permissions(value)
            }
            _ => Ok(()),
        }
    }

    pub fn check_declaration(&mut self, var: &HirVariable) -> Result<(), String> {
        var.permissions.check_permission_combination()?;
        
        // Register the variable in the alias table
        self.alias_table.register_variable(&var.name, &var.permissions)?;
        
        if let Some(init) = &var.initializer {
            if let HirValue::Variable(source_name, _) = init {
                // This is an alias - register it as such
                self.alias_table.register_alias(&var.name, source_name, &var.permissions)?;
            }
            self.check_permissions(init)?;
        }
        
        // Record in the regular permissions map too
        self.permissions.insert(var.name.clone(), var.permissions.clone());
        Ok(())
    }

    pub fn check_permissions(&mut self, value: &HirValue) -> Result<(), String> {
        match value {
            HirValue::Variable(name, _) => {
                self.alias_table.check_read_access(name)
            },
            HirValue::Binary { left, right, .. } => {
                self.check_permissions(left)?;
                self.check_permissions(right)
            },
            HirValue::Consume(inner) => {
                match **inner {
                    HirValue::Variable(ref name, _) => {
                        if let Some(perms) = self.permissions.get(name) {
                            if perms.can_be_consumed() {
                                Ok(())
                            } else {
                                Err(format!("Cannot consume '{}' - not consumable", name))
                            }
                        } else {
                            Err(format!("Variable '{}' not found", name))
                        }
                    },
                    _ => Err("Can only consume variables".to_string()),
                }
            },
            _ => Ok(()),
        }
    }

    pub fn check_assignment(&mut self, assign: &HirAssignment) -> Result<(), String> {
        // Check if target exists and has write permission
        self.alias_table.check_write_access(&assign.target)?;
        
        // Check permissions for the value being assigned
        self.check_permissions(&assign.value)
    }

    pub fn check_value_permissions(&mut self, value: &HirValue) -> Result<(), String> {
        self.check_permissions(value)
    }

    pub fn print_alias_table(&self) -> String {
        self.alias_table.visualize()
    }

    pub fn save_alias_table(&self, path: &str) -> std::io::Result<()> {
        self.alias_table.save_visualization(path)
    }
}

/// Validate that a program's permissions are consistent
pub fn validate_permissions(program: &HirProgram) -> Result<(), String> {
    let mut var_permissions = HashMap::new();
    let mut alias_table = AliasTable::new();
    
    // First register all variables
    for stmt in &program.statements {
        if let HirStatement::Declaration(var) = stmt {
            alias_table.register_variable(&var.name, &var.permissions)?;
            var_permissions.insert(var.name.clone(), var.permissions.clone());
        }
    }
    
    // Then check all initializers and assignments
    for stmt in &program.statements {
        match stmt {
            HirStatement::Declaration(var) => {
                // Check if initializer is an alias
                if let Some(HirValue::Variable(source_name, _)) = &var.initializer {
                    // Register the alias
                    alias_table.register_alias(
                        &var.name,
                        source_name,
                        &var.permissions
                    )?;
                }
            },
            HirStatement::Assignment(assign) => {
                // Verify write permission
                alias_table.check_write_access(&assign.target)?;
                
                // Check any variables read in the expression
                check_read_permissions_in_expr(&assign.value, &alias_table)?;
            },
            // Check other statement types
            _ => {}
        }
    }
    
    Ok(())
}

#[derive(Debug)]
pub struct AliasTable {
    // Maps memory locations to all variables that reference them
    aliases: HashMap<String, AliasGroup>,
    // For visualization/debugging
    access_history: Vec<AccessEvent>,
    // Maps variable names to their permissions
    pub permissions: HashMap<String, PermissionInfo>,
}

#[derive(Debug, Clone)]
pub struct AliasGroup {
    // The primary variable (original allocation)
    pub primary: String,
    // All variables that alias this memory
    pub aliases: HashSet<String>,
    // Current permission state for this memory
    pub state: AliasState,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AliasState {
    // Exclusive access (read+write)
    Exclusive { owner: String },
    // Multiple readers, no writers
    SharedRead { readers: HashSet<String> },
    // One writer, multiple readers
    ReadsWrite { writer: String, readers: HashSet<String> },
    // Multiple writers (technically invalid, for error reporting)
    Conflict { description: String },
    // New state for reads+writes
    SharedWrite { writers: HashSet<String>, readers: HashSet<String> },
}

#[derive(Debug, Clone)]
pub struct AccessEvent {
    var_name: String,
    access_type: AccessType,
    result: Result<(), String>,
    timestamp: usize,
}

#[derive(Debug, Clone)]
pub enum AccessType {
    Read,
    Write,
    ReadExclusively,
    Clone,
    Peak,
    Consume,
}

// Helper function to check read permissions in expressions
fn check_read_permissions_in_expr(value: &HirValue, alias_table: &AliasTable) -> Result<(), String> {
    match value {
        HirValue::Variable(name, _) => {
            alias_table.check_read_access_immut(name)
        },
        HirValue::Binary { left, right, .. } => {
            check_read_permissions_in_expr(left, alias_table)?;
            check_read_permissions_in_expr(right, alias_table)
        },
        HirValue::Consume(inner) => {
            check_read_permissions_in_expr(inner, alias_table)
        },
        _ => Ok(()),
    }
}

impl AliasTable {
    pub fn new() -> Self {
        Self {
            aliases: HashMap::new(),
            access_history: Vec::new(),
            permissions: HashMap::new(),
        }
    
        
    }
    
    // Immutable version of check_read_access that doesn't record history
    pub fn check_read_access_immut(&self, var: &str) -> Result<(), String> {
        if let Some(group) = self.aliases.get(var) {
            match &group.state {
                AliasState::Exclusive { owner } if owner == var => Ok(()),
                AliasState::SharedRead { readers } if readers.contains(var) => Ok(()),
                AliasState::ReadsWrite { readers, .. } if readers.contains(var) => Ok(()),
                AliasState::SharedWrite { readers, .. } if readers.contains(var) => Ok(()),
                _ => {
                    // Look up the actual permission of this specific variable
                    if let Some(permissions) = self.permissions.get(var) {
                        if permissions.has_read_access() {
                            return Ok(());
                        }
                    }
                    Err(format!("Cannot read from '{}' - no read permission", var))
                },
            }
        } else {
            Err(format!("Variable '{}' not found", var))
        }
    }
    
    // Register a new variable with permissions
    pub fn register_variable(&mut self, name: &str, permissions: &PermissionInfo) -> Result<(), String> {
        // Make sure this code checks for both Write and Writes
        let state = if permissions.has_exclusive_permissions() {
            AliasState::Exclusive { owner: name.to_string() }
        } else if permissions.is_reads_write() {
            // This might not be handling reads+writes properly
            let mut readers = HashSet::new();
            readers.insert(name.to_string());
            AliasState::ReadsWrite { writer: name.to_string(), readers }
        } else if permissions.permissions.contains(&PermissionType::Writes) {
            // Add the missing readers HashSet for SharedWrite
            let mut writers = HashSet::new();
            writers.insert(name.to_string());
            let mut readers = HashSet::new();
            if permissions.has_read_access() {
                readers.insert(name.to_string());
            }
            AliasState::SharedWrite { writers, readers }
        } else {
            let mut readers = HashSet::new();
            readers.insert(name.to_string());
            AliasState::SharedRead { readers }
        };

        let group = AliasGroup {
            primary: name.to_string(),
            aliases: HashSet::from([name.to_string()]),
            state,
        };
        
        self.aliases.insert(name.to_string(), group);
        Ok(())
    }
    
    // Register a variable that aliases an existing one
    pub fn register_alias(&mut self, 
                        name: &str, 
                        source: &str, 
                        permissions: &PermissionInfo) -> Result<(), String> {
        if !self.aliases.contains_key(source) {
            return Err(format!("Source variable '{}' not found", source));
        }
        
        // Get the primary variable this aliases
        let primary = self.aliases[source].primary.clone();
        
        // Create a modified group outside the borrow
        let mut new_group;
        {
            let group = self.aliases.get_mut(&primary).unwrap();
            new_group = group.clone();
            
            // Check if this alias is allowed based on current state
            match &group.state {
                AliasState::Exclusive { owner } => {
                    // Can't alias an exclusive variable
                    if permissions.has_write_access() {
                        return Err(format!(
                            "Cannot create write alias to '{}' - it has exclusive permissions", 
                            source
                        ));
                    }
                    
                    // For read-only access, need to transition to ReadsWrite
                    if permissions.has_read_access() {
                        let mut readers = HashSet::new();
                        readers.insert(name.to_string());
                        readers.insert(owner.clone());
                        
                        group.state = AliasState::ReadsWrite { 
                            writer: owner.clone(),
                            readers 
                        };
                        new_group.state = group.state.clone();
                    }
                },
                AliasState::SharedRead { readers } => {
                    // Can add another reader, but not a writer
                    if permissions.has_write_access() {
                        return Err(format!(
                            "Cannot create write alias to '{}' - it already has readers", 
                            source
                        ));
                    }
                    
                    let mut new_readers = readers.clone();
                    new_readers.insert(name.to_string());
                    group.state = AliasState::SharedRead { readers: new_readers };
                    new_group.state = group.state.clone();
                },
                AliasState::ReadsWrite { writer, readers } => {
                    // Can add another reader, but not another writer
                    if permissions.has_write_access() && !name.eq(writer) {
                        return Err(format!(
                            "Cannot create write alias to '{}' - it already has a writer ({})", 
                            source, writer
                        ));
                    }
                    
                    let mut new_readers = readers.clone();
                    new_readers.insert(name.to_string());
                    group.state = AliasState::ReadsWrite { 
                        writer: writer.clone(),
                        readers: new_readers 
                    };
                    new_group.state = group.state.clone();
                },
                AliasState::SharedWrite { writers, readers } => {
                    // Can add another writer
                    let mut new_writers = writers.clone();
                    let mut new_readers = readers.clone();
                    
                    // Only add to writers if it has write permission
                    if permissions.has_write_access() {
                        new_writers.insert(name.to_string());
                    }
                    
                    // Only add to readers if it has read permission
                    if permissions.has_read_access() {
                        new_readers.insert(name.to_string());
                    }
                    
                    group.state = AliasState::SharedWrite { writers: new_writers, readers: new_readers };
                    new_group.state = group.state.clone();
                },
                AliasState::Conflict { .. } => {
                    return Err(format!("Cannot create alias to '{}' - it's in a conflicted state", source));
                }
            }
            
            // Add to alias group
            group.aliases.insert(name.to_string());
            new_group.aliases = group.aliases.clone();
        }
        
        // Add a reference to the alias table
        self.aliases.insert(name.to_string(), new_group);
        
        Ok(())
    }
    
    // Record an access for history/debugging
    pub fn record_access(&mut self, var: &str, access: AccessType, result: Result<(), String>) {
        self.access_history.push(AccessEvent {
            var_name: var.to_string(),
            access_type: access,
            result,
            timestamp: self.access_history.len(),
        });
    }
    
    // Check if a read access is allowed
    pub fn check_read_access(&mut self, var: &str) -> Result<(), String> {
        if let Some(group) = self.aliases.get(var) {
            // First check if the variable has read permission based on its state
            let result = match &group.state {
                AliasState::Exclusive { owner } if owner == var => Ok(()),
                AliasState::SharedRead { readers } if readers.contains(var) => Ok(()),
                AliasState::ReadsWrite { readers, .. } if readers.contains(var) => Ok(()),
                
                // Fix this condition - only allow reading if in the readers set
                AliasState::SharedWrite { readers, .. } if readers.contains(var) => Ok(()),
                
                // Check if this variable specifically has read permission
                _ => {
                    // Look up the actual permission of this specific variable
                    if let Some(permissions) = self.permissions.get(var) {
                        if permissions.has_read_access() {
                            return Ok(());
                        }
                    }
                    Err(format!("Cannot read from '{}' - no read permission", var))
                },
            };
            
            self.record_access(var, AccessType::Read, result.clone());
            result
        } else {
            let err = Err(format!("Variable '{}' not found", var));
            self.record_access(var, AccessType::Read, err.clone());
            err
        }
    }
    
    // Check if a write access is allowed
    pub fn check_write_access(&mut self, var: &str) -> Result<(), String> {
        if let Some(group) = self.aliases.get(var) {
            let result = match &group.state {
                AliasState::Exclusive { owner } if owner == var => Ok(()),
                AliasState::ReadsWrite { writer, .. } if writer == var => Ok(()),
                AliasState::SharedWrite { writers, readers: _ } if writers.contains(var) => Ok(()),
                _ => Err(format!("Cannot write to '{}' - no write permission", var)),
            };
            
            self.record_access(var, AccessType::Write, result.clone());
            result
        } else {
            let err = Err(format!("Variable '{}' not found", var));
            self.record_access(var, AccessType::Write, err.clone());
            err
        }
    }
    
    // Generate a visualization of the current alias state
    pub fn visualize(&self) -> String {
        let mut result = String::new();
        
        // Header
        result.push_str("===== MEMORY ALIAS TABLE =====\n");
        result.push_str("Memory | Variables | State\n");
        result.push_str("-------------------------------\n");
        
        // Collect groups by primary
        let mut groups_by_primary: HashMap<&String, Vec<&AliasGroup>> = HashMap::new();
        for (name, group) in &self.aliases {
            groups_by_primary
                .entry(&group.primary)
                .or_default()
                .push(group);
        }
        
        // Print each primary group once
        for primary in groups_by_primary.keys() {
            let group = &self.aliases[*primary];
            let alias_list = group.aliases.iter()
                .map(|a| a.to_string())
                .collect::<Vec<_>>()
                .join(", ");
                
            let state_str = match &group.state {
                AliasState::Exclusive { owner } => 
                    format!("Exclusive (owner: {})", owner),
                AliasState::SharedRead { readers } =>
                    format!("Shared Read ({} readers)", readers.len()),
                AliasState::ReadsWrite { writer, readers } =>
                    format!("Reads+Write (writer: {}, {} readers)", writer, readers.len()),
                AliasState::SharedWrite { writers, readers } =>
                    format!("Shared Write ({} writers, {} readers)", writers.len(), readers.len()),
                AliasState::Conflict { description } =>
                    format!("CONFLICT: {}", description),
            };
            
            result.push_str(&format!("{:<8}| {:<20}| {}\n", primary, alias_list, state_str));
        }
        
        // Print access history
        result.push_str("\n===== ACCESS HISTORY =====\n");
        for (i, event) in self.access_history.iter().enumerate() {
            let status = if event.result.is_ok() { "✓" } else { "✗" };
            let access_type = format!("{:?}", event.access_type);
            let message = match &event.result {
                Ok(_) => "Success".to_string(),
                Err(msg) => format!("Error: {}", msg),
            };
            
            result.push_str(&format!(
                "{:3}. {} {:8} {} - {}\n", 
                i + 1, status, access_type, event.var_name, message
            ));
        }
        
        result
    }

    // Save visualization to a file
    pub fn save_visualization(&self, path: &str) -> std::io::Result<()> {
        use std::fs::File;
        use std::io::Write;
        
        let mut file = File::create(path)?;
        file.write_all(self.visualize().as_bytes())?;
        Ok(())
    }
}