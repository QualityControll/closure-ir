use inkwell::{builder::Builder, context::Context, module::Module, types::{BasicType, BasicTypeEnum}, values::{BasicValueEnum, FunctionValue, PointerValue}};
use crate::{expr::{Block, Closure, Expr}, jit::{CompiledClosure, DynamicCompiledClosure}, statement_lowering::lower_closure_block, types::{CompileType, TypeInfo}};
pub struct Compiler<'ctx>{pub(crate) context:&'ctx Context}
impl<'ctx> Compiler<'ctx>{pub fn new(context:&'ctx Context)->Self{Self{context}} pub fn compile<Args,Ret>(&self,closure:&Closure)->Result<CompiledClosure<'ctx,Args,Ret>,String> where Args:CompileType,Ret:CompileType+'static{let c=self.context;let m=c.create_module("closure_module");let f="compiled_closure";self.generate_function(c,&m,f,closure)?;let e=m.create_jit_execution_engine(inkwell::OptimizationLevel::None).map_err(|e|format!("failed to create JIT: {:?}",e))?;Ok(CompiledClosure::new(e,f.to_string()))} pub fn compile_dynamic(&self,closure:&Closure)->Result<DynamicCompiledClosure<'ctx>,String>{let c=self.context;let m=c.create_module("closure_dynamic_module");let f="compiled_dynamic_closure";self.generate_dynamic_function(c,&m,f,closure)?;let e=m.create_jit_execution_engine(inkwell::OptimizationLevel::None).map_err(|e|format!("failed to create JIT: {:?}",e))?;Ok(DynamicCompiledClosure::new(e,f.to_string(),closure.arguments.clone(),closure.return_type.clone()))} fn generate_function(&self,c:&'ctx Context,m:&Module<'ctx>,name:&str,closure:&Closure)->Result<(),String>{let p=c.ptr_type(inkwell::AddressSpace::default());let ft=c.void_type().fn_type(&[p.into(),p.into()],false);let f=m.add_function(name,ft,None);let entry=c.append_basic_block(f,"entry");let b=c.create_builder();b.position_at_end(entry);let ap=f.get_nth_param(0).ok_or("missing function argument")?.into_pointer_value();let rp=f.get_nth_param(1).ok_or("missing result pointer")?.into_pointer_value();let args=self.build_argument_pointers(c,&b,ap,&closure.arguments)?;let v=lower_closure_block(c,m,&b,f,&args,&closure.arguments,&closure.return_type,&closure.body)?;b.build_store(rp,v).map_err(|e|format!("failed to store return value: {:?}",e))?;b.build_return(None).map_err(|e|format!("failed to build return: {:?}",e))?;if f.verify(true){Ok(())}else{Err("LLVM function verification failed".into())}} fn generate_dynamic_function(&self,c:&'ctx Context,m:&Module<'ctx>,name:&str,closure:&Closure)->Result<(),String>{let p=c.ptr_type(inkwell::AddressSpace::default());let ft=c.void_type().fn_type(&[p.into(),p.into()],false);let f=m.add_function(name,ft,None);let e=c.append_basic_block(f,"entry");let b=c.create_builder();b.position_at_end(e);let ap=f.get_nth_param(0).ok_or("missing dynamic argument array")?.into_pointer_value();let rp=f.get_nth_param(1).ok_or("missing result pointer")?.into_pointer_value();let args=self.build_dynamic_argument_pointers(c,&b,ap,&closure.arguments)?;let v=lower_closure_block(c,m,&b,f,&args,&closure.arguments,&closure.return_type,&closure.body)?;b.build_store(rp,v).map_err(|e|format!("failed to store dynamic return value: {:?}",e))?;b.build_return(None).map_err(|e|format!("failed to build dynamic return: {:?}",e))?;if f.verify(true){Ok(())}else{Err("LLVM dynamic function verification failed".into())}} fn build_argument_pointers(&self,c:&'ctx Context,b:&Builder<'ctx>,ap:PointerValue<'ctx>,ats:&[TypeInfo])->Result<Vec<PointerValue<'ctx>>,String>{if ats.is_empty(){return Ok(vec![])}let ts=ats.iter().map(|t|llvm_type(c,t)).collect::<Result<Vec<_>,_>>()?;let st=c.struct_type(&ts,false);(0..ats.len()).map(|i|b.build_struct_gep(st,ap,i as u32,&format!("arg{}_ptr",i)).map_err(|e|format!("failed to build argument {} GEP: {:?}",i,e))).collect()} fn build_dynamic_argument_pointers(&self,c:&'ctx Context,b:&Builder<'ctx>,ap:PointerValue<'ctx>,ats:&[TypeInfo])->Result<Vec<PointerValue<'ctx>>,String>{let pt=c.ptr_type(inkwell::AddressSpace::default());(0..ats.len()).map(|i|{let iv=c.i64_type().const_int(i as u64,false);let p=unsafe{b.build_gep(pt,ap,&[iv],&format!("dynamic_arg{}_slot",i))}.map_err(|e|format!("failed to build dynamic argument GEP: {:?}",e))?;b.build_load(pt,p,&format!("dynamic_arg{}_ptr",i)).map_err(|e|format!("failed to load dynamic argument {} pointer: {:?}",i,e)).map(|v|v.into_pointer_value())}).collect()}}
pub(crate) fn llvm_type<'ctx>(c:&'ctx Context,t:&TypeInfo)->Result<BasicTypeEnum<'ctx>,String>{match t{TypeInfo::F32=>Ok(c.f32_type().into()),TypeInfo::F64=>Ok(c.f64_type().into()),TypeInfo::I8|TypeInfo::U8=>Ok(c.i8_type().into()),TypeInfo::I16|TypeInfo::U16=>Ok(c.i16_type().into()),TypeInfo::I32|TypeInfo::U32=>Ok(c.i32_type().into()),TypeInfo::I64|TypeInfo::U64|TypeInfo::Usize=>Ok(c.i64_type().into()),TypeInfo::I128|TypeInfo::U128=>Ok(c.i128_type().into()),TypeInfo::Bool=>Ok(c.bool_type().into()),TypeInfo::Array{element,length}=>Ok(llvm_type(c,element)?.array_type(*length as u32).into()),TypeInfo::Slice{..}=>Ok(c.struct_type(&[c.ptr_type(inkwell::AddressSpace::default()).into(),c.i64_type().into()],false).into()),TypeInfo::Struct{fields,..}=>Ok(c.struct_type(&fields.iter().map(|f|llvm_type(c,&f.type_info)).collect::<Result<Vec<_>,_>>()?,false).into())}}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indexed_sequence_lowering_contains_runtime_trap() {
        let context = Context::create();
        let compiler = Compiler::new(&context);
        let array = TypeInfo::Array { element: Box::new(TypeInfo::I32), length: 4 };
        let slice = TypeInfo::Slice { element: Box::new(TypeInfo::I32) };
        for sequence in [array, slice] {
            let closure = Closure {
                arguments: vec![sequence.clone()],
                return_type: TypeInfo::I32,
                body: Block::expression(Expr::Index {
                    sequence: Box::new(Expr::Argument(0)),
                    index: Box::new(Expr::Constant(crate::value::Value::Usize(0))),
                }),
            };
            let module = context.create_module("bounds_test");
            compiler.generate_function(&context, &module, "test", &closure).unwrap();
            let ir = module.print_to_string().to_string();
            assert!(ir.contains("llvm.trap"), "generated IR did not contain a bounds trap:\n{}", ir);
        }
    }
}
