#[cfg(test)]
mod tests {
    use super::*;
    use middle_end::mir::{MirFunction, MirInstruction, MirValue};

    #[test]
    fn test_basic_codegen() {
        let context = Context::create();
        let mut codegen = CodeGen::new(&context, "test");

        let mir = MirFunction {
            instructions: vec![
                MirInstruction::WriteBarrier { reference: "counter".to_string() },
                MirInstruction::Load { target: 0, value: MirValue::Number(2) },
                MirInstruction::Store { target: "counter".to_string(), value: MirValue::Temporary(0) },
            ]
        };

        codegen.compile(&mir);
        assert!(codegen.module.verify().is_ok());
    }
}