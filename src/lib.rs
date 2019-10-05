use std::collections::HashMap;

pub type Result<T> = std::result::Result<T, String>;

const OPEN_PARENTHESIS: &'static str = "(";
const CLOSED_PARENTHESIS: &'static str = ")";

#[derive(Debug)]
enum Value {
    Number(f64),
    Placeholder(String),
}

#[derive(Debug)]
enum Operation {
    Sum, Difference, Multiplication, Division,
}

impl Operation {
    fn execute_binary(&self, x: f64, y: f64) -> f64 {
        match self {
            Operation::Sum => x + y,
            Operation::Difference => x - y,
            Operation::Multiplication => x * y,
            Operation::Division => x / y,
        }
    }
}

#[derive(Debug)]
enum Tree {
    Leaf(Value),
    Node {
        node: Operation,
        left_operand: Option<Box<Tree>>,
        right_operand: Option<Box<Tree>>,
    },
}

#[derive(Debug)]
enum ParsedToken {
    Operand(Tree),
    Operation(Operation),
}

fn parse_string(s: &str) -> Result<Tree> {
    // TODO deal with adjacent spaces and missing spaces
    let tokens: Vec<_> = s.split(' ').collect();
    parse_tokens(&tokens)
}

fn parse_tokens(tokens: &[&str]) -> Result<Tree> {
    let parsed_tokens = intermediate_parse(tokens)?;

    let length = parsed_tokens.len();
    if length == 0 {
        return Err(format!("So empty around here! {:?}", parsed_tokens));
    }

    let mut parsed_tokens = parsed_tokens.into_iter();

    let mut result = match parsed_tokens.next().unwrap() {
        ParsedToken::Operand(t) => t,
        x => return Err(format!("Expected an operation here! {:?}", x))
    };

    loop {
        let operation = match parsed_tokens.next() {
            None => break,
            Some(ParsedToken::Operation(o)) => o,
            Some(x) => return Err(format!("Expected an operation here! {:?}", x))
        };

        let operand = match parsed_tokens.next() {
            None => return Err(format!("Missing operand: {}", "?")),
            Some(ParsedToken::Operand(t)) => t,
            Some(x) => return Err(format!("Expected an operation here! {:?}", x))
        };

        result = Tree::Node{
            node: operation,
            left_operand: Some(Box::new(result)),
            right_operand: Some(Box::new(operand))
        };
    }

    Ok(result)

    // TODO make implementation with priorities
    // TODO deal with non binary operators (unary?)
    // TODO deal with errors
}

fn intermediate_parse(tokens: &[&str]) -> Result<Vec<ParsedToken>> {
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
        let operand = parse_tokens(&tokens[pos+1..closing_parenthesis_pos])?;
        return Ok((ParsedToken::Operand(operand), closing_parenthesis_pos + 1));
    }

    let operation = try_parse_operation(tokens[pos]);
    if operation.is_some() {
        return Ok((ParsedToken::Operation(operation.unwrap()), pos + 1));
    }

    let value = try_parse_value(tokens[pos]);
    if value.is_some() {
        return Ok((ParsedToken::Operand(Tree::Leaf(value.unwrap())), pos + 1));
    }

    Err(format!("Cannot parse token {}", tokens[pos]))
}

fn try_parse_value(token: &str) -> Option<Value> {
    let number = token.parse::<f64>();
    if number.is_ok() {
        Some(Value::Number(number.unwrap()))
    } else {
        Some(Value::Placeholder(token.to_string())) // TODO restrict variable names!
    }
}

fn try_parse_operation(token: &str) -> Option<Operation> {
    match token {
        "+" => Some(Operation::Sum),
        "-" => Some(Operation::Difference),
        "*" => Some(Operation::Multiplication),
        "/" => Some(Operation::Division),
        _ => None
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

impl Tree {
    fn execute(&self, variables: &HashMap<&str, f64>) -> Result<f64> {
        match self {
            Tree::Leaf(Value::Number(n)) => Ok(*n),
            Tree::Leaf(Value::Placeholder(x)) => {
                match variables.get(x.as_str()) {
                    Some(n) => Ok(*n),
                    None => Err(format!("Value for variable {} must be provided", x)),
                }
            },
            Tree::Node{node, left_operand, right_operand} => {
                let left = match left_operand {
                    Some(operand) => operand.execute(variables)?,
                    None => unimplemented!(),
                };
                let right = match right_operand {
                    Some(operand) => operand.execute(variables)?,
                    None => unimplemented!(),
                };
                Ok(node.execute_binary(left, right))
            }
        }
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
        assert_eq!(3_f64, parse_tokens(&tokens).unwrap().execute(&HashMap::new()).unwrap());

        let tokens = ["x"];
        let variables = [("x", 4_f64)].iter().cloned().collect();
        assert_eq!(4_f64, parse_tokens(&tokens).unwrap().execute(&variables).unwrap());

        let tokens = ["x", "+", "3"];
        let variables = [("x", 4_f64)].iter().cloned().collect();
        assert_eq!(7_f64, parse_tokens(&tokens).unwrap().execute(&variables).unwrap());

        let tokens = ["(", "x", "+", "3", ")", "*", "4", "+", "(", "4", "+", "y", ")"];
        let variables = [("x", 4_f64), ("y", 1_f64)].iter().cloned().collect();
        assert_eq!(33_f64, parse_tokens(&tokens).unwrap().execute(&variables).unwrap());

        let s = "3 + 4 * ( 2 + yy / ( 3 - xz ) * ( ( 2 ) ) )";
        let variables = [("xz", 4_f64), ("yy", 1_f64)].iter().cloned().collect();
        assert_eq!(-42_f64, parse_string(s).unwrap().execute(&variables).unwrap());
    }
}
