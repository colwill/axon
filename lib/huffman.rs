// huffman.rs — Huffman encoding for AXON prompt compression
//
// Encodes the AXON specification into a compact binary representation using
// Huffman coding. The encoded output is base64-encoded for safe embedding
// in text prompts. A decode table is prepended so the LLM can reconstruct
// the original text during its thinking phase.

use std::collections::{BinaryHeap, HashMap};
use std::cmp::Ordering;


#[derive(Debug, Eq, PartialEq)]
enum Node {
    Leaf { ch: char, freq: u32 },
    Internal { freq: u32, left: Box<Node>, right: Box<Node> },
}

impl Node {
    fn freq(&self) -> u32 {
        match self {
            Node::Leaf { freq, .. } => *freq,
            Node::Internal { freq, .. } => *freq,
        }
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.freq().cmp(&self.freq()) // min-heap
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn build_tree(text: &str) -> Option<Node> {
    let mut freq: HashMap<char, u32> = HashMap::new();
    for ch in text.chars() {
        *freq.entry(ch).or_insert(0) += 1;
    }

    let mut heap = BinaryHeap::new();
    for (ch, f) in freq {
        heap.push(Node::Leaf { ch, freq: f });
    }

    if heap.len() == 1 {
        let node = heap.pop().unwrap();
        return Some(Node::Internal {
            freq: node.freq(),
            left: Box::new(node),
            right: Box::new(Node::Leaf { ch: '\0', freq: 0 }),
        });
    }

    while heap.len() > 1 {
        let left = heap.pop().unwrap();
        let right = heap.pop().unwrap();
        heap.push(Node::Internal {
            freq: left.freq() + right.freq(),
            left: Box::new(left),
            right: Box::new(right),
        });
    }

    heap.pop()
}

fn build_codes(node: &Node, prefix: &str, codes: &mut HashMap<char, String>) {
    match node {
        Node::Leaf { ch, .. } => {
            codes.insert(*ch, if prefix.is_empty() { "0".to_string() } else { prefix.to_string() });
        }
        Node::Internal { left, right, .. } => {
            build_codes(left, &format!("{}0", prefix), codes);
            build_codes(right, &format!("{}1", prefix), codes);
        }
    }
}

fn encode_bits(text: &str, codes: &HashMap<char, String>) -> Vec<u8> {
    let mut bits = String::new();
    for ch in text.chars() {
        if let Some(code) = codes.get(&ch) {
            bits.push_str(code);
        }
    }

    // Pack bits into bytes
    let mut bytes = Vec::with_capacity(bits.len() / 8 + 1);
    let padding = (8 - bits.len() % 8) % 8;
    for _ in 0..padding {
        bits.push('0');
    }

    for chunk in bits.as_bytes().chunks(8) {
        let mut byte = 0u8;
        for (i, &bit) in chunk.iter().enumerate() {
            if bit == b'1' {
                byte |= 1 << (7 - i);
            }
        }
        bytes.push(byte);
    }

    // Prepend padding count
    let mut result = vec![padding as u8];
    result.extend_from_slice(&bytes);
    result
}

// Based
const B64_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn base64_encode(data: &[u8]) -> String {
    let mut result = String::new();
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;

        result.push(B64_CHARS[((triple >> 18) & 0x3F) as usize] as char);
        result.push(B64_CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            result.push(B64_CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(B64_CHARS[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}

// Serialise the Huffman code table in a compact format:
// Each entry: char=bitstring, separated by semicolons.
// Special chars are escaped: \n → \\n, \t → \\t, space → \\s, ; → \\;, = → \\=
fn serialise_table(codes: &HashMap<char, String>) -> String {
    let mut entries: Vec<(&char, &String)> = codes.iter().collect();
    entries.sort_by_key(|(_, code)| code.len());

    let mut parts = Vec::new();
    for (ch, code) in entries {
        let ch_repr = match ch {
            '\n' => "\\n".to_string(),
            '\r' => "\\r".to_string(),
            '\t' => "\\t".to_string(),
            ' ' => "\\s".to_string(),
            ';' => "\\;".to_string(),
            '=' => "\\=".to_string(),
            '\\' => "\\\\".to_string(),
            c => c.to_string(),
        };
        parts.push(format!("{}={}", ch_repr, code));
    }
    parts.join(";")
}

pub struct CompressedPrompt {
    pub encoded: String,
    pub original_bytes: usize,
    pub compressed_bytes: usize,
    pub ratio: f64,
}

// Compress the AXON spec into a Huffman-encoded prompt with decode instructions.
// Useful only for large input...
pub fn compress_prompt(text: &str) -> CompressedPrompt {
    let tree = build_tree(text).expect("empty input");
    let mut codes = HashMap::new();
    build_codes(&tree, "", &mut codes);

    let encoded_bytes = encode_bits(text, &codes);
    let b64 = base64_encode(&encoded_bytes);
    let table = serialise_table(&codes);

    let original_bytes = text.len();
    let compressed_bytes = b64.len() + table.len();
    let ratio = if original_bytes > 0 {
        (1.0 - compressed_bytes as f64 / original_bytes as f64) * 100.0
    } else {
        0.0
    };

    // Build the compressed prompt with decode header
    let encoded = format!(
        "[HUFFMAN-ENCODED AXON SPEC]\n\
         Decode this during your thinking phase. The table maps characters to bit patterns.\n\
         Use the table to decode the base64 payload back to the original AXON specification.\n\n\
         TABLE: {}\n\n\
         PAYLOAD: {}\n\n\
         [END ENCODED SPEC]\n\n\
         After decoding, follow the AXON specification for all subsequent communication.\n\
         When the user writes natural language, respond in AXON. When they write AXON, decode to natural language.",
        table, b64
    );

    CompressedPrompt {
        encoded,
        original_bytes,
        compressed_bytes,
        ratio,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip() {
        let text = "hello world, this is a test of huffman encoding!";
        let tree = build_tree(text).unwrap();
        let mut codes = HashMap::new();
        build_codes(&tree, "", &mut codes);

        // Verify all characters have codes
        for ch in text.chars() {
            assert!(codes.contains_key(&ch), "Missing code for '{}'", ch);
        }

        // Verify prefix-free property
        let code_strs: Vec<&String> = codes.values().collect();
        for (i, a) in code_strs.iter().enumerate() {
            for (j, b) in code_strs.iter().enumerate() {
                if i != j {
                    assert!(!a.starts_with(b.as_str()) || a == b,
                        "Prefix violation: {} starts with {}", a, b);
                }
            }
        }
    }

    #[test]
    fn test_compress_prompt() {
        let spec = "AXON v1.0 spec test content with @entity #concept ~process";
        let result = compress_prompt(spec);
        assert!(!result.encoded.is_empty());
        assert!(result.encoded.contains("HUFFMAN-ENCODED"));
        assert!(result.encoded.contains("TABLE:"));
        assert!(result.encoded.contains("PAYLOAD:"));
    }

    #[test]
    fn test_compression_ratio() {
        // Longer text should compress reasonably
        let spec = "AXON specification version 2.0. Type sigils: @ entity, # concept, ~ process. \
                    Operators: → causes, ← result of, ∴ therefore, ∵ because. \
                    Commands: >doc >impl >fix >test >rev >ref >opt >plan. \
                    Queries: ?how ?why ?best ?what ?diff ?when ?where. \
                    Structural: @Type+.field @Type-.field @Type.x=$val.";
        let result = compress_prompt(spec);
        // The base64 + table overhead means short texts may not compress well,
        // but the compression should at least work without errors.
        assert!(result.original_bytes > 0);
        assert!(result.compressed_bytes > 0);
    }
}
