use unscanny::Scanner;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Number(i64),
    Ident(String),
    Plus,
    Minus,
    Colon,
    Slash,
    Eof,
    Illegal,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Token::Number(n) => write!(f, "Number({})", n),
            Token::Ident(s) => write!(f, "Ident({})", s),
            Token::Plus => write!(f, "Plus"),
            Token::Minus => write!(f, "Minus"),
            Token::Colon => write!(f, "Colon"),
            Token::Slash => write!(f, "Slash"),
            Token::Eof => write!(f, "Eof"),
            Token::Illegal => write!(f, "Illegal"),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Lexer<'s> {
    s: Scanner<'s>,
}

impl<'a> Lexer<'a> {
    pub fn new(string: &'a str) -> Self {
        Self {
            s: Scanner::new(string),
        }
    }

    pub fn next_token(&mut self) -> Token {
        let token = match self.s.eat() {
            Some('+') => Token::Plus,
            Some('-') => Token::Minus,
            Some(':') => Token::Colon,
            Some('/') => Token::Slash,
            Some(' ') => self.whitespace(),
            Some('0'..='9') => self.number(),
            Some('a'..='z') | Some('A'..='Z') => self.ident(),
            None => Token::Eof,
            _ => Token::Illegal,
        };

        token
    }

    fn whitespace(&mut self) -> Token {
        self.s.eat_whitespace();
        self.next_token()
    }

    fn number(&mut self) -> Token {
        self.s.uneat();
        let number = self.s.eat_while(char::is_ascii_digit);
        match number.parse() {
            Ok(n) => Token::Number(n),
            Err(_) => Token::Illegal, // Number too large for i64
        }
    }

    fn ident(&mut self) -> Token {
        self.s.uneat();
        let ident = self.s.eat_while(char::is_ascii_alphabetic);
        Token::Ident(ident.to_string())
    }
}

impl<'s> Iterator for Lexer<'s> {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        Some(self.next_token())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_token_random() {
        let input = "+    -:/ 1223abcd";
        let mut lexer = Lexer::new(input);

        assert_eq!(lexer.next_token(), Token::Plus);
        assert_eq!(lexer.next_token(), Token::Minus);
        assert_eq!(lexer.next_token(), Token::Colon);
        assert_eq!(lexer.next_token(), Token::Slash);
        assert_eq!(lexer.next_token(), Token::Number(1223));
        assert_eq!(lexer.next_token(), Token::Ident("abcd".to_string()));
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_next_token_plausible() {
        let input = "today - 2hours + 1 year";
        let mut lexer = Lexer::new(input);

        assert_eq!(lexer.next_token(), Token::Ident("today".to_string()));
        assert_eq!(lexer.next_token(), Token::Minus);
        assert_eq!(lexer.next_token(), Token::Number(2));
        assert_eq!(lexer.next_token(), Token::Ident("hours".to_string()));
        assert_eq!(lexer.next_token(), Token::Plus);
        assert_eq!(lexer.next_token(), Token::Number(1));
        assert_eq!(lexer.next_token(), Token::Ident("year".to_string()));
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_next_token_plausible_2() {
        let input = "2am + 3h";
        let mut lexer = Lexer::new(input);

        assert_eq!(lexer.next_token(), Token::Number(2));
        assert_eq!(lexer.next_token(), Token::Ident("am".to_string()));
        assert_eq!(lexer.next_token(), Token::Plus);
        assert_eq!(lexer.next_token(), Token::Number(3));
        assert_eq!(lexer.next_token(), Token::Ident("h".to_string()));
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_illegal_token() {
        let mut lexer = Lexer::new("@");
        assert_eq!(lexer.next_token(), Token::Illegal);
    }

    #[test]
    fn test_number_overflow() {
        // Number larger than i64::MAX (9223372036854775807)
        let input = "99999999999999999999";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::Illegal);
    }
}
