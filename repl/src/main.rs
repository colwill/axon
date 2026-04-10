mod decoder;
mod protocol;
use std::io::{self, BufRead, IsTerminal, Write};
use axon::huffman::compress_prompt;
use axon::translator::Translator;
use axon::estimate_tokens;
use decoder::Decoder;
use protocol::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let json_mode = args.contains(&"--json".to_string()) || !io::stdin().is_terminal();

    let translator = Translator::new();
    let decoder = Decoder::new();

    if json_mode {
        run_json_loop(&translator, &decoder);
    } else {
        run_interactive_loop(&translator, &decoder);
    }
}

fn run_interactive_loop(translator: &Translator, decoder: &Decoder) {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    println!("AXON v1.0 REPL");
    println!("Commands: encode <text>, decode <axon>, tokens <text>, compress <text>, help, quit");
    println!("Or just type text — auto-detects AXON vs plain language.\n");

    loop {
        print!("axon> ");
        stdout.flush().unwrap();

        let mut line = String::new();
        match stdin.lock().read_line(&mut line) {
            Ok(0) => break, // EOF
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                break;
            }
        }

        let input = line.trim();
        if input.is_empty() {
            continue;
        }

        match input {
            "quit" | "exit" => break,
            "help" => print_help(),
            _ => handle_interactive_command(input, translator, decoder),
        }
    }
}

fn handle_interactive_command(input: &str, translator: &Translator, decoder: &Decoder) {
    // Check for explicit command prefixes
    if let Some(text) = input.strip_prefix("encode ") {
        print_encode(text.trim(), translator);
    } else if let Some(axon) = input.strip_prefix("decode ") {
        print_decode(axon.trim(), decoder);
    } else if let Some(text) = input.strip_prefix("tokens ") {
        let count = estimate_tokens(text.trim());
        println!("  tokens: {}\n", count);
    } else if let Some(text) = input.strip_prefix("compress ") {
        print_compress(text.trim());
    } else if looks_like_axon(input) {
        // Auto-detect: decode AXON
        print_decode(input, decoder);
    } else {
        // Auto-detect: encode plain text
        print_encode(input, translator);
    }
}

fn print_encode(text: &str, translator: &Translator) {
    let result = translator.translate(text);
    let inp_t = estimate_tokens(text);
    let axon_t = estimate_tokens(&result.axon);
    let savings = if inp_t > 0 {
        inp_t.saturating_sub(axon_t) * 100 / inp_t
    } else {
        0
    };

    println!("  axon:       {}", result.axon);
    println!("  annotation: {}", result.annotation);
    println!("  tokens:     {} -> {} ({}% savings)\n", inp_t, axon_t, savings);
}

fn print_decode(axon: &str, decoder: &Decoder) {
    let text = decoder.decode(axon);
    println!("  text: {}\n", text);
}

fn print_compress(text: &str) {
    let result = compress_prompt(text);
    println!("  encoded:    {}...", &result.encoded[..result.encoded.len().min(80)]);
    println!("  bytes:      {} -> {} ({:.1}% reduction)\n",
        result.original_bytes, result.compressed_bytes, result.ratio);
}

fn print_help() {
    println!("
AXON REPL (AXON v1.0 Spec) — Commands:

  encode <text>     Translate plain text to AXON notation
  decode <axon>     Translate AXON notation back to plain text
  tokens <text>     Estimate BPE token count
  compress <text>   Huffman-compress text for smaller context
  help              Show this help
  quit / exit       Exit the REPL

Auto-detection: type without a command prefix and the REPL will
detect whether your input is AXON (decode) or plain text (encode).
");
}

// does this input look like AXON notation??
fn looks_like_axon(input: &str) -> bool {
    let tokens: Vec<&str> = input.split_whitespace().collect();
    if tokens.is_empty() {
        return false;
    }

    let sigil_count = tokens
        .iter()
        .filter(|t| {
            t.starts_with('@')
                || t.starts_with('#')
                || t.starts_with('~')
                || t.starts_with('>')
                || t.starts_with('$')
                || t.starts_with('^')
        })
        .count();

    let has_operators = input.contains(" -> ")
        || input.contains(" <- ")
        || input.contains(" :. ")
        || input.contains(" && ")
        || input.contains(" || ");

    // If more than 30% of tokens have sigils, or operators are present then yeah probably
    sigil_count * 3 > tokens.len() || has_operators
}

fn run_json_loop(translator: &Translator, decoder: &Decoder) {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        let line = line.trim().to_string();
        if line.is_empty() {
            continue;
        }

        let req: Request = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                let resp = ErrorResponse::new(&format!("invalid JSON: {}", e));
                let _ = writeln!(stdout, "{}", serde_json::to_string(&resp).unwrap());
                continue;
            }
        };

        let output = match req.action.as_str() {
            "encode" => handle_json_encode(&req.text, translator),
            "decode" => handle_json_decode(&req.axon, decoder),
            "tokens" => handle_json_tokens(&req.text),
            "compress" => handle_json_compress(&req.text),
            other => serde_json::to_string(&ErrorResponse::new(
                &format!("unknown action: {}", other),
            ))
            .unwrap(),
        };

        let _ = writeln!(stdout, "{}", output);
        let _ = stdout.flush();
    }
}

fn handle_json_encode(text: &str, translator: &Translator) -> String {
    let result = translator.translate(text);
    let inp_t = estimate_tokens(text);
    let axon_t = estimate_tokens(&result.axon);
    let savings = if inp_t > 0 {
        inp_t.saturating_sub(axon_t) * 100 / inp_t
    } else {
        0
    };

    serde_json::to_string(&EncodeResponse {
        ok: true,
        axon: result.axon,
        annotation: result.annotation,
        input_tokens: inp_t,
        axon_tokens: axon_t,
        savings_pct: savings,
    })
    .unwrap()
}

fn handle_json_decode(axon: &str, decoder: &Decoder) -> String {
    let text = decoder.decode(axon);
    serde_json::to_string(&DecodeResponse { ok: true, text }).unwrap()
}

fn handle_json_tokens(text: &str) -> String {
    let tokens = estimate_tokens(text);
    serde_json::to_string(&TokensResponse { ok: true, tokens }).unwrap()
}

fn handle_json_compress(text: &str) -> String {
    let result = compress_prompt(text);
    serde_json::to_string(&CompressResponse {
        ok: true,
        encoded: result.encoded,
        original_bytes: result.original_bytes,
        compressed_bytes: result.compressed_bytes,
        ratio: result.ratio,
    })
    .unwrap()
}
