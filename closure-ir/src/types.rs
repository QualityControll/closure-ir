use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TypeInfo { F32, F64, I8, I16, I32, I64, I128, U8, U16, U32, U64, U128, Usize, Bool, Array { element: Box<TypeInfo>, length: usize }, Slice { element: Box<TypeInfo> }, Struct { name: String, fields: Vec<FieldInfo> } }
impl TypeInfo {
    pub fn is_float(&self)->bool{matches!(self,Self::F32|Self::F64)}
    pub fn is_signed_integer(&self)->bool{matches!(self,Self::I8|Self::I16|Self::I32|Self::I64|Self::I128)}
    pub fn is_unsigned_integer(&self)->bool{matches!(self,Self::U8|Self::U16|Self::U32|Self::U64|Self::U128|Self::Usize)}
    pub fn is_integer(&self)->bool{self.is_signed_integer()||self.is_unsigned_integer()||self.is_bool()}
    pub fn is_bool(&self)->bool{matches!(self,Self::Bool)}
    pub fn is_numeric(&self)->bool{self.is_float()||self.is_signed_integer()||self.is_unsigned_integer()}
    pub fn indexed_element_type(&self)->Option<&TypeInfo>{match self{Self::Array{element,..}|Self::Slice{element}=>Some(element),_=>None}}
    pub fn is_indexable(&self)->bool{self.indexed_element_type().is_some()}
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)] pub struct FieldInfo { pub name:String, pub type_info:TypeInfo }
pub trait CompileType { fn type_info()->TypeInfo; }
macro_rules! primitive { ($t:ty,$v:expr)=>{impl CompileType for $t {fn type_info()->TypeInfo{$v}}}; }
primitive!(f32,TypeInfo::F32); primitive!(f64,TypeInfo::F64); primitive!(i8,TypeInfo::I8); primitive!(i16,TypeInfo::I16); primitive!(i32,TypeInfo::I32); primitive!(i64,TypeInfo::I64); primitive!(i128,TypeInfo::I128); primitive!(u8,TypeInfo::U8); primitive!(u16,TypeInfo::U16); primitive!(u32,TypeInfo::U32); primitive!(u64,TypeInfo::U64); primitive!(u128,TypeInfo::U128); primitive!(bool,TypeInfo::Bool);
impl CompileType for usize {fn type_info()->TypeInfo{TypeInfo::Usize}}
impl<T:CompileType,const N:usize> CompileType for [T;N] {fn type_info()->TypeInfo{TypeInfo::Array{element:Box::new(T::type_info()),length:N}}}
impl<'a,T:CompileType> CompileType for &'a [T] {fn type_info()->TypeInfo{TypeInfo::Slice{element:Box::new(T::type_info())}}}
impl<'a,T:CompileType> CompileType for &'a mut [T] {fn type_info()->TypeInfo{TypeInfo::Slice{element:Box::new(T::type_info())}}}
impl CompileType for () {fn type_info()->TypeInfo{TypeInfo::Struct{name:"()".into(),fields:Vec::new()}}}
macro_rules! impl_tuple_compile_type { ($($T:ident:$index:tt),+)=>{impl<$($T:CompileType),+> CompileType for ($($T,)+){fn type_info()->TypeInfo{TypeInfo::Struct{name:stringify!(($($T,)+)).into(),fields:vec![$(FieldInfo{name:stringify!($index).into(),type_info:$T::type_info()}),+]}}}}; }
impl_tuple_compile_type!(A:0); impl_tuple_compile_type!(A:0,B:1); impl_tuple_compile_type!(A:0,B:1,C:2); impl_tuple_compile_type!(A:0,B:1,C:2,D:3); impl_tuple_compile_type!(A:0,B:1,C:2,D:3,E:4); impl_tuple_compile_type!(A:0,B:1,C:2,D:3,E:4,F:5); impl_tuple_compile_type!(A:0,B:1,C:2,D:3,E:4,F:5,G:6); impl_tuple_compile_type!(A:0,B:1,C:2,D:3,E:4,F:5,G:6,H:7); impl_tuple_compile_type!(A:0,B:1,C:2,D:3,E:4,F:5,G:6,H:7,I:8); impl_tuple_compile_type!(A:0,B:1,C:2,D:3,E:4,F:5,G:6,H:7,I:8,J:9); impl_tuple_compile_type!(A:0,B:1,C:2,D:3,E:4,F:5,G:6,H:7,I:8,J:9,K:10); impl_tuple_compile_type!(A:0,B:1,C:2,D:3,E:4,F:5,G:6,H:7,I:8,J:9,K:10,L:11); impl_tuple_compile_type!(A:0,B:1,C:2,D:3,E:4,F:5,G:6,H:7,I:8,J:9,K:10,L:11,M:12); impl_tuple_compile_type!(A:0,B:1,C:2,D:3,E:4,F:5,G:6,H:7,I:8,J:9,K:10,L:11,M:12,N:13); impl_tuple_compile_type!(A:0,B:1,C:2,D:3,E:4,F:5,G:6,H:7,I:8,J:9,K:10,L:11,M:12,N:13,O:14); impl_tuple_compile_type!(A:0,B:1,C:2,D:3,E:4,F:5,G:6,H:7,I:8,J:9,K:10,L:11,M:12,N:13,O:14,P:15);