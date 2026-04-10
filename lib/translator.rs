// translator.rs — Rule-based AXON v1.0 translation engine
//
// Unified pipeline for both general NLP and programming queries:
//   0. Try programming translation (commands, queries, structural ops)
//   1. Extract confidence marker from hedge words
//   2. Extract temporal markers from time expressions
//   3. Detect negation
//   4. Detect causal / logical structure (->  :.  bc  &&  ||)
//   5. Tag each phrase token with its sigil  (@ # ~ $ !)
//   6. Abbreviate common terms
//   7. Merge consecutive bare tokens into hyphenated compounds
//   8. Assemble final AXON string + annotation
//
// Token-reduction strategies:
//   - ASCII operators instead of Unicode (-> not →, && not ∧, etc.)
//   - Abbreviation dictionary for common technical terms
//   - Conditional sigils: only prefix tokens in the known entity/concept/verb
//     databases; fallback tokens get no sigil (saves 1 BPE token each)
//   - Compound merging: consecutive unsigiled tokens hyphenate into one
use std::collections::HashSet;

use crate::code_translator::CodeTranslator;

pub struct Translation {
    pub axon: String,
    pub annotation: String,
}


// Well-known named entities → @
const KNOWN_ENTITIES: &[&str] = &[
    // Astronomy
    "sun", "moon", "earth", "mars", "jupiter", "saturn", "venus", "mercury",
    "uranus", "neptune", "pluto", "milky way", "universe", "cosmos",
    // Orgs / tech
    "openai", "anthropic", "google", "microsoft", "meta", "apple", "amazon",
    "nasa", "who", "un", "fbi", "cia", "nsa", "gpt", "claude", "gemini",
    // Notable people (lowercase)
    "einstein", "newton", "darwin", "turing", "feynman",
    // Common agents
    "user", "model", "agent", "server", "client", "system", "network",
    "browser", "database", "api", "llm", "ai", "bot", "human",
    // Chemicals / elements often treated as entities
    "co2", "h2o", "dna", "rna", "atp", "oxygen", "nitrogen", "carbon",
    "hydrogen", "iron", "gold", "silicon",
];

// Abstract concepts → #
const KNOWN_CONCEPTS: &[&str] = &[
    // Physics
    "gravity", "energy", "force", "mass", "momentum", "entropy", "pressure",
    "velocity", "acceleration", "friction", "radiation", "magnetism",
    "electricity", "heat", "light", "sound", "wave", "particle", "field",
    "quantum", "relativity", "thermodynamics", "electromagnetism",
    // Bio / health
    "life", "death", "survival", "evolution", "mutation", "fitness",
    "disease", "infection", "immunity", "health", "pain", "growth", "decay",
    "metabolism", "photosynthesis", "respiration", "reproduction",
    // Cognitive / epistemic
    "intelligence", "consciousness", "knowledge", "belief", "truth", "logic",
    "reasoning", "memory", "learning", "understanding", "thought",
    "perception", "cognition", "intuition", "creativity", "bias",
    // Social / political
    "justice", "freedom", "democracy", "authority", "law", "power",
    "conflict", "war", "peace", "cooperation", "competition", "trust",
    "equality", "inequality", "rights", "duty", "responsibility",
    // Econ
    "economy", "market", "trade", "money", "wealth", "poverty", "inflation",
    "growth", "recession", "capital", "labour", "profit", "cost", "value",
    "risk", "uncertainty", "investment", "debt", "supply", "demand",
    // Information / CS
    "information", "data", "signal", "noise", "algorithm", "computation",
    "network", "system", "complexity", "chaos", "order", "pattern",
    "encryption", "security", "privacy", "bandwidth", "latency",
    // Environment
    "climate", "weather", "temperature", "pollution", "ecosystem",
    "biodiversity", "habitat", "atmosphere", "ocean", "soil",
    // Abstract
    "change", "stability", "equilibrium", "symmetry", "structure",
    "process", "outcome", "result", "effect", "cause", "reason", "purpose",
    "probability", "evidence", "proof", "theory", "hypothesis", "fact",
    "claim", "argument", "assumption", "prediction", "model",
    "safety", "danger", "threat", "protection", "damage",
    "time", "space", "distance", "size", "scale", "rate", "frequency",
    "uv", "uv-radiation", "ultraviolet", "infrared",
    // Compounds that should stay as concepts
    "treatment", "medicine", "drug", "vaccine", "therapy", "surgery",
    "education", "research", "science", "technology", "innovation",
    "culture", "society", "community", "population", "generation",
];

// Verb stems → ~  (common forms normalised to stem)
const KNOWN_VERBS: &[&str] = &[
    "emit", "absorb", "reflect", "transmit", "conduct", "radiate",
    "cause", "produce", "generate", "create", "destroy", "remove",
    "increase", "decrease", "grow", "shrink", "expand", "contract",
    "rise", "fall", "drop", "climb", "accelerate", "decelerate",
    "move", "travel", "propagate", "flow", "spread",
    "learn", "adapt", "evolve", "mutate", "transform", "change", "shift",
    "predict", "estimate", "measure", "observe", "detect", "sense",
    "require", "need", "demand", "depend", "rely", "support",
    "fail", "succeed", "work", "function", "operate", "run",
    "store", "process", "compute", "calculate", "optimize", "compile",
    "communicate", "send", "receive", "encode", "decode", "parse",
    "affect", "influence", "control", "regulate", "inhibit", "enable",
    "activate", "deactivate", "trigger", "initiate", "terminate",
    "exist", "occur", "happen", "persist", "continue", "end", "begin",
    "show", "demonstrate", "prove", "disprove", "indicate", "suggest",
    "think", "believe", "know", "assume", "expect", "hope",
    "improve", "worsen", "stabilise", "stabilize", "fluctuate",
    "heat", "cool", "melt", "freeze", "evaporate", "condense",
    "accelerate", "brake", "oscillate", "vibrate", "rotate", "spin",
    "interact", "collide", "merge", "split", "combine", "separate",
    "consume", "use", "release", "store", "convert", "exchange",
    "attack", "defend", "protect", "threaten", "harm", "help",
    "build", "develop", "design", "test", "deploy", "maintain",
];

// ---------------------------------------------------------------------------
// Abbreviation tiers
// ---------------------------------------------------------------------------
//
// AXON supports four abbreviation levels, each inclusive of the previous:
//
//   L0 (default)  — 85+ well-known terms universally understood by developers
//   L1 (extended) — broader technical vocabulary: networking, OS, data, DevOps
//   L2 (aggressive) — common English words shortened for max compression
//   L3 (maximum)  — single-char and extreme shortenings for expert users
//
// The translator accepts an `AbbrevLevel` and merges all tiers up to that
// level into a single lookup table at construction time.

/// Abbreviation compression level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AbbrevLevel {
    /// L0: conservative defaults — 85+ standard dev terms (v1.0 baseline)
    L0 = 0,
    /// L1: extended technical vocabulary
    L1 = 1,
    /// L2: aggressive general-purpose compression
    L2 = 2,
    /// L3: maximum compression, expert-only
    L3 = 3,
}

impl Default for AbbrevLevel {
    fn default() -> Self {
        AbbrevLevel::L0
    }
}

// L0 — conservative defaults (the original v1.0 dictionary)
pub const ABBREV_L0: &[(&str, &str)] = &[
    ("object", "obj"),
    ("function", "fn"),
    ("component", "comp"),
    ("documentation", "docs"),
    ("document", "doc"),
    ("implementation", "impl"),
    ("implement", "impl"),
    ("authentication", "auth"),
    ("authorization", "authz"),
    ("application", "app"),
    ("configuration", "cfg"),
    ("configure", "cfg"),
    ("environment", "env"),
    ("development", "dev"),
    ("production", "prod"),
    ("repository", "repo"),
    ("parameter", "param"),
    ("argument", "arg"),
    ("reference", "ref"),
    ("variable", "var"),
    ("constant", "const"),
    ("directory", "dir"),
    ("dependency", "dep"),
    ("dependencies", "deps"),
    ("performance", "perf"),
    ("optimization", "opt"),
    ("optimize", "opt"),
    ("database", "db"),
    ("message", "msg"),
    ("request", "req"),
    ("response", "res"),
    ("middleware", "mw"),
    ("specification", "spec"),
    ("information", "info"),
    ("property", "prop"),
    ("properties", "props"),
    ("attribute", "attr"),
    ("attributes", "attrs"),
    ("expression", "expr"),
    ("condition", "cond"),
    ("temporary", "tmp"),
    ("template", "tpl"),
    ("maximum", "max"),
    ("minimum", "min"),
    ("number", "num"),
    ("string", "str"),
    ("boolean", "bool"),
    ("integer", "int"),
    ("character", "char"),
    ("subscription", "sub"),
    ("notification", "notif"),
    ("generation", "gen"),
    ("operation", "op"),
    ("operations", "ops"),
    ("administrator", "admin"),
    ("previous", "prev"),
    ("current", "curr"),
    ("source", "src"),
    ("destination", "dest"),
    ("original", "orig"),
    ("description", "desc"),
    ("infrastructure", "infra"),
    ("transaction", "txn"),
    ("synchronous", "sync"),
    ("asynchronous", "async"),
    ("memory", "mem"),
    ("iteration", "iter"),
    ("iterator", "iter"),
    ("executable", "exec"),
    ("execution", "exec"),
    ("navigation", "nav"),
    ("initialization", "init"),
    ("initialize", "init"),
    ("connection", "conn"),
    ("communication", "comm"),
    ("certificate", "cert"),
    ("technology", "tech"),
    ("management", "mgmt"),
    ("calculate", "calc"),
    ("calculation", "calc"),
    ("callback", "cb"),
    ("render", "rnd"),
    ("rendering", "rnd"),
    ("inline", "inl"),
];

// L1 — extended technical vocabulary
pub const ABBREV_L1: &[(&str, &str)] = &[
    // Networking / protocols
    ("protocol", "proto"),
    ("address", "addr"),
    ("network", "net"),
    ("socket", "sock"),
    ("bandwidth", "bw"),
    ("latency", "lat"),
    ("throughput", "tput"),
    ("packet", "pkt"),
    ("connection", "conn"),
    ("disconnect", "dconn"),
    ("transmission", "tx"),
    ("receiver", "rx"),
    // OS / systems
    ("process", "proc"),
    ("processes", "procs"),
    ("processor", "proc"),
    ("thread", "thd"),
    ("scheduler", "sched"),
    ("filesystem", "fs"),
    ("semaphore", "sem"),
    ("allocation", "alloc"),
    ("allocator", "alloc"),
    ("register", "reg"),
    ("interrupt", "irq"),
    ("permission", "perm"),
    ("permissions", "perms"),
    // Data / storage
    ("collection", "coll"),
    ("element", "elem"),
    ("elements", "elems"),
    ("index", "idx"),
    ("length", "len"),
    ("count", "cnt"),
    ("buffer", "buf"),
    ("sequence", "seq"),
    ("frequency", "freq"),
    ("column", "col"),
    ("columns", "cols"),
    ("record", "rec"),
    ("records", "recs"),
    ("aggregate", "agg"),
    ("accumulator", "acc"),
    // DevOps / infra
    ("container", "ctr"),
    ("kubernetes", "k8s"),
    ("namespace", "ns"),
    ("instance", "inst"),
    ("instances", "insts"),
    ("availability", "avail"),
    ("deployment", "deploy"),
    ("monitoring", "mon"),
    ("observability", "obs"),
    ("telemetry", "telem"),
    ("replication", "repl"),
    // Web / API
    ("endpoint", "ep"),
    ("endpoints", "eps"),
    ("middleware", "mw"),
    ("controller", "ctrl"),
    ("serialization", "ser"),
    ("deserialization", "deser"),
    ("pagination", "page"),
    ("validation", "val"),
    ("validator", "val"),
    // General dev
    ("constructor", "ctor"),
    ("destructor", "dtor"),
    ("exception", "exc"),
    ("library", "lib"),
    ("libraries", "libs"),
    ("package", "pkg"),
    ("packages", "pkgs"),
    ("interface", "iface"),
    ("abstract", "abs"),
    ("algorithm", "algo"),
    ("architecture", "arch"),
    ("version", "ver"),
    ("revision", "rev"),
    ("extension", "ext"),
    ("extensions", "exts"),
    ("overflow", "ovfl"),
    ("underflow", "udfl"),
    ("definition", "def"),
    ("declaration", "decl"),
    ("statement", "stmt"),
    ("annotation", "annot"),
    ("utility", "util"),
    ("utilities", "utils"),
    ("debug", "dbg"),
    ("debugging", "dbg"),
    ("profiling", "prof"),
    ("benchmark", "bench"),
    ("regression", "regr"),
    ("migration", "migr"),
    ("integration", "integ"),
    ("refactoring", "refac"),
    ("compilation", "compile"),
    ("compiler", "cc"),
    ("documentation", "docs"),
];

// L2 — aggressive general-purpose compression
pub const ABBREV_L2: &[(&str, &str)] = &[
    // Common English shortened
    ("between", "btw"),
    ("without", "w/o"),
    ("because", "bc"),
    ("example", "ex"),
    ("examples", "exs"),
    ("different", "diff"),
    ("difference", "diff"),
    ("important", "imp"),
    ("multiple", "mult"),
    ("available", "avail"),
    ("required", "reqd"),
    ("optional", "opt"),
    ("alternative", "alt"),
    ("alternatives", "alts"),
    ("comparison", "cmp"),
    ("approximately", "approx"),
    ("especially", "esp"),
    ("including", "incl"),
    ("following", "foll"),
    ("regarding", "re"),
    ("usually", "usu"),
    ("probably", "prob"),
    ("especially", "esp"),
    ("significant", "sig"),
    ("significant", "sig"),
    ("however", "howev"),
    ("typically", "typ"),
    // Technical discourse
    ("specific", "spec"),
    ("specifically", "spec"),
    ("particular", "partic"),
    ("generally", "gen"),
    ("essentially", "ess"),
    ("basically", "basic"),
    ("additional", "addl"),
    ("separately", "sep"),
    ("automatic", "auto"),
    ("automatically", "auto"),
    ("independent", "indep"),
    ("independently", "indep"),
    ("separate", "sep"),
    ("completely", "compl"),
    ("immediately", "immed"),
    ("consistent", "consist"),
    ("simultaneously", "simul"),
    ("distributed", "distrib"),
    ("distributed", "distrib"),
    ("concurrent", "concur"),
    ("horizontal", "horiz"),
    ("vertical", "vert"),
    ("duplicate", "dup"),
    ("duplicates", "dups"),
    ("strategy", "strat"),
    ("strategies", "strats"),
    ("mechanism", "mech"),
];

// L3 — maximum compression, expert-only
pub const ABBREV_L3: &[(&str, &str)] = &[
    // Extreme shortenings
    ("function", "f"),
    ("variable", "v"),
    ("object", "o"),
    ("string", "s"),
    ("number", "n"),
    ("integer", "i"),
    ("boolean", "b"),
    ("return", "ret"),
    ("value", "val"),
    ("error", "err"),
    ("result", "res"),
    ("context", "ctx"),
    ("config", "cfg"),
    ("message", "m"),
    ("data", "d"),
    ("type", "t"),
    ("event", "evt"),
    ("handler", "h"),
    ("service", "svc"),
    ("server", "srv"),
    ("client", "cli"),
    ("command", "cmd"),
    ("argument", "a"),
    ("option", "o"),
    ("input", "in"),
    ("output", "out"),
    ("channel", "ch"),
    ("signal", "sig"),
    ("method", "m"),
    ("module", "mod"),
    ("resource", "rsc"),
    ("identifier", "id"),
    ("property", "p"),
    ("attribute", "a"),
    ("element", "el"),
    ("container", "c"),
    ("manager", "mgr"),
    ("factory", "fac"),
    ("provider", "prov"),
    ("consumer", "cons"),
    ("producer", "prod"),
    ("subscriber", "sub"),
    ("publisher", "pub"),
    ("listener", "lsnr"),
    ("observer", "obs"),
    ("wrapper", "wrap"),
];

/// Build a merged abbreviation table for a given level (inclusive of all lower levels).
pub fn abbreviations_for_level(level: AbbrevLevel) -> Vec<(&'static str, &'static str)> {
    let mut table: Vec<(&str, &str)> = Vec::new();
    // Higher levels override lower levels, so we add in order and deduplicate
    // by keeping the LAST entry for each key (higher tier wins).
    table.extend_from_slice(ABBREV_L0);
    if level >= AbbrevLevel::L1 {
        table.extend_from_slice(ABBREV_L1);
    }
    if level >= AbbrevLevel::L2 {
        table.extend_from_slice(ABBREV_L2);
    }
    if level >= AbbrevLevel::L3 {
        table.extend_from_slice(ABBREV_L3);
    }
    // Deduplicate: later entries win (higher tier overrides)
    let mut seen = std::collections::HashMap::new();
    for (i, &(full, abbr)) in table.iter().enumerate() {
        seen.insert(full, (i, abbr));
    }
    let mut deduped: Vec<(usize, &str, &str)> = seen
        .into_iter()
        .map(|(full, (i, abbr))| (i, full, abbr))
        .collect();
    deduped.sort_by_key(|&(i, _, _)| i);
    deduped.iter().map(|&(_, full, abbr)| (full, abbr)).collect()
}

// Backward-compatible alias for the default (L0) table
pub const ABBREVIATIONS: &[(&str, &str)] = ABBREV_L0;

// Words that signal CAUSATION (->)
const CAUSAL_FORWARD: &[&str] = &[
    "causes", "cause", "leads to", "lead to", "results in", "result in",
    "produces", "produce", "generates", "generate", "creates", "create",
    "triggers", "trigger", "drives", "drive", "induces", "induce",
    "brings about", "gives rise to", "makes",
];

// Words that signal REVERSE CAUSATION (←)
const CAUSAL_BACKWARD: &[&str] = &[
    "caused by", "due to", "as a result of", "owing to", "stemming from",
    "arising from", "resulting from",
];

// Words that signal BECAUSE (∵)
const BECAUSE_WORDS: &[&str] = &[
    "because", "since", "as", "given that", "considering that",
    "in light of", "on account of",
];

// Words that signal THEREFORE (∴)
const THEREFORE_WORDS: &[&str] = &[
    "therefore", "thus", "hence", "so", "consequently", "as a result",
    "it follows that", "this means",
];

// Confidence hedge words
struct HedgeEntry {
    words: &'static [&'static str],
    marker: &'static str,
    label: &'static str,
}

fn confidence_table() -> Vec<HedgeEntry> {
    vec![
        HedgeEntry {
            words: &[
                "certainly", "definitely", "absolutely", "undoubtedly",
                "without doubt", "clearly", "obviously", "always", "invariably",
            ],
            marker: "!!",
            label: "certain",
        },
        HedgeEntry {
            words: &[
                "likely", "probably", "generally", "typically",
                "usually", "normally", "in most cases", "strongly",
            ],
            marker: "!",
            label: "high-confidence",
        },
        HedgeEntry {
            words: &[
                "possibly", "perhaps", "maybe", "conceivably", "potentially",
                "might", "could", "may", "it is possible", "there is a chance",
            ],
            marker: "**",
            label: "speculative",
        },
        HedgeEntry {
            words: &[
                "uncertain", "unclear", "unsure", "unknown",
                "not sure", "don't know", "not known", "questionable",
            ],
            marker: "?",
            label: "unknown",
        },

        // moderate is the default — no extra words needed, but match some light hedges
        HedgeEntry {
            words: &[
                "somewhat", "partly", "partially", "to some extent",
                "in some cases", "often", "frequently",
            ],
            marker: "~",
            label: "moderate",
        },
    ]
}

// Noise phrases — stripped wholesale from input BEFORE the pipeline runs.
//
// These are multi-word expressions that carry zero semantic payload:
// politeness markers, meta-commentary, filler openers, and social scaffolding.
// Order matters: longer / more specific phrases must come before shorter ones.
const NOISE_PHRASES: &[&str] = &[
    // Politeness openers / closers
    "could you please", "can you please", "would you please",
    "if you don't mind", "if you wouldn't mind", "if you don't mind me asking",
    "i'd appreciate it if", "i would appreciate it if",
    "i'd be grateful if", "i would be grateful if",
    "i'd be happy if", "i was wondering if",
    "would it be possible to", "would it be possible for you to",
    "do you think you could", "do you think you can",
    "i hope you don't mind", "i hope you can help",

    // Thank-you variants
    "thank you very much", "thank you so much", "thank you kindly",
    "many thanks", "thanks a lot", "thanks so much", "thanks very much",
    "thank you", "thanks", "cheers",

    // Greeting / sign-off
    "good morning", "good afternoon", "good evening", "good day",
    "hi there", "hello there", "hey there", "greetings",
    "hi", "hey", "hello", "howdy",
    "kind regards", "best regards", "warm regards",
    "yours sincerely", "yours faithfully", "yours truly",
    "best wishes", "all the best", "take care",

    // Filler / meta-commentary openers
    "i was just wondering", "i was wondering",
    "i just wanted to ask", "i wanted to ask",
    "i just wanted to know", "i wanted to know",
    "i just wanted to say", "i wanted to say",
    "i just want to know", "i want to know",
    "i was curious about", "i am curious about", "i'm curious about",
    "could you tell me", "can you tell me", "please tell me",
    "could you explain", "can you explain", "please explain",
    "could you clarify", "can you clarify", "please clarify",
    "i need to understand", "help me understand",
    "i'd like to know", "i would like to know",
    "i'd like to understand", "i would like to understand",
    "i'd like to ask", "i would like to ask",
    "i have a question about", "i have a question",
    "quick question", "just a quick question",

    // Apology / hedging preamble
    "i'm sorry if this is a silly question",
    "sorry if this is obvious",
    "apologies if this is a dumb question",
    "forgive me if",
    "pardon me",
    "excuse me",
    "no worries if not",
    "feel free to ignore this if",

    // Affirmations / acknowledgements with no payload
    "of course", "sure thing", "absolutely", "definitely", "certainly",
    "no problem", "not a problem", "no worries", "fair enough",
    "that makes sense", "that's fair",
    "i see", "i understand", "i get it", "got it",
    "noted", "understood", "sounds good", "great",
    "awesome", "wonderful", "fantastic", "excellent", "perfect",
    "ok", "okay", "alright", "right",
    "yes", "yeah", "yep", "yup", "nope", "nah",

    // Conversational softeners
    "in other words", "to put it another way", "that is to say",
    "so to speak", "as it were", "you know", "you see",
    "i mean", "like i said", "as i said", "as i mentioned",
    "at the end of the day", "when all is said and done",
    "needless to say", "goes without saying", "it goes without saying",
    "to be honest", "to be fair", "to be clear",
    "frankly speaking", "frankly", "honestly",
    "basically", "essentially", "fundamentally", "ultimately",
    "in short", "in brief", "in summary", "in a nutshell",
    "long story short", "to cut a long story short",
    "as you know", "as we know", "as everyone knows",
    "it's worth noting", "it's worth mentioning", "it should be noted",
    "interestingly", "interestingly enough", "importantly",
    "obviously", "clearly", "evidently", "apparently",
    "of note", "notably",

    "please", "kindly",
];

// Stop words — ignored during per-token classification.
//
// These are single tokens that carry no semantic payload once the sentence
// has been stripped of noise phrases.
const STOP_WORDS: &[&str] = &[
    // Articles / determiners
    "a", "an", "the", "this", "that", "these", "those",
    // Copula / auxiliaries
    "is", "are", "was", "were", "be", "been", "being",
    "will", "would", "shall", "should", "must", "may",
    "have", "has", "had", "do", "does", "did",
    // Prepositions
    "of", "in", "on", "at", "to", "for", "with", "by", "from", "as",
    "into", "onto", "upon", "about", "over", "under", "between", "among",
    "through", "during", "before", "after", "above", "below",
    "up", "down", "out", "off", "along", "across", "against", "within",
    // Pronouns
    "it", "i", "we", "you", "he", "she", "they", "them",
    "my", "our", "your", "his", "her", "their", "its",
    "me", "us", "him",
    // Conjunctions / relativizers
    "if", "when", "while", "although", "though", "unless", "until",
    "whether", "which", "who", "whom", "whose", "what", "where", "how",
    "such", "so",
    // Adverbs with no semantic weight
    "very", "quite", "rather", "too", "also", "just", "even", "still",
    "already", "yet", "then", "now", "here", "there", "thus",
    // Determiners / quantifiers (handled separately if needed)
    "both", "all", "each", "every", "any", "some",
    // Motion / existence verbs when used as copula
    "get", "got", "go", "goes", "went", "come", "came",
    // Filler interjections left over after phrase stripping
    "well", "indeed", "anyway", "besides",
];

pub struct Translator {
    entities: HashSet<String>,
    concepts: HashSet<String>,
    verbs: HashSet<String>,
    stop_words: HashSet<String>,
    abbrev_table: Vec<(&'static str, &'static str)>,
    abbrev_level: AbbrevLevel,
    code: CodeTranslator,
}

impl Translator {
    pub fn new() -> Self {
        Self::with_level(AbbrevLevel::L0)
    }

    pub fn with_level(level: AbbrevLevel) -> Self {
        let table = abbreviations_for_level(level);
        Self {
            entities: KNOWN_ENTITIES.iter().map(|s| s.to_string()).collect(),
            concepts: KNOWN_CONCEPTS.iter().map(|s| s.to_string()).collect(),
            verbs: KNOWN_VERBS.iter().map(|s| s.to_string()).collect(),
            stop_words: STOP_WORDS.iter().map(|s| s.to_string()).collect(),
            abbrev_table: table,
            abbrev_level: level,
            code: CodeTranslator::with_level(level),
        }
    }

    pub fn abbrev_level(&self) -> AbbrevLevel {
        self.abbrev_level
    }

    pub fn translate(&self, input: &str) -> Translation {
        let code_result = self.code.translate(input);
        if code_result.matched {
            return Translation {
                axon: code_result.axon,
                annotation: code_result.annotation,
            };
        }

        let text = input.trim().trim_end_matches(['.', '!', '?', ',']).trim();
        let lower = text.to_lowercase();

        let mut annotations: Vec<String> = Vec::new();

        let (lower, stripped_count) = self.strip_noise(&lower);
        if stripped_count > 0 {
            annotations.push(format!("stripped {}\u{00d7}noise", stripped_count));
        }

        let (confidence, lower_stripped_conf) = self.extract_confidence(&lower, &mut annotations);
        let (temporal, working) = self.extract_temporal(&lower_stripped_conf, &mut annotations);
        let (negated, working) = self.extract_negation(&working);
        if negated {
            annotations.push("negation(!)".into());
        }

        let axon_body = self.parse_structure(&working, negated, &mut annotations);
        let mut axon = axon_body;

        if !confidence.is_empty() {
            axon.push_str(&confidence);
        }
        if !temporal.is_empty() {
            axon.push(' ');
            axon.push_str(&temporal);
        }

        let annotation = if annotations.is_empty() {
            "direct encoding".into()
        } else {
            annotations.join(" · ")
        };

        Translation { axon, annotation }
    }


    fn strip_noise(&self, text: &str) -> (String, usize) {
        let mut working = text.to_string();
        let mut count = 0usize;

        // Iteratively strip until stable (handles e.g. "please, hi, can you tell me...")
        let mut changed = true;
        while changed {
            changed = false;

            for phrase in NOISE_PHRASES {
                if working.starts_with(phrase) {
                    let rest = working[phrase.len()..].to_string();
                    // Strip any trailing punctuation / whitespace from the phrase
                    working = rest
                        .trim_start_matches(|c: char| c.is_ascii_punctuation() || c == ' ')
                        .to_string();
                    count += 1;
                    changed = true;
                    continue;
                }

                // end of string - Allow ", phrase" or ". phrase" suffix patterns
                for prefix in &[", ", " - ", ". ", "! ", "? "] {
                    let suffix = format!("{}{}", prefix, phrase);
                    if working.ends_with(&suffix) {
                        working = working[..working.len() - suffix.len()]
                            .trim_end_matches(|c: char| c.is_ascii_punctuation() || c == ' ')
                            .to_string();
                        count += 1;
                        changed = true;
                        break;
                    }
                }
                if changed { continue; }

                // Plain end match
                if working.ends_with(phrase) && working.len() > phrase.len() {
                    let candidate = &working[..working.len() - phrase.len()];
                    // Only strip if preceded by a word boundary (space/punctuation)
                    if candidate.ends_with(|c: char| c == ' ' || c.is_ascii_punctuation()) {
                        working = candidate
                            .trim_end_matches(|c: char| c.is_ascii_punctuation() || c == ' ')
                            .to_string();
                        count += 1;
                        changed = true;
                        continue;
                    }
                }

                // inline
                for (open, close) in &[(", ", ", "), (", ", " "), (", ", "! "), (", ", "? ")] {
                    let pattern = format!("{}{}{}", open, phrase, close.trim_end());
                    // Find as ", phrase," then close
                    let search = format!("{}{}", open, phrase);
                    if let Some(pos) = working.find(&search) {
                        let after = pos + search.len();
                        let rest_start = &working[after..];
                        if rest_start.starts_with(|c: char| c == ',' || c == '.' || c == '!' || c == '?' || c == ' ') {
                            let remainder = rest_start.trim_start_matches(|c: char| c == ',' || c == ' ');
                            working = format!("{} {}", &working[..pos].trim_end(), remainder).trim().to_string();
                            count += 1;
                            changed = true;
                            break;
                        }
                    }
                }
            }
        }

        // Final cleanup: strip any leading/trailing stray punctuation then normalise whitespace
        let cleaned = working
            .trim_matches(|c: char| c.is_ascii_punctuation() || c == ' ')
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        (cleaned, count)
    }

    fn extract_confidence(&self, text: &str, annotations: &mut Vec<String>) -> (String, String) {
        for entry in confidence_table() {
            for &word in entry.words {
                if let Some(pos) = text.find(word) {
                    let stripped = format!(
                        "{}{}",
                        &text[..pos],
                        &text[pos + word.len()..]
                    )
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ");
                    annotations.push(format!("confidence({}={})", entry.marker, entry.label));
                    return (entry.marker.to_string(), stripped);
                }
            }
        }

        // Default: no marker (implicitly "!" / high confidence for plain statements)
        ("".into(), text.to_string())
    }

    fn extract_temporal(&self, text: &str, annotations: &mut Vec<String>) -> (String, String) {
        // Ordered from most specific to least
        let patterns: &[(&str, &str)] = &[
            ("always", "^A.t"),
            ("forever", "^A.t"),
            ("at all times", "^A.t"),
            ("universally", "^A.t"),
            ("right now", "^now"),
            ("currently", "^now"),
            ("at present", "^now"),
            ("today", "^now"),
            ("now", "^now"),
            ("at the moment", "^now"),
            ("yesterday", "^T-1d"),
            ("last week", "^T-7d"),
            ("last month", "^T-30d"),
            ("last year", "^T-365d"),
            ("tomorrow", "^T+1d"),
            ("next week", "^T+7d"),
            ("next month", "^T+30d"),
            ("next year", "^T+365d"),
            ("in the future", "^T+∞"),
            ("in the past", "^T-∞"),
            ("historically", "^T-∞"),
            ("recently", "^T-30d"),
            ("soon", "^T+7d"),
        ];

        for (phrase, marker) in patterns {
            if let Some(pos) = text.find(phrase) {
                let stripped = format!(
                    "{}{}",
                    &text[..pos],
                    &text[pos + phrase.len()..]
                )
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");
                annotations.push(format!("temporal({})", marker));
                return (marker.to_string(), stripped);
            }
        }

        // Try to detect "in N days/weeks/months"
        if let Some(result) = self.extract_relative_time(text) {
            let (marker, stripped) = result;
            annotations.push(format!("temporal({})", marker));
            return (marker, stripped);
        }

        ("".into(), text.to_string())
    }

    fn extract_relative_time(&self, text: &str) -> Option<(String, String)> {
        // Pattern: "in N days/weeks/months/years"
        let words: Vec<&str> = text.split_whitespace().collect();
        for i in 0..words.len().saturating_sub(2) {
            if words[i] == "in" || words[i] == "within" {
                if let Ok(n) = words[i + 1].parse::<u32>() {
                    let unit = words.get(i + 2).unwrap_or(&"");
                    let marker = match *unit {
                        "day" | "days" => format!("^T+{}d", n),
                        "week" | "weeks" => format!("^T+{}d", n * 7),
                        "month" | "months" => format!("^T+{}d", n * 30),
                        "year" | "years" => format!("^T+{}d", n * 365),
                        _ => continue,
                    };
                    // Remove "in N unit" from text
                    let stripped: Vec<&str> = words
                        .iter()
                        .enumerate()
                        .filter(|(j, _)| *j != i && *j != i + 1 && *j != i + 2)
                        .map(|(_, w)| *w)
                        .collect();
                    return Some((marker, stripped.join(" ")));
                }
            }

            // "N days/weeks ago"
            if let Ok(n) = words[i].parse::<u32>() {
                let unit = words.get(i + 1).unwrap_or(&"");
                let suffix = words.get(i + 2).unwrap_or(&"");
                if *suffix == "ago" {
                    let marker = match *unit {
                        "day" | "days" => format!("^T-{}d", n),
                        "week" | "weeks" => format!("^T-{}d", n * 7),
                        "month" | "months" => format!("^T-{}d", n * 30),
                        "year" | "years" => format!("^T-{}d", n * 365),
                        _ => continue,
                    };
                    let stripped: Vec<&str> = words
                        .iter()
                        .enumerate()
                        .filter(|(j, _)| *j != i && *j != i + 1 && *j != i + 2)
                        .map(|(_, w)| *w)
                        .collect();
                    return Some((marker, stripped.join(" ")));
                }
            }
        }
        None
    }

    fn extract_negation(&self, text: &str) -> (bool, String) {
        let neg_phrases = [
            "there is no ", "there are no ", "there is not ", "there are not ",
            "no evidence", "no proof", "no sign",
            "without ", "lacking ", "absence of ", "lack of ",
        ];
        let neg_words = ["not ", "never ", "nobody ", "nothing ", "nowhere "];

        for phrase in &neg_phrases {
            if text.contains(phrase) {
                let stripped = text.replace(phrase, "").split_whitespace().collect::<Vec<_>>().join(" ");
                return (true, stripped);
            }
        }
        for word in &neg_words {
            if text.contains(word) {
                let stripped = text.replace(word, "").split_whitespace().collect::<Vec<_>>().join(" ");
                return (true, stripped);
            }
        }
        (false, text.to_string())
    }

    fn parse_structure(&self, text: &str, negated: bool, annotations: &mut Vec<String>) -> String {
        // Try causal forward: "A causes B"
        for trigger in CAUSAL_FORWARD {
            if let Some(pos) = text.find(trigger) {
                let left = text[..pos].trim().to_string();
                let right = text[pos + trigger.len()..].trim().to_string();
                if !left.is_empty() && !right.is_empty() {
                    annotations.push("causal(->)".into());
                    let l = self.tag_phrase(&left, false);
                    let r = self.tag_phrase(&right, negated);
                    return format!("{} -> {}", l, r);
                }
            }
        }

        // Try causal backward: "B caused by A"
        for trigger in CAUSAL_BACKWARD {
            if let Some(pos) = text.find(trigger) {
                let left = text[..pos].trim().to_string();
                let right = text[pos + trigger.len()..].trim().to_string();
                if !left.is_empty() && !right.is_empty() {
                    annotations.push("causal(<-)".into());
                    let l = self.tag_phrase(&left, negated);
                    let r = self.tag_phrase(&right, false);
                    return format!("{} <- {}", l, r);
                }
            }
        }

        // Try "therefore": "A, therefore B"
        for trigger in THEREFORE_WORDS {
            if let Some(pos) = text.find(trigger) {
                let left = text[..pos].trim().trim_end_matches(',').trim().to_string();
                let right = text[pos + trigger.len()..].trim().to_string();
                if !left.is_empty() && !right.is_empty() {
                    annotations.push("logical(:.)".into());
                    let l = self.tag_phrase(&left, false);
                    let r = self.tag_phrase(&right, negated);
                    return format!("{} :. {}", l, r);
                }
            }
        }

        // Try "because": "A because B"
        for trigger in BECAUSE_WORDS {
            if let Some(pos) = text.find(trigger) {
                let left = text[..pos].trim().to_string();
                let right = text[pos + trigger.len()..].trim().to_string();
                if !left.is_empty() && !right.is_empty() {
                    annotations.push("logical(bc)".into());
                    let l = self.tag_phrase(&left, negated);
                    let r = self.tag_phrase(&right, false);
                    return format!("{} bc {}", l, r);
                }
            }
        }

        // Try "and" conjunction
        if let Some(pos) = self.find_conjunction(text, " and ") {
            let left = &text[..pos];
            let right = &text[pos + 5..];
            // Only split if both sides are substantial
            if left.split_whitespace().count() >= 2 && right.split_whitespace().count() >= 2 {
                annotations.push("conjunction(&&)".into());
                let l = self.tag_phrase(left.trim(), false);
                let r = self.tag_phrase(right.trim(), negated);
                return format!("{} && {}", l, r);
            }
        }

        // Try "or" disjunction
        if let Some(pos) = self.find_conjunction(text, " or ") {
            let left = &text[..pos];
            let right = &text[pos + 4..];
            if left.split_whitespace().count() >= 2 && right.split_whitespace().count() >= 2 {
                annotations.push("disjunction(||)".into());
                let l = self.tag_phrase(left.trim(), false);
                let r = self.tag_phrase(right.trim(), negated);
                return format!("{} || {}", l, r);
            }
        }

        // Check for universal quantifier
        let (universal, text_no_quant) = self.extract_quantifier(text);

        // Default: tag the whole phrase
        let tagged = self.tag_phrase(&text_no_quant, negated);
        if !universal.is_empty() {
            annotations.push(format!("quantifier({})", universal));
            format!("{}{}", universal, tagged)
        } else {
            tagged
        }
    }

    fn find_conjunction(&self, text: &str, conj: &str) -> Option<usize> {
        // Don't split on conjunctions that are inside parentheses
        let pos = text.find(conj)?;
        // Simple heuristic: must be after at least 3 chars
        if pos < 3 {
            return None;
        }
        Some(pos)
    }

    fn extract_quantifier(&self, text: &str) -> (String, String) {
        let universal_triggers = [
            "all ", "every ", "each ", "any ", "for all ", "invariably ",
        ];
        let existential_triggers = [
            "some ", "there exist", "there is a ", "there are some ",
        ];

        for t in &universal_triggers {
            if text.starts_with(t) || text.contains(&format!(" {}", t.trim())) {
                let stripped = text.replacen(t, "", 1);
                return ("A.".into(), stripped.trim().to_string());
            }
        }
        for t in &existential_triggers {
            if text.starts_with(t) {
                let stripped = text[t.len()..].trim().to_string();
                return ("E.".into(), stripped);
            }
        }
        ("".into(), text.to_string())
    }

    fn tag_phrase(&self, phrase: &str, negated: bool) -> String {
        let words: Vec<&str> = phrase.split_whitespace().collect();
        if words.is_empty() {
            return String::new();
        }

        // Build compound tokens (bi-grams first for multi-word concepts)
        let mut tokens: Vec<String> = Vec::new();
        let mut i = 0;
        while i < words.len() {
            let w = words[i].to_lowercase();
            let w_clean = w.trim_matches(|c: char| !c.is_alphanumeric() && c != '-');

            // Skip stop words
            if self.stop_words.contains(w_clean) {
                i += 1;
                continue;
            }

            // Try bi-gram match first
            if i + 1 < words.len() {
                let next_clean = words[i + 1].to_lowercase();
                let next_clean = next_clean.trim_matches(|c: char| !c.is_alphanumeric());
                let bigram = format!("{} {}", w_clean, next_clean);
                if self.concepts.contains(&bigram) {
                    let slug = self.abbreviate(&bigram.replace(' ', "-"));
                    tokens.push(if negated { format!("!{}", slug) } else { format!("#{}", slug) });
                    i += 2;
                    continue;
                }
                if self.entities.contains(&bigram) {
                    let slug = self.abbreviate(&bigram.replace(' ', "-"));
                    tokens.push(format!("@{}", slug));
                    i += 2;
                    continue;
                }
            }

            // Single token classification
            let token = self.classify_token(w_clean, words[i], negated);
            if let Some(t) = token {
                tokens.push(t);
            }
            i += 1;
        }

        if tokens.is_empty() {
            // Fallback: take first meaningful word
            for w in &words {
                let clean = w.to_lowercase();
                let clean = clean.trim_matches(|c: char| !c.is_alphanumeric());
                if !self.stop_words.contains(clean) {
                    let abbr = self.abbreviate(clean);
                    return if negated {
                        format!("!{}", abbr)
                    } else {
                        abbr
                    };
                }
            }
            return "unknown".into();
        }

        // Merge consecutive bare tokens (no sigil prefix) into hyphenated compounds
        self.merge_bare_tokens(&tokens)
    }

    /// Merge consecutive tokens that have no sigil prefix into hyphenated compounds.
    /// e.g. ["new", "obj", "#ref"] → ["new-obj", "#ref"]
    fn merge_bare_tokens(&self, tokens: &[String]) -> String {
        let mut merged: Vec<String> = Vec::new();
        let mut bare_run: Vec<String> = Vec::new();

        for token in tokens {
            if self.has_sigil(token) {
                // Flush any accumulated bare tokens
                if !bare_run.is_empty() {
                    merged.push(bare_run.join("-"));
                    bare_run.clear();
                }
                merged.push(token.clone());
            } else {
                bare_run.push(token.clone());
            }
        }
        // Flush remaining
        if !bare_run.is_empty() {
            merged.push(bare_run.join("-"));
        }

        merged.join(" ")
    }

    /// Check whether a token starts with a known sigil prefix.
    fn has_sigil(&self, token: &str) -> bool {
        let first = token.chars().next().unwrap_or(' ');
        matches!(first, '@' | '#' | '~' | '$' | '!' | '?' | '^')
    }

    /// Abbreviate a word using the abbreviation dictionary.
    /// For hyphenated compounds, abbreviate each component.
    fn abbreviate(&self, word: &str) -> String {
        if word.contains('-') {
            word.split('-')
                .map(|part| self.abbreviate_single(part))
                .collect::<Vec<_>>()
                .join("-")
        } else {
            self.abbreviate_single(word)
        }
    }

    fn abbreviate_single(&self, word: &str) -> String {
        for &(full, abbr) in &self.abbrev_table {
            if word == full {
                return abbr.to_string();
            }
        }
        word.to_string()
    }

    fn classify_token(&self, lower: &str, original: &str, negated: bool) -> Option<String> {
        if lower.is_empty() {
            return None;
        }

        let abbr = self.abbreviate(lower);

        // Numbers → $
        if lower.parse::<f64>().is_ok() {
            return Some(format!("${}", lower));
        }

        // Percentages → $
        if lower.ends_with('%') {
            return Some(format!("${}", lower));
        }

        // Known entity → @  (always sigil — entities are proper nouns)
        if self.entities.contains(lower) {
            return Some(format!("@{}", abbr));
        }

        // Mid-sentence capitalisation → likely a proper noun → @
        let first_char = original.chars().next().unwrap_or('a');
        if first_char.is_uppercase() && lower != "i" {
            return Some(format!("@{}", abbr));
        }

        // Known concept → # only for known concepts (disambiguation)
        if self.concepts.contains(lower) {
            return Some(if negated {
                format!("!{}", abbr)
            } else {
                format!("#{}", abbr)
            });
        }

        // Known verb or normalised verb stem → ~ only for known verbs
        let stem = self.verb_stem(lower);
        if self.verbs.contains(lower) || self.verbs.contains(stem.as_str()) {
            let label = if lower.ends_with("ing") {
                lower.trim_end_matches("ing").trim_end_matches('n').to_string()
            } else {
                stem.clone()
            };
            return Some(format!("~{}", self.abbreviate(&label)));
        }

        // Adjectives ending in common suffixes that indicate scale → $
        if lower.ends_with("er") || lower.ends_with("est") {
            if lower.len() > 4 {
                return Some(format!("${}", abbr));
            }
        }

        // Fallback: NO sigil — bare token (saves 1 BPE token per word)
        // Apply abbreviation and negation prefix only
        Some(if negated {
            format!("!{}", abbr)
        } else {
            abbr
        })
    }

    // Strip common English verb suffixes to get approximate stem
    fn verb_stem<'a>(&self, word: &'a str) -> String {
        if word.ends_with("ies") && word.len() > 4 {
            return format!("{}y", &word[..word.len() - 3]);
        }
        if word.ends_with("ied") && word.len() > 4 {
            return format!("{}y", &word[..word.len() - 3]);
        }
        if word.ends_with("ing") && word.len() > 5 {
            let base = &word[..word.len() - 3];
            // Handle doubled consonant: running → run
            if base.len() > 2 {
                let chars: Vec<char> = base.chars().collect();
                let n = chars.len();
                if chars[n - 1] == chars[n - 2] {
                    return chars[..n - 1].iter().collect();
                }
            }
            return base.to_string();
        }
        if word.ends_with("ed") && word.len() > 4 {
            let base = &word[..word.len() - 2];
            if base.ends_with('e') {
                return base.to_string();
            }
            return base.to_string();
        }
        if word.ends_with("es") && word.len() > 4 && !word.ends_with("ies") {
            return word[..word.len() - 2].to_string();
        }
        if word.ends_with('s') && word.len() > 4 {
            return word[..word.len() - 1].to_string();
        }
        word.to_string()
    }
}

impl Default for Translator {
    fn default() -> Self {
        Self::new()
    }
}
