/// Comparison tests between AXON notation and the Caveman prompt-compression
/// benchmark dataset (https://github.com/JuliusBrussee/caveman).
///
/// The dataset contains 10 technical prompts with responses from 6 arms:
///   __baseline__  — unmodified prompt
///   __terse__     — "Answer concisely." prefix
///   caveman       — caveman English
///   caveman-cn    — caveman Chinese
///   caveman-es    — caveman Spanish
///   compress      — generic compression prompt
///
/// These tests translate the same prompts through AXON and compare:
///   1. Prompt-side token savings (AXON vs original)
///   2. Response-side token counts across all arms
///   3. AXON translation of response text (thinking/output compression)

use axon::estimate_tokens;
use axon::translator::Translator;
use serde_json::Value;
use std::collections::BTreeMap;

const FIXTURE: &str = include_str!("fixtures/caveman_results.json");

fn load_fixture() -> Value {
    serde_json::from_str(FIXTURE).expect("failed to parse caveman_results.json")
}

fn prompts(data: &Value) -> Vec<&str> {
    data["prompts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect()
}

fn arm_responses<'a>(data: &'a Value, arm: &str) -> Vec<&'a str> {
    data["arms"][arm]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect()
}

fn arm_names(data: &Value) -> Vec<String> {
    data["arms"]
        .as_object()
        .unwrap()
        .keys()
        .cloned()
        .collect()
}

#[test]
fn test_all_caveman_prompts_translate() {
    let t = Translator::new();
    let data = load_fixture();
    let prompts = prompts(&data);

    for (i, prompt) in prompts.iter().enumerate() {
        let result = t.translate(prompt);
        assert!(
            !result.axon.is_empty(),
            "Prompt #{} produced empty AXON: {:?}",
            i,
            prompt
        );
        assert!(
            result.axon != *prompt,
            "Prompt #{} was not transformed: {:?}",
            i,
            prompt
        );
    }
}

#[test]
fn test_axon_prompt_token_savings() {
    let t = Translator::new();
    let data = load_fixture();
    let prompts = prompts(&data);

    let mut total_input = 0usize;
    let mut total_axon = 0usize;

    eprintln!("\n=== AXON Prompt Token Savings ===");
    eprintln!(
        "{:<70} {:>6} {:>6} {:>7}",
        "Prompt", "Input", "AXON", "Saving"
    );
    eprintln!("{}", "-".repeat(93));

    for prompt in &prompts {
        let result = t.translate(prompt);
        let input_tok = estimate_tokens(prompt);
        let axon_tok = estimate_tokens(&result.axon);
        total_input += input_tok;
        total_axon += axon_tok;

        let saving_pct = if input_tok > 0 {
            ((input_tok as f64 - axon_tok as f64) / input_tok as f64) * 100.0
        } else {
            0.0
        };

        let display: String = prompt.chars().take(67).collect();
        eprintln!(
            "{:<70} {:>6} {:>6} {:>6.1}%",
            display, input_tok, axon_tok, saving_pct
        );
    }

    let avg_savings = if total_input > 0 {
        ((total_input as f64 - total_axon as f64) / total_input as f64) * 100.0
    } else {
        0.0
    };

    eprintln!("{}", "-".repeat(93));
    eprintln!(
        "{:<70} {:>6} {:>6} {:>6.1}%",
        "TOTAL", total_input, total_axon, avg_savings
    );
    eprintln!();

    // AXON should achieve meaningful savings on these prompts
    assert!(
        avg_savings > 30.0,
        "Expected >30% average prompt savings, got {:.1}%",
        avg_savings
    );
}

#[test]
fn test_response_token_counts_across_arms() {
    let data = load_fixture();
    let prompts = prompts(&data);
    let arms = arm_names(&data);

    // arm_name -> total tokens across all 10 responses
    let mut arm_totals: BTreeMap<String, usize> = BTreeMap::new();

    eprintln!("\n=== Response Token Counts by Arm ===");
    eprintln!("{:<12} {}", "Arm", "Total Response Tokens (BPE est.)");
    eprintln!("{}", "-".repeat(50));

    for arm in &arms {
        let responses = arm_responses(&data, arm);
        assert_eq!(
            responses.len(),
            prompts.len(),
            "Arm {} has {} responses, expected {}",
            arm,
            responses.len(),
            prompts.len()
        );
        let total: usize = responses.iter().map(|r| estimate_tokens(r)).sum();
        arm_totals.insert(arm.clone(), total);
        eprintln!("{:<12} {:>6}", arm, total);
    }

    let baseline_total = *arm_totals.get("__baseline__").unwrap();

    eprintln!("{}", "-".repeat(50));
    eprintln!("\nResponse savings vs baseline:");
    for (arm, total) in &arm_totals {
        if arm == "__baseline__" {
            continue;
        }
        let saving = if baseline_total > 0 {
            ((baseline_total as f64 - *total as f64) / baseline_total as f64) * 100.0
        } else {
            0.0
        };
        eprintln!("  {:<12} {:>6.1}%", arm, saving);
    }
    eprintln!();

    // Caveman responses should be shorter than baseline
    let caveman_total = *arm_totals.get("caveman").unwrap();
    assert!(
        caveman_total < baseline_total,
        "Expected caveman responses ({}) to use fewer tokens than baseline ({})",
        caveman_total,
        baseline_total
    );
}

#[test]
fn test_per_prompt_response_comparison() {
    let data = load_fixture();
    let prompts = prompts(&data);
    let arms = arm_names(&data);

    eprintln!("\n=== Per-Prompt Response Token Comparison ===");

    for (i, prompt) in prompts.iter().enumerate() {
        let display: String = prompt.chars().take(60).collect();
        eprintln!("\n[{}] {}", i, display);

        for arm in &arms {
            let responses = arm_responses(&data, arm);
            let tokens = estimate_tokens(responses[i]);
            eprintln!("  {:<14} {:>5} tokens", arm, tokens);
        }
    }
    eprintln!();
}

#[test]
fn test_axon_response_compression() {
    let t = Translator::new();
    let data = load_fixture();

    // Translate each arm's responses through AXON and measure savings
    let arms_to_test = ["__baseline__", "caveman", "compress"];

    eprintln!("\n=== AXON Compression of Response Text ===");
    eprintln!(
        "{:<14} {:>10} {:>10} {:>8}",
        "Arm", "Orig Tok", "AXON Tok", "Saving"
    );
    eprintln!("{}", "-".repeat(46));

    for arm in &arms_to_test {
        let responses = arm_responses(&data, arm);
        let mut total_orig = 0usize;
        let mut total_axon = 0usize;

        for response in &responses {
            let orig_tok = estimate_tokens(response);
            // Translate each sentence/line through AXON
            let axon_tok: usize = response
                .lines()
                .filter(|l| !l.trim().is_empty())
                .map(|line| {
                    let result = t.translate(line.trim());
                    estimate_tokens(&result.axon)
                })
                .sum();
            total_orig += orig_tok;
            total_axon += axon_tok;
        }

        let saving = if total_orig > 0 {
            ((total_orig as f64 - total_axon as f64) / total_orig as f64) * 100.0
        } else {
            0.0
        };

        eprintln!(
            "{:<14} {:>10} {:>10} {:>7.1}%",
            arm, total_orig, total_axon, saving
        );
    }
    eprintln!();
}

#[test]
fn test_axon_thinking_output_per_prompt() {
    let t = Translator::new();
    let data = load_fixture();
    let all_prompts = prompts(&data);

    eprintln!("\n=== AXON Thinking Output: Prompt + Response Compression ===");
    eprintln!(
        "{:<4} {:<14} {:>8} {:>8} {:>8} {:>8} {:>8}",
        "#", "Arm", "P_orig", "P_axon", "R_orig", "R_axon", "Total%"
    );
    eprintln!("{}", "-".repeat(72));

    let arms_to_test = ["__baseline__", "__terse__", "caveman", "compress"];

    for (i, prompt) in all_prompts.iter().enumerate() {
        let prompt_orig_tok = estimate_tokens(prompt);
        let prompt_result = t.translate(prompt);
        let prompt_axon_tok = estimate_tokens(&prompt_result.axon);

        for arm in &arms_to_test {
            let responses = arm_responses(&data, arm);
            let response = responses[i];

            let resp_orig_tok = estimate_tokens(response);
            let resp_axon_tok: usize = response
                .lines()
                .filter(|l| !l.trim().is_empty())
                .map(|line| {
                    let r = t.translate(line.trim());
                    estimate_tokens(&r.axon)
                })
                .sum();

            let total_orig = prompt_orig_tok + resp_orig_tok;
            let total_axon = prompt_axon_tok + resp_axon_tok;
            let saving = if total_orig > 0 {
                ((total_orig as f64 - total_axon as f64) / total_orig as f64) * 100.0
            } else {
                0.0
            };

            eprintln!(
                "{:<4} {:<14} {:>8} {:>8} {:>8} {:>8} {:>7.1}%",
                i, arm, prompt_orig_tok, prompt_axon_tok, resp_orig_tok, resp_axon_tok, saving
            );
        }
        eprintln!();
    }
}

#[test]
fn test_axon_vs_caveman_efficiency() {
    let t = Translator::new();
    let data = load_fixture();
    let prompts = prompts(&data);

    let baseline_responses = arm_responses(&data, "__baseline__");
    let caveman_responses = arm_responses(&data, "caveman");
    let terse_responses = arm_responses(&data, "__terse__");
    let compress_responses = arm_responses(&data, "compress");

    let mut axon_prompt_savings_total = 0i64;
    let mut caveman_resp_savings_total = 0i64;
    let mut terse_resp_savings_total = 0i64;
    let mut compress_resp_savings_total = 0i64;
    let mut baseline_prompt_total = 0usize;
    let mut baseline_resp_total = 0usize;

    eprintln!("\n=== AXON vs Caveman: Efficiency Comparison ===");
    eprintln!(
        "{:<4} {:>10} {:>10} | {:>10} {:>10} {:>10}",
        "#", "P_saved", "AXON_P%", "Cave_R%", "Terse_R%", "Compr_R%"
    );
    eprintln!("{}", "-".repeat(62));

    for (i, prompt) in prompts.iter().enumerate() {
        let p_orig = estimate_tokens(prompt);
        let p_axon = estimate_tokens(&t.translate(prompt).axon);
        let p_saved = p_orig as i64 - p_axon as i64;
        baseline_prompt_total += p_orig;
        axon_prompt_savings_total += p_saved;

        let r_baseline = estimate_tokens(baseline_responses[i]);
        let r_caveman = estimate_tokens(caveman_responses[i]);
        let r_terse = estimate_tokens(terse_responses[i]);
        let r_compress = estimate_tokens(compress_responses[i]);
        baseline_resp_total += r_baseline;

        caveman_resp_savings_total += r_baseline as i64 - r_caveman as i64;
        terse_resp_savings_total += r_baseline as i64 - r_terse as i64;
        compress_resp_savings_total += r_baseline as i64 - r_compress as i64;

        let axon_p_pct = if p_orig > 0 {
            (p_saved as f64 / p_orig as f64) * 100.0
        } else {
            0.0
        };
        let cave_r_pct = if r_baseline > 0 {
            ((r_baseline as f64 - r_caveman as f64) / r_baseline as f64) * 100.0
        } else {
            0.0
        };
        let terse_r_pct = if r_baseline > 0 {
            ((r_baseline as f64 - r_terse as f64) / r_baseline as f64) * 100.0
        } else {
            0.0
        };
        let compr_r_pct = if r_baseline > 0 {
            ((r_baseline as f64 - r_compress as f64) / r_baseline as f64) * 100.0
        } else {
            0.0
        };

        eprintln!(
            "{:<4} {:>10} {:>9.1}% | {:>9.1}% {:>9.1}% {:>9.1}%",
            i, p_saved, axon_p_pct, cave_r_pct, terse_r_pct, compr_r_pct
        );
    }

    let axon_avg = if baseline_prompt_total > 0 {
        (axon_prompt_savings_total as f64 / baseline_prompt_total as f64) * 100.0
    } else {
        0.0
    };
    let cave_avg = if baseline_resp_total > 0 {
        (caveman_resp_savings_total as f64 / baseline_resp_total as f64) * 100.0
    } else {
        0.0
    };
    let terse_avg = if baseline_resp_total > 0 {
        (terse_resp_savings_total as f64 / baseline_resp_total as f64) * 100.0
    } else {
        0.0
    };
    let compr_avg = if baseline_resp_total > 0 {
        (compress_resp_savings_total as f64 / baseline_resp_total as f64) * 100.0
    } else {
        0.0
    };

    eprintln!("{}", "-".repeat(62));
    eprintln!(
        "AVG  {:>10} {:>9.1}% | {:>9.1}% {:>9.1}% {:>9.1}%",
        "", axon_avg, cave_avg, terse_avg, compr_avg
    );
    eprintln!();
    eprintln!("AXON saves tokens on the PROMPT side (input cost).");
    eprintln!("Caveman/terse/compress save tokens on the RESPONSE side (output cost).");
    eprintln!("These are complementary — AXON prompts could be combined with any response style.");
    eprintln!();
}

#[test]
fn test_axon_translation_samples() {
    let t = Translator::new();
    let data = load_fixture();
    let prompts = prompts(&data);

    eprintln!("\n=== AXON Translation Samples ===");
    for (i, prompt) in prompts.iter().enumerate() {
        let result = t.translate(prompt);
        let input_tok = estimate_tokens(prompt);
        let axon_tok = estimate_tokens(&result.axon);
        let saving = if input_tok > 0 {
            ((input_tok as f64 - axon_tok as f64) / input_tok as f64) * 100.0
        } else {
            0.0
        };

        eprintln!("[{}] Original:   {}", i, prompt);
        eprintln!("    AXON:       {}", result.axon);
        eprintln!("    Annotation: {}", result.annotation);
        eprintln!(
            "    Tokens:     {} -> {} ({:.1}% saving)",
            input_tok, axon_tok, saving
        );
        eprintln!();
    }
}

#[test]
fn test_total_token_cost_comparison() {
    let t = Translator::new();
    let data = load_fixture();
    let prompts = prompts(&data);
    let arms = ["__baseline__", "__terse__", "caveman", "compress"];

    eprintln!("\n=== Total Token Cost: Prompt + Response ===");
    eprintln!(
        "{:<14} {:>12} {:>12} {:>12} {:>8}",
        "Arm", "Prompt Tok", "Resp Tok", "Total", "vs Base"
    );
    eprintln!("{}", "-".repeat(62));

    let baseline_total;

    // First: baseline with original prompts
    {
        let responses = arm_responses(&data, "__baseline__");
        let p_total: usize = prompts.iter().map(|p| estimate_tokens(p)).sum();
        let r_total: usize = responses.iter().map(|r| estimate_tokens(r)).sum();
        baseline_total = p_total + r_total;
        eprintln!(
            "{:<14} {:>12} {:>12} {:>12} {:>8}",
            "baseline", p_total, r_total, baseline_total, "—"
        );
    }

    // Other arms with original prompts
    for arm in &arms[1..] {
        let responses = arm_responses(&data, arm);
        let p_total: usize = prompts.iter().map(|p| estimate_tokens(p)).sum();
        let r_total: usize = responses.iter().map(|r| estimate_tokens(r)).sum();
        let total = p_total + r_total;
        let vs_base = if baseline_total > 0 {
            ((baseline_total as f64 - total as f64) / baseline_total as f64) * 100.0
        } else {
            0.0
        };
        eprintln!(
            "{:<14} {:>12} {:>12} {:>12} {:>7.1}%",
            arm, p_total, r_total, total, vs_base
        );
    }

    // AXON: translated prompts + baseline responses (AXON only compresses the prompt)
    {
        let responses = arm_responses(&data, "__baseline__");
        let p_total: usize = prompts
            .iter()
            .map(|p| estimate_tokens(&t.translate(p).axon))
            .sum();
        let r_total: usize = responses.iter().map(|r| estimate_tokens(r)).sum();
        let total = p_total + r_total;
        let vs_base = if baseline_total > 0 {
            ((baseline_total as f64 - total as f64) / baseline_total as f64) * 100.0
        } else {
            0.0
        };
        eprintln!(
            "{:<14} {:>12} {:>12} {:>12} {:>7.1}%",
            "AXON+base", p_total, r_total, total, vs_base
        );
    }

    // AXON prompts + caveman responses (complementary combination)
    {
        let responses = arm_responses(&data, "caveman");
        let p_total: usize = prompts
            .iter()
            .map(|p| estimate_tokens(&t.translate(p).axon))
            .sum();
        let r_total: usize = responses.iter().map(|r| estimate_tokens(r)).sum();
        let total = p_total + r_total;
        let vs_base = if baseline_total > 0 {
            ((baseline_total as f64 - total as f64) / baseline_total as f64) * 100.0
        } else {
            0.0
        };
        eprintln!(
            "{:<14} {:>12} {:>12} {:>12} {:>7.1}%",
            "AXON+caveman", p_total, r_total, total, vs_base
        );
    }

    eprintln!();
}
