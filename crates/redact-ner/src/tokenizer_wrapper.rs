// Placeholder for tokenizer wrapper
// This will wrap the tokenizers crate and provide helpers for
// converting between token offsets and character offsets

use anyhow::Result;

pub struct TokenizerWrapper {
    // TODO: Wrap tokenizers::Tokenizer
}

impl TokenizerWrapper {
    pub fn from_file(path: &str) -> Result<Self> {
        // TODO: Load tokenizer from file
        Ok(Self {})
    }

    pub fn encode(&self, text: &str) -> Result<Encoding> {
        // TODO: Tokenize text
        Ok(Encoding {
            ids: vec![],
            tokens: vec![],
            offsets: vec![],
        })
    }
}

pub struct Encoding {
    pub ids: Vec<u32>,
    pub tokens: Vec<String>,
    pub offsets: Vec<(usize, usize)>,
}
