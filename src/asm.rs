use std::convert::TryFrom;

use crate::token::{IntoTokens, Token, Tokens};

#[derive(Clone, Debug)]
pub struct Label(pub String, pub Vec<Op>);

#[derive(Clone, Debug, PartialEq)]
pub struct Op(pub String, pub Vec<OpArg>);

#[derive(Clone, Debug, PartialEq)]
pub enum OpArg {
    Ident(String),
    Number(usize),
    String(String),
}

impl IntoTokens for Label {
    fn into_tokens(self) -> Result<Tokens, String> {
        let ops = self.1;

        let op_tokens = ops
            .into_iter()
            .map(|arg| arg.into_tokens())
            .collect::<Result<Vec<_>, _>>()?;

        Ok(op_tokens.into_iter().flatten().collect())
    }
}

impl IntoTokens for Op {
    fn into_tokens(self) -> Result<Tokens, String> {
        let (name, args) = (self.0, self.1);

        let (op_bytes, num_args) = match name.clone().as_str() {
            "halt" => Ok((vec![0x00], 0)),
            "load" => Ok((vec![0x10], 1)),
            "lload" => Ok((vec![0x11], 1)),
            "store" => Ok((vec![0x20], 1)),
            "add" => Ok((vec![0x30], 1)),
            "sub" => Ok((vec![0x40], 1)),
            "jmp" => Ok((vec![0x50], 1)),
            "cjmp" => Ok((vec![0x51], 1)),
            "icjmp" => Ok((vec![0x52], 1)),
            "read" => Ok((vec![0x70], 1)),
            "rread" => Ok((vec![0x71], 0)),
            "write" => Ok((vec![0x78], 1)),
            "rwrite" => Ok((vec![0x79], 0)),
            ".data" | ".static" => Ok((vec![], 1)),
            _ => Err(format!("cannot resolve opcode ‘{}’", name)),
        }?;

        if args.len() != num_args {
            Err(format!(
                "wrong number of arguments for ‘{}’ (expected {}, got {})",
                name,
                num_args,
                args.len()
            ))?;
        }

        let arg_tokens = args
            .into_iter()
            .map(|arg| arg.into_tokens())
            .collect::<Result<Vec<_>, _>>()?;

        Ok(op_bytes
            .into_iter()
            .map(Token::Byte)
            .chain(arg_tokens.into_iter().flatten())
            .collect())
    }
}

impl IntoTokens for OpArg {
    fn into_tokens(self) -> Result<Tokens, String> {
        match self {
            OpArg::Ident(s) => Ok(vec![Token::Label(s)]),
            OpArg::Number(n) => u8::try_from(n)
                .map(|byte| vec![Token::Byte(byte)])
                .map_err(|_| format!("integers >1 byte are currently not supported ({} > 255)", n)),
            OpArg::String(s) => {
                let mut bytes = s.into_bytes();
                bytes.push(0u8);
                Ok(bytes.into_iter().map(Token::Byte).collect())
            }
        }
    }
}
