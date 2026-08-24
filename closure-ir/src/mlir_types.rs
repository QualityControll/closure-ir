use melior::{ir::Type, Context};

use crate::types::TypeInfo;

/// Maps the frontend's type metadata to MLIR types.
///
/// This is intentionally additive during the migration: the existing LLVM
/// `CompileType` implementation remains in place until the MLIR backend has
/// feature parity.
pub(crate) fn mlir_type<'c>(context: &'c Context, type_info: &TypeInfo) -> Result<Type<'c>, String> {
    let spelling = match type_info {
        TypeInfo::F32 => "f32",
        TypeInfo::F64 => "f64",
        TypeInfo::I8 => "i8",
        TypeInfo::I16 => "i16",
        TypeInfo::I32 => "i32",
        TypeInfo::I64 => "i64",
        TypeInfo::I128 => "i128",
        TypeInfo::U8 => "i8",
        TypeInfo::U16 => "i16",
        TypeInfo::U32 => "i32",
        TypeInfo::U64 => "i64",
        TypeInfo::U128 => "i128",
        // Keep the same representation as the current Inkwell backend.
        TypeInfo::Usize => "i64",
        TypeInfo::Bool => "i1",
        TypeInfo::Array { .. }
        | TypeInfo::Slice { .. }
        | TypeInfo::Struct { .. } => {
            return Err(format!("MLIR type lowering not implemented for {type_info:?}"));
        }
    };

    Type::parse(context, spelling)
        .ok_or_else(|| format!("failed to parse MLIR type `{spelling}`"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lowers_primitive_types() {
        let context = Context::new();

        for type_info in [
            TypeInfo::F32,
            TypeInfo::F64,
            TypeInfo::I8,
            TypeInfo::I16,
            TypeInfo::I32,
            TypeInfo::I64,
            TypeInfo::I128,
            TypeInfo::U8,
            TypeInfo::U16,
            TypeInfo::U32,
            TypeInfo::U64,
            TypeInfo::U128,
            TypeInfo::Usize,
            TypeInfo::Bool,
        ] {
            assert!(mlir_type(&context, &type_info).is_ok(), "failed for {type_info:?}");
        }
    }

    #[test]
    fn reports_unsupported_aggregate_types() {
        let context = Context::new();
        let ty = TypeInfo::Array {
            element: Box::new(TypeInfo::I32),
            length: 4,
        };

        let error = mlir_type(&context, &ty).unwrap_err();
        assert!(error.contains("MLIR type lowering not implemented"));
    }
}
