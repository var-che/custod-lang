use crate::hir::permissions::{AliasTable, AccessType, PermissionInfo};
use crate::hir::types::*;
use front_end::token::PermissionType;

#[test]
fn test_register_variable() {
    let mut table = AliasTable::new();
    
    // Register variable with Read+Write permission
    let x_perms = PermissionInfo::new(vec![PermissionType::Read, PermissionType::Write]);
    let result = table.register_variable("x", &x_perms);
    
    assert!(result.is_ok(), "Should be able to register variable");
    
    // Check read access
    assert!(table.check_read_access("x").is_ok(), "Should be able to read from x");
    
    // Check write access
    assert!(table.check_write_access("x").is_ok(), "Should be able to write to x");
    
    // Try accessing nonexistent variable
    assert!(table.check_read_access("y").is_err(), "Should not be able to read nonexistent variable");
}

#[test]
fn test_alias_read_only() {
    let mut table = AliasTable::new();
    
    // Register original variable with Reads+Write permission
    let x_perms = PermissionInfo::new(vec![PermissionType::Reads, PermissionType::Write]);
    table.register_variable("x", &x_perms).unwrap();
    
    // Register read-only alias to x
    let y_perms = PermissionInfo::new(vec![PermissionType::Read]);
    let result = table.register_alias("y", "x", &y_perms);
    
    assert!(result.is_ok(), "Should be able to create read-only alias");
    
    // Check access permissions
    assert!(table.check_read_access("y").is_ok(), "y should have read access");
    assert!(table.check_write_access("y").is_err(), "y should not have write access");
    assert!(table.check_write_access("x").is_ok(), "x should still have write access");
    
    // Print visualization for debugging
    println!("After creating read-only alias:\n{}", table.visualize());
}

#[test]
fn test_alias_exclusive_variable_fails() {
    let mut table = AliasTable::new();
    
    // Register original variable with exclusive Read+Write permission
    let x_perms = PermissionInfo::new(vec![PermissionType::Read, PermissionType::Write]);
    table.register_variable("x", &x_perms).unwrap();
    
    // Try to create a write alias (should fail)
    let y_perms = PermissionInfo::new(vec![PermissionType::Write]);
    let result = table.register_alias("y", "x", &y_perms);
    
    assert!(result.is_err(), "Should not be able to create write alias to exclusive variable");
    
    // Try to create a read alias
    let z_perms = PermissionInfo::new(vec![PermissionType::Read]);
    let result = table.register_alias("z", "x", &z_perms);
    
    assert!(result.is_ok(), "Should be able to create read alias");
    
    // Original should now be in ReadsWrite state
    assert!(table.check_write_access("x").is_ok(), "x should still have write access");
    assert!(table.check_read_access("z").is_ok(), "z should have read access");
    
    // Print visualization for debugging
    println!("After aliasing exclusive variable:\n{}", table.visualize());
}

#[test]
fn test_multiple_readers() {
    let mut table = AliasTable::new();
    
    // Register original variable with Reads permission
    let x_perms = PermissionInfo::new(vec![PermissionType::Reads]);
    table.register_variable("x", &x_perms).unwrap();
    
    // Create multiple read aliases
    let y_perms = PermissionInfo::new(vec![PermissionType::Read]);
    let z_perms = PermissionInfo::new(vec![PermissionType::Read]);
    
    table.register_alias("y", "x", &y_perms).unwrap();
    table.register_alias("z", "x", &z_perms).unwrap();
    
    // All should have read access
    assert!(table.check_read_access("x").is_ok());
    assert!(table.check_read_access("y").is_ok());
    assert!(table.check_read_access("z").is_ok());
    
    // None should have write access
    assert!(table.check_write_access("x").is_err());
    assert!(table.check_write_access("y").is_err());
    assert!(table.check_write_access("z").is_err());
    
    // Print visualization
    println!("Multiple readers scenario:\n{}", table.visualize());
}

#[test]
fn test_reads_write_transition() {
    let mut table = AliasTable::new();
    
    // Register original variable with Reads+Write permission
    let x_perms = PermissionInfo::new(vec![PermissionType::Reads, PermissionType::Write]);
    table.register_variable("x", &x_perms).unwrap();
    
    // Record some access events
    table.check_read_access("x").unwrap();
    table.check_write_access("x").unwrap();
    
    // Create some read aliases
    let y_perms = PermissionInfo::new(vec![PermissionType::Read]);
    let z_perms = PermissionInfo::new(vec![PermissionType::Read]);
    
    table.register_alias("y", "x", &y_perms).unwrap();
    table.register_alias("z", "x", &z_perms).unwrap();
    
    // x should still have write access, others just read
    assert!(table.check_write_access("x").is_ok());
    assert!(table.check_write_access("y").is_err());
    assert!(table.check_read_access("z").is_ok());
    
    // Try to create a write alias (should fail)
    let w_perms = PermissionInfo::new(vec![PermissionType::Write]);
    let result = table.register_alias("w", "x", &w_perms);
    assert!(result.is_err(), "Should not be able to create second writer");
    
    // Print visualization 
    println!("Reads+Write scenario:\n{}", table.visualize());
}

#[test]
fn test_reads_writes_scenario() {
    let mut table = AliasTable::new();
    
    // 1. Create variable with reads+writes permissions
    // reads writes c = 5
    let c_perms = PermissionInfo::new(vec![PermissionType::Reads, PermissionType::Writes]);
    table.register_variable("c", &c_perms).unwrap();
    
    // Verify initial state
    assert!(table.check_read_access("c").is_ok(), "Should be able to read from c");
    assert!(table.check_write_access("c").is_ok(), "Should be able to write to c");
    
    // 2. Create write-only alias
    // write d = c
    let d_perms = PermissionInfo::new(vec![PermissionType::Write]);
    let result = table.register_alias("d", "c", &d_perms);
    assert!(result.is_ok(), "Should be able to create write alias to reads+writes variable");
    
    // Verify both can write
    assert!(table.check_write_access("c").is_ok(), "c should still have write access");
    assert!(table.check_write_access("d").is_ok(), "d should have write access");
    assert!(table.check_read_access("c").is_ok(), "c should have read access");
    assert!(table.check_read_access("d").is_err(), "d should not have read access");
    
    // 3. Create read-only alias using peak
    // read r = peak c
    let r_perms = PermissionInfo::new(vec![PermissionType::Read]);
    let result = table.register_alias("r", "c", &r_perms);
    assert!(result.is_ok(), "Should be able to create read alias with peak");
    
    // Verify final state - everyone maintains their permissions
    assert!(table.check_write_access("c").is_ok(), "c should still have write access");
    assert!(table.check_write_access("d").is_ok(), "d should still have write access");
    assert!(table.check_read_access("c").is_ok(), "c should still have read access");
    assert!(table.check_read_access("r").is_ok(), "r should have read access");
    
    // Print visualization to see the complex permission state
    println!("Reads+Writes scenario with multiple writers and readers:\n{}", table.visualize());
    
    // The special part here: multiple writers should be allowed
    // Try some write operations
    let write1 = table.record_access("c", AccessType::Write, Ok(()));
    let write2 = table.record_access("d", AccessType::Write, Ok(()));
    
    // Try to read after writing
    let read1 = table.record_access("r", AccessType::Read, Ok(()));
    let read2 = table.record_access("c", AccessType::Read, Ok(()));
    
    // Print final state with all operations
    println!("Final state after operations:\n{}", table.visualize());
}

