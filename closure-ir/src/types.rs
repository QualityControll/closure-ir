use inkwell::{context::Context, types::{BasicTypeEnum, BasicType}};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TypeInfo { F32, F64, I8, I16, I32, I64, I128, U8, U16, U32, U64, U128, Usize, Bool, Array { element: Box<TypeInfo>, length: usize }, Struct { name: String, fields: Vec<FieldInfo> } }
impl TypeInfo {
    pub fn is_float(&self)->bool{matches!(self,Self::F32|Self::F64)}
    pub fn is_signed_integer(&self)->bool{matches!(self,Self::I8|Self::I16|Self::I32|Self::I64|Self::I128)}
    pub fn is_unsigned_integer(&self)->bool{matches!(self,Self::U8|Self::U16|Self::U32|Self::U64|Self::U128|Self::Usize)}
    pub fn is_integer(&self)->bool{self.is_signed_integer()||self.is_unsigned_integer()||self.is_bool()}
    pub fn is_bool(&self)->bool{matches!(self,Self::Bool)}
    pub fn is_numeric(&self)->bool{self.is_float()||self.is_signed_integer()||self.is_unsigned_integer()}
    pub fn indexed_element_type(&self)->Option<&TypeInfo>{match self{Self::Array{element,..}=>Some(element),_=>None}}
    pub fn is_indexable(&self)->bool{self.indexed_element_type().is_some()}
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)] pub struct FieldInfo { pub name:String, pub type_info:TypeInfo }
pub trait CompileType { fn type_info()->TypeInfo; fn llvm_type<'ctx>(context:&'ctx Context)->BasicTypeEnum<'ctx>; }
macro_rules! primitive { ($t:ty,$v:expr,$m:ident)=>{impl CompileType for $t {fn type_info()->TypeInfo{$v} fn llvm_type<'ctx>(c:&'ctx Context)->BasicTypeEnum<'ctx>{c.$m().into()}}}; }
primitive!(f32,TypeInfo::F32,f32_type); primitive!(f64,TypeInfo::F64,f64_type); primitive!(i8,TypeInfo::I8,i8_type); primitive!(i16,TypeInfo::I16,i16_type); primitive!(i32,TypeInfo::I32,i32_type); primitive!(i64,TypeInfo::I64,i64_type); primitive!(i128,TypeInfo::I128,i128_type); primitive!(u8,TypeInfo::U8,i8_type); primitive!(u16,TypeInfo::U16,i16_type); primitive!(u32,TypeInfo::U32,i32_type); primitive!(u64,TypeInfo::U64,i64_type); primitive!(u128,TypeInfo::U128,i128_type); primitive!(bool,TypeInfo::Bool,bool_type);
impl CompileType for usize {fn type_info()->TypeInfo{TypeInfo::Usize} fn llvm_type<'ctx>(c:&'ctx Context)->BasicTypeEnum<'ctx>{c.i64_type().into()}}
impl<T:CompileType,const N:usize> CompileType for [T;N] {fn type_info()->TypeInfo{TypeInfo::Array{element:Box::new(T::type_info()),length:N}} fn llvm_type<'ctx>(c:&'ctx Context)->BasicTypeEnum<'ctx>{T::llvm_type(c).array_type(N as u32).into()}}
impl CompileType for () {fn type_info()->TypeInfo{TypeInfo::Struct{name:"()".into(),fields:Vec::new()}} fn llvm_type<'ctx>(c:&'ctx Context)->BasicTypeEnum<'ctx>{c.struct_type(&[],false).into()}}
macro_rules! impl_tuple_compile_type { ($($T:ident:$index:tt),+)=>{impl<$($T:CompileType),+> CompileType for ($($T,)+){fn type_info()->TypeInfo{TypeInfo::Struct{name:stringify!(($($T,)+)).into(),fields:vec![$(FieldInfo{name:stringify!($index).into(),type_info:$T::type_info()}),+]}} fn llvm_type<'ctx>(c:&'ctx Context)->BasicTypeEnum<'ctx>{c.struct_type(&[$($T::llvm_type(c)),+],false).into()}}}; }
impl_tuple_compile_type!(A:0); impl_tuple_compile_type!(A:0,B:1); impl_tuple_compile_type!(A:0,B:1,C:2); impl_tuple_compile_type!(A:0,B:1,C:2,D:3); impl_tuple_compile_type!(A:0,B:1,C:2,D:3,E:4); impl_tuple_compile_type!(A:0,B:1,C:2,D:3,E:4,F:5); impl_tuple_compile_type!(A:0,B:1,C:2,D:3,E:4,F:5,G:6); impl_tuple_compile_type!(A:0,B:1,C:2,D:3,E:4,F:5,G:6,H:7); impl_tuple_compile_type!(A:0,B:1,C:2,D:3,E:4,F:5,G:6,H:7,I:8); impl_tuple_compile_type!(A:0,B:1,C:2,D:3,E:4,F:5,G:6,H:7,I:8,J:9); impl_tuple_compile_type!(A:0,B:1,C:2,D:3,E:4,F:5,G:6,H:7,I:8,J:9,K:10); impl_tuple_compile_type!(A:0,B:1,C:2,D:3,E:4,F:5,G:6,H:7,I:8,J:9,K:10,L:11); impl_tuple_compile_type!(A:0,B:1,C:2,D:3,E:4,F:5,G:6,H:7,I:8,J:9,K:10,L:11,M:12); impl_tuple_compile_type!(A:0,B:1,C:2,D:3,E:4,F:5,G:6,H:7,I:8,J:9,K:10,L:11,M:12,N:13); impl_tuple_compile_type!(A:0,B:1,C:2,D:3,E:4,F:5,G:6,H:7,I:8,J:9,K:10,L:11,M:12,N:13,O:14); impl_tuple_compile_type!(A:0,B:1,C:2,D:3,E:4,F:5,G:6,H:7,I:8,J:9,K:10,L:11,M:12,N:13,O:14,P:15);