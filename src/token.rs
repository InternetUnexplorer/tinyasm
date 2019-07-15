use crate::assemble::AddressMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Byte(u8),
    Label(String),
}

pub type Tokens = Vec<Token>;

pub trait IntoTokens {
    fn into_tokens(self) -> Result<Tokens, String>;
}

impl Token {
    pub fn as_byte(&self, address_map: &AddressMap) -> Result<u8, String> {
        match self {
            Token::Byte(byte) => Ok(*byte),
            Token::Label(label) => address_map
                .get(label.as_str())
                .copied()
                .ok_or_else(|| format!("cannot resolve label ‘{}’", label)),
        }
    }
}
