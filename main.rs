use std::{collections::HashMap, io::{stdin, Stdin, stdout, Write, Read}};
use crossterm::{*, style::*};

#[derive(Clone, Debug, PartialEq)]
enum Token {
    Function{
        param: String,
        body: Box<Token>
    },
    NonSetParam(String),
    Call{
        func: Box<Token>,
        input: Vec<Token>
    },
    Ident(String)
}
impl Token{
    fn apply_param(&mut self, mut input: Token, param_name: String) {
        // unbox func
        // recursively replace all NonSetParam == func.param with input[0]
        // print the new
        match self {
            Token::Function { ref mut param, body } => {
                if param.as_str() == param_name.as_str() {
                    // tried to avoid repeating parameters name but it just won't work
                    let mut new = param.clone();
                    new.push('+');
                    input.apply_param(Token::NonSetParam(new.clone()), param.clone());
                    *param = new;
                }

                body.apply_param(input, param_name);
            }
            Token::Call { func, input: innput } => {
                func.apply_param(input.clone(), param_name.clone());
                for e in innput {
                    e.apply_param(input.clone(), param_name.clone());
                }
            }
            Token::NonSetParam(name) => {
                if *name == param_name {
                    *self = input;
                }
            }
            _=>{}
        }
    }
    fn to_string(&self) -> String{
        // (>f g h.f g (h h))(>x y.x)(>t.t t)
        // (>g.>h.(>x y.x) g (h h))(>t.t t)

        // (>g.>h.(>x.>y.(x) g (h h)))(>t.(t t))
        // (>g.>h.)
        
        match self{
            Token::Function { param, body } => {
                let mut acc = format!(">{}.", param);
                acc.push_str(&body.to_string());
                acc
            }
            Token::Call { func, input } => {
                let mut acc = if let Token::Function{..} = &**func {
                    format!("({})",func.to_string())
                } else {
                    format!("{}",func.to_string())
                };
                format_call_input(input, &mut acc);
                acc
            }
            Token::NonSetParam(param) => {
                param.clone()
            }
            Token::Ident(ident) => {
                ident.clone()
            }
        }
    }
}

fn format_call_input(input: &Vec<Token>, acc: &mut String) {
    for token in input{
        if let Token::Function{..} | Token::Call{..} = token {
            acc.push(' ');
            acc.push('(');
            acc.push_str(&token.to_string());
            acc.push(')');
        } else {
            acc.push(' ');
            acc.push_str(&token.to_string());
        }
    }
}

struct Session{
   functions: HashMap<String, Token> 
}
impl Session {
    fn new() -> Session {
        Session{functions: HashMap::new()}
    }
}


fn main() {
    let mut sess = Session::new();
    let stdi = stdin();
    let mut stdo = stdout();
    sess.functions.insert("I".to_string(),Token::Function{
        param: "x".to_owned(),
        body: Box::new(Token::NonSetParam("x".to_string()))
    });

    let mut user = String::new();

    let welcome = "Welcome to the lamda calculus comand line interpreter.
Syntax for declaration is FUNCTIONNAME>parameters.body
Parameters must be single characters and you can assign a name only to the outmost function\n
$ ";
    execute!(stdo, Print(welcome)).unwrap();

    stdi.read_line(&mut user).unwrap();
    loop {
        if user.as_str() == "quit" {break;}
        let inp  = sess.parse_input(&user);
        // execute!(stdo, Print(format!("{:#?}", &inp))).unwrap();
        if let Some(input) = inp {
            let new =eval(input, &sess);
            if user == new {
                user.clear();
            } else {
                user = new;
            }
        } else {
            match user.as_str() {
                "finish\n" => user.clear(),
                "quit\n" => break,
                _ => {
                    user.clear();
                }
            }
        }

        if user.len() == 0 {
            execute!(stdo, Print("$ ")).unwrap();
            stdi.read_line(&mut user).unwrap();
        } else {
            execute!(stdo, Print("$ "), Print(user.to_string())).unwrap();

            let mut action = String::new();

            stdi.read_line(&mut action).unwrap();
            match action.as_str() {
                "finish\n" => user.clear(),
                "quit\n" => break,
                _ => {}
            }
        } 
    }
}

fn eval(inp: Token, sess: &Session) -> String{
    let mut user;
    match inp {
        Token::Call{func, mut input} => {
            if input.len() == 0 {
                return eval(*func, sess);
            }
            if let Token::NonSetParam(p) = *func {
                user = p;
                format_call_input(&input, &mut user);
                return user;
            }
            match *func {

                Token::Function { param, mut body } => {
                    body.apply_param(input.remove(0), param);
                    let mut b = format!("({})", body.to_string());

                    format_call_input(&input, &mut b);

                    user = b;
                }

                Token::Call { func: func1, input: input1 } => {
                    let mut b = format!("({})", func1.to_string());
        
                    format_call_input(&input1, &mut b);
                    format_call_input(&input, &mut b);

                    user = b;
                }
    
                Token::Ident(i) => {
                    if let Some(Token::Function{..}) = sess.functions.get(&i){
                        let mut b = format!("({})", sess.functions.get(&i).unwrap().to_string());

                        format_call_input(&input, &mut b);

                        user = b;
                    } else {
                        user = String::new();
                    }
                }
    
                Token::NonSetParam(p) => {
                    user = format!("{} ", p);
                    for el in input{
                        user.push_str(&el.to_string());
                    }
                }
            }
        }

        Token::Function{body, param} => {
            user = format!(">{}.", param);
            user.push_str(&eval(*body, sess));
        }

        Token::Ident(i) => {
            user = sess.functions.get(&i).unwrap().to_string();
        }
        Token::NonSetParam(p) => {
            user = p;
        }
    }

    user
}

// FN_NAME > fn
// fn == param [ fn |  ('.' body) ]
// body == ['>' fn | (ident)+ | '(' body ')']


impl Session {
    fn parse_input(&mut self, dec: &str) -> Option<Token>{
        let mut strings = Vec::new();
        let mut buff = String::new();
        for chr in dec.chars(){
            match chr {
                ' ' => {
                    if buff.len() > 0 { 
                        strings.push(buff); 
                        buff = "".to_string();
                    }
                }
                '>' | '.' |
                '(' | ')'=> {
                    if buff.len() > 0 { 
                        strings.push(buff); 
                        strings.push(chr.to_string());
                        buff = "".to_string();
                    } else {
                        strings.push(chr.to_string());
                        buff = "".to_string();
                    }
                }
                _ if chr != '\n' => {
                    buff.push(chr);
                }
                _ => {}
            }
        }
        if buff.len() != 0 {strings.push(buff)}
        let vec:Vec<&str> =strings.iter().map(|e| e.as_str()).collect();
        // println!("segmented string:\n{:?}", &vec);
        if vec.len() == 0 {
            None
        } else {
            self.parse_item(&vec[0..])
        }
    }
    fn parse_item(&mut self, vec: &[&str]) -> Option<Token> {
        match *vec.get(0)? {
            "(" => {
                // println!("Crating Call with {:?}", vec);
                let (f, mut rest) = self.one_token(&vec[1..], &mut Vec::new())?;
                let mut input = Vec::new();
                // println!("Looping with rest\n{:?}", rest);
                while let Some((token, other)) = self.one_token(&rest[0..], &mut Vec::new()){
                    input.push(token);
                    rest = other; 
                    // println!("Looping with rest\n{:?}", rest);
                }
                let func = Box::new(f);
                Some(Token::Call{func, input})
            }
            ident if vec.get(1) == Some(&">") => {
                let body = self.parse_fn(&vec[2..], &mut Vec::new())?;
                self.functions.insert(ident.to_string(), body);
                None
            }
            ">" => {
                self.parse_fn(&vec[1..], &mut Vec::new())
            }
            ident => {
                if self.functions.contains_key(&ident.to_string()) {
                    let (f, mut rest) = self.one_token(&vec[0..], &mut Vec::new())?;
                    let mut input = vec![f];
                    while let Some((token, other)) = self.one_token(&rest[0..], &mut Vec::new()){
                       input.push(token);
                        rest = other; 
                    }
                    let func = Box::new(Token::Ident(ident.to_string()));
                    Some(Token::Call{func, input})
                } else {
                     None
                    
                }
            }
        }
    }
    fn parse_fn(&self, s: &[&str], params: &mut Vec<String>) -> Option<Token>{
        let param = s.get(0)?.to_string();
        
        if params.contains(&param){ return None; }

        // println!("Parsing function with param {}\n{:?}\n", param, s);
        params.push(param.clone());
        let body = Box::new(
            match *s.get(1)? {
                "." => {
                    // println!("Parsing a call\n{:?}\n", s);
                    self.parse_body(&s[2..], params)?
                }
                ">" | "(" | ")" => {return None;} 
                _ => {
                    self.parse_fn(&s[1..], params)?
            }
        });
        Some(Token::Function{param, body})
    }
    fn parse_body(&self, s: &[&str], params: &mut Vec<String>) -> Option<Token>{
        match *s.get(0)? {
            ">" => {self.parse_fn(&s[1..], params)}
            _ => {
                let mut input:Vec<Token> = Vec::new();
                // println!("Parsing one token\n{:?}\n", s);
                let (f, mut other) = self.one_token(&s[0..], params)?;
                // println!("Token found: {:?}\nWhole input\n{:?}\n", f,s);
                while let Some((token, rest)) = self.one_token(&other[0..], params){
                    // println!("Looping token\n{:?}\n", rest);
                    input.push(token);
                    other = rest;
                }

                let func = Box::new(f);
                if input.len() != 0{
                    Some(Token::Call{func, input})
                } else {
                    Some(*func)
                }
            }
        }
    }

    fn one_token<'a> (&self, s: &'a [&str], params: &mut Vec<String>) -> Option<(Token, &'a [&'a str])> {
        match *s.get(0)? {
            ">" => {
                let mut par = 0;
                let mut d = 0;
                for (idx, item) in s.iter().enumerate() {
                    if item == &"(" {par += 1;}
                    if item == &")" {par -= 1;}
                    d = idx;
                    if par < 0 {
                        break;
                    }
                }

                // println!("Recognised function\n{:?}\n", &s[1..d]);
                Some(
                    (self.parse_fn(&s[1..d], params)?, 
                    &s[d..])
                )
            }
            "(" if *s.get(1)? == ">" => {
                let mut par = 0;
                let mut d = 0;
                for (idx, item) in s.iter().enumerate() {
                    if item == &"(" {par += 1;}
                    if item == &")" {par -= 1;}
                    d = idx;
                    if par == 0 {
                        break;
                    }
                }

                // println!("Recognised function\n{:?}\n", &s[2..d]);
                Some(
                    (self.parse_fn(&s[2..d], params)?, 
                    &s[d..])
                )
            }
            "(" => {
                let mut par = 0;
                let mut d = 0;
                for (idx, item) in s.iter().enumerate() {
                    if item == &"(" {par += 1;}
                    if item == &")" {par -= 1;}
                    d = idx;
                    if par == 0 {
                        break;
                    }
                }

                // println!("Recognised InParenthesis\n", );
                // println!("Parsing a call\n{:?}\n", &s[1..d]);
                Some(
                    (self.parse_body(&s[1..d], params)?, 
                    &s[d..])
                )
            }
            ")" => {
                self.one_token(&s[1..], params)
            }
            ident => {
                if params.contains(&ident.to_string()) {
                    // println!("Recognised param {}\nAll{:?}", ident,&s[0..]);
                    Some((Token::NonSetParam(ident.to_string()), &s[1..]))
                } else if self.functions.contains_key(&ident.to_string()) {
                    // println!("Recognised identifier {}\n", ident);
                    Some((Token::Ident(ident.to_string()), &s[1..]))
                } else {
                    None
                }
            }
        }
    } 
}


#[test]
fn declaration(){
}
#[test]
fn onlytoken(){
    println!("{:?}",Session::one_token(&Session::new(),&[
        ">", "a", ".", "a", ")"
    ], &mut Vec::new()));
}
