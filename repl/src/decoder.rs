// AXON v1.0 -> plain text decoder.
//
// Best-effort reverse translation: sigil stripping, operator expansion,
// abbreviation reversal, temporal marker expansion. The encoding is lossy
// (articles, stop words, and noise phrases are stripped), so the decoder
// produces a readable paraphrase, not the original verbatim text.

use std::collections::HashMap;

use axon::translator::ABBREVIATIONS;

// Standalone operator tokens and their prose expansions.
const OPERATORS: &[(&str, &str)] = &[
    ("->", "causes"),
    ("<-", "is caused by"),
    (":.", "therefore"),
    ("&&", "and"),
    ("||", "or"),
];

pub struct Decoder {
    // Reverse abbreviation map: "obj" -> "object", "fn" -> "function", etc.
    abbrev_reverse: HashMap<String, String>,
}

impl Decoder {
    pub fn new() -> Self {
        let mut abbrev_reverse = HashMap::new();
        for &(full, abbr) in ABBREVIATIONS {
            abbrev_reverse.insert(abbr.to_string(), full.to_string());
        }
        Self { abbrev_reverse }
    }

    // Decode an AXON expression into plain text.
    pub fn decode(&self, axon: &str) -> String {
        let tokens: Vec<&str> = axon.split_whitespace().collect();
        if tokens.is_empty() {
            return String::new();
        }

        let mut words: Vec<String> = Vec::new();

        for token in &tokens {
            let expanded = self.expand_token(token);
            words.extend(expanded);
        }

        if words.is_empty() {
            return axon.to_string();
        }

        // Capitalize first word and add period
        if let Some(first) = words.first_mut() {
            let mut chars = first.chars();
            if let Some(c) = chars.next() {
                *first = c.to_uppercase().to_string() + chars.as_str();
            }
        }

        let mut sentence = words.join(" ");

        // Strip trailing confidence markers from sentence and append as qualifier
        let confidence = self.extract_trailing_confidence(&mut sentence);
        if !confidence.is_empty() {
            sentence = format!("{} ({})", sentence.trim(), confidence);
        }

        format!("{}.", sentence.trim_end_matches('.'))
    }

    // Expand a single AXON token into one or more plain text words.
    fn expand_token(&self, token: &str) -> Vec<String> {
        // Standalone operators
        for &(op, expansion) in OPERATORS {
            if token == op {
                return vec![expansion.to_string()];
            }
        }

        // "bc" as standalone operator
        if token == "bc" {
            return vec!["because".to_string()];
        }

        // Standalone quantifiers (rare — usually prefixed to next token)
        if token == "A." {
            return vec!["all".to_string()];
        }
        if token == "E." {
            return vec!["there exists".to_string()];
        }

        // Quantifier prefix: "A.something" or "E.something"
        if let Some(rest) = token.strip_prefix("A.") {
            let mut result = vec!["all".to_string()];
            result.extend(self.expand_body_parts(rest));
            return result;
        }
        if let Some(rest) = token.strip_prefix("E.") {
            let mut result = vec!["there exists".to_string()];
            result.extend(self.expand_body_parts(rest));
            return result;
        }

        // Temporal markers
        if let Some(temporal) = self.expand_temporal(token) {
            return vec![temporal];
        }

        // Confidence markers (standalone)
        match token {
            "!!" => return vec!["(certain)".to_string()],
            "*" => return vec!["(possibly)".to_string()],
            "**" => return vec!["(speculatively)".to_string()],
            _ => {}
        }

        // Handle scope separator ":"  e.g. ">doc login-comp:auth-service"
        // First strip any sigil, then check for ":"
        let (body, _sigil_kind) = self.strip_sigil(token);

        // Split on ":" for scope (e.g., "login-comp:auth-service" -> "login comp in auth service")
        if let Some(colon_pos) = body.find(':') {
            let subject = &body[..colon_pos];
            let scope = &body[colon_pos + 1..];
            let mut parts = self.expand_body_parts(subject);
            parts.push("in".to_string());
            parts.extend(self.expand_body_parts(scope));
            return parts;
        }

        // Expand the body: reverse abbreviations, unhyphenate
        self.expand_body_parts(body)
    }

    /// Strip the sigil prefix from a token, returning (body, sigil_kind).
    fn strip_sigil<'a>(&self, token: &'a str) -> (&'a str, &'static str) {
        if let Some(rest) = token.strip_prefix('@') {
            (rest, "entity")
        } else if let Some(rest) = token.strip_prefix('#') {
            (rest, "concept")
        } else if let Some(rest) = token.strip_prefix('~') {
            (rest, "action")
        } else if let Some(rest) = token.strip_prefix('$') {
            (rest, "scalar")
        } else if let Some(rest) = token.strip_prefix('!') {
            (rest, "negation")
        } else if let Some(rest) = token.strip_prefix('>') {
            (rest, "command")
        } else if let Some(rest) = token.strip_prefix('?') {
            (rest, "query")
        } else if let Some(rest) = token.strip_prefix('^') {
            (rest, "temporal")
        } else {
            (token, "bare")
        }
    }

    fn expand_body_parts(&self, body: &str) -> Vec<String> {
        body.split('-')
            .map(|part| {
                self.abbrev_reverse
                    .get(part)
                    .cloned()
                    .unwrap_or_else(|| part.to_string())
            })
            .collect()
    }

    fn expand_temporal(&self, token: &str) -> Option<String> {
        if token == "^now" {
            return Some("now".to_string());
        }
        if token == "^A.t" {
            return Some("always".to_string());
        }

        // ^T+Nd or ^T-Nd
        if let Some(rest) = token.strip_prefix("^T+") {
            if let Some(days) = rest.strip_suffix('d') {
                if let Ok(n) = days.parse::<u32>() {
                    return Some(match n {
                        1 => "tomorrow".to_string(),
                        7 => "next week".to_string(),
                        30 => "next month".to_string(),
                        365 => "next year".to_string(),
                        _ => format!("in {} days", n),
                    });
                }
            }
        }
        if let Some(rest) = token.strip_prefix("^T-") {
            if let Some(days) = rest.strip_suffix('d') {
                if let Ok(n) = days.parse::<u32>() {
                    return Some(match n {
                        1 => "yesterday".to_string(),
                        7 => "last week".to_string(),
                        30 => "last month".to_string(),
                        365 => "last year".to_string(),
                        _ => format!("{} days ago", n),
                    });
                }
            }
        }

        // Generic temporal — strip ^ and return
        if token.starts_with('^') {
            return Some(token[1..].to_string());
        }
        None
    }

    fn extract_trailing_confidence(&self, sentence: &mut String) -> String {
        let markers = [
            ("!!", "certain"),
            ("**", "speculative"),
            ("*", "low confidence"),
            ("~", "moderate confidence"),
        ];

        for (marker, label) in &markers {
            if sentence.ends_with(marker) {
                *sentence = sentence[..sentence.len() - marker.len()].trim().to_string();
                return label.to_string();
            }
        }
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_simple_encode() {
        let d = Decoder::new();
        let result = d.decode("@sun ~emit #uv-radiation");
        let lower = result.to_lowercase();
        assert!(lower.contains("sun"));
        assert!(lower.contains("emit"));
        assert!(lower.contains("radiation"));
    }

    #[test]
    fn test_decode_operators() {
        let d = Decoder::new();
        let result = d.decode("#gravity -> obj ~fall");
        let lower = result.to_lowercase();
        assert!(lower.contains("gravity"));
        assert!(lower.contains("causes"));
        assert!(lower.contains("fall"));
    }

    #[test]
    fn test_decode_abbreviations_reversed() {
        let d = Decoder::new();
        let result = d.decode(">doc login-comp:auth-service");
        let lower = result.to_lowercase();
        assert!(lower.contains("component"), "expected 'component' in: {}", result);
        assert!(lower.contains("auth"), "expected 'auth' in: {}", result);
    }

    #[test]
    fn test_decode_temporal() {
        let d = Decoder::new();
        let result = d.decode("@server ~fail ^T-1d");
        assert!(result.contains("yesterday"), "expected 'yesterday' in: {}", result);
    }

    #[test]
    fn test_decode_quantifier() {
        let d = Decoder::new();
        let result = d.decode("A.new-obj-ref $rnd");
        let lower = result.to_lowercase();
        assert!(result.starts_with("All"), "expected starts with 'All' in: {}", result);
        assert!(lower.contains("object"), "expected 'object' in: {}", result);
        assert!(lower.contains("render"), "expected 'render' in: {}", result);
    }

    #[test]
    fn test_decode_negation() {
        let d = Decoder::new();
        let result = d.decode("!#evidence :. !~work #treatment");
        let lower = result.to_lowercase();
        assert!(lower.contains("evidence"), "expected 'evidence' in: {}", result);
        assert!(lower.contains("therefore"), "expected 'therefore' in: {}", result);
    }

    #[test]
    fn test_decode_roundtrip_code_command() {
        let d = Decoder::new();
        let result = d.decode(">fix bug:auth-service");
        let lower = result.to_lowercase();
        assert!(lower.contains("fix"), "expected 'fix' in: {}", result);
        assert!(lower.contains("auth"), "expected 'auth' in: {}", result);
    }
}
