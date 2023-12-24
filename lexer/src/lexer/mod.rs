use crate::types::{Token, TokenKind, TokenValue};
use crate::utils::report_error;
use std::str::Chars;

pub struct Lexer<'a> {
    /// Path of the source file
    path: &'a str,

    /// Source Text
    source: &'a str,

    /// The remaining characters
    chars: Chars<'a>,
}

impl<'a> Lexer<'a> {
    pub fn new(path: &'a str, source: &'a str) -> Self {
        Self {
            path,
            source,
            chars: source.chars(),
        }
    }

    pub fn next(&mut self) -> Option<char> {
        self.chars.next()
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        loop {
            let token = self.next_token();
            if token.kind == TokenKind::Eof {
                break;
            }
            tokens.push(token);
        }

        tokens
    }

    pub fn skip_trivia(&mut self) {
        while let Some(c) = self.peek() {
            match c {
                ' ' | '\t' | '\n' | '\r' => {
                    self.next();
                }
                _ => break,
            }
        }
    }

    pub fn next_kind(&mut self) -> TokenKind {
        while let Some(c) = self.next() {
            match c {
                '+' => return self.read_one_more('=', TokenKind::PlusAssign, TokenKind::Plus),
                '-' => return self.read_one_more('=', TokenKind::MinusAssign, TokenKind::Minus),
                '*' => return self.read_one_more('=', TokenKind::MultiplyAssign, TokenKind::Star),
                '/' => return self.read_one_more('=', TokenKind::DivideAssign, TokenKind::Slash),
                '^' => return self.read_one_more('=', TokenKind::PowerAssign, TokenKind::Power),
                '%' => return self.read_one_more('=', TokenKind::ModuloAssign, TokenKind::Modulo),

                '<' => return self.read_one_more('=', TokenKind::LessEqual, TokenKind::Less),
                '>' => return self.read_one_more('=', TokenKind::GreaterEqual, TokenKind::Greater),

                '!' => return self.read_one_more('=', TokenKind::NotEqual, TokenKind::Not),

                '(' => return TokenKind::LParen,
                ')' => return TokenKind::RParen,
                '{' => return TokenKind::LBrace,
                '}' => return TokenKind::RBrace,
                '[' => return TokenKind::LBracket,
                ']' => return TokenKind::RBracket,

                ';' => return TokenKind::Semicolon,
                ',' => return TokenKind::Comma,
                ':' => return self.read_one_more('=', TokenKind::FormulaAssign, TokenKind::Colon),

                '.' => return self.read_dot(),

                '=' => return self.read_one_more('=', TokenKind::Equal, TokenKind::Assign),
                '0'..='9' => return self.read_number(),
                'a'..='z' | 'A'..='Z' | '_' => return self.read_identifier(),
                '"' | '\'' | '`' => return self.read_string(c),
                '#' => return self.read_comment(),
                _ => return TokenKind::Unexpected,
            };
        }
        TokenKind::Eof
    }

    fn read_dot(&mut self) -> TokenKind {
        if self.peek() == Some('.') {
            self.next();
            return TokenKind::Range;
        } else if ("0"..="9").contains(&self.peek().unwrap_or_default().to_string().as_str()) {
            return self.read_number();
        }
        return TokenKind::Dot;
    }

    fn read_number(&mut self) -> TokenKind {
        while let Some(c) = self.peek() {
            match c {
                '0'..='9' => {
                    self.next();
                }
                '.' | 'e' | 'E' => {
                    if let Some(c) = self.peek_two() {
                        match c {
                            '0'..='9' => {
                                self.next();
                                self.next();
                            }
                            _ => {
                                break;
                            }
                        }
                    } else {
                        break;
                    }
                }
                _ => break,
            };
        }

        TokenKind::Number
    }

    fn read_comment(&mut self) -> TokenKind {
        while let Some(c) = self.peek() {
            match c {
                '\n' => {
                    self.next();
                    break;
                }
                _ => {
                    self.next();
                }
            };
        }
        TokenKind::Comment
    }

    fn read_string(&mut self, init_char: char) -> TokenKind {
        while let Some(c) = self.peek() {
            match c {
                c if c == init_char => {
                    self.next();
                    return TokenKind::String;
                }
                '\\' => {
                    self.next();
                    self.next();
                }
                _ => {
                    self.next();
                }
            };
        }
        TokenKind::Unexpected
    }

    fn read_identifier(&mut self) -> TokenKind {
        while let Some(c) = self.peek() {
            match c {
                'a'..='z' | 'A'..='Z' | '_' | '0'..='9' => {
                    self.next();
                }
                _ => break,
            };
        }

        TokenKind::Identifier
    }

    fn read_one_more(
        &mut self,
        ch: char,
        kind_expected: TokenKind,
        kind_unexpected: TokenKind,
    ) -> TokenKind {
        match self.peek() {
            Some(c) if c == ch => {
                self.next();
                return kind_expected;
            }
            _ => return kind_unexpected,
        }
    }

    fn next_token(&mut self) -> Token {
        self.skip_trivia();
        let start = self.offset();
        let mut kind = self.next_kind();
        let end = self.offset();

        let s = self.source[start..end].trim();

        let mut value = TokenValue::None;

        match kind {
            TokenKind::Number => {
                value = TokenValue::Number(s.trim().parse::<f64>().unwrap_or_default());
            }
            TokenKind::Identifier => {
                kind = self.match_keyword(&s);

                match kind {
                    TokenKind::If | TokenKind::While | TokenKind::For => {}
                    _ => {
                        value = TokenValue::String(s.to_string());
                    }
                }
            }

            TokenKind::String => {
                value = TokenValue::String(s[1..s.len() - 1].to_string());
            }

            TokenKind::Comment => {
                value = TokenValue::String(s[1..].to_string());
            }

            TokenKind::Unexpected => {
                report_error(self.path, self.source, "Unexpected token", start, end)
            }
            _ => {}
        }

        Token {
            kind,
            start,
            end,
            value,
        }
    }

    fn match_keyword(&self, ident: &str) -> TokenKind {
        // all keywords are 1 <= length <= 10
        if ident.len() == 1 || ident.len() > 10 {
            return TokenKind::Identifier;
        }

        match ident {
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "while" => TokenKind::While,
            "loop" => TokenKind::Loop,
            "for" => TokenKind::For,
            "let" => TokenKind::Let,
            "fn" => TokenKind::Function,
            "return" => TokenKind::Return,
            "break" => TokenKind::Break,
            "continue" => TokenKind::Continue,
            "in" => TokenKind::In,

            "true" => TokenKind::True,
            "false" => TokenKind::False,

            _ => TokenKind::Identifier,
        }
    }

    /// Get the length offset from the source text, in UTF-8 bytes
    fn offset(&self) -> usize {
        self.source.len() - self.chars.as_str().len()
    }

    fn peek(&self) -> Option<char> {
        self.chars.as_str().chars().next()
    }

    fn peek_two(&self) -> Option<char> {
        let new_chars = self.chars.as_str();
        new_chars.chars().next();
        new_chars.chars().next()
    }
}