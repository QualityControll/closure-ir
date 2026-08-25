use std::fmt::Write;
use crate::{expr::Closure, types::TypeInfo};
use super::{Ref, RefKind};

pub(crate) struct MlirBuilder<'a>{pub(crate) name:&'a str,pub(crate) closure:&'a Closure,pub(crate) dynamic:bool,pub(crate) text:String,pub(crate) next_value:usize,pub(crate) next_block:usize,pub(crate) current_terminated:bool,pub(crate) refs:Vec<Ref>,pub(crate) args:Vec<Ref>,pub(crate) local_count:usize}
impl<'a> MlirBuilder<'a>{
 pub(crate) fn new(name:&'a str,closure:&'a Closure,dynamic:bool)->Self{Self{name,closure,dynamic,text:String::new(),next_value:0,next_block:0,current_terminated:false,refs:Vec::new(),args:Vec::new(),local_count:0}}
 pub(crate) fn value(&mut self)->String{let s=format!("%v{}",self.next_value);self.next_value+=1;s}
 pub(crate) fn block(&mut self,prefix:&str)->String{let s=format!("{}_{}",prefix,self.next_block);self.next_block+=1;s}
 pub(crate) fn ty(t:&TypeInfo)->String{match t{TypeInfo::F32=>"f32".into(),TypeInfo::F64=>"f64".into(),TypeInfo::I8|TypeInfo::U8=>"i8".into(),TypeInfo::I16|TypeInfo::U16=>"i16".into(),TypeInfo::I32|TypeInfo::U32=>"i32".into(),TypeInfo::I64|TypeInfo::U64|TypeInfo::Usize=>"i64".into(),TypeInfo::I128|TypeInfo::U128=>"i128".into(),TypeInfo::Bool=>"i1".into(),TypeInfo::Array{element,length}=>format!("!llvm.array<{} x {}>",length,Self::ty(element)),TypeInfo::Slice{..}=>"!llvm.struct<(ptr, i64)>".into(),TypeInfo::Struct{fields,..}=>format!("!llvm.struct<({})>",fields.iter().map(|f|Self::ty(&f.type_info)).collect::<Vec<_>>().join(", "))}}
 pub(crate) fn one(&mut self)->String{let v=self.value();self.text.push_str(&format!("    {} = llvm.mlir.constant(1 : i64) : i64\n",v));v}
 pub(crate) fn c_i64(&mut self,n:i64)->String{let v=self.value();self.text.push_str(&format!("    {} = llvm.mlir.constant({} : i64) : i64\n",v,n));v}
 pub(crate) fn c_bool(&mut self,b:bool)->String{let v=self.value();let n=if b{"1"}else{"0"};self.text.push_str(&format!("    {} = llvm.mlir.constant({} : i1) : i1\n",v,n));v}
 pub(crate) fn c_int(&mut self,n:&str,ty:&str)->String{let v=self.value();self.text.push_str(&format!("    {} = llvm.mlir.constant({} : {}) : {}\n",v,n,ty,ty));v}
 pub(crate) fn c_float(&mut self,n:&str,ty:&str)->String{let v=self.value();let literal=if n.contains('.')||n.contains('e')||n.contains('E'){n.to_string()}else{format!("{}.0",n)};self.text.push_str(&format!("    {} = llvm.mlir.constant({} : {}) : {}\n",v,literal,ty,ty));v}
 pub(crate) fn alloca(&mut self,t:&TypeInfo)->String{let n=self.one();let v=self.value();self.text.push_str(&format!("    {} = llvm.alloca {} x {} : (i64) -> !llvm.ptr\n",v,n,Self::ty(t)));v}
 pub(crate) fn load(&mut self,p:&str,t:&TypeInfo)->String{self.load_raw(p,&Self::ty(t))}
 pub(crate) fn load_raw(&mut self,p:&str,t:&str)->String{let v=self.value();self.text.push_str(&format!("    {} = llvm.load {} : !llvm.ptr -> {}\n",v,p,t));v}
 pub(crate) fn store(&mut self,v:&str,p:&str,t:&TypeInfo){self.text.push_str(&format!("    llvm.store {}, {} : {}, !llvm.ptr\n",v,p,Self::ty(t)));}
 pub(crate) fn gep_const(&mut self,p:&str,t:&TypeInfo,indices:&[usize])->String{let v=self.value();let idx=indices.iter().map(|i|i.to_string()).collect::<Vec<_>>().join(", ");self.text.push_str(&format!("    {} = llvm.getelementptr {}[{}] : (!llvm.ptr) -> !llvm.ptr, {}\n",v,p,idx,Self::ty(t)));v}
 pub(crate) fn gep_dynamic(&mut self,p:&str,t:&TypeInfo,idx:&str)->String{self.gep_raw(p,&Self::ty(t),idx)}
 pub(crate) fn gep_array_dynamic(&mut self,p:&str,t:&TypeInfo,idx:&str)->String{let v=self.value();self.text.push_str(&format!("    {} = llvm.getelementptr {}[0, {}] : (!llvm.ptr, i64) -> !llvm.ptr, {}\n",v,p,idx,Self::ty(t)));v}
 pub(crate) fn gep_raw(&mut self,p:&str,elem_ty:&str,idx:&str)->String{let v=self.value();self.text.push_str(&format!("    {} = llvm.getelementptr {}[{}] : (!llvm.ptr, i64) -> !llvm.ptr, {}\n",v,p,idx,elem_ty));v}
 pub(crate) fn emit_branch(&mut self,target:&str){self.text.push_str(&format!("    llvm.br ^{}\n",target));self.current_terminated=true}
 pub(crate) fn emit_cond(&mut self,c:&str,t:&str,f:&str){self.text.push_str(&format!("    llvm.cond_br {}, ^{}, ^{}\n",c,t,f));self.current_terminated=true}
 pub(crate) fn label(&mut self,label:&str){self.text.push_str(&format!("  ^{}:\n",label));self.current_terminated=false}
 pub(crate) fn bounds(&mut self,index:&str,len:&str)->Result<(),String>{let cmp=self.value();self.text.push_str(&format!("    {} = llvm.icmp \"ult\" {}, {} : i64\n",cmp,index,len));let ok=self.block("bounds_ok");let trap=self.block("bounds_trap");self.emit_cond(&cmp,&ok,&trap);self.label(&trap);self.text.push_str("    llvm.intr.trap\n    llvm.unreachable\n");self.current_terminated=true;self.label(&ok);Ok(())}
 pub(crate) fn build(mut self)->Result<String,String>{
  let arg_struct=TypeInfo::Struct{name:"args".into(),fields:self.closure.arguments.iter().enumerate().map(|(i,t)|crate::types::FieldInfo{name:i.to_string(),type_info:t.clone()}).collect()};
  let cap_struct=TypeInfo::Struct{name:"captures".into(),fields:self.closure.captures.iter().enumerate().map(|(i,t)|crate::types::FieldInfo{name:i.to_string(),type_info:t.clone()}).collect()};
  writeln!(self.text,"module {{").unwrap();
  for f in &self.closure.external_functions { let args=f.arguments.iter().map(Self::ty).collect::<Vec<_>>().join(", "); writeln!(self.text,"  llvm.func @{}({}) -> {}",f.name,args,Self::ty(&f.return_type)).unwrap(); }
  writeln!(self.text,"  llvm.func @{}(%captures: !llvm.ptr, %args: !llvm.ptr, %result: !llvm.ptr) attributes {{ llvm.emit_c_interface }} {{",self.name).unwrap();
  self.current_terminated=false;
  for(i,t)in self.closure.captures.iter().enumerate(){let p=self.gep_const("%captures",&cap_struct,&[0,i]);let v=self.load(&p,t);let r=Ref{name:v,ty:t.clone(),kind:RefKind::Value};self.refs.push(r);}
  for(i,t)in self.closure.arguments.iter().enumerate(){let v=if self.dynamic{let idx=self.c_i64(i as i64);let p=self.gep_raw("%args","!llvm.ptr",&idx);let ap=self.load_raw(&p,"!llvm.ptr");self.load(&ap,t)}else{let p=self.gep_const("%args",&arg_struct,&[0,i]);self.load(&p,t)};let r=Ref{name:v,ty:t.clone(),kind:RefKind::Value};self.args.push(r.clone());self.refs.push(r)}
  let result=self.lower_block(&self.closure.body,Some(&self.closure.return_type))?;if let Some(r)=result{self.store(&r.name,"%result",&r.ty)}if !self.current_terminated{self.text.push_str("    llvm.return\n")}
  writeln!(self.text,"  }}").unwrap();
  writeln!(self.text,"  llvm.func @_mlir_ciface_{}(%captures: !llvm.ptr, %args: !llvm.ptr, %result: !llvm.ptr) {{",self.name).unwrap();
  writeln!(self.text,"    llvm.call @{}(%captures, %args, %result) : (!llvm.ptr, !llvm.ptr, !llvm.ptr) -> ()",self.name).unwrap();writeln!(self.text,"    llvm.return\n  }}\n}}\n").unwrap();Ok(self.text)
 }
}
