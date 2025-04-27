use inkwell::context::Context;
use inkwell::builder::Builder;
use inkwell::module::Module;
use inkwell::values::{IntValue, PointerValue};
use std::collections::HashMap;
use middle_end::mir::{MirFunction, MirInstruction, MirValue};

pub struct CodeGen<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    variables: HashMap<String, PointerValue<'ctx>>,
}

impl<'ctx> CodeGen<'ctx> {
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();
        
        CodeGen {
            context,
            module,
            builder,
            variables: HashMap::new(),
        }
    }

    pub fn compile(&mut self, mir: &MirFunction) {
        let i64_type = self.context.i64_type();
        let fn_type = i64_type.fn_type(&[], false);
        let function = self.module.add_function("main", fn_type, None);
        let basic_block = self.context.append_basic_block(function, "entry");
        
        self.builder.position_at_end(basic_block);

        for instruction in &mir.instructions {
            self.compile_instruction(instruction);
        }

        // Return 0
        let ret_val = i64_type.const_int(0, false);
        self.builder.build_return(Some(&ret_val));
    }

    fn compile_instruction(&self, instruction: &MirInstruction) {
        // We'll implement this next
    }
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
