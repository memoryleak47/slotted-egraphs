use crate::lamcalc::*;

impl Ast {
    pub fn parse(s: &str) -> Ast {
        let tokens = tokenize(s);

        let (ast, rest) = assemble_impl(&tokens);
        assert!(rest.is_empty());

        ast
    }
}

///// assemble

fn assemble_impl(tk: &[Token]) -> (Ast, &[Token]) {
    match tk {
        [Token::Ident(x), rest@..] => (Ast::Var(String::from(x)), rest),
        [Token::LParen, Token::Lam, Token::Ident(x), rest@..] => {
            let (b, rest) = assemble_impl(rest);
            let [Token::RParen, rest@..] = rest else { panic!() };
            let out = Ast::Lam(String::from(x), Box::new(b));
            (out, rest)
        },
        [Token::LParen, Token::App, rest@..] => {
            let (l, rest) = assemble_impl(rest);
            let (r, rest) = assemble_impl(rest);
            let [Token::RParen, rest@..] = rest else { panic!() };
            let out = Ast::App(Box::new(l), Box::new(r));
            (out, rest)
        },
        _ => panic!("parser error: invalid format!"),
    }
}

///// tokenization

enum Token {
    LParen,
    RParen,
    Lam, App,
    Ident(String),
}

fn tokenize(s: &str) -> Vec<Token> {
    let mut current_string: Option<String> = None;
    let mut tokens = Vec::new();

    let cleanup_current_string = |current_string: &mut Option<String>, tokens: &mut Vec<Token>| {
        if let Some(x) = current_string.take() {
            match &*x {
                "lam" => tokens.push(Token::Lam),
                "app" => tokens.push(Token::App),
                x => tokens.push(Token::Ident(String::from(x))),
            }
        }
    };

    for ch in s.chars() {
        if ch.is_alphanumeric() || ch == '_' {
            if current_string.is_none() {
                current_string = Some(String::new());
            }

            current_string.as_mut().unwrap().push(ch);
        } else {
            cleanup_current_string(&mut current_string, &mut tokens);

            if ch.is_whitespace() {
                continue;
            } else if ch == '(' {
                tokens.push(Token::LParen);
            } else if ch == ')' {
                tokens.push(Token::RParen);
            } else {
                panic!("invalid character {ch}!");
            }
        }
    }

    cleanup_current_string(&mut current_string, &mut tokens);

    tokens
}
