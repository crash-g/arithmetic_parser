//! A simple parser and evaluator for arithmetic expressions.
//!
//! # Usage example
//! ```
//! use arithmetic_parser as parser;
//! let expression = parser::ArithmeticExpression::parse("(x+y)/(x-y)").unwrap();
//! let variables = [("x", 5_f64), ("y", 1_f64)].iter().cloned().collect();
//! assert_eq!(1.5, expression.evaluate(&variables).unwrap());
//! ```

#![deny(rust_2018_idioms)]

#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;

mod data_structures;

pub use data_structures::ArithmeticExpression;

pub type Result<T> = std::result::Result<T, String>;

use data_structures::{pop_operand, pop_operator, Operator, ParsedToken};

const OPEN_PARENTHESIS: &str = "(";
const CLOSED_PARENTHESIS: &str = ")";
const COMMA: &str = ",";

const OPEN_PARENTHESIS_CHAR: char = '(';
const CLOSED_PARENTHESIS_CHAR: char = ')';
const COMMA_CHAR: char = ',';

impl ArithmeticExpression {
    /// Parse an arithmetic expression and return a tree representation.
    ///
    /// An arithmetic expression is made of *numbers*, *variables*,
    /// *operators* and *special characters* (parenthesis and comma).
    /// Operators can be *functional*
    /// if their arguments follow them (e.g., `sqrt`), or *infix* if they are
    /// placed between their arguments (e.g., `+`). Infix operators support
    /// precedence.
    ///
    /// ## Caveats:
    /// - Variable names must satisfy the following regex: `[a-zA-Z0-9]+`.
    /// - Spaces can be omitted around parenthesis, commas, symbolic
    ///   operators (`+`, `-`, `*`, `/`).
    /// - Arguments for function operators must be surrounded by parenthesis
    ///   and separated by commas. Parenthesis can be omitted if there is only
    ///   one argument.
    ///
    /// ## Examples:
    /// ```
    /// use arithmetic_parser as parser;
    /// parser::ArithmeticExpression::parse("3 + 2");
    /// parser::ArithmeticExpression::parse("2 + x*4");
    /// parser::ArithmeticExpression::parse("(1.34+sqrt x)*(2.2/(+(0.1,0.2,0.3)))");
    /// ```
    pub fn parse(s: &str) -> Result<ArithmeticExpression> {
        let tokens: Vec<_> = s
            .split_whitespace()
            .flat_map(|x| {
                let mut tokens = Vec::new();
                let mut pos = 0;
                let len = x.len();
                while let Some(i) = find_restricted_character(x, pos, len) {
                    if pos != i {
                        tokens.push(&x[pos..i]);
                    }
                    tokens.push(&x[i..i + 1]);
                    pos = i + 1;
                }
                if pos != len {
                    tokens.push(&x[pos..len]);
                }
                tokens
            })
            .collect();
        parse_tokens(&tokens)
    }

    /// Evaluate an arithmetic expression to produce a value.
    ///
    /// A HashMap with the values of all the variables must be provided. A
    /// variable which is missing from the expression is ignored, but if
    /// a variable is not present in the HashMap an error is returned.
    ///
    /// Example:
    /// ```
    /// use arithmetic_parser as parser;
    /// let expression = parser::ArithmeticExpression::parse("(x+y)/(x-y)").unwrap();
    /// let variables = [("x", 5_f64), ("y", 1_f64)].iter().cloned().collect();
    /// assert_eq!(1.5, expression.evaluate(&variables).unwrap());
    /// ```
    pub fn evaluate(&self, variables: &HashMap<&str, f64>) -> Result<f64> {
        match self {
            ArithmeticExpression::NumberLeaf(n) => Ok(*n),
            ArithmeticExpression::VariableLeaf(x) => match variables.get(x.as_str()) {
                Some(n) => Ok(*n),
                None => Err(format!("Value for variable {} must be provided", x)),
            },
            ArithmeticExpression::Node { node, operands } => {
                let mut resolved_operands = Vec::with_capacity(operands.len());
                for operand in operands {
                    resolved_operands.push(operand.evaluate(variables)?);
                }
                Ok(node.apply(resolved_operands))
            }
        }
    }
}

fn parse_tokens(tokens: &[&str]) -> Result<ArithmeticExpression> {
    let parsed_tokens = preliminary_parse(tokens)?;

    let mut token_stack = Vec::new();
    for parsed_token in parsed_tokens {
        match parsed_token {
            operand @ ParsedToken::Operand(_) => token_stack.push(operand),
            ParsedToken::Operator(operator) => {
                resolve_operators(&mut token_stack, operator.get_priority())?;
                token_stack.push(ParsedToken::Operator(operator));
            }
        }
    }
    resolve_operators(&mut token_stack, 0)?;
    if token_stack.len() == 1 {
        Ok(pop_operand(&mut token_stack).unwrap())
    } else {
        // TODO deal with errors (adjacent operators, adjacent operands, starting or finishing operator)
        panic!()
    }
}

fn resolve_operators(token_stack: &mut Vec<ParsedToken>, minimum_priority: u8) -> Result<()> {
    resolve_function_operators(token_stack)?;
    resolve_infix_operators(token_stack, minimum_priority)?;
    Ok(())
}

fn resolve_function_operators(token_stack: &mut Vec<ParsedToken>) -> Result<()> {
    let pos = find_last_function_operator_pos(token_stack);
    if pos.is_some() {
        let pos = pos.unwrap();
        let num_operands = token_stack.len() - pos - 1;
        if num_operands > 0 {
            if token_stack[pos].is_nary(num_operands) {
                let mut operands = Vec::with_capacity(num_operands);
                operands.reverse();
                for _ in 0..num_operands {
                    operands.push(pop_operand(token_stack).unwrap());
                }
                let node = ArithmeticExpression::Node {
                    node: pop_operator(token_stack).unwrap(),
                    operands,
                };
                token_stack.push(ParsedToken::Operand(node));
            } else {
                return Err(format!(
                    "{:?} is not a function operator which accepts {} arguments",
                    token_stack[pos], num_operands
                ));
            }
        }
    }
    Ok(())
}

fn find_last_function_operator_pos(token_stack: &[ParsedToken]) -> Option<usize> {
    token_stack
        .iter()
        .rposition(|token| token.is_operator())
        .and_then(|last_operator_pos| {
            if last_operator_pos == 0 || token_stack[last_operator_pos - 1].is_operator() {
                Some(last_operator_pos)
            } else {
                None
            }
        })
}

fn resolve_infix_operators(token_stack: &mut Vec<ParsedToken>, minimum_priority: u8) -> Result<()> {
    let mut stack_length = token_stack.len();
    while stack_length >= 3
        && token_stack[stack_length - 3].is_operand()
        && token_stack[stack_length - 2].is_operator()
        && token_stack[stack_length - 1].is_operand()
    {
        match &token_stack[stack_length - 2] {
            ParsedToken::Operator(operator) => {
                if operator.get_priority() < minimum_priority {
                    break;
                }
            }
            _ => panic!(),
        }
        let right_operand = pop_operand(token_stack).unwrap();
        let operator = pop_operator(token_stack).unwrap();
        let left_operand = pop_operand(token_stack).unwrap();
        if !operator.is_nary(2) {
            return Err(format!("{:?} is not an infix operator", operator));
        }
        let node = ArithmeticExpression::Node {
            node: operator,
            operands: vec![left_operand, right_operand],
        };
        token_stack.push(ParsedToken::Operand(node));
        stack_length = token_stack.len();
    }
    Ok(())
}

fn preliminary_parse(tokens: &[&str]) -> Result<Vec<ParsedToken>> {
    let tokens_len = tokens.len();
    let mut current_pos = 0;
    let mut result = Vec::new();

    while current_pos < tokens_len {
        if tokens[current_pos] == OPEN_PARENTHESIS {
            let closing_parenthesis_pos = find_closing_parenthesis_pos(tokens, current_pos)?;
            let operands = tokens[current_pos + 1..closing_parenthesis_pos]
                .split(|token| token == &COMMA)
                .map(|subtokens| parse_tokens(subtokens));
            for operand in operands {
                if operand.is_ok() {
                    result.push(ParsedToken::Operand(operand.unwrap()))
                } else {
                    return Err(operand.unwrap_err());
                }
            }
            current_pos = closing_parenthesis_pos + 1;
        } else {
            let parsed_token = try_parse(tokens[current_pos])?;
            result.push(parsed_token);
            current_pos += 1;
        }
    }

    Ok(result)
}

fn try_parse(token: &str) -> Result<ParsedToken> {
    let operator = try_parse_operator(token);
    if operator.is_some() {
        return Ok(ParsedToken::Operator(operator.unwrap()));
    }

    let number = try_parse_number(token);
    if number.is_some() {
        return Ok(ParsedToken::Operand(ArithmeticExpression::NumberLeaf(
            number.unwrap(),
        )));
    }

    let variable = try_parse_variable(token);
    if variable.is_some() {
        return Ok(ParsedToken::Operand(ArithmeticExpression::VariableLeaf(
            variable.unwrap(),
        )));
    }

    Err(format!("Cannot parse token {}", token))
}

fn try_parse_number(token: &str) -> Option<f64> {
    let number = token.parse::<f64>();
    if number.is_ok() {
        Some(number.unwrap())
    } else {
        None
    }
}

fn try_parse_variable(token: &str) -> Option<String> {
    Some(token.to_string()) // TODO restrict variable names!
}

fn try_parse_operator(token: &str) -> Option<Operator> {
    for operator in Operator::get_all() {
        if operator.as_str() == token {
            return Some(operator.clone());
        }
    }
    None
}

fn find_restricted_character(s: &str, left: usize, right: usize) -> Option<usize> {
    s[left..right]
        .find(|c| {
            c == OPEN_PARENTHESIS_CHAR
                || c == CLOSED_PARENTHESIS_CHAR
                || c == COMMA_CHAR
                || Operator::get_all_infix().contains(&c)
        })
        .map(|i| i + left)
}

fn find_closing_parenthesis_pos(tokens: &[&str], pos: usize) -> Result<usize> {
    let tokens_len = tokens.len();
    let mut current_pos = pos;
    let mut count = 1;

    while count > 0 && current_pos < tokens_len - 1 {
        current_pos += 1;
        if tokens[current_pos] == OPEN_PARENTHESIS {
            count += 1;
        } else if tokens[current_pos] == CLOSED_PARENTHESIS {
            count -= 1;
        }
    }

    if count == 0 {
        Ok(current_pos)
    } else {
        Err(format!("Parenthesis at pos {} is not balanced!", pos))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_closing_parenthesis() {
        let tokens = ["a", "(", "(", "f", ")", "(", "b", "fer", ")", ")"];
        assert_eq!(find_closing_parenthesis_pos(&tokens, 1).unwrap(), 9);

        let tokens = ["a", "(", "(", "f", ")", "(", "b", "fer", ")"];
        assert!(find_closing_parenthesis_pos(&tokens, 1).is_err());
    }

    #[test]
    fn test_evaluate() {
        let tokens = ["3"];
        assert_eq!(
            3_f64,
            parse_tokens(&tokens)
                .unwrap()
                .evaluate(&HashMap::new())
                .unwrap()
        );

        let tokens = ["x"];
        let variables = [("x", 4_f64)].iter().cloned().collect();
        assert_eq!(
            4_f64,
            parse_tokens(&tokens).unwrap().evaluate(&variables).unwrap()
        );

        let tokens = ["x", "+", "3"];
        let variables = [("x", 4_f64)].iter().cloned().collect();
        assert_eq!(
            7_f64,
            parse_tokens(&tokens).unwrap().evaluate(&variables).unwrap()
        );

        let tokens = [
            "(", "x", "+", "3", ")", "*", "4", "+", "(", "4", "+", "y", ")",
        ];
        let variables = [("x", 4_f64), ("y", 1_f64)].iter().cloned().collect();
        assert_eq!(
            33_f64,
            parse_tokens(&tokens).unwrap().evaluate(&variables).unwrap()
        );

        let s = "3 + 4 * (2 + yy / (3-xz) * ((5)))";
        let variables = [("xz", 4_f64), ("yy", 1_f64)].iter().cloned().collect();
        let result = ArithmeticExpression::parse(s)
            .unwrap()
            .evaluate(&variables)
            .unwrap();
        assert_eq!(-9_f64, result);

        let s = "-x";
        let variables = [("x", 4_f64)].iter().cloned().collect();
        assert_eq!(
            -4_f64,
            ArithmeticExpression::parse(s)
                .unwrap()
                .evaluate(&variables)
                .unwrap()
        );

        let s = "3 * sqrt 4 - 2 * x + +(2,3)";
        let variables = [("x", 3_f64)].iter().cloned().collect();
        assert_eq!(
            5_f64,
            ArithmeticExpression::parse(s)
                .unwrap()
                .evaluate(&variables)
                .unwrap()
        );

        let s = "* (3 + x*2, sqrt y - 1)";
        let variables = [("x", 3_f64), ("y", 9_f64)].iter().cloned().collect();
        assert_eq!(
            18_f64,
            ArithmeticExpression::parse(s)
                .unwrap()
                .evaluate(&variables)
                .unwrap()
        );

        let s = "3 + sqrt 4 * 2";
        assert_eq!(
            7_f64,
            ArithmeticExpression::parse(s)
                .unwrap()
                .evaluate(&HashMap::new())
                .unwrap()
        );
    }
}
