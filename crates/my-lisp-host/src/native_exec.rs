use my_lisp::{
    eval_expr, Environment, ErrorKind, Exactness, Expr, LanguageError, Span, Value,
};
use std::ffi::c_void;

const PROT_READ: i32 = 0x1;
const PROT_WRITE: i32 = 0x2;
const PROT_EXEC: i32 = 0x4;
const MAP_PRIVATE: i32 = 0x02;
const MAP_ANONYMOUS: i32 = 0x20;
const MAX_EXACT_LISP_INTEGER: u64 = 9_007_199_254_740_991;
const MAX_NATIVE_ARENA_BYTES: usize = 1_048_576;

extern "C" {
    fn mmap(
        address: *mut c_void,
        length: usize,
        protection: i32,
        flags: i32,
        file_descriptor: i32,
        offset: isize,
    ) -> *mut c_void;
    fn mprotect(address: *mut c_void, length: usize, protection: i32) -> i32;
    fn munmap(address: *mut c_void, length: usize) -> i32;
}

fn mechanism_error(operation: &str, action: &str, span: Span) -> LanguageError {
    LanguageError::new(
        ErrorKind::InvalidForm,
        format!(
            "{operation}: {action} failed: {}",
            std::io::Error::last_os_error()
        ),
        span,
    )
}

fn expect_machine_bytes(
    value: &Value,
    operation: &str,
    span: Span,
) -> Result<Vec<u8>, LanguageError> {
    let mut bytes = Vec::new();
    let mut current = value;

    loop {
        match current {
            Value::Nil => return Ok(bytes),
            Value::Pair(head, tail) => {
                let Value::Number(number, Exactness::Exact) = **head else {
                    return Err(LanguageError::new(
                        ErrorKind::Type,
                        format!("{operation} expects exact byte integers 0-255"),
                        span,
                    ));
                };
                if number.fract() != 0.0 || !(0.0..=255.0).contains(&number) {
                    return Err(LanguageError::new(
                        ErrorKind::Type,
                        format!("{operation} expects exact byte integers 0-255"),
                        span,
                    ));
                }
                bytes.push(number as u8);
                current = tail;
            }
            _ => {
                return Err(LanguageError::new(
                    ErrorKind::Type,
                    format!(
                        "{operation} expects a proper list of exact byte integers 0-255"
                    ),
                    span,
                ));
            }
        }
    }
}

fn expect_arena_length(value: &Value, span: Span) -> Result<usize, LanguageError> {
    let Value::Number(number, Exactness::Exact) = value else {
        return Err(LanguageError::new(
            ErrorKind::Type,
            "native-call-u64-raw expects an exact positive arena byte count",
            span,
        ));
    };
    if number.fract() != 0.0
        || *number < 1.0
        || *number > MAX_NATIVE_ARENA_BYTES as f64
    {
        return Err(LanguageError::new(
            ErrorKind::InvalidForm,
            format!(
                "native-call-u64-raw arena byte count must be 1..={MAX_NATIVE_ARENA_BYTES}"
            ),
            span,
        ));
    }
    Ok(*number as usize)
}

fn prepare_executable(
    bytes: &[u8],
    operation: &str,
    span: Span,
) -> Result<*mut c_void, LanguageError> {
    if bytes.is_empty() {
        return Err(LanguageError::new(
            ErrorKind::InvalidForm,
            format!("{operation} refuses an empty machine-code buffer"),
            span,
        ));
    }

    let length = bytes.len();
    let memory = unsafe {
        mmap(
            std::ptr::null_mut(),
            length,
            PROT_READ | PROT_WRITE,
            MAP_PRIVATE | MAP_ANONYMOUS,
            -1,
            0,
        )
    };

    if memory as isize == -1 {
        return Err(mechanism_error(operation, "mmap RW", span));
    }

    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), memory.cast::<u8>(), length);
    }

    if unsafe { mprotect(memory, length, PROT_READ | PROT_EXEC) } != 0 {
        let error = mechanism_error(operation, "mprotect RW->RX", span);
        unsafe {
            munmap(memory, length);
        }
        return Err(error);
    }

    Ok(memory)
}

fn execute_u64(bytes: &[u8], span: Span) -> Result<u64, LanguageError> {
    let operation = "native-call-u64-raw";
    let memory = prepare_executable(bytes, operation, span)?;
    let function: unsafe extern "C" fn() -> u64 = unsafe { std::mem::transmute(memory) };
    let result = unsafe { function() };

    if unsafe { munmap(memory, bytes.len()) } != 0 {
        return Err(mechanism_error(operation, "munmap code", span));
    }

    Ok(result)
}

fn execute_u64_with_arena(
    bytes: &[u8],
    arena_length: usize,
    span: Span,
) -> Result<u64, LanguageError> {
    let operation = "native-call-u64-raw";
    let code = prepare_executable(bytes, operation, span)?;
    let arena = unsafe {
        mmap(
            std::ptr::null_mut(),
            arena_length,
            PROT_READ | PROT_WRITE,
            MAP_PRIVATE | MAP_ANONYMOUS,
            -1,
            0,
        )
    };

    if arena as isize == -1 {
        let error = mechanism_error(operation, "mmap arena RW", span);
        unsafe {
            munmap(code, bytes.len());
        }
        return Err(error);
    }

    let function: unsafe extern "C" fn(*mut u8) -> u64 = unsafe { std::mem::transmute(code) };
    let result = unsafe { function(arena.cast::<u8>()) };

    let code_unmap = unsafe { munmap(code, bytes.len()) };
    let arena_unmap = unsafe { munmap(arena, arena_length) };
    if code_unmap != 0 {
        return Err(mechanism_error(operation, "munmap code", span));
    }
    if arena_unmap != 0 {
        return Err(mechanism_error(operation, "munmap arena", span));
    }

    Ok(result)
}

fn result_value(result: u64, operation: &str, span: Span) -> Result<Value, LanguageError> {
    if result > MAX_EXACT_LISP_INTEGER {
        return Err(LanguageError::new(
            ErrorKind::InvalidForm,
            format!("{operation} result exceeds my-lisp's exact integer range"),
            span,
        ));
    }

    Ok(Value::Number(result as f64, Exactness::Exact))
}

pub(crate) fn evaluate_native_call_u64_raw(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    let operation = "native-call-u64-raw";
    if !(1..=2).contains(&arguments.len()) {
        return Err(LanguageError::new(
            ErrorKind::Arity,
            format!(
                "{operation}: expected 1 or 2 arguments; received {}",
                arguments.len()
            ),
            span,
        ));
    }

    let byte_value = eval_expr(&arguments[0], environment)?;
    let bytes = expect_machine_bytes(&byte_value, operation, arguments[0].span)?;
    let result = if arguments.len() == 1 {
        execute_u64(&bytes, span)?
    } else {
        let arena_value = eval_expr(&arguments[1], environment)?;
        let arena_length = expect_arena_length(&arena_value, arguments[1].span)?;
        execute_u64_with_arena(&bytes, arena_length, span)?
    };
    result_value(result, operation, span)
}
