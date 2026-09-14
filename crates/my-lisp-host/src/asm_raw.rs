//! Raw x86-64 assembly mechanism for Lisp-owned assembler atoms.
//!
//! This module deliberately owns no public instruction surface and no opcode
//! table. Lisp supplies instruction data; iced-x86 owns physical x86 encoding.

use iced_x86::code_asm::CodeAssembler;
use my_lisp::{
    eval_expr, exact_arity, Environment, ErrorKind, Exactness, Expr, LanguageError, Span, Value,
};

fn assembly_error(detail: impl std::fmt::Display, span: Span) -> LanguageError {
    LanguageError::new(
        ErrorKind::InvalidForm,
        format!("x86-assemble-raw: {detail}"),
        span,
    )
}

fn malformed(detail: &str, span: Span) -> LanguageError {
    LanguageError::new(
        ErrorKind::InvalidForm,
        format!("x86-assemble-raw: {detail}"),
        span,
    )
}

fn emit_instruction(
    assembler: &mut CodeAssembler,
    instruction: &Value,
    span: Span,
) -> Result<(), LanguageError> {
    let Value::Pair(operator, operands) = instruction else {
        return Err(malformed("each instruction must be a Lisp list", span));
    };
    let Value::Symbol(operator) = operator.as_ref() else {
        return Err(malformed("instruction operator must be a symbol", span));
    };

    match operator.as_ref() {
        "ret" => {
            if !matches!(operands.as_ref(), Value::Nil) {
                return Err(malformed("ret takes no operands", span));
            }
            assembler
                .ret()
                .map_err(|error| assembly_error(error, span))?;
            Ok(())
        }
        other => Err(malformed(
            &format!("unsupported raw instruction: {other}"),
            span,
        )),
    }
}

fn emit_program(
    assembler: &mut CodeAssembler,
    program: &Value,
    span: Span,
) -> Result<(), LanguageError> {
    let mut current = program;
    loop {
        match current {
            Value::Nil => return Ok(()),
            Value::Pair(instruction, rest) => {
                emit_instruction(assembler, instruction.as_ref(), span)?;
                current = rest.as_ref();
            }
            _ => return Err(malformed("program must be a proper list of instructions", span)),
        }
    }
}

pub(crate) fn evaluate_x86_assemble_raw(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("x86-assemble-raw", arguments, 1, span)?;
    let program = eval_expr(&arguments[0], environment)?;

    let mut assembler = CodeAssembler::new(64).map_err(|error| assembly_error(error, span))?;
    emit_program(&mut assembler, &program, span)?;
    let bytes = assembler
        .assemble(0)
        .map_err(|error| assembly_error(error, span))?;

    Ok(Value::list(
        bytes
            .into_iter()
            .map(|byte| Value::Number(byte as f64, Exactness::Exact)),
    ))
}
