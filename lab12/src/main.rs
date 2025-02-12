use std::fmt::Write;
use std::io;

#[derive(Debug)]
enum ProgramError {
    InvalidOperation,
    IncompleteList,
    IncompleteString,
    IncompleteQuotation,
    StackEmpty,
    ExpectedBool,
    ExpectedList,
    ExpectedNumber,
    ExpectedString,
}

#[derive(Debug, Clone)]
enum Datatype {
    Int(i128),
    Float(f64),
    Boolean(bool),
    List(Vec<Datatype>),
    String(String),
    Code(String),
}

fn main() {
    println!("Welcome to the bprog interpreter!\nTesting or interpreting? (t/i)");
    let input = read_line();
    
    if input.contains('t') {
        tests();
    } else {
        loop { 
            let result = interpreter(&read_line());
            match result {
                Ok(value) => println!("{}", format_stack_item(value)),
                Err(e) => println!("Error: {:?}", e),
            }
        }
    }
    
}

fn interpreter(line : &String) -> Result<Datatype, ProgramError> {
    let mut tokens: Vec<_> = line.trim().split(' ').rev().collect();
    
    let mut stack : Vec<Datatype> = Vec::new();

    while !tokens.is_empty() {
        let token = tokens.pop().unwrap();

        match datatype(token, &mut tokens) {
            Some(value) => stack.push(value),
            None => {
                 //Checking the input for operators and function-calls, returns Some, if there are anything to be pushed back in the stack
                let result : Option<Result<Datatype, ProgramError>> = match token {
                    "+" => Some(add(stack.pop().unwrap(), stack.pop().unwrap())),
                    "-" => Some(subtract(stack.pop().unwrap(), stack.pop().unwrap())),
                    "*" => Some(multiply(stack.pop().unwrap(), stack.pop().unwrap())),
                    "/" => Some(divide(stack.pop().unwrap(), stack.pop().unwrap())),
                    "&&" => Some(and(stack.pop().unwrap(), stack.pop().unwrap())),
                    "||" => Some(or(stack.pop().unwrap(), stack.pop().unwrap())),
                    "not" => Some(not(stack.pop().unwrap())),
                    "<" => Some(less_than(stack.pop().unwrap(), stack.pop().unwrap())),
                    ">" => Some(larger_than(stack.pop().unwrap(), stack.pop().unwrap())),
                    "div" => Some(div(stack.pop().unwrap(), stack.pop().unwrap())),
                    "==" => Some(equal(stack.pop().unwrap(), stack.pop().unwrap())),
                    "swap" => {
                        let a = stack.pop().unwrap();
                        let b = stack.pop().unwrap();
                        stack.push(a);
                        stack.push(b);
                        None
                    },
                    "pop" => {
                        stack.pop();
                        None
                    },
                    "dup" => {
                        let a = stack.pop().unwrap();
                        stack.push(a.clone());
                        stack.push(a);
                        None
                    },
                    "length" => Some(length(stack.pop().unwrap())),
                    "words" => Some(words(stack.pop().unwrap())),
                    "parseInteger" => Some(parse_integer(stack.pop().unwrap())),
                    "parseFloat" => Some(parse_float(stack.pop().unwrap())),
                    "empty" => Some(empty(stack.pop().unwrap())),
                    "head" => Some(head(stack.pop().unwrap())),
                    "tail" => Some(tail(stack.pop().unwrap())),
                    "cons" => Some(cons(stack.pop().unwrap(), stack.pop().unwrap())),
                    "append" => Some(append(stack.pop().unwrap(), stack.pop().unwrap())),
                    "exec" => Some(Ok(stack.pop().unwrap())),
                    "map" => Some(map(stack.pop().unwrap(), &mut tokens)),
                    "if" => Some(if_(stack.pop().unwrap(), &mut tokens)),
                    "each" => Some(each(stack.pop().unwrap(), &mut tokens, &mut stack)),
                    "foldl" => Some(foldl(stack.pop().unwrap(), stack.pop().unwrap(), &mut tokens)),
                    "times" => Some(times(stack.pop().unwrap(), &mut tokens, &mut stack)),






                    _ => Some(Err(ProgramError::InvalidOperation)),
                };

                match result {
                    Some(Ok(value)) => stack.push(value),
                    Some(Err(e)) => println!("Error: {:?}", e),
                    None => (),
                }
            },
        } 
    }

    //Handling the error case if the stack is empty
    if stack.len() == 0 {
        return Err(ProgramError::StackEmpty)
    } else if stack.len() == 1 {
        let last_element = stack.pop().unwrap();

        //Evaluation of the last element in the stack if it is a code block, otherwise return the element
        match last_element {
            Datatype::Code(code) => {
                match interpreter(&code) {
                    Ok(value) => return Ok(value),
                    Err(e) => return Err(e),
                }
            },
            _ => return Ok(last_element)
        }
        
    } else {
        
        //Final evaluation of the stack
        let mut final_evaluation = String::new();
        for item in stack {
            final_evaluation.push_str(&format!("{} ",format_stack_item(item).as_str()));
        }
        //println!("Final evaluation: {}", final_evaluation);
        return match interpreter(&final_evaluation) {
            Ok(value) => Ok(value),
            Err(e) => Err(e),
        };
    }
    
}

fn foldl(init_accumulator : Datatype, list : Datatype, tokens : &mut Vec<&str>) -> Result<Datatype, ProgramError> {
    
    let token = tokens.pop().unwrap();
    let operation = match datatype(token, tokens) {
        Some(dt) => format_stack_item(dt),
        None => token.to_string()
    };

    let iterable_list = match list {
        Datatype::List(list) => list,
        _ => return Err(ProgramError::ExpectedList),
    };

    let mut final_accumulation = match init_accumulator{
        Datatype::Int(value) => Datatype::Int(value),
        _ => return Err(ProgramError::ExpectedNumber),
    };

    for item in iterable_list {

        //Does operation to the accumulated value with the list item as argument
        let code_to_execute = format!("{} {} {}", format_stack_item(final_accumulation), format_stack_item(item), operation);
        
        match interpreter(&code_to_execute){
            Ok(new_acc_value) => final_accumulation = new_acc_value,
            Err(e) => return Err(e),
        }
    }

    Ok(final_accumulation)
}

fn each(token : Datatype, tokens : &mut Vec<&str>, stack : &mut Vec<Datatype>) -> Result<Datatype, ProgramError> {
    
    let old_list = match token {
        Datatype::List(list) => list,
        _ => return Err(ProgramError::InvalidOperation),
    };

    //Call on map and get a list with elements
    let new_list = match map(Datatype::List(old_list), tokens) {
        Ok(Datatype::List(list)) => list,
        _ => return Err(ProgramError::InvalidOperation),
    };

    //Iterate over this list and push the element onto the stack 
    for (i, item) in new_list.iter().enumerate() {
        
        if i != new_list.len() - 1 {
            stack.push(item.clone());
        } else {
            return Ok(item.clone())
        }
    }

    return Err(ProgramError::InvalidOperation)
    
}

fn if_(predicate : Datatype, tokens: &mut Vec<&str>) -> Result<Datatype, ProgramError>{
    
    let first_expression = tokens.pop().unwrap();

    let true_expression = match datatype(first_expression, tokens){
        Some(dt) => dt,
        None => Datatype::Code(first_expression.to_string()),
    };

    let second_expression = tokens.pop().unwrap();

    let false_expression = match datatype(second_expression, tokens){
        Some(dt) => dt,
        None => Datatype::Code(second_expression.to_string()),
    };
    // println!("True expression: {:?}", true_expression);
    // println!("False expression: {:?}", false_expression);

    let expression : Datatype;
    
    match predicate {
        Datatype::Boolean(boolean) => {
            if boolean {
                expression = true_expression;
            } else {
                expression = false_expression;
            }

            Ok(expression)
        },
        _ => Err(ProgramError::ExpectedBool)
    }
}

fn map(list: Datatype, tokens: &mut Vec<&str>) -> Result<Datatype, ProgramError> {
    let operation = tokens.pop().ok_or(ProgramError::InvalidOperation)?;
    let code_block = match datatype(operation, tokens) {
        Some(Datatype::Code(code)) => code,
        _ => operation.to_string(),
    };

    //println!("Code block in map: {}", code_block);

    let new_list = match list {
        Datatype::List(list) => {
            let mut new_list = Vec::new();
            for item in list {
                let formatted_item = format_stack_item(item);
                let formatted_code = format!("{} {}", formatted_item, &code_block);
                //println!("Formatted code: {}", formatted_code);
                let result = interpreter(&formatted_code)?;
                match result {
                    Datatype::Code(code_exp) => {
                        let code_to_execute = format!("{} {}", formatted_item, code_exp);
                        new_list.push(interpreter(&code_to_execute)?);
                    }
                    _ => new_list.push(result),
                }
            }
            Datatype::List(new_list)
        }
        _ => return Err(ProgramError::ExpectedList),
    };

    Ok(new_list)
}

fn read_line() -> String {
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_) => input,
        Err(error) => panic!("Error: {}", error), 
    }
}

fn datatype(token: &str, tokens: &mut Vec<&str>) -> Option<Datatype> {
    match token {
        "{" => {
            match code(tokens) {
                Ok(value) => Some(value),
                Err(_) => None,
            }
        }
        "[" => {
            match list(token, tokens) {
                Ok(value) => Some(value),
                Err(_) => None,
            }
        }
        _ if token.parse::<f64>().is_ok() && token.contains('.') => {
            Some(Datatype::Float(token.parse::<f64>().unwrap()))
        }
        _ if token.parse::<i128>().is_ok() => {
            Some(Datatype::Int(token.parse::<i128>().unwrap()))
        }
        "False" | "True" => {
            let bool_value = token.to_ascii_lowercase();
            Some(Datatype::Boolean(bool_value.parse::<bool>().unwrap()))
        }
        "\"" => {
            match string(tokens) {
                Ok(value) => Some(value),
                Err(_) => None,
            }
        }
        _ => None,
    }
}


fn list(token: &str, tokens: &mut Vec<&str>) -> Result<Datatype, ProgramError> {
    let mut list_ : Vec<Datatype> = Vec::new();

    //Looping over all the list elements till the closing bracket
    while token != "]" {

        //If the token is a list, call the list function recursively
        let new_token = tokens.pop().unwrap();
        if new_token == "]" {
            return Ok(Datatype::List(list_));
        }
        match datatype(new_token, tokens) {
            Some(value) => list_.push(value),
            None => return Err(ProgramError::IncompleteList),
        }
    }
    Ok(Datatype::List(list_))
}

fn string(tokens: &mut Vec<&str>) -> Result<Datatype, ProgramError> {
    let mut string_ = String::new();
    let mut token = tokens.pop().unwrap();
    while token != "\"" {
        string_.push_str(format!(" {}", token).as_str());
        if tokens.is_empty() {
            return Err(ProgramError::IncompleteString)
        }
        token = tokens.pop().unwrap();
    }
    Ok(Datatype::String(string_.trim().to_string()))
}

fn code(tokens: &mut Vec<&str>) -> Result<Datatype, ProgramError> {
    let mut code_ = String::new();
    let mut new_token = tokens.pop().unwrap();
    while new_token != "}" {
        
        if new_token == "{" {
            match code(tokens) {
                Ok(value) => {
                    let inner_code = format_stack_item(value);
                    code_.push_str(format!(" {{ {} }}", inner_code).as_str());
                },
                Err(e) => return Err(e)
            }
        }

        if new_token == "}" {
            return Ok(Datatype::Code(code_));
        }

        if new_token != "{" {
            code_.push_str(format!(" {}", new_token).as_str());
        }

        if tokens.is_empty() {
            return Err(ProgramError::IncompleteQuotation)
        }
        new_token = tokens.pop().unwrap();
    }
    //println!("Code: {}", code_.trim().to_string());
    Ok(Datatype::Code(code_.trim().to_string()))
    

}

fn empty(a : Datatype) -> Result<Datatype, ProgramError> {
    let result = match a {
        Datatype::List(list) => Ok(Datatype::Boolean(list.is_empty())),
        _ => Err(ProgramError::InvalidOperation),
    };
    result
}

fn append(a : Datatype, b : Datatype) -> Result<Datatype, ProgramError> {
    let result = match (a, b) {
        (Datatype::List(mut list), Datatype::List(mut list2)) => {
            list2.append(&mut list);
            Ok(Datatype::List(list2))
        },
        _ => Err(ProgramError::InvalidOperation),
    };
    result
}

fn cons(a : Datatype, b : Datatype) -> Result<Datatype, ProgramError> {
    let result = match (a, b) {
        (Datatype::List(mut list), item) => {
            list.insert(0, item);
            Ok(Datatype::List(list))
        },
        _ => Err(ProgramError::InvalidOperation),
    };
    result
}

fn tail(a : Datatype) -> Result<Datatype, ProgramError> {
    let result = match a {
        Datatype::List(list) => {
            if list.is_empty() {
                Err(ProgramError::InvalidOperation)
            } else {
                let mut new_list = list.clone();
                new_list.remove(0);
                Ok(Datatype::List(new_list))
            }
        },
        _ => Err(ProgramError::InvalidOperation),
    };
    result
}

fn head(a : Datatype) -> Result<Datatype, ProgramError> {
    let result = match a {
        Datatype::List(list) => {
            if list.is_empty() {
                Err(ProgramError::InvalidOperation)
            } else {
                Ok(list[0].clone())
            }
        },
        _ => Err(ProgramError::InvalidOperation),
    };
    result
}

fn length(a : Datatype) -> Result<Datatype, ProgramError> {
    match a {
        Datatype::List(list) => Ok(Datatype::Int(list.len() as i128)),
        Datatype::String(string) => Ok(Datatype::Int(string.len() as i128)),
        Datatype::Code(string) => Ok(Datatype::Int(string.split(' ').collect::<Vec<&str>>().len() as i128)),
        _ => Err(ProgramError::InvalidOperation),
    }
}

fn parse_integer(a : Datatype) -> Result<Datatype, ProgramError> {
    match a {
        Datatype::String(value) => {
            match value.parse::<i128>() {
                Ok(value) => Ok(Datatype::Int(value)),
                Err(_) => Err(ProgramError::InvalidOperation),
            }
        },
        _ => Err(ProgramError::InvalidOperation),
    }
}

fn parse_float(a : Datatype) -> Result<Datatype, ProgramError> {
    match a {
        Datatype::String(value) => {
            match value.parse::<f64>() {
                Ok(value) => Ok(Datatype::Float(value)),
                Err(_) => Err(ProgramError::InvalidOperation),
            }
        },
        _ => Err(ProgramError::InvalidOperation),
    }
}

fn words(a : Datatype) -> Result<Datatype, ProgramError> {
    let result = match a {
        Datatype::String(value) => {
            let words : Vec<Datatype> = value.split(' ').map(|x| Datatype::String(x.to_string())).collect();
            Ok(Datatype::List(words))
        },
        _ => Err(ProgramError::ExpectedString),
    };
    result
}


fn div(a : Datatype, b : Datatype) -> Result<Datatype, ProgramError> {
    let result = match (a, b) {
        (Datatype::Int(a), Datatype::Int(b)) => Ok(Datatype::Int(b / a)),
        (Datatype::Float(a), Datatype::Float(b)) => Ok(Datatype::Int((b as i128) / (a as i128))),
        (Datatype::Int(a), Datatype::Float(b)) => Ok(Datatype::Int((b as i128) / a as i128)),
        (Datatype::Float(a), Datatype::Int(b)) => Ok(Datatype::Int((b as i128) / (a as i128))),
        _ => Err(ProgramError::ExpectedNumber),
    };
    result
}

fn equal(a : Datatype, b : Datatype) -> Result<Datatype, ProgramError> {
    let result = match (a, b) {
        (Datatype::Int(a), Datatype::Int(b)) => Ok(Datatype::Boolean(b == a)),
        (Datatype::Float(a), Datatype::Float(b)) => Ok(Datatype::Boolean(b == a)),
        (Datatype::Int(a), Datatype::Float(b)) => Ok(Datatype::Boolean(b == a as f64)),
        (Datatype::Float(a), Datatype::Int(b)) => Ok(Datatype::Boolean((b as f64) == a)),
        (Datatype::Boolean(a), Datatype::Boolean(b)) => Ok(Datatype::Boolean(b == a)),
        (Datatype::String(a), Datatype::String(b)) => Ok(Datatype::Boolean(b == a)),
        (Datatype::List(a), Datatype::List(b)) => Ok(Datatype::Boolean(equal_list(a, b))),
        _ => Err(ProgramError::InvalidOperation),
    };
    result
}

fn equal_list(a : Vec<Datatype>, b : Vec<Datatype>) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut valid : bool = true;
    for (i, item) in a.iter().enumerate() {
        if let Datatype::List(f) = item {
            if let Datatype::List(g) = &b[i] {
                if !equal_list(f.clone(), g.clone()) {
                    return false;
                }
            } else {
                return false;
            }
        } else if let Ok(e) = equal(item.clone(), b[i].clone()) {
            if let Datatype::Boolean(c) = e {
                valid = valid && c;
            } 
        }
    }
    return true
}

fn larger_than(a : Datatype, b : Datatype) -> Result<Datatype, ProgramError> {
    let result = match (a, b) {
        (Datatype::Int(a), Datatype::Int(b)) => Ok(Datatype::Boolean(b > a)),
        (Datatype::Float(a), Datatype::Float(b)) => Ok(Datatype::Boolean(b > a)),
        (Datatype::Int(a), Datatype::Float(b)) => Ok(Datatype::Boolean(b > a as f64)),
        (Datatype::Float(a), Datatype::Int(b)) => Ok(Datatype::Boolean((b as f64) > a)),
        _ => Err(ProgramError::InvalidOperation),
    };
    result
}

fn less_than(a : Datatype, b : Datatype) -> Result<Datatype, ProgramError> {
    let result = match (a, b) {
        (Datatype::Int(a), Datatype::Int(b)) => Ok(Datatype::Boolean(b < a)),
        (Datatype::Float(a), Datatype::Float(b)) => Ok(Datatype::Boolean(b < a)),
        (Datatype::Int(a), Datatype::Float(b)) => Ok(Datatype::Boolean(b < a as f64)),
        (Datatype::Float(a), Datatype::Int(b)) => Ok(Datatype::Boolean((b as f64) < a)),
        _ => Err(ProgramError::ExpectedNumber),
    };
    result
}



fn and(a : Datatype, b : Datatype) -> Result<Datatype, ProgramError> {
    let result = match (a, b) {
        (Datatype::Boolean(a), Datatype::Boolean(b)) => Ok(Datatype::Boolean(a && b)),
        _ => Err(ProgramError::ExpectedBool),
    };
    result
}

fn or(a : Datatype, b : Datatype) -> Result<Datatype, ProgramError> {
    let result = match (a, b) {
        (Datatype::Boolean(a), Datatype::Boolean(b)) => Ok(Datatype::Boolean(a || b)),
        _ => Err(ProgramError::ExpectedBool),
    };
    result
}

fn not(a : Datatype) -> Result<Datatype, ProgramError> {
    let result = match a {
        Datatype::Boolean(a) => Ok(Datatype::Boolean(!a)),
        _ => Err(ProgramError::InvalidOperation),
    };
    result
}

fn add(a : Datatype, b : Datatype) -> Result<Datatype, ProgramError> {
    let result = match (a, b) {
        (Datatype::Int(a), Datatype::Int(b)) => Ok(Datatype::Int(a + b)),
        (Datatype::Float(a), Datatype::Float(b)) => Ok(Datatype::Float(a + b)),
        (Datatype::Int(a), Datatype::Float(b)) => Ok(Datatype::Float(a as f64 + b)),
        (Datatype::Float(a), Datatype::Int(b)) => Ok(Datatype::Float(a + b as f64)),
        _ => Err(ProgramError::ExpectedNumber),
    };
    result
}

fn subtract(a : Datatype, b : Datatype) -> Result<Datatype, ProgramError> {
    let result = match (a, b) {
        (Datatype::Int(a), Datatype::Int(b)) => Ok(Datatype::Int(b - a)),
        (Datatype::Float(a), Datatype::Float(b)) => Ok(Datatype::Float(b - a)),
        (Datatype::Float(a), Datatype::Int(b)) => Ok(Datatype::Float(b as f64 - a)),
        (Datatype::Int(a), Datatype::Float(b)) => Ok(Datatype::Float(b - a as f64)),
        _ => Err(ProgramError::ExpectedNumber),
    };
    result
}

fn times(number : Datatype, tokens : &mut Vec<&str>, stack : &mut Vec<Datatype>) -> Result<Datatype, ProgramError> {
    let token = tokens.pop().unwrap();
    let evaluated_codeblock = match datatype(token, tokens) {
        Some(Datatype::Code(code)) => match interpreter(&code) {
            Ok(dt) => dt,
            Err(e) => return Err(e),
        },
        Some(dt) => dt,
        None => Datatype::Code(token.to_string()),
    };

    match number {
        Datatype::Int(value) => {
            let new_list = Vec::new();
            for i in 0..value {
                if i < value - 1 {
                    stack.push(evaluated_codeblock.clone());
                } else {
                    return Ok(evaluated_codeblock.clone());
                }
            }
            Ok(Datatype::List(new_list))
        },
        _ => Err(ProgramError::InvalidOperation),
    }
}

fn multiply(a : Datatype, b : Datatype) -> Result<Datatype, ProgramError> {
    let result = match (a, b) {
        (Datatype::Int(a), Datatype::Int(b)) => Ok(Datatype::Int(a * b)),
        (Datatype::Float(a), Datatype::Float(b)) => Ok(Datatype::Float(a * b)),
        (Datatype::Int(a), Datatype::Float(b)) => Ok(Datatype::Float(a as f64 * b)),
        (Datatype::Float(a), Datatype::Int(b)) => Ok(Datatype::Float(a * b as f64)),
        _ => Err(ProgramError::InvalidOperation),
    };
    result
}

fn divide(a : Datatype, b : Datatype) -> Result<Datatype, ProgramError> {
    let result = match (a, b) {
        (Datatype::Int(a), Datatype::Int(b)) => Ok(Datatype::Float((b as f64) / (a as f64))),
        (Datatype::Float(a), Datatype::Float(b)) => Ok(Datatype::Float(b / a)),
        (Datatype::Int(a), Datatype::Float(b)) => Ok(Datatype::Float(b / a as f64)),
        (Datatype::Float(a), Datatype::Int(b)) => Ok(Datatype::Float(b as f64 / a)),
        _ => Err(ProgramError::InvalidOperation),
    };
    result
}


fn format_stack_item(stack_item : Datatype) -> String {
    match stack_item {
        Datatype::Int(value) => format!("{}", value),
        Datatype::Float(value) => format!("{:?}", value),
        Datatype::Boolean(value) => format!("{}", if value { "True" } else { "False" }),
        Datatype::List(list) => format!("[{}]", format_list(&list)),
        Datatype::String(value) => format!("\" {} \"", value),
        Datatype::Code(value) => format!("{}", value),
    }
}

fn format_list(list : &Vec<Datatype>) -> String {
    let mut list_str = String::new();
    let length = list.len();
    for (i, item) in list.iter().enumerate() {
        if let Datatype::List(f) = item {
            write!(list_str, "[{}]", format_list(f)).unwrap();
        } else {
            write!(list_str, "{}", format_stack_item(item.clone())).unwrap();
        }
        if i < length - 1 {
            write!(list_str, ",").unwrap();
        }
    }
    if list_str == "" {
        return " ".to_string()
    } else {
        return list_str
    }
}

fn tests() {
    let testings: Vec<(String, String)> = vec![
        ("3".to_string(), "3".to_string()),
        ("121231324135634563456363567".to_string(), "121231324135634563456363567".to_string()),
        ("1.0".to_string(), "1.0".to_string()),
        ("0.0".to_string(), "0.0".to_string()),
        ("-1".to_string(), "-1".to_string()),
        ("-1.1".to_string(), "-1.1".to_string()),
        ("False".to_string(), "False".to_string()),
        ("True".to_string(), "True".to_string()),
        ("[ [ ] [ ] ]".to_string(), "[[ ],[ ]]".to_string()),
        ("[ False [ ] True [ 1 2 ] ]".to_string(), "[False,[ ],True,[1,2]]".to_string()),
        ("\" [ so { not if ] and } \"".to_string(), "\" [ so { not if ] and } \"".to_string()),
        //("{ 20 10 + }".to_string(), "{ 20 10 + }".to_string()),
        //("[ { + } { 10 + } { 20 10 + } ]".to_string(), "[{ + },{ 10 + },{ 20 10 + }]".to_string()),
        ("1 1 +".to_string(), "2".to_string()),
        ("10 20 *".to_string(), "200".to_string()),
        ("20 2 div".to_string(), "10".to_string()),
        ("20 2 /".to_string(), "10.0".to_string()),
        ("1 1.0 +".to_string(), "2.0".to_string()),
        ("10 20.0 *".to_string(), "200.0".to_string()),
        ("20 2.0 div".to_string(), "10".to_string()),
        ("20.0 2.0 div".to_string(), "10".to_string()),
        ("False False &&".to_string(), "False".to_string()),
        ("False True ||".to_string(), "True".to_string()),
        ("False not".to_string(), "True".to_string()),
        ("True not".to_string(), "False".to_string()),
        ("20 10 <".to_string(), "False".to_string()),
    ("20 10 >".to_string(), "True".to_string()),
    ("20 10.0 >".to_string(), "True".to_string()),
    ("20.0 20.0 >".to_string(), "False".to_string()),
    ("10 10 ==".to_string(), "True".to_string()),
    ("10 10.0 ==".to_string(), "True".to_string()),
    ("True True ==".to_string(), "True".to_string()),
    ("True 40 40 == ==".to_string(), "True".to_string()),
    ("\" abba \" \" abba \" ==".to_string(), "True".to_string()),
    ("[ ] [ ] ==".to_string(), "True".to_string()),
    ("[ 1 2 ] [ 1 2 ] ==".to_string(), "True".to_string()),
    ("[ [ ] ] [ [ ] ] ==".to_string(), "True".to_string()),

    // Stack operations
    ("10 20 swap pop".to_string(), "20".to_string()),
    ("10 dup dup + swap pop".to_string(), "20".to_string()),
    ("10 20 swap dup + div".to_string(), "1".to_string()),

    // Length
    ("\" hello \" length".to_string(), "5".to_string()),
    ("\" hello world \" length".to_string(), "11".to_string()),
    ("[ 1 2 3 [ ] ] length".to_string(), "4".to_string()),
    ("{ 10 20 + } length".to_string(), "3".to_string()),

    // String parsing
    ("\" 12 \" parseInteger".to_string(), "12".to_string()),
    ("\" 12.34 \" parseFloat".to_string(), "12.34".to_string()),
    ("\" adam bob charlie \" words".to_string(), "[\" adam \",\" bob \",\" charlie \"]".to_string()),

    // Lists
    ("[ 1 2 3 ]".to_string(), "[1,2,3]".to_string()),
    ("[ 1 \" bob \" ]".to_string(), "[1,\" bob \"]".to_string()),
    ("[ 1 2 ] empty".to_string(), "False".to_string()),
    ("[ ] empty".to_string(), "True".to_string()),
    ("[ 1 2 3 ] head".to_string(), "1".to_string()),
    ("[ 1 2 3 ] length".to_string(), "3".to_string()),
    ("[ 1 2 3 ] tail".to_string(), "[2,3]".to_string()),
    ("1 [ ] cons".to_string(), "[1]".to_string()),
    ("1 [ 2 3 ] cons".to_string(), "[1,2,3]".to_string()),
    ("[ 1 2 ] [ ] append".to_string(), "[1,2]".to_string()),
    ("[ 1 ] [ 2 3 ] append".to_string(), "[1,2,3]".to_string()),
    ("[ 1 ] [ 2 3 ] cons".to_string(), "[[1],2,3]".to_string()),

    
    // If statements
    ("True if { 20 } { }".to_string(), "20".to_string()), 
    ("True if { 20 10 + } { 3 }".to_string(), "30".to_string()), 
    ("10 5 5 == if { 10 + } { 100 + }".to_string(), "20".to_string()),
    ("False if { } { 45 }".to_string(), "45".to_string()),
    ("True if { False if { 50 } { 100 } } { 30 }".to_string(), "100".to_string()), 

    // If without quotation
    ("True if 20 { }".to_string(), "20".to_string()), 
    ("True if { 20 10 + } 3".to_string(), "30".to_string()),
    ("10 10 5 5 == if + { 100 + }".to_string(), "20".to_string()),
    ("False if { } 45".to_string(), "45".to_string()),
    ("True if { False if 50 100 } 30".to_string(), "100".to_string()),

    // List quotations
    ("[ 1 2 3 ] map { 10 * }".to_string(), "[10,20,30]".to_string()),
    ("[ 1 2 3 ] map { 1 + }".to_string(), "[2,3,4]".to_string()),
    ("[ 1 2 3 4 ] map { dup 2 > if { 10 * } { 2 * } }".to_string(), "[2,4,30,40]".to_string()),
    ("[ 1 2 3 4 ] each { 10 * } + + +".to_string(), "100".to_string()),
    ("[ 1 2 3 4 ] 0 foldl { + }".to_string(), "10".to_string()), 
    ("[ 2 5 ] 20 foldl { div }".to_string(), "2".to_string()),
    ("[ \" 1 \" \" 2 \" \" 3 \" ] each { parseInteger } [ ] cons cons cons".to_string(), "[1,2,3]".to_string()), 
    ("[ 1 2 3 4 ] 0 foldl +".to_string(), "10".to_string()),
    ("[ 2 5 ] 20 foldl div".to_string(), "2".to_string()),
    ("[ \" 1 \" \" 2 \" \" 3 \" ] each parseInteger [ ] 3 times cons".to_string(), "[1,2,3]".to_string()),



    // Quotations
    ("{ 20 10 + } exec".to_string(), "30".to_string()),
    ("10 { 20 + } exec".to_string(), "30".to_string()),
    ("10 20 { + } exec".to_string(), "30".to_string()),
    ("{ { 10 20 + } exec } exec".to_string(), "30".to_string()),
    //("{ { 10 20 + } exec 20 + } exec".to_string(), "50".to_string()),
    

    // Times
     ("1 times { 100 50 + }".to_string(), "150".to_string()),
    //("5 times { 1 } [ ] 5 times { cons } 0 foldl { + }".to_string(), "5".to_string()),
    // ("5 times 1 [ ] 5 times cons 0 foldl +".to_string(), "5".to_string()),
    // ("5 times { 10 } + + + +".to_string(), "50".to_string()),
    // ("5 times 10 4 times +".to_string(), "50".to_string()),
    

    // // Assignments
    // ("age".to_string(), "age".to_string()),
    // ("age 10 := age".to_string(), "10".to_string()),
    // ("10 age swap := age".to_string(), "10".to_string()),
    // ("[ 1 2 3 ] list swap := list".to_string(), "[1,2,3]".to_string()),
    // ("age 20 := [ 10 age ]".to_string(), "[10,20]".to_string()),

    // // Functions
    // ("inc { 1 + } fun 1 inc".to_string(), "2".to_string()),
    // ("mul10 { 10 * } fun inc { 1 + } fun 10 inc mul10".to_string(), "110".to_string()),


    // // Loop
    // ("1 loop { dup 4 > } { dup 1 + } [ ] 5 times { cons }".to_string(), "[1,2,3,4,5]".to_string()),
    // ("1 loop { dup 4 > } { dup 1 + } [ ] 5 times cons".to_string(), "[1,2,3,4,5]".to_string()),
    // ("[ 1 ] loop { dup length 9 > } { dup head 1 + swap cons }".to_string(), "[10,9,8,7,6,5,4,3,2,1]".to_string()),

    ];

    println!("Running tests...");

    let mut tests_passed : usize = 0;
    for (index, (input, output)) in testings.iter().enumerate() {
        println!("Test {} passed", index+1);
        tests_passed = index;
        let result = match interpreter(&input){
            Ok(value) => format_stack_item(value),
            Err(e) => format!("{:?}", e),
        };
        assert!(result == *output, "FAIL on test {}\n- test: {}\n- result: {}\n- expected: {}", index, input, result, output);
    }

    println!("{} successful tests!", tests_passed+1);

}