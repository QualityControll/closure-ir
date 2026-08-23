use inkwell::{
    context::Context,
    types::BasicTypeEnum,
};

use serde::{Serialize, Deserialize};


// ============================================================
// Type information
// ============================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TypeInfo {
    F32,
    F64,

    I8,
    I16,
    I32,
    I64,
    I128,

    U8,
    U16,
    U32,
    U64,
    U128,

    Bool,

    Struct {
        name: String,
        fields: Vec<FieldInfo>,
    },
}

impl TypeInfo {
    pub fn is_float(&self) -> bool {
        matches!(self, Self::F32 | Self::F64)
    }

    pub fn is_signed_integer(&self) -> bool {
        matches!(
            self,
            Self::I8
                | Self::I16
                | Self::I32
                | Self::I64
                | Self::I128
        )
    }

    pub fn is_unsigned_integer(&self) -> bool {
        matches!(
            self,
            Self::U8
                | Self::U16
                | Self::U32
                | Self::U64
                | Self::U128
        )
    }

    pub fn is_integer(&self) -> bool {
        self.is_signed_integer()
            || self.is_unsigned_integer()
            || self.is_bool()
    }

    pub fn is_bool(&self) -> bool {
        matches!(self, Self::Bool)
    }

    pub fn is_numeric(&self) -> bool {
        self.is_float()
            || self.is_signed_integer()
            || self.is_unsigned_integer()
    }
}


#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldInfo {
    pub name: String,
    pub type_info: TypeInfo,
}


// ============================================================
// CompileType
// ============================================================

pub trait CompileType {
    fn type_info() -> TypeInfo;

    fn llvm_type<'ctx>(
        context: &'ctx Context,
    ) -> BasicTypeEnum<'ctx>;
}


// ============================================================
// Primitive CompileType implementations
// ============================================================

impl CompileType for f32 {
    fn type_info() -> TypeInfo {
        TypeInfo::F32
    }

    fn llvm_type<'ctx>(
        context: &'ctx Context,
    ) -> BasicTypeEnum<'ctx> {
        context.f32_type().into()
    }
}

impl CompileType for f64 {
    fn type_info() -> TypeInfo {
        TypeInfo::F64
    }

    fn llvm_type<'ctx>(
        context: &'ctx Context,
    ) -> BasicTypeEnum<'ctx> {
        context.f64_type().into()
    }
}

impl CompileType for i8 {
    fn type_info() -> TypeInfo {
        TypeInfo::I8
    }

    fn llvm_type<'ctx>(
        context: &'ctx Context,
    ) -> BasicTypeEnum<'ctx> {
        context.i8_type().into()
    }
}

impl CompileType for i16 {
    fn type_info() -> TypeInfo {
        TypeInfo::I16
    }

    fn llvm_type<'ctx>(
        context: &'ctx Context,
    ) -> BasicTypeEnum<'ctx> {
        context.i16_type().into()
    }
}

impl CompileType for i32 {
    fn type_info() -> TypeInfo {
        TypeInfo::I32
    }

    fn llvm_type<'ctx>(
        context: &'ctx Context,
    ) -> BasicTypeEnum<'ctx> {
        context.i32_type().into()
    }
}

impl CompileType for i64 {
    fn type_info() -> TypeInfo {
        TypeInfo::I64
    }

    fn llvm_type<'ctx>(
        context: &'ctx Context,
    ) -> BasicTypeEnum<'ctx> {
        context.i64_type().into()
    }
}

impl CompileType for i128 {
    fn type_info() -> TypeInfo {
        TypeInfo::I128
    }

    fn llvm_type<'ctx>(
        context: &'ctx Context,
    ) -> BasicTypeEnum<'ctx> {
        context.i128_type().into()
    }
}

impl CompileType for u8 {
    fn type_info() -> TypeInfo {
        TypeInfo::U8
    }

    fn llvm_type<'ctx>(
        context: &'ctx Context,
    ) -> BasicTypeEnum<'ctx> {
        context.i8_type().into()
    }
}

impl CompileType for u16 {
    fn type_info() -> TypeInfo {
        TypeInfo::U16
    }

    fn llvm_type<'ctx>(
        context: &'ctx Context,
    ) -> BasicTypeEnum<'ctx> {
        context.i16_type().into()
    }
}

impl CompileType for u32 {
    fn type_info() -> TypeInfo {
        TypeInfo::U32
    }

    fn llvm_type<'ctx>(
        context: &'ctx Context,
    ) -> BasicTypeEnum<'ctx> {
        context.i32_type().into()
    }
}

impl CompileType for u64 {
    fn type_info() -> TypeInfo {
        TypeInfo::U64
    }

    fn llvm_type<'ctx>(
        context: &'ctx Context,
    ) -> BasicTypeEnum<'ctx> {
        context.i64_type().into()
    }
}

impl CompileType for u128 {
    fn type_info() -> TypeInfo {
        TypeInfo::U128
    }

    fn llvm_type<'ctx>(
        context: &'ctx Context,
    ) -> BasicTypeEnum<'ctx> {
        context.i128_type().into()
    }
}

impl CompileType for bool {
    fn type_info() -> TypeInfo {
        TypeInfo::Bool
    }

    fn llvm_type<'ctx>(
        context: &'ctx Context,
    ) -> BasicTypeEnum<'ctx> {
        context.bool_type().into()
    }
}


// ============================================================
// Tuple CompileType implementations
// ============================================================

impl CompileType for () {
    fn type_info() -> TypeInfo {
        TypeInfo::Struct {
            name: "()".to_string(),
            fields: Vec::new(),
        }
    }

    fn llvm_type<'ctx>(
        context: &'ctx Context,
    ) -> BasicTypeEnum<'ctx> {
        context.struct_type(&[], false).into()
    }
}


macro_rules! impl_tuple_compile_type {
    ($($T:ident : $index:tt),+) => {
        impl<$($T: CompileType),+> CompileType
            for ($($T,)+)
        {
            fn type_info() -> TypeInfo {
                TypeInfo::Struct {
                    name: stringify!(($($T,)+)).to_string(),

                    fields: vec![
                        $(
                            FieldInfo {
                                name: stringify!($index).to_string(),
                                type_info: $T::type_info(),
                            }
                        ),+
                    ],
                }
            }

            fn llvm_type<'ctx>(
                context: &'ctx Context,
            ) -> BasicTypeEnum<'ctx> {
                context
                    .struct_type(
                        &[
                            $(
                                $T::llvm_type(context)
                            ),+
                        ],
                        false,
                    )
                    .into()
            }
        }
    };
}

impl_tuple_compile_type!(A: 0);
impl_tuple_compile_type!(A: 0, B: 1);
impl_tuple_compile_type!(A: 0, B: 1, C: 2);
impl_tuple_compile_type!(A: 0, B: 1, C: 2, D: 3);
impl_tuple_compile_type!(A: 0, B: 1, C: 2, D: 3, E: 4);
impl_tuple_compile_type!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5);
impl_tuple_compile_type!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6);
impl_tuple_compile_type!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7);
impl_tuple_compile_type!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8);
impl_tuple_compile_type!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9);
impl_tuple_compile_type!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10);
impl_tuple_compile_type!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10, L: 11);
impl_tuple_compile_type!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10, L: 11, M: 12);
impl_tuple_compile_type!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10, L: 11, M: 12, N: 13);
impl_tuple_compile_type!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10, L: 11, M: 12, N: 13, O: 14);
impl_tuple_compile_type!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10, L: 11, M: 12, N: 13, O: 14, P: 15);