use crate::hir::permissions::{AliasTable, PermissionInfo};
use crate::hir::types::*;
use front_end::token::PermissionType;

#[test]
fn test_alias_table_visualization() {
    // Create a table with several different permission scenarios
    let mut table = AliasTable::new();
    
    // 1. Exclusive variable
    let excl_perms = PermissionInfo::new(vec![PermissionType::Read, PermissionType::Write]);
    table.register_variable("exclusive", &excl_perms).unwrap();
    
    // 2. Shared read variable
    let shared_perms = PermissionInfo::new(vec![PermissionType::Reads]);
    table.register_variable("shared", &shared_perms).unwrap();
    let read_perms = PermissionInfo::new(vec![PermissionType::Read]);
    table.register_alias("shared_reader1", "shared", &read_perms).unwrap();
    table.register_alias("shared_reader2", "shared", &read_perms).unwrap();
    
    // 3. Reads+Write variable with readers
    let rw_perms = PermissionInfo::new(vec![PermissionType::Reads, PermissionType::Write]);
    table.register_variable("owner", &rw_perms).unwrap();
    table.register_alias("reader1", "owner", &read_perms).unwrap();
    table.register_alias("reader2", "owner", &read_perms).unwrap();
    
    // 4. Try some operations
    table.check_read_access("shared").unwrap();
    table.check_read_access("shared_reader1").unwrap();
    table.check_write_access("exclusive").unwrap();
    table.check_write_access("shared").expect_err("Should not have write access");
    table.check_write_access("owner").unwrap();
    table.check_read_access("reader1").unwrap();
    
    // Generate and print visualization
    let visualization = table.visualize();
    println!("Alias Table Visualization:\n{}", visualization);
    
    // Basic assertion to ensure visualization contains key elements
    assert!(visualization.contains("MEMORY ALIAS TABLE"), "Visualization should contain table header");
    assert!(visualization.contains("ACCESS HISTORY"), "Visualization should contain access history");
    assert!(visualization.contains("exclusive"), "Visualization should contain exclusive variable");
    assert!(visualization.contains("Exclusive (owner:"), "Visualization should show exclusive state");
    assert!(visualization.contains("shared_reader"), "Visualization should show shared readers");
}

#[test]
fn test_visualization_in_error_cases() {
    let mut table = AliasTable::new();
    
    // Set up variables
    let excl_perms = PermissionInfo::new(vec![PermissionType::Read, PermissionType::Write]);
    table.register_variable("exclusive", &excl_perms).unwrap();
    
    let rw_perms = PermissionInfo::new(vec![PermissionType::Reads, PermissionType::Write]);
    table.register_variable("owner", &rw_perms).unwrap();
    
    // Generate some errors
    let _ = table.check_read_access("nonexistent");
    let _ = table.check_write_access("nonexistent");
    
    // Try to make a write alias to exclusive (should fail)
    let write_perms = PermissionInfo::new(vec![PermissionType::Write]);
    let _ = table.register_alias("write_alias", "exclusive", &write_perms);
    
    // Generate visualization with errors included
    let visualization = table.visualize();
    println!("Error Case Visualization:\n{}", visualization);
    
    // Check that errors are shown in visualization
    assert!(visualization.contains("✗"), "Visualization should show error markers");
    assert!(visualization.contains("Error:"), "Visualization should contain error messages");
}