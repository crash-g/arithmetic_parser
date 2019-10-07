use super::HashMap;
use super::Result;

#[derive(Debug)]
pub enum Operator {
    Plus,
    Minus,
    Star,
    Slash,
}

impl Operator {
    pub fn get_priority(&self) -> u8 {
        match self {
            Operator::Plus => 0,
            Operator::Minus => 0,
            Operator::Star => 1,
            Operator::Slash => 1,
        }
    }

    pub fn execute_unary(&self, x: f64) -> f64 {
        match self {
            Operator::Plus => x,
            Operator::Minus => -x,
            Operator::Star => panic!("Not supported!"),
            Operator::Slash => panic!("Not supported!"),
        }
    }

    pub fn execute_binary(&self, x: f64, y: f64) -> f64 {
        match self {
            Operator::Plus => x + y,
            Operator::Minus => x - y,
            Operator::Star => x * y,
            Operator::Slash => x / y,
        }
    }

    pub fn is_unary(&self) -> bool {
        match self {
            Operator::Plus => true,
            Operator::Minus => true,
            Operator::Star => false,
            Operator::Slash => false,
        }
    }

    pub fn is_binary(&self) -> bool {
        match self {
            Operator::Plus => true,
            Operator::Minus => true,
            Operator::Star => true,
            Operator::Slash => true,
        }
    }
}

#[derive(Debug)]
pub enum Tree {
    NumberLeaf(f64),
    VariableLeaf(String),
    Node {
        node: Operator,
        left_operand: Option<Box<Tree>>,
        right_operand: Box<Tree>,
    },
}

impl Tree {
    pub fn execute(&self, variables: &HashMap<&str, f64>) -> Result<f64> {
        match self {
            Tree::NumberLeaf(n) => Ok(*n),
            Tree::VariableLeaf(x) => match variables.get(x.as_str()) {
                Some(n) => Ok(*n),
                None => Err(format!("Value for variable {} must be provided", x)),
            },
            Tree::Node {
                node,
                left_operand,
                right_operand,
            } => {
                let left = match left_operand {
                    Some(operand) => Some(operand.execute(variables)?),
                    None => None,
                };
                let right = right_operand.execute(variables)?;
                match left {
                    None => Ok(node.execute_unary(right)),
                    Some(left) => Ok(node.execute_binary(left, right)),
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum ParsedToken {
    Operand(Tree),
    Operator(Operator),
}

impl ParsedToken {
    pub fn is_operand(&self) -> bool {
        match self {
            ParsedToken::Operand(_) => true,
            _ => false,
        }
    }

    pub fn is_operator(&self) -> bool {
        match self {
            ParsedToken::Operator(_) => true,
            _ => false,
        }
    }
}

pub fn pop_operator(token_stack: &mut Vec<ParsedToken>) -> Option<Operator> {
    let can_pop = match token_stack.peek() {
        Some(ParsedToken::Operator(_)) => true,
        _ => false,
    };
    if can_pop {
        match token_stack.pop() {
            Some(ParsedToken::Operator(operator)) => Some(operator),
            _ => panic!("How could this happen!"),
        }
    } else {
        None
    }
}

pub fn pop_operand(token_stack: &mut Vec<ParsedToken>) -> Option<Tree> {
    let can_pop = match token_stack.peek() {
        Some(ParsedToken::Operand(_)) => true,
        _ => false,
    };
    if can_pop {
        match token_stack.pop() {
            Some(ParsedToken::Operand(operand)) => Some(operand),
            _ => panic!("How could this happen!"),
        }
    } else {
        None
    }
}

trait Peekable<T> {
    fn peek(&self) -> Option<&T>;
}

impl<T> Peekable<T> for Vec<T> {
    fn peek(&self) -> Option<&T> {
        if !self.is_empty() {
            Some(&self[self.len() - 1])
        } else {
            None
        }
    }
}
