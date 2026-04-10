#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

pub mod code_translator;
pub mod huffman;
pub mod translator;

#[cfg(feature = "wasm")]
use translator::Translator;

/// Estimate BPE token count by treating each whitespace-separated word as at
/// least one token and counting non-alphanumeric characters as additional
/// tokens (BPE tokenizers almost always split on symbol boundaries).
pub fn estimate_tokens(text: &str) -> usize {
    let mut count = 0usize;
    for word in text.split_whitespace() {
        count += 1;
        for ch in word.chars() {
            if !ch.is_alphanumeric() {
                count += 1;
            }
        }
    }
    count.max(1)
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct AxonTranslator {
    inner: Translator,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct TranslationResult {
    axon: String,
    annotation: String,
    input_tokens: usize,
    axon_tokens: usize,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl TranslationResult {
    #[wasm_bindgen(getter)]
    pub fn axon(&self) -> String {
        self.axon.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn annotation(&self) -> String {
        self.annotation.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn savings(&self) -> usize {
        if self.input_tokens == 0 {
            return 0;
        }
        let saved = self.input_tokens.saturating_sub(self.axon_tokens);
        (saved * 100 / self.input_tokens).min(95)
    }
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl AxonTranslator {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Translator::new(),
        }
    }

    pub fn translate(&self, input: &str) -> TranslationResult {
        let result = self.inner.translate(input);
        let input_tokens = estimate_tokens(input);
        let axon_tokens = estimate_tokens(&result.axon);
        TranslationResult {
            axon: result.axon,
            annotation: result.annotation,
            input_tokens,
            axon_tokens,
        }
    }

    /// Compress text using Huffman encoding for smaller context usage.
    pub fn compress(&self, text: &str) -> CompressionResult {
        let result = huffman::compress_prompt(text);
        CompressionResult {
            encoded: result.encoded,
            original_bytes: result.original_bytes,
            compressed_bytes: result.compressed_bytes,
            ratio: result.ratio,
        }
    }
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct CompressionResult {
    encoded: String,
    original_bytes: usize,
    compressed_bytes: usize,
    ratio: f64,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl CompressionResult {
    #[wasm_bindgen(getter)]
    pub fn encoded(&self) -> String {
        self.encoded.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn original_bytes(&self) -> usize {
        self.original_bytes
    }

    #[wasm_bindgen(getter)]
    pub fn compressed_bytes(&self) -> usize {
        self.compressed_bytes
    }

    #[wasm_bindgen(getter)]
    pub fn ratio(&self) -> f64 {
        self.ratio
    }
}
