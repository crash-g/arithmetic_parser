use arithmetic_parser as parser;
use std::collections::HashMap;

fn main() {
    let stdin = std::io::stdin();

    loop {
        println!("Enter expression. CTRL-C to quit.");
        let mut line = String::new();
        stdin.read_line(&mut line).unwrap();

        let expression = match parser::ArithmeticExpression::parse(&line) {
            Ok(e) => e,
            Err(e) => {
                println!("Error: {}", e);
                continue;
            }
        };

        println!(
            "Now enter list of space separated variable values (e.g., x 2 y 1). CTRL-C to quit."
        );
        line.clear();
        stdin.read_line(&mut line).unwrap();

        let mut variables: HashMap<&str, f64> = HashMap::new();
        let tokens: Vec<&str> = line.trim().split(' ').filter(|t| t != &"").collect();
        let tokens_len = tokens.len();
        assert!(tokens_len % 2 == 0);
        let mut i = 0;
        while i < tokens_len {
            variables.insert(tokens[i], tokens[i + 1].parse().unwrap());
            i += 2;
        }

        match expression.evaluate(&variables) {
            Ok(r) => println!("Result is: {}", r),
            Err(e) => println!("Error: {}", e),
        }
    }
}
