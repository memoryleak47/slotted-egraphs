use crate::*;

#[derive(Debug)]
pub enum ParseError {
    TokenState(String),
    ParseState(Vec<Token>),
    RemainingRest(Vec<Token>),
    FromOpFailed(String, Vec<Child>)
}

#[derive(Debug, Clone)]
enum Token {
    Slot(Slot), // s42
    Ident(String), // map
    PVar(String), // ?x
    ColonEquals, // :=
    LParen, // (
    RParen, // )
    LBracket, // [
    RBracket, // ]
}

fn ident_char(c: char) -> bool {
    if c.is_whitespace() { return false; }
    // TODO re-add '?' character here.
    if "()[]$:=".contains(c) { return false; }
    true
}

fn crop_ident(s: &str) -> Result<(/*ident*/ &str, /*rest*/ &str), ParseError> {
    let out = if let Some((i, _)) = s.char_indices().find(|(_, x)| !ident_char(*x)) {
        (&s[..i], &s[i..])
    } else {
        (s, "")
    };

    if out.0.is_empty() { return Err(ParseError::TokenState(s.to_string())); }

    Ok(out)
}

fn tokenize(mut s: &str) -> Result<Vec<Token>, ParseError> {
    let mut current = String::new();
    let mut tokens = Vec::new();

    loop {
        s = s.trim_start();
        if s.is_empty() { break; }

        if s.starts_with('(') {
            tokens.push(Token::LParen);
            s = &s[1..];
        } else if s.starts_with(')') {
            tokens.push(Token::RParen);
            s = &s[1..];
        } else if s.starts_with('[') {
            tokens.push(Token::LBracket);
            s = &s[1..];
        } else if s.starts_with(']') {
            tokens.push(Token::RBracket);
            s = &s[1..];
        } else if s.starts_with(":=") {
            tokens.push(Token::ColonEquals);
            s = &s[2..];
        } else if s.starts_with('?') && false { // temporary disable.
            let (op, rst) = crop_ident(&s[1..])?;
            tokens.push(Token::PVar(op.to_string()));
            s = rst;
        } else if s.starts_with('$') {
            let (op, rst) = crop_ident(&s[1..])?;
            tokens.push(Token::Slot(Slot::named(op)));
            s = rst;
        } else {
            let (op, rst) = crop_ident(s)?;
            tokens.push(Token::Ident(op.to_string()));
            s = rst;
        }
    }

    Ok(tokens)
}

// parse:
impl<L: Language> RecExpr<L> {
    pub fn parse(s: &str) -> Result<Self, ParseError> {
        let tok = tokenize(s)?;
        let (re, rest) = parse_rec_expr(&tok)?;

        if !rest.is_empty() {
            return Err(ParseError::RemainingRest(to_vec(rest)));
        }

        Ok(re)
    }
}

fn parse_rec_expr<L: Language>(mut tok: &[Token]) -> Result<(RecExpr<L>, &[Token]), ParseError> {
    if let Token::LParen = tok[0] {
        tok = &tok[1..];

        let Token::Ident(op) = &tok[0] else { return Err(ParseError::ParseState(to_vec(tok))) };
        tok = &tok[1..];

        let mut children = Vec::new();
        loop {
            if let Token::RParen = tok[0] { break };

            let (child, tok2) = parse_child(tok)?;
            tok = tok2;
            children.push(child);
        }
        tok = &tok[1..];

        let children_mock: Vec<_> = children.iter().map(|x|
            match x {
                ChildImpl::Slot(s) => Child::Slot(*s),
                ChildImpl::RecExpr(_) => Child::AppliedId(AppliedId::null()),
            }
        ).collect();
        let node = L::from_op(op, children_mock.clone()).ok_or_else(|| ParseError::FromOpFailed(op.to_string(), children_mock))?;
        let children = children.into_iter().filter_map(|x| match x {
            ChildImpl::RecExpr(re) => Some(re),
            ChildImpl::Slot(_) => None,
        }).collect();
        let re = RecExpr { node, children };
        Ok((re, tok))
    } else {
        let Token::Ident(op) = &tok[0] else { return Err(ParseError::ParseState(to_vec(tok))) };
        tok = &tok[1..];

        let node = L::from_op(op, vec![]).ok_or_else(|| ParseError::FromOpFailed(op.to_string(), vec![]))?;
        let re = RecExpr { node, children: Vec::new() };
        Ok((re, tok))
    }
}

enum ChildImpl<L: Language> {
    RecExpr(RecExpr<L>),
    Slot(Slot),
}

fn parse_child<L: Language>(tok: &[Token]) -> Result<(ChildImpl<L>, &[Token]), ParseError> {
    if let Token::Slot(slot) = tok[0] {
        return Ok((ChildImpl::Slot(slot), &tok[1..]));
    }

    parse_rec_expr::<L>(tok).map(|(x, rest)| (ChildImpl::RecExpr(x), rest))
}

// print:
impl<L: Language> std::fmt::Display for RecExpr<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (x, rest) = self.node.to_op();

        if rest.is_empty() {
            return write!(f, "{}", x);
        }

        write!(f, "({} ", x)?;
        let mut child_idx = 0;
        let n = rest.len();
        for (i, r) in rest.into_iter().enumerate() {
            match r {
                Child::AppliedId(_) => {
                    write!(f, "{}", &self.children[child_idx])?;
                    child_idx += 1;
                },
                Child::Slot(slot) => {
                    write!(f, "{}", slot.to_string())?;
                },
            }
            if i != n-1 { write!(f, " ")?; }
        }
        write!(f, ")")
    }
}

impl<L: Language> std::fmt::Debug for RecExpr<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RE({})", self)
    }
}


fn to_vec<T: Clone>(t: &[T]) -> Vec<T> {
    t.iter().cloned().collect()
}
