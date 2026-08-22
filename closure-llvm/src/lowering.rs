use inkwell::{
    builder::Builder,
    context::Context,
    types::{
        BasicTypeEnum,
        StructType,
    },
    values::{
        BasicValueEnum,
        FloatValue,
        FunctionValue,
        IntValue,
        PointerValue,
    },
    FloatPredicate,
    IntPredicate,
};

use crate::{
    compiler::llvm_type,
    expr::Expr,
    operators::{
        binary_operand_type,
        BinaryOp,
        UnaryOp,
    },
    types::TypeInfo,
    value::Value,
};


// ============================================================
// Lowered value
// ============================================================

pub(crate) enum LoweredValue<'ctx> {
    Value(BasicValueEnum<'ctx>),

    Pointer {
        pointer: PointerValue<'ctx>,
        type_info: TypeInfo,
    },
}


// ============================================================
// Lowering
// ============================================================

pub(crate) struct Lowering;


impl<'ctx> Lowering {

    // ========================================================
    // Expression lowering
    // ========================================================

    pub(crate) fn lower_expr(
        &self,
        context: &'ctx Context,
        builder: &Builder<'ctx>,
        function: FunctionValue<'ctx>,
        arguments: &[PointerValue<'ctx>],
        argument_types: &[TypeInfo],
        expected_type: &TypeInfo,
        expr: &Expr,
    ) -> Result<LoweredValue<'ctx>, String> {
        match expr {
            Expr::Argument(index) => {
                let pointer =
                    *arguments
                        .get(*index)
                        .ok_or_else(|| {
                            format!(
                                "argument index {} out of bounds",
                                index
                            )
                        })?;

                let type_info =
                    argument_types
                        .get(*index)
                        .cloned()
                        .ok_or_else(|| {
                            format!(
                                "argument type index {} out of bounds",
                                index
                            )
                        })?;

                Ok(
                    LoweredValue::Pointer {
                        pointer,
                        type_info,
                    }
                )
            }

            Expr::Constant(value) =>
                self.lower_constant(
                    context,
                    value,
                ),

            Expr::Field {
                object,
                name,
            } =>
                self.lower_field(
                    context,
                    builder,
                    function,
                    arguments,
                    argument_types,
                    expected_type,
                    object,
                    name,
                ),

            Expr::IfElse {
                condition,
                then_branch,
                else_branch,
            } =>
                self.lower_if_else(
                    context,
                    builder,
                    function,
                    arguments,
                    argument_types,
                    expected_type,
                    condition,
                    then_branch,
                    else_branch,
                ),

            Expr::Add { lhs, rhs } =>
                self.lower_binary(
                    context,
                    builder,
                    function,
                    arguments,
                    argument_types,
                    expected_type,
                    lhs,
                    rhs,
                    BinaryOp::Add,
                ),

            Expr::Sub { lhs, rhs } =>
                self.lower_binary(
                    context,
                    builder,
                    function,
                    arguments,
                    argument_types,
                    expected_type,
                    lhs,
                    rhs,
                    BinaryOp::Sub,
                ),

            Expr::Mul { lhs, rhs } =>
                self.lower_binary(
                    context,
                    builder,
                    function,
                    arguments,
                    argument_types,
                    expected_type,
                    lhs,
                    rhs,
                    BinaryOp::Mul,
                ),

            Expr::Div { lhs, rhs } =>
                self.lower_binary(
                    context,
                    builder,
                    function,
                    arguments,
                    argument_types,
                    expected_type,
                    lhs,
                    rhs,
                    BinaryOp::Div,
                ),

            Expr::Rem { lhs, rhs } =>
                self.lower_binary(
                    context,
                    builder,
                    function,
                    arguments,
                    argument_types,
                    expected_type,
                    lhs,
                    rhs,
                    BinaryOp::Rem,
                ),

            Expr::Eq { lhs, rhs } =>
                self.lower_binary(
                    context,
                    builder,
                    function,
                    arguments,
                    argument_types,
                    expected_type,
                    lhs,
                    rhs,
                    BinaryOp::Eq,
                ),

            Expr::Ne { lhs, rhs } =>
                self.lower_binary(
                    context,
                    builder,
                    function,
                    arguments,
                    argument_types,
                    expected_type,
                    lhs,
                    rhs,
                    BinaryOp::Ne,
                ),

            Expr::Lt { lhs, rhs } =>
                self.lower_binary(
                    context,
                    builder,
                    function,
                    arguments,
                    argument_types,
                    expected_type,
                    lhs,
                    rhs,
                    BinaryOp::Lt,
                ),

            Expr::Le { lhs, rhs } =>
                self.lower_binary(
                    context,
                    builder,
                    function,
                    arguments,
                    argument_types,
                    expected_type,
                    lhs,
                    rhs,
                    BinaryOp::Le,
                ),

            Expr::Gt { lhs, rhs } =>
                self.lower_binary(
                    context,
                    builder,
                    function,
                    arguments,
                    argument_types,
                    expected_type,
                    lhs,
                    rhs,
                    BinaryOp::Gt,
                ),

            Expr::Ge { lhs, rhs } =>
                self.lower_binary(
                    context,
                    builder,
                    function,
                    arguments,
                    argument_types,
                    expected_type,
                    lhs,
                    rhs,
                    BinaryOp::Ge,
                ),

            Expr::And { lhs, rhs } =>
                self.lower_binary(
                    context,
                    builder,
                    function,
                    arguments,
                    argument_types,
                    expected_type,
                    lhs,
                    rhs,
                    BinaryOp::And,
                ),

            Expr::Or { lhs, rhs } =>
                self.lower_binary(
                    context,
                    builder,
                    function,
                    arguments,
                    argument_types,
                    expected_type,
                    lhs,
                    rhs,
                    BinaryOp::Or,
                ),

            Expr::BitAnd { lhs, rhs } =>
                self.lower_binary(
                    context,
                    builder,
                    function,
                    arguments,
                    argument_types,
                    expected_type,
                    lhs,
                    rhs,
                    BinaryOp::BitAnd,
                ),

            Expr::BitOr { lhs, rhs } =>
                self.lower_binary(
                    context,
                    builder,
                    function,
                    arguments,
                    argument_types,
                    expected_type,
                    lhs,
                    rhs,
                    BinaryOp::BitOr,
                ),

            Expr::BitXor { lhs, rhs } =>
                self.lower_binary(
                    context,
                    builder,
                    function,
                    arguments,
                    argument_types,
                    expected_type,
                    lhs,
                    rhs,
                    BinaryOp::BitXor,
                ),

            Expr::Shl { lhs, rhs } =>
                self.lower_binary(
                    context,
                    builder,
                    function,
                    arguments,
                    argument_types,
                    expected_type,
                    lhs,
                    rhs,
                    BinaryOp::Shl,
                ),

            Expr::Shr { lhs, rhs } =>
                self.lower_binary(
                    context,
                    builder,
                    function,
                    arguments,
                    argument_types,
                    expected_type,
                    lhs,
                    rhs,
                    BinaryOp::Shr,
                ),

            Expr::Not { operand } =>
                self.lower_unary(
                    context,
                    builder,
                    function,
                    arguments,
                    argument_types,
                    expected_type,
                    operand,
                    UnaryOp::Not,
                ),

            Expr::Neg { operand } =>
                self.lower_unary(
                    context,
                    builder,
                    function,
                    arguments,
                    argument_types,
                    expected_type,
                    operand,
                    UnaryOp::Neg,
                ),
        }
    }


    // ========================================================
    // Field
    // ========================================================

    fn lower_field(
        &self,
        context: &'ctx Context,
        builder: &Builder<'ctx>,
        function: FunctionValue<'ctx>,
        arguments: &[PointerValue<'ctx>],
        argument_types: &[TypeInfo],
        expected_type: &TypeInfo,
        object: &Expr,
        name: &str,
    ) -> Result<LoweredValue<'ctx>, String> {
        let object =
            self.lower_expr(
                context,
                builder,
                function,
                arguments,
                argument_types,
                expected_type,
                object,
            )?;

        let (object_pointer, object_type) =
            match object {
                LoweredValue::Pointer {
                    pointer,
                    type_info,
                } => (pointer, type_info),

                LoweredValue::Value(_) =>
                    return Err(format!(
                        "cannot access field `{}` on a value",
                        name
                    )),
            };

        let fields =
            match &object_type {
                TypeInfo::Struct {
                    fields,
                    ..
                } => fields,

                _ =>
                    return Err(format!(
                        "cannot access field `{}` on non-struct type",
                        name
                    )),
            };

        let (field_index, field_type) =
            fields
                .iter()
                .enumerate()
                .find_map(|(index, field)| {
                    if field.name == name {
                        Some((
                            index,
                            field.type_info.clone(),
                        ))
                    } else {
                        None
                    }
                })
                .ok_or_else(|| {
                    format!(
                        "field `{}` not found",
                        name
                    )
                })?;

        let struct_type =
            llvm_struct_type(
                context,
                &object_type,
            )?;

        let field_pointer =
            builder
                .build_struct_gep(
                    struct_type,
                    object_pointer,
                    field_index as u32,
                    &format!("{}_ptr", name),
                )
                .map_err(|error| {
                    format!(
                        "failed to build GEP for field `{}`: {:?}",
                        name,
                        error
                    )
                })?;

        Ok(
            LoweredValue::Pointer {
                pointer: field_pointer,
                type_info: field_type,
            }
        )
    }


    // ========================================================
    // If / else
    // ========================================================

    fn lower_if_else(
        &self,
        context: &'ctx Context,
        builder: &Builder<'ctx>,
        function: FunctionValue<'ctx>,
        arguments: &[PointerValue<'ctx>],
        argument_types: &[TypeInfo],
        expected_type: &TypeInfo,
        condition: &Expr,
        then_branch: &Expr,
        else_branch: &Expr,
    ) -> Result<LoweredValue<'ctx>, String> {
        let condition =
            self.lower_expr(
                context,
                builder,
                function,
                arguments,
                argument_types,
                &TypeInfo::Bool,
                condition,
            )?;

        let condition =
            self.materialize_value(
                context,
                builder,
                condition,
            )?;

        let condition =
            match condition {
                BasicValueEnum::IntValue(value)
                    if value.get_type().get_bit_width() == 1 =>
                {
                    value
                }

                BasicValueEnum::IntValue(_) =>
                    return Err(
                        "if condition must be bool"
                            .to_string()
                    ),

                _ =>
                    return Err(
                        "if condition must be bool"
                            .to_string()
                    ),
            };

        let then_block =
            context.append_basic_block(
                function,
                "then",
            );

        let else_block =
            context.append_basic_block(
                function,
                "else",
            );

        let merge_block =
            context.append_basic_block(
                function,
                "if_merge",
            );

        builder
            .build_conditional_branch(
                condition,
                then_block,
                else_block,
            )
            .map_err(|error| {
                format!(
                    "failed to build conditional branch: {:?}",
                    error
                )
            })?;

        // ----------------------------------------------------
        // Then
        // ----------------------------------------------------

        builder.position_at_end(then_block);

        let then_value =
            self.lower_expr(
                context,
                builder,
                function,
                arguments,
                argument_types,
                expected_type,
                then_branch,
            )?;

        let then_value =
            self.materialize_value(
                context,
                builder,
                then_value,
            )?;

        let then_end =
            builder
                .get_insert_block()
                .ok_or_else(|| {
                    "missing then block"
                        .to_string()
                })?;

        if then_end.get_terminator().is_none() {
            builder
                .build_unconditional_branch(
                    merge_block,
                )
                .map_err(|error| {
                    format!(
                        "failed to branch from then block: {:?}",
                        error
                    )
                })?;
        }

        // ----------------------------------------------------
        // Else
        // ----------------------------------------------------

        builder.position_at_end(else_block);

        let else_value =
            self.lower_expr(
                context,
                builder,
                function,
                arguments,
                argument_types,
                expected_type,
                else_branch,
            )?;

        let else_value =
            self.materialize_value(
                context,
                builder,
                else_value,
            )?;

        let else_end =
            builder
                .get_insert_block()
                .ok_or_else(|| {
                    "missing else block"
                        .to_string()
                })?;

        if else_end.get_terminator().is_none() {
            builder
                .build_unconditional_branch(
                    merge_block,
                )
                .map_err(|error| {
                    format!(
                        "failed to branch from else block: {:?}",
                        error
                    )
                })?;
        }

        // ----------------------------------------------------
        // Merge
        // ----------------------------------------------------

        builder.position_at_end(merge_block);

        let phi =
            builder
                .build_phi(
                    llvm_type(
                        context,
                        expected_type,
                    )?,
                    "if_result",
                )
                .map_err(|error| {
                    format!(
                        "failed to build if/else PHI: {:?}",
                        error
                    )
                })?;

        phi.add_incoming(&[
            (&then_value, then_end),
            (&else_value, else_end),
        ]);

        Ok(
            LoweredValue::Value(
                phi.as_basic_value()
            )
        )
    }


    // ========================================================
    // Constants
    // ========================================================

    fn lower_constant(
        &self,
        context: &'ctx Context,
        value: &Value,
    ) -> Result<LoweredValue<'ctx>, String> {
        let value =
            match value {
                Value::F32(value) =>
                    context
                        .f32_type()
                        .const_float(*value as f64)
                        .into(),

                Value::F64(value) =>
                    context
                        .f64_type()
                        .const_float(*value)
                        .into(),

                Value::I8(value) =>
                    context
                        .i8_type()
                        .const_int(
                            *value as i64 as u64,
                            true,
                        )
                        .into(),

                Value::I16(value) =>
                    context
                        .i16_type()
                        .const_int(
                            *value as i64 as u64,
                            true,
                        )
                        .into(),

                Value::I32(value) =>
                    context
                        .i32_type()
                        .const_int(
                            *value as i64 as u64,
                            true,
                        )
                        .into(),

                Value::I64(value) =>
                    context
                        .i64_type()
                        .const_int(
                            *value as u64,
                            true,
                        )
                        .into(),

                Value::I128(value) =>
                    context
                        .i128_type()
                        .const_int_arbitrary_precision(
                            &[
                                *value as u128 as u64,
                                ((*value as u128) >> 64) as u64,
                            ],
                        )
                        .into(),

                Value::U8(value) =>
                    context
                        .i8_type()
                        .const_int(
                            *value as u64,
                            false,
                        )
                        .into(),

                Value::U16(value) =>
                    context
                        .i16_type()
                        .const_int(
                            *value as u64,
                            false,
                        )
                        .into(),

                Value::U32(value) =>
                    context
                        .i32_type()
                        .const_int(
                            *value as u64,
                            false,
                        )
                        .into(),

                Value::U64(value) =>
                    context
                        .i64_type()
                        .const_int(
                            *value,
                            false,
                        )
                        .into(),

                Value::U128(value) =>
                    context
                        .i128_type()
                        .const_int_arbitrary_precision(
                            &[
                                *value as u64,
                                (*value >> 64) as u64,
                            ],
                        )
                        .into(),

                Value::Bool(value) =>
                    context
                        .bool_type()
                        .const_int(
                            if *value { 1 } else { 0 },
                            false,
                        )
                        .into(),
            };

        Ok(
            LoweredValue::Value(value)
        )
    }


    // ========================================================
    // Unary
    // ========================================================

    fn lower_unary(
        &self,
        context: &'ctx Context,
        builder: &Builder<'ctx>,
        function: FunctionValue<'ctx>,
        arguments: &[PointerValue<'ctx>],
        argument_types: &[TypeInfo],
        expected_type: &TypeInfo,
        operand: &Expr,
        operation: UnaryOp,
    ) -> Result<LoweredValue<'ctx>, String> {
        let operand =
            self.lower_expr(
                context,
                builder,
                function,
                arguments,
                argument_types,
                expected_type,
                operand,
            )?;

        let operand =
            self.materialize_value(
                context,
                builder,
                operand,
            )?;

        match operation {
            UnaryOp::Not => {
                let value =
                    match operand {
                        BasicValueEnum::IntValue(value) =>
                            value,

                        _ =>
                            return Err(
                                "unary ! requires an integer or bool operand"
                                    .to_string()
                            ),
                    };

                let result =
                    builder
                        .build_not(
                            value,
                            "not",
                        )
                        .map_err(|e| {
                            format!(
                                "failed to build not: {:?}",
                                e
                            )
                        })?;

                Ok(
                    LoweredValue::Value(
                        result.into()
                    )
                )
            }

            UnaryOp::Neg => {
                match operand {
                    BasicValueEnum::IntValue(value) => {
                        let result =
                            builder
                                .build_int_neg(
                                    value,
                                    "neg",
                                )
                                .map_err(|e| {
                                    format!(
                                        "failed to build integer negation: {:?}",
                                        e
                                    )
                                })?;

                        Ok(
                            LoweredValue::Value(
                                result.into()
                            )
                        )
                    }

                    BasicValueEnum::FloatValue(value) => {
                        let result =
                            builder
                                .build_float_neg(
                                    value,
                                    "neg",
                                )
                                .map_err(|e| {
                                    format!(
                                        "failed to build float negation: {:?}",
                                        e
                                    )
                                })?;

                        Ok(
                            LoweredValue::Value(
                                result.into()
                            )
                        )
                    }

                    _ =>
                        Err(
                            "unary - requires numeric operand"
                                .to_string()
                        ),
                }
            }
        }
    }


    // ========================================================
    // Binary
    // ========================================================

    fn lower_binary(
        &self,
        context: &'ctx Context,
        builder: &Builder<'ctx>,
        function: FunctionValue<'ctx>,
        arguments: &[PointerValue<'ctx>],
        argument_types: &[TypeInfo],
        expected_type: &TypeInfo,
        lhs: &Expr,
        rhs: &Expr,
        operation: BinaryOp,
    ) -> Result<LoweredValue<'ctx>, String> {
        let operand_type =
            binary_operand_type(
                argument_types,
                lhs,
                rhs,
                expected_type,
                &operation,
            )?;

        let lhs =
            self.lower_expr(
                context,
                builder,
                function,
                arguments,
                argument_types,
                &operand_type,
                lhs,
            )?;

        let lhs =
            self.materialize_value(
                context,
                builder,
                lhs,
            )?;

        let rhs =
            self.lower_expr(
                context,
                builder,
                function,
                arguments,
                argument_types,
                &operand_type,
                rhs,
            )?;

        let rhs =
            self.materialize_value(
                context,
                builder,
                rhs,
            )?;

        match (lhs, rhs) {
            (
                BasicValueEnum::FloatValue(lhs),
                BasicValueEnum::FloatValue(rhs),
            ) =>
                self.lower_float_binary(
                    builder,
                    lhs,
                    rhs,
                    operation,
                ),

            (
                BasicValueEnum::IntValue(lhs),
                BasicValueEnum::IntValue(rhs),
            ) =>
                self.lower_int_binary(
                    builder,
                    lhs,
                    rhs,
                    operation,
                    &operand_type,
                ),

            _ =>
                Err(
                    "binary operands must have matching numeric or integer types"
                        .to_string()
                ),
        }
    }


    fn lower_float_binary(
        &self,
        builder: &Builder<'ctx>,
        lhs: FloatValue<'ctx>,
        rhs: FloatValue<'ctx>,
        operation: BinaryOp,
    ) -> Result<LoweredValue<'ctx>, String> {
        match operation {
            BinaryOp::Add =>
                Ok(LoweredValue::Value(
                    builder
                        .build_float_add(
                            lhs,
                            rhs,
                            "add",
                        )
                        .map_err(|e| format!("{:?}", e))?
                        .into()
                )),

            BinaryOp::Sub =>
                Ok(LoweredValue::Value(
                    builder
                        .build_float_sub(
                            lhs,
                            rhs,
                            "sub",
                        )
                        .map_err(|e| format!("{:?}", e))?
                        .into()
                )),

            BinaryOp::Mul =>
                Ok(LoweredValue::Value(
                    builder
                        .build_float_mul(
                            lhs,
                            rhs,
                            "mul",
                        )
                        .map_err(|e| format!("{:?}", e))?
                        .into()
                )),

            BinaryOp::Div =>
                Ok(LoweredValue::Value(
                    builder
                        .build_float_div(
                            lhs,
                            rhs,
                            "div",
                        )
                        .map_err(|e| format!("{:?}", e))?
                        .into()
                )),

            BinaryOp::Rem =>
                Ok(LoweredValue::Value(
                    builder
                        .build_float_rem(
                            lhs,
                            rhs,
                            "rem",
                        )
                        .map_err(|e| format!("{:?}", e))?
                        .into()
                )),

            BinaryOp::Eq =>
                self.float_compare(
                    builder,
                    lhs,
                    rhs,
                    FloatPredicate::OEQ,
                ),

            BinaryOp::Ne =>
                self.float_compare(
                    builder,
                    lhs,
                    rhs,
                    FloatPredicate::ONE,
                ),

            BinaryOp::Lt =>
                self.float_compare(
                    builder,
                    lhs,
                    rhs,
                    FloatPredicate::OLT,
                ),

            BinaryOp::Le =>
                self.float_compare(
                    builder,
                    lhs,
                    rhs,
                    FloatPredicate::OLE,
                ),

            BinaryOp::Gt =>
                self.float_compare(
                    builder,
                    lhs,
                    rhs,
                    FloatPredicate::OGT,
                ),

            BinaryOp::Ge =>
                self.float_compare(
                    builder,
                    lhs,
                    rhs,
                    FloatPredicate::OGE,
                ),

            _ =>
                Err(
                    "unsupported floating-point operator"
                        .to_string()
                ),
        }
    }


    fn float_compare(
        &self,
        builder: &Builder<'ctx>,
        lhs: FloatValue<'ctx>,
        rhs: FloatValue<'ctx>,
        predicate: FloatPredicate,
    ) -> Result<LoweredValue<'ctx>, String> {
        let result =
            builder
                .build_float_compare(
                    predicate,
                    lhs,
                    rhs,
                    "cmp",
                )
                .map_err(|e| {
                    format!(
                        "failed to build float comparison: {:?}",
                        e
                    )
                })?;

        Ok(
            LoweredValue::Value(
                result.into()
            )
        )
    }


    fn lower_int_binary(
        &self,
        builder: &Builder<'ctx>,
        lhs: IntValue<'ctx>,
        rhs: IntValue<'ctx>,
        operation: BinaryOp,
        type_info: &TypeInfo,
    ) -> Result<LoweredValue<'ctx>, String> {
        let unsigned =
            type_info.is_unsigned_integer();

        let result =
            match operation {
                BinaryOp::Add =>
                    builder.build_int_add(
                        lhs,
                        rhs,
                        "add",
                    ),

                BinaryOp::Sub =>
                    builder.build_int_sub(
                        lhs,
                        rhs,
                        "sub",
                    ),

                BinaryOp::Mul =>
                    builder.build_int_mul(
                        lhs,
                        rhs,
                        "mul",
                    ),

                BinaryOp::Div => {
                    if unsigned {
                        builder.build_int_unsigned_div(
                            lhs,
                            rhs,
                            "div",
                        )
                    } else {
                        builder.build_int_signed_div(
                            lhs,
                            rhs,
                            "div",
                        )
                    }
                }

                BinaryOp::Rem => {
                    if unsigned {
                        builder.build_int_unsigned_rem(
                            lhs,
                            rhs,
                            "rem",
                        )
                    } else {
                        builder.build_int_signed_rem(
                            lhs,
                            rhs,
                            "rem",
                        )
                    }
                }

                BinaryOp::Eq =>
                    builder.build_int_compare(
                        IntPredicate::EQ,
                        lhs,
                        rhs,
                        "eq",
                    ),

                BinaryOp::Ne =>
                    builder.build_int_compare(
                        IntPredicate::NE,
                        lhs,
                        rhs,
                        "ne",
                    ),

                BinaryOp::Lt =>
                    builder.build_int_compare(
                        if unsigned {
                            IntPredicate::ULT
                        } else {
                            IntPredicate::SLT
                        },
                        lhs,
                        rhs,
                        "lt",
                    ),

                BinaryOp::Le =>
                    builder.build_int_compare(
                        if unsigned {
                            IntPredicate::ULE
                        } else {
                            IntPredicate::SLE
                        },
                        lhs,
                        rhs,
                        "le",
                    ),

                BinaryOp::Gt =>
                    builder.build_int_compare(
                        if unsigned {
                            IntPredicate::UGT
                        } else {
                            IntPredicate::SGT
                        },
                        lhs,
                        rhs,
                        "gt",
                    ),

                BinaryOp::Ge =>
                    builder.build_int_compare(
                        if unsigned {
                            IntPredicate::UGE
                        } else {
                            IntPredicate::SGE
                        },
                        lhs,
                        rhs,
                        "ge",
                    ),

                BinaryOp::And =>
                    builder.build_and(
                        lhs,
                        rhs,
                        "and",
                    ),

                BinaryOp::Or =>
                    builder.build_or(
                        lhs,
                        rhs,
                        "or",
                    ),

                BinaryOp::BitAnd =>
                    builder.build_and(
                        lhs,
                        rhs,
                        "bitand",
                    ),

                BinaryOp::BitOr =>
                    builder.build_or(
                        lhs,
                        rhs,
                        "bitor",
                    ),

                BinaryOp::BitXor =>
                    builder.build_xor(
                        lhs,
                        rhs,
                        "bitxor",
                    ),

                BinaryOp::Shl =>
                    builder.build_left_shift(
                        lhs,
                        rhs,
                        "shl",
                    ),

                BinaryOp::Shr =>
                    builder.build_right_shift(
                        lhs,
                        rhs,
                        !unsigned,
                        "shr",
                    ),
            }
            .map_err(|e| {
                format!(
                    "failed to build integer operation: {:?}",
                    e
                )
            })?;

        Ok(
            LoweredValue::Value(
                result.into()
            )
        )
    }


    // ========================================================
    // Materialize
    // ========================================================

    pub(crate) fn materialize_value(
        &self,
        context: &'ctx Context,
        builder: &Builder<'ctx>,
        value: LoweredValue<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        match value {
            LoweredValue::Value(value) =>
                Ok(value),

            LoweredValue::Pointer {
                pointer,
                type_info,
            } => {
                let llvm_type =
                    llvm_type(
                        context,
                        &type_info,
                    )?;

                builder
                    .build_load(
                        llvm_type,
                        pointer,
                        "load",
                    )
                    .map_err(|error| {
                        format!(
                            "failed to build load: {:?}",
                            error
                        )
                    })
            }
        }
    }
}


// ============================================================
// Struct type
// ============================================================

fn llvm_struct_type<'ctx>(
    context: &'ctx Context,
    type_info: &TypeInfo,
) -> Result<StructType<'ctx>, String> {
    match llvm_type(
        context,
        type_info,
    )? {
        BasicTypeEnum::StructType(struct_type) =>
            Ok(struct_type),

        _ =>
            Err(
                "expected struct LLVM type"
                    .to_string()
            ),
    }
}