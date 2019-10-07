mod data_structures;

use data_structures::{pop_operand, pop_operator, Operator, ParsedToken, Tree};
use std::collections::HashMap;

pub type Result<T> = std::result::Result<T, String>;

const OPEN_PARENTHESIS: &'static str = "(";
const CLOSED_PARENTHESIS: &'static str = ")";

pub fn parse_string(s: &str) -> Result<Tree> {
    // TODO improve parsing of tokens
    let s = str::replace(s, OPEN_PARENTHESIS, " ( ");
    let s = str::replace(&s, CLOSED_PARENTHESIS, " ) ");
    let s = str::replace(&s, "+", " + ");
    let s = str::replace(&s, "-", " - ");
    let s = str::replace(&s, "*", " * ");
    let s = str::replace(&s, "/", " / ");

    let tokens: Vec<_> = s.split(' ').filter(|t| t != &"").collect();
    parse_tokens(&tokens)
}

fn parse_tokens(tokens: &[&str]) -> Result<Tree> {
    let parsed_tokens = preliminary_parse(tokens)?;

    let mut token_stack = Vec::new();
    for parsed_token in parsed_tokens {
        match parsed_token {
            operand @ ParsedToken::Operand(_) => {
                token_stack.push(operand);
                resolve_unary_operators(&mut token_stack)?;
            }
            ParsedToken::Operator(operator) => {
                resolve_binary_operators(&mut token_stack, operator.get_priority())?;
                token_stack.push(ParsedToken::Operator(operator));
            }
        }
    }
    resolve_binary_operators(&mut token_stack, 0)?;
    if token_stack.len() == 1 {
        Ok(pop_operand(&mut token_stack).unwrap())
    } else {
        // TODO deal with errors (adjacent operators, adjacent operands, starting or finishing operator)
        panic!()
    }
}

fn resolve_unary_operators(token_stack: &mut Vec<ParsedToken>) -> Result<()> {
    let mut stack_length = token_stack.len();
    while stack_length >= 2
        && token_stack[stack_length - 1].is_operand()
        && token_stack[stack_length - 2].is_operator()
        && (stack_length == 2 || token_stack[stack_length - 3].is_operator())
    {
        let operand = pop_operand(token_stack).unwrap();
        let operator = pop_operator(token_stack).unwrap();
        if !operator.is_unary() {
            return Err(format!("Invalid non-unary operator found: {:?}", operator));
        }
        let node = Tree::Node {
            node: operator,
            left_operand: None,
            right_operand: Box::new(operand),
        };
        token_stack.push(ParsedToken::Operand(node));
        stack_length = token_stack.len();
    }
    Ok(())
}

fn resolve_binary_operators(
    token_stack: &mut Vec<ParsedToken>,
    minimum_priority: u8,
) -> Result<()> {
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
        if !operator.is_binary() {
            return Err(format!("Invalid non-unary operator found: {:?}", operator));
        }
        let node = Tree::Node {
            node: operator,
            left_operand: Some(Box::new(left_operand)),
            right_operand: Box::new(right_operand),
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
        let (parsed_token, end_pos) = try_parse_next(tokens, current_pos)?;
        result.push(parsed_token);
        current_pos = end_pos;
    }

    Ok(result)
}

fn try_parse_next(tokens: &[&str], pos: usize) -> Result<(ParsedToken, usize)> {
    if tokens[pos] == OPEN_PARENTHESIS {
        let closing_parenthesis_pos = find_closing_parenthesis_pos(tokens, pos)?;
        let operand = parse_tokens(&tokens[pos + 1..closing_parenthesis_pos])?;
        return Ok((ParsedToken::Operand(operand), closing_parenthesis_pos + 1));
    }

    let operator = try_parse_operator(tokens[pos]);
    if operator.is_some() {
        return Ok((ParsedToken::Operator(operator.unwrap()), pos + 1));
    }

    let number = try_parse_number(tokens[pos]);
    if number.is_some() {
        return Ok((
            ParsedToken::Operand(Tree::NumberLeaf(number.unwrap())),
            pos + 1,
        ));
    }

    let variable = try_parse_variable(tokens[pos]);
    if variable.is_some() {
        return Ok((
            ParsedToken::Operand(Tree::VariableLeaf(variable.unwrap())),
            pos + 1,
        ));
    }

    Err(format!("Cannot parse token {}", tokens[pos]))
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
    match token {
        "+" => Some(Operator::Plus),
        "-" => Some(Operator::Minus),
        "*" => Some(Operator::Star),
        "/" => Some(Operator::Slash),
        _ => None,
    }
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
    }
}
