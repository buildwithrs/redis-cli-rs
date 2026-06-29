use crate::{errors::CliErrors};

pub mod decoder;
pub mod encoder;

pub struct CmdParser {
    pub tokens: Vec<char>,
    pub cur: usize,
}

impl CmdParser {
    pub fn new(cmd: &str) -> Self {
        Self {
            tokens: cmd.chars().collect(),
            cur: 0 as usize,
        }
    }

    pub fn parse(&mut self) -> Result<Vec<String>, CliErrors> {
        if self.tokens.len() == 0 {
            return Ok(Vec::new());
        }

        println!("parsing tokens: {:?}", self.tokens);

        let mut results = Vec::new();
        let mut chars = Vec::new();

        while !self.is_end() {
            let tk = self.tokens[self.cur];
            match tk {
                ' ' | '\r' | '\n' | '\t' => {
                    let ch = chars.clone();
                    // handle space splited string
                    if ch.len() > 0 {
                        println!("[parse] build string from: {:?}", ch);
                        results.push(String::from_iter(ch));
                        chars.clear();
                    }

                    self.cur += 1;
                    continue;
                }

                '"' => {
                    println!("reading string at: {}", self.cur);
                    results.push(self.read_string()?);
                }

                _ => {
                    // just append the tk into chars, no need check
                    println!("[parse] pushing {} to chars", tk);
                    chars.push(tk);
                    self.cur += 1;
                }
            }
        }

        // handle the remain chars
        if chars.len() > 0 {
            results.push(String::from_iter(chars));
        }

        Ok(results)
    }

    fn is_end(&self) -> bool {
        self.cur >= self.tokens.len()
    }

    fn consume_char(&mut self) -> Option<char> {
        if !self.is_end() {
            let ch = self.tokens[self.cur];
            self.cur += 1;
            return Some(ch);
        }

        None
    }

    fn read_string(&mut self) -> Result<String, CliErrors> {
        // consume '"'
        self.cur += 1;
        println!("start read string at: {}", self.cur);

        let mut chars = Vec::new();
        let mut meet_end = false;

        while !self.is_end() {
            if let Some(ch) = self.consume_char() {
                if ch == '"' {
                    println!("meet string end");
                    meet_end = true;
                    break;
                }

                println!("pushing {} to chars", ch);
                chars.push(ch);
            } else {
                break;
            }
        }

        if self.is_end() && !meet_end {
            return Err(CliErrors::UnterminatedString);
        }

        let ret = Ok(String::from_iter(chars));
        ret
    }
}

pub fn parse_cmd_to_strings(cmd: &str) -> Result<Vec<String>, CliErrors> {
    let mut parser = CmdParser::new(cmd);
    let values = parser.parse()?;
    Ok(values)
}

#[cfg(test)]
mod tests {
    use crate::codec::parse_cmd_to_strings;

    #[test]
    fn test_parse_cmds_to_strings() {
        let cmd = r#"HSET user:002 name "Bob" age 19 job SWE"#;
        let res = parse_cmd_to_strings(cmd);
        assert!(res.is_ok());
        assert_eq!(
            res.unwrap(),
            r#"HSET user:002 name Bob age 19 job SWE"#
                .split(" ")
                .map(|v| v.to_string())
                .collect::<Vec<String>>(),
        );

        
    }

    #[test]
    fn test_read_string() {
        let cmd = r#"HSET product:001 addr "US Oregon""#;
        let res = parse_cmd_to_strings(cmd);
        assert!(res.is_ok());
        assert_eq!(
            res.unwrap(),
            vec![
                "HSET".to_string(),
                "product:001".to_string(),
                "addr".to_string(),
                "US Oregon".to_string(),
            ]
        );
    }
}
