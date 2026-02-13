use crate::compiler::tokens::Token;

pub struct Lexer {
    pub input: String,
    pub position: usize,
}

impl Lexer {
    fn get_token(&mut self, ch: char) -> Token {
        match ch {
            '{' | '}' | '(' | ')' => Token::Punctuation(ch),
            '=' | '>' | '<' | '~' | '^' => Token::Operator(ch),
            '"' => Token::StringLiteral(self.read_string()),
            ch if ch.is_ascii_digit() => Token::IntegerLiteral(self.read_number()),
            ch if self.is_ident_start(ch) => Token::Identifier(self.read_identifier()),
            _ => Token::Error,
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens: Vec<Token> = Vec::new();

        while self.position < self.input.len() {
            let current_char = self.input.chars().nth(self.position);

            match current_char {
                Some(ch) => {
                    if ch.is_whitespace() {
                        self.advance();
                        continue;
                    }
                    let token = self.get_token(ch);
                    tokens.push(token);
                    self.advance();
                }
                None => break,
            }
        }

        tokens
    }

    fn read_identifier(&mut self) -> String {
        let mut value = String::new();

        while let Some(ch) = self.peek() {
            if self.is_ident_continue(ch) {
                value.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        value
    }

    fn read_number(&mut self) -> u32 {
        let mut num = String::new();

        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                num.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        num.parse::<u32>().unwrap()
    }

    fn read_string(&mut self) -> String {
        let mut string = String::new();

        self.advance();
        while let Some(ch) = self.peek() {
            if ch == '"' {
                break;
            } else {
                string.push(ch);
            }
            self.advance();
        }

        string
    }

    fn peek(&self) -> Option<char> {
        self.input.chars().nth(self.position)
    }

    fn advance(&mut self) {
        self.position += 1;
    }

    fn is_ident_start(&self, ch: char) -> bool {
        ch.is_ascii_alphabetic() || ch == '_'
    }

    fn is_ident_continue(&self, ch: char) -> bool {
        self.is_ident_start(ch) || ch.is_ascii_digit()
    }
}
