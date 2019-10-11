#[macro_use]
extern crate lazy_static;

mod data_structures;

use data_structures::{pop_operand, pop_operator, ArithmeticExpression, Operator, ParsedToken};
use std::collections::HashMap;

pub type Result<T> = std::result::Result<T, String>;

const OPEN_PARENTHESIS: &str = "(";
const CLOSED_PARENTHESIS: &str = ")";
const COMMA: &str = ",";

pub fn parse_string(s: &str) -> Result<ArithmeticExpression> {
    // TODO improve parsing of tokens
    let mut with_spaces = s.to_string();
    with_spaces = str::replace(
        &with_spaces,
        OPEN_PARENTHESIS,
        &format!(" {} ", OPEN_PARENTHESIS),
    );
    with_spaces = str::replace(
        &with_spaces,
        CLOSED_PARENTHESIS,
        &format!(" {} ", CLOSED_PARENTHESIS),
    );
    with_spaces = str::replace(&with_spaces, COMMA, &format!(" {} ", COMMA));
    for operator in Operator::get_all() {
        let operator = operator.as_str();
        with_spaces = str::replace(&with_spaces, operator, &format!(" {} ", operator));
    }

    let tokens: Vec<_> = with_spaces.split(' ').filter(|t| t != &"").collect();
    parse_tokens(&tokens)
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
    fn test_execute() {
        let tokens = ["3"];
        assert_eq!(
            3_f64,
            parse_tokens(&tokens)
                .unwrap()
                .execute(&HashMap::new())
                .unwrap()
        );

        let tokens = ["x"];
        let variables = [("x", 4_f64)].iter().cloned().collect();
        assert_eq!(
            4_f64,
            parse_tokens(&tokens).unwrap().execute(&variables).unwrap()
        );

        let tokens = ["x", "+", "3"];
        let variables = [("x", 4_f64)].iter().cloned().collect();
        assert_eq!(
            7_f64,
            parse_tokens(&tokens).unwrap().execute(&variables).unwrap()
        );

        let tokens = [
            "(", "x", "+", "3", ")", "*", "4", "+", "(", "4", "+", "y", ")",
        ];
        let variables = [("x", 4_f64), ("y", 1_f64)].iter().cloned().collect();
        assert_eq!(
            33_f64,
            parse_tokens(&tokens).unwrap().execute(&variables).unwrap()
        );

        let s = "3 + 4 * (2 + yy / (3-xz) * ((5)))";
        let variables = [("xz", 4_f64), ("yy", 1_f64)].iter().cloned().collect();
        let result = parse_string(s).unwrap().execute(&variables).unwrap();
        assert_eq!(-9_f64, result);

        let s = "-x";
        let variables = [("x", 4_f64)].iter().cloned().collect();
        assert_eq!(
            -4_f64,
            parse_string(s).unwrap().execute(&variables).unwrap()
        );

        let s = "3 * sqrt 4 - 2 * x + +(2,3)";
        let variables = [("x", 3_f64)].iter().cloned().collect();
        assert_eq!(5_f64, parse_string(s).unwrap().execute(&variables).unwrap());

        let s = "* (3 + x*2, sqrt y - 1)";
        let variables = [("x", 3_f64), ("y", 9_f64)].iter().cloned().collect();
        assert_eq!(
            18_f64,
            parse_string(s).unwrap().execute(&variables).unwrap()
        );

        let s = "3 + sqrt 4 * 2";
        assert_eq!(
            7_f64,
            parse_string(s).unwrap().execute(&HashMap::new()).unwrap()
        );
    }
}
