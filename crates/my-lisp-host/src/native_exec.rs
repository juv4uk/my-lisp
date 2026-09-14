use my_lisp::{
    eval_expr, exact_arity, Environment, ErrorKind, Exactness, Expr, LanguageError, Span, Value,
};
use std::ffi::c_void;

const PROT_READ: i32 = 0x1;
const PROT_WRITE: i32 = 0x2;
const PROT_EXEC: i32 = 0x4;
const MAP_PRIVATE: i32 = 0x02;
const MAP_ANONYMOUS: i32 = 0x20;
const MAX_EXACT_LISP_INTEGER: u64 = 9_007_199_254_740_991;

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

fn mechanism_error(action: &str, span: Span) -> LanguageError {
    LanguageError::new(
        ErrorKind::InvalidForm,
        format!(
            "native-call-u64-raw: {action} failed: {}",
            std::io::Error::last_os_error()
        ),
        span,
    )
}

fn expect_machine_bytes(value: &Value, span: Span) -> Result<Vec<u8>, LanguageError> {
    let mut bytes = Vec::new();
    let mut current = value;

    loop {
        match current {
            Value::Nil => return Ok(bytes),
            Value::Pair(head, tail) => {
                let Value::Number(number, Exactness::Exact) = **head else {
                    return Err(LanguageError::new(
                        ErrorKind::Type,
                        "native-call-u64-raw expects exact byte integers 0-255",
                        span,
                    ));
                };
                if number.fract() != 0.0 || !(0.0..=255.0).contains(&number) {
                    return Err(LanguageError::new(
                        ErrorKind::Type,
                        "native-call-u64-raw expects exact byte integers 0-255",
                        span,
                    ));
                }
                bytes.push(number as u8);
                current = tail;
            }
            _ => {
                return Err(LanguageError::new(
                    ErrorKind::Type,
                    "native-call-u64-raw expects a proper list of exact byte integers 0-255",
                    span,
                ));
            }
        }
    }
}

fn execute_u64(bytes: &[u8], span: Span) -> Result<u64, LanguageError> {
    if bytes.is_empty() {
        return Err(LanguageError::new(
            ErrorKind::InvalidForm,
            "native-call-u64-raw refuses an empty machine-code buffer",
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
        return Err(mechanism_error("mmap RW", span));
    }

    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), memory.cast::<u8>(), length);
    }

    if unsafe { mprotect(memory, length, PROT_READ | PROT_EXEC) } != 0 {
        let error = mechanism_error("mprotect RW->RX", span);
        unsafe {
            munmap(memory, length);
        }
        return Err(error);
    }

    let function: unsafe extern "C" fn() -> u64 = unsafe { std::mem::transmute(memory) };
    let result = unsafe { function() };

    if unsafe { munmap(memory, length) } != 0 {
        return Err(mechanism_error("munmap", span));
    }

    Ok(result)
}

pub(crate) fn evaluate_native_call_u64_raw(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("native-call-u64-raw", arguments, 1, span)?;
    let value = eval_expr(&arguments[0], environment)?;
    let bytes = expect_machine_bytes(&value, arguments[0].span)?;
    let result = execute_u64(&bytes, span)?;

    if result > MAX_EXACT_LISP_INTEGER {
        return Err(LanguageError::new(
            ErrorKind::InvalidForm,
            "native-call-u64-raw result exceeds my-lisp's exact integer range",
            span,
        ));
    }

    Ok(Value::Number(result as f64, Exactness::Exact))
}
