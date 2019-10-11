#[derive(Debug)]
pub enum ArithmeticExpression {
    NumberLeaf(f64),
    VariableLeaf(String),
    Node {
        node: Operator,
        operands: Vec<ArithmeticExpression>,
    },
}

#[derive(Debug)]
pub enum ParsedToken {
    Operand(ArithmeticExpression),
    Operator(Operator),
}

#[derive(Debug, Clone)]
pub enum Operator {
    Plus,
    Minus,
    Star,
    Slash,
    Sqrt,
}

lazy_static! {
    static ref OPERATORS: Vec<Operator> = vec![
        Operator::Plus,
        Operator::Minus,
        Operator::Star,
        Operator::Slash,
        Operator::Sqrt,
    ];
}

impl Operator {
    pub fn get_all() -> &'static [Operator] {
        &OPERATORS
    }

    pub fn get_priority(&self) -> u8 {
        match self {
            Operator::Plus => 0,
            Operator::Minus => 0,
            Operator::Star => 1,
            Operator::Slash => 1,
            Operator::Sqrt => 1,
        }
    }

    pub fn execute(&self, args: Vec<f64>) -> f64 {
        match self {
            Operator::Plus => args.iter().sum(),
            Operator::Minus => match args.len() {
                1 => -args[0],
                2 => args[0] - args[1],
                _ => panic!("Not supported!"),
            },
            Operator::Star => match args.len() {
                2 => args[0] * args[1],
                _ => panic!("Not supported!"),
            },
            Operator::Slash => match args.len() {
                2 => args[0] / args[1],
                _ => panic!("Not supported!"),
            },
            Operator::Sqrt => match args.len() {
                1 => args[0].sqrt(),
                _ => panic!("Not supported!"),
            },
        }
    }

    pub fn is_nary(&self, n: usize) -> bool {
        if n == 0 {
            false
        } else {
            match self {
                Operator::Plus => true,
                Operator::Minus => n == 1 || n == 2,
                Operator::Star => n == 2,
                Operator::Slash => n == 2,
                Operator::Sqrt => n == 1,
            }
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Operator::Plus => "+",
            Operator::Minus => "-",
            Operator::Star => "*",
            Operator::Slash => "/",
            Operator::Sqrt => "sqrt",
        }
    }
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

    pub fn is_nary(&self, n: usize) -> bool {
        match self {
            ParsedToken::Operator(o) => o.is_nary(n),
            _ => panic!("Only operators support this method!"),
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

pub fn pop_operand(token_stack: &mut Vec<ParsedToken>) -> Option<ArithmeticExpression> {
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

impl<T> Peekable<T> for [T] {
    fn peek(&self) -> Option<&T> {
        if !self.is_empty() {
            Some(&self[self.len() - 1])
        } else {
            None
        }
    }
}
