use crate::*;

#[derive(Debug)]
pub enum ParseError {
    TokenState(String),
    ParseState(Vec<Token>),
    RemainingRest(Vec<Token>),
    FromOpFailed(String, Vec<Child>),
    ExpectedColonEquals(Vec<Token>),
    ExpectedRBracket(Vec<Token>)
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
    if "()[]".contains(c) { return false; }
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
        } else if s.starts_with('?') {
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
impl<L: Language> Pattern<L> {
    pub fn parse(s: &str) -> Result<Self, ParseError> {
        let tok = tokenize(s)?;
        let (re, rest) = parse_pattern(&tok)?;

        if !rest.is_empty() {
            return Err(ParseError::RemainingRest(to_vec(rest)));
        }

        Ok(re)
    }
}

impl<L: Language> RecExpr<L> {
    pub fn parse(s: &str) -> Result<Self, ParseError> {
        let pat = Pattern::parse(s)?;
        Ok(pattern_to_re(&pat))
    }
}

fn parse_pattern<L: Language>(tok: &[Token]) -> Result<(Pattern<L>, &[Token]), ParseError> {
    let (mut pat, mut tok) = parse_pattern_nosubst(tok)?;
    while let Some(Token::LBracket) = tok.get(0) {
        tok = &tok[1..];
        let (l, tok2) = parse_pattern(tok)?;
        tok = tok2;

        let Token::ColonEquals = &tok[0] else { return Err(ParseError::ExpectedColonEquals(to_vec(tok))) };
        tok = &tok[1..];

        let (r, tok2) = parse_pattern(tok)?;
        tok = tok2;

        let Token::RBracket = &tok[0] else { return Err(ParseError::ExpectedRBracket(to_vec(tok))) };
        tok = &tok[1..];

        pat = Pattern::Subst(Box::new(pat), Box::new(l), Box::new(r));
    }
    Ok((pat, tok))
}

fn parse_pattern_nosubst<L: Language>(mut tok: &[Token]) -> Result<(Pattern<L>, &[Token]), ParseError> {
    if let Token::PVar(p) = &tok[0] {
        let pat = Pattern::PVar(p.to_string());
        return Ok((pat, &tok[1..]));
    }

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
                ChildImpl::Pattern(_) => Child::AppliedId(AppliedId::null()),
            }
        ).collect();
        let node = L::from_op(op, children_mock.clone()).ok_or_else(|| ParseError::FromOpFailed(op.to_string(), children_mock))?;
        let children = children.into_iter().filter_map(|x| match x {
            ChildImpl::Pattern(pat) => Some(pat),
            ChildImpl::Slot(_) => None,
        }).collect();
        let re = Pattern::ENode(node, children);
        Ok((re, tok))
    } else {
        let Token::Ident(op) = &tok[0] else { return Err(ParseError::ParseState(to_vec(tok))) };
        tok = &tok[1..];

        let node = L::from_op(op, vec![]).ok_or_else(|| ParseError::FromOpFailed(op.to_string(), vec![]))?;
        let pat = Pattern::ENode(node, Vec::new());
        Ok((pat, tok))
    }
}

enum ChildImpl<L: Language> {
    Pattern(Pattern<L>),
    Slot(Slot),
}

fn parse_child<L: Language>(tok: &[Token]) -> Result<(ChildImpl<L>, &[Token]), ParseError> {
    if let Token::Slot(slot) = tok[0] {
        return Ok((ChildImpl::Slot(slot), &tok[1..]));
    }

    parse_pattern::<L>(tok).map(|(x, rest)| (ChildImpl::Pattern(x), rest))
}

// print:
impl<L: Language> std::fmt::Display for Pattern<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Pattern::ENode(node, children) => {
                let (x, rest) = node.to_op();

                if rest.is_empty() {
                    return write!(f, "{}", x);
                }

                write!(f, "({} ", x)?;
                let mut child_idx = 0;
                let n = rest.len();
                for (i, r) in rest.into_iter().enumerate() {
                    match r {
                        Child::AppliedId(_) => {
                            write!(f, "{}", &children[child_idx])?;
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
            Pattern::PVar(p) => write!(f, "?{p}"),
            Pattern::Subst(b, x, t) => write!(f, "{b}[{x} := {t}]"),
        }
    }
}

impl<L: Language> std::fmt::Debug for Pattern<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl<L: Language> std::fmt::Display for RecExpr<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", re_to_pattern(self))
    }
}

impl<L: Language> std::fmt::Debug for RecExpr<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", re_to_pattern(self))
    }
}


fn to_vec<T: Clone>(t: &[T]) -> Vec<T> {
    t.iter().cloned().collect()
}
