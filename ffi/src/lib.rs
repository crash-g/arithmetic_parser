#![deny(rust_2018_idioms)]

use libc::{c_char, c_double};
use std::ffi::CStr;

use std::collections::HashMap;

use arithmetic_parser::ArithmeticExpression;

pub struct Wrapper {
    expression: ArithmeticExpression,
    variables: HashMap<String, f64>,
}

impl Wrapper {
    fn parse(s: &str) -> Wrapper {
        Wrapper {
            expression: ArithmeticExpression::parse(s).unwrap(),
            variables: HashMap::new(),
        }
    }

    fn add_variable(&mut self, variable: String, value: f64) {
        self.variables.insert(variable, value);
    }

    fn evaluate(&self) -> f64 {
        let variables_ref = self
            .variables
            .iter()
            .map(|(x, y)| (x.as_ref(), *y))
            .collect();
        self.expression.evaluate(&variables_ref).unwrap()
    }
}

#[no_mangle]
pub extern "C" fn arithmetic_parser_parse(s: *const c_char) -> *mut Wrapper {
    let c_str = unsafe {
        assert!(!s.is_null());
        CStr::from_ptr(s)
    };

    let r_str = c_str.to_str().unwrap();

    Box::into_raw(Box::new(Wrapper::parse(r_str)))
}

#[no_mangle]
pub extern "C" fn arithmetic_parser_add_variable(
    ptr: *mut Wrapper,
    variable: *const c_char,
    value: c_double,
) {
    let wrapper = unsafe {
        assert!(!ptr.is_null());
        &mut *ptr
    };
    let variable_str = unsafe {
        assert!(!variable.is_null());
        CStr::from_ptr(variable)
    };

    wrapper.add_variable(variable_str.to_str().unwrap().to_string(), value);
}

#[no_mangle]
pub extern "C" fn arithmetic_parser_evaluate(ptr: *mut Wrapper) -> c_double {
    let wrapper = unsafe {
        assert!(!ptr.is_null());
        &mut *ptr
    };

    wrapper.evaluate()
}

#[no_mangle]
pub extern "C" fn arithmetic_parser_free(ptr: *mut Wrapper) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        Box::from_raw(ptr);
    }
}
