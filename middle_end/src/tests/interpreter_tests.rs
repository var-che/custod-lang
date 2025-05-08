use crate::interpreter::Interpreter;
use crate::mir::{MirFunction, MirInstruction, MirValue};

#[test]
fn test_basic_arithmetic() {
    let mir = MirFunction {
        instructions: vec![
            // Store 2 in counter
            MirInstruction::Store {
                target: "counter".to_string(),
                value: MirValue::Number(2),
            },
            // Load counter into temp 0
            MirInstruction::Load {
                target: 0,
                value: MirValue::Variable("counter".to_string()),
            },
            // Add 3 to temp 0, store in temp 1
            MirInstruction::Add {
                target: 1,
                left: MirValue::Temporary(0),
                right: MirValue::Number(3),
            },
            // Store result back in counter
            MirInstruction::Store {
                target: "counter".to_string(),
                value: MirValue::Temporary(1),
            },
        ]
    };

    let mut interpreter = Interpreter::new();
    interpreter.execute(&mir).expect("Execution failed");
    
    // Get final value of counter
    assert_eq!(interpreter.get_variable("counter"), Some(5));
}

#[test]
fn test_permission_barriers() {
    let mir = MirFunction {
        instructions: vec![
            // Write with proper barrier
            MirInstruction::WriteBarrier { 
                reference: "counter".to_string() 
            },
            MirInstruction::Store {
                target: "counter".to_string(),
                value: MirValue::Number(1),
            },
            // Read with proper barrier
            MirInstruction::ReadBarrier { 
                reference: "counter".to_string() 
            },
            MirInstruction::Load {
                target: 0,
                value: MirValue::Variable("counter".to_string()),
            },
            MirInstruction::Print {
                value: MirValue::Temporary(0),
            },
        ]
    };

    let mut interpreter = Interpreter::new();
    interpreter.execute(&mir).expect("Execution failed");
    assert_eq!(interpreter.get_variable("counter"), Some(1));
}

#[test]
fn test_multiple_variables() {
    let mir = MirFunction {
        instructions: vec![
            MirInstruction::Store {
                target: "x".to_string(),
                value: MirValue::Number(10),
            },
            MirInstruction::Store {
                target: "y".to_string(),
                value: MirValue::Number(20),
            },
            MirInstruction::Load {
                target: 0,
                value: MirValue::Variable("x".to_string()),
            },
            MirInstruction::Load {
                target: 1,
                value: MirValue::Variable("y".to_string()),
            },
            MirInstruction::Add {
                target: 2,
                left: MirValue::Temporary(0),
                right: MirValue::Temporary(1),
            },
            MirInstruction::Store {
                target: "sum".to_string(),
                value: MirValue::Temporary(2),
            },
        ]
    };

    let mut interpreter = Interpreter::new();
    interpreter.execute(&mir).expect("Execution failed");
    assert_eq!(interpreter.get_variable("sum"), Some(30));
}