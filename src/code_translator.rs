/// code_translator.rs — AXON programming query & structural operation translator
///
/// Handles the programming subset of the unified AXON notation: command verbs,
/// query types, structural code operations (field/method/type modifications),
/// and import/inheritance operators.
///
/// Syntax (part of AXON v2.0):
///   Commands:    >verb subject[:scope]
///   Queries:     ?type subject[:scope]
///   Transform:   subject→target
///   Structural:  @Type+.field  @Type.field=$val  @Type:impl(@Trait)
///   Import:      +use(module)  -use(module)

// ─── Action trigger table ────────────────────────────────────────────────────
//
// Each entry: (&[trigger phrases], axon_verb)
// Ordered longest-first within each group so greedy prefix match works.

const ACTION_PATTERNS: &[(&[&str], &str)] = &[
    // ── Documentation ────────────────────────────────────────────────────────
    (&[
        "create documentation for",
        "write documentation for",
        "generate documentation for",
        "create docs for",
        "write docs for",
        "generate docs for",
        "add documentation for",
        "add docs for",
        "create documentation about",
        "write documentation about",
        "document the",
        "document",
    ], "doc"),

    // ── Implementation ───────────────────────────────────────────────────────
    (&[
        "implement the feature",
        "implement a new",
        "implement the",
        "implement a",
        "implement",
        "build a new",
        "build the",
        "build a",
        "build",
        "create a new",
        "create the",
        "create a",
        "develop a new",
        "develop the",
        "develop a",
        "develop",
        "code a",
        "code the",
        "code up",
        "scaffold a",
        "scaffold the",
        "scaffold",
        "stub out",
        "wire up",
    ], "impl"),

    // ── Bug fixing ───────────────────────────────────────────────────────────
    (&[
        "fix the bug in",
        "fix the bug with",
        "fix the issue with",
        "fix the issue in",
        "fix the error in",
        "fix the error with",
        "fix the problem with",
        "fix the problem in",
        "debug the issue with",
        "debug the issue in",
        "debug the",
        "debug",
        "fix the",
        "fix a",
        "fix",
        "resolve the issue with",
        "resolve the issue in",
        "resolve the",
        "resolve",
        "troubleshoot the",
        "troubleshoot",
        "patch the",
        "patch",
        "hotfix",
    ], "fix"),

    // ── Testing ──────────────────────────────────────────────────────────────
    (&[
        "write unit tests for",
        "write integration tests for",
        "write end to end tests for",
        "write e2e tests for",
        "write tests for",
        "create unit tests for",
        "create integration tests for",
        "create tests for",
        "add unit tests for",
        "add integration tests for",
        "add tests for",
        "add test coverage for",
        "test the",
        "test",
    ], "test"),

    // ── Code review ──────────────────────────────────────────────────────────
    (&[
        "review the code in",
        "review the code for",
        "review the code of",
        "code review for",
        "code review of",
        "review the pull request for",
        "review the pr for",
        "review the",
        "review",
        "check the code in",
        "check the code for",
        "check the code of",
    ], "rev"),

    // ── Refactoring ──────────────────────────────────────────────────────────
    (&[
        "refactor the code in",
        "refactor the code for",
        "refactor the",
        "refactor",
        "restructure the",
        "restructure",
        "reorganize the",
        "reorganize",
        "reorganise the",
        "reorganise",
        "clean up the code in",
        "clean up the code for",
        "clean up the",
        "clean up",
        "simplify the",
        "simplify",
    ], "ref"),

    // ── Optimization ─────────────────────────────────────────────────────────
    (&[
        "optimize the performance of",
        "optimise the performance of",
        "optimize the",
        "optimise the",
        "optimize",
        "optimise",
        "improve the performance of",
        "improve performance of",
        "improve performance for",
        "speed up the",
        "speed up",
        "make faster",
        "reduce latency of",
        "reduce latency for",
        "reduce latency in",
        "reduce memory usage in",
        "reduce memory usage of",
    ], "opt"),

    // ── Planning / Design ────────────────────────────────────────────────────
    (&[
        "plan the following feature",
        "plan the implementation of",
        "plan the implementation for",
        "plan the feature",
        "plan the",
        "plan out",
        "plan",
        "design the architecture for",
        "design the architecture of",
        "design the system for",
        "design the",
        "design a",
        "design",
        "architect the",
        "architect a",
        "architect",
        "outline the approach for",
        "outline the approach to",
        "outline",
        "draft a plan for",
        "draft a plan to",
    ], "plan"),

    // ── Deployment ───────────────────────────────────────────────────────────
    (&[
        "deploy the application to",
        "deploy the app to",
        "deploy the service to",
        "deploy the",
        "deploy to",
        "deploy",
        "release the",
        "release a new version of",
        "release",
        "ship the",
        "ship to",
        "ship",
        "push to production",
        "push to staging",
        "push to",
        "publish the",
        "publish to",
        "publish",
    ], "dep"),

    // ── Add ──────────────────────────────────────────────────────────────────
    (&[
        "add a new feature for",
        "add a new feature to",
        "add a new",
        "add a feature for",
        "add a feature to",
        "add support for",
        "add the ability to",
        "add a",
        "add the",
        "add an",
        "add",
        "include support for",
        "include a",
        "include the",
        "include",
        "integrate with",
        "integrate the",
        "integrate a",
        "integrate",
        "introduce a",
        "introduce",
    ], "add"),

    // ── Remove ───────────────────────────────────────────────────────────────
    (&[
        "remove the dependency on",
        "remove the",
        "remove all",
        "remove",
        "delete the",
        "delete all",
        "delete",
        "drop the",
        "drop",
        "get rid of",
        "eliminate the",
        "eliminate",
        "deprecate the",
        "deprecate",
    ], "rm"),

    // ── Update / Upgrade ─────────────────────────────────────────────────────
    (&[
        "update the version of",
        "update the dependency",
        "update the",
        "update all",
        "update to",
        "update",
        "upgrade the version of",
        "upgrade the",
        "upgrade all",
        "upgrade to",
        "upgrade",
        "bump the version of",
        "bump the",
        "bump",
    ], "up"),

    // ── Rename / Move ────────────────────────────────────────────────────────
    (&[
        "rename the file",
        "rename the function",
        "rename the class",
        "rename the variable",
        "rename the method",
        "rename the",
        "rename",
        "move the file",
        "move the function",
        "move the class",
        "move the",
        "move",
    ], "mv"),

    // ── Configure / Setup ────────────────────────────────────────────────────
    (&[
        "configure the settings for",
        "configure the",
        "configure",
        "set up the environment for",
        "set up the",
        "set up a",
        "set up",
        "setup the",
        "setup a",
        "setup",
        "initialise the",
        "initialise",
        "initialize the",
        "initialize",
        "bootstrap the",
        "bootstrap a",
        "bootstrap",
    ], "cfg"),

    // ── Migrate ──────────────────────────────────────────────────────────────
    (&[
        "migrate the database",
        "migrate the data from",
        "migrate the codebase from",
        "migrate the code from",
        "migrate the application from",
        "migrate the",
        "migrate from",
        "migrate to",
        "migrate",
        "port the code from",
        "port the application from",
        "port the",
        "port from",
        "port to",
        "port",
        "convert the code from",
        "convert the",
        "convert from",
        "convert to",
        "convert",
    ], "mig"),

    // ── Benchmark / Profile ──────────────────────────────────────────────────
    (&[
        "benchmark the performance of",
        "benchmark the",
        "benchmark",
        "profile the performance of",
        "profile the",
        "profile",
        "measure the performance of",
        "measure the latency of",
        "measure",
        "load test the",
        "load test",
        "stress test the",
        "stress test",
    ], "bench"),

    // ── Lint / Format ────────────────────────────────────────────────────────
    (&[
        "lint the code in",
        "lint the",
        "lint",
        "format the code in",
        "format the",
        "format",
        "auto format",
        "autoformat",
        "run the linter on",
        "run the formatter on",
        "run linter",
        "run formatter",
    ], "lint"),

    // ── Logging ──────────────────────────────────────────────────────────────
    (&[
        "add logging to the",
        "add logging to",
        "add logging for the",
        "add logging for",
        "add logging",
        "add logs to the",
        "add logs to",
        "add logs for the",
        "add logs for",
        "add logs",
        "add tracing to the",
        "add tracing to",
        "add tracing for the",
        "add tracing for",
        "add tracing",
        "add observability to the",
        "add observability to",
        "add observability",
        "add metrics to the",
        "add metrics to",
        "add metrics for the",
        "add metrics for",
        "add metrics",
        "instrument the",
        "instrument",
    ], "log"),

    // ── Security ─────────────────────────────────────────────────────────────
    (&[
        "audit the security of",
        "audit the security for",
        "audit security for",
        "audit security of",
        "security audit of",
        "security audit for",
        "security audit",
        "run a security scan on",
        "run a security scan",
        "secure the",
        "secure",
        "harden the",
        "harden",
        "check for vulnerabilities in",
        "check for vulnerabilities",
        "scan for vulnerabilities in",
        "scan for vulnerabilities",
    ], "sec"),

    // ── Merge ────────────────────────────────────────────────────────────────
    (&[
        "merge the branch into",
        "merge the branch",
        "merge the pull request for",
        "merge the pull request",
        "merge the pr for",
        "merge the pr",
        "merge the",
        "merge into the",
        "merge into",
        "merge",
        "squash and merge the",
        "squash and merge",
        "rebase and merge the",
        "rebase and merge",
    ], "merge"),

    // ── Type annotations ─────────────────────────────────────────────────────
    (&[
        "add type annotations to the",
        "add type annotations to",
        "add type annotations for the",
        "add type annotations for",
        "add type annotations",
        "add types to the",
        "add types to",
        "add types for the",
        "add types for",
        "add types",
        "type check the",
        "type check",
        "add typing to the",
        "add typing to",
        "add typing for the",
        "add typing for",
        "add typing",
    ], "type"),

    // ── Database ─────────────────────────────────────────────────────────────
    (&[
        "create a database migration for",
        "create a database migration",
        "create a migration for",
        "create a migration",
        "write a database query for",
        "write a query for",
        "write a query to",
        "add a database index on",
        "add a database index for",
        "add an index on",
        "add an index for",
        "create a database table for",
        "create a table for",
        "create a table",
        "modify the database schema for",
        "modify the schema for",
        "modify the schema",
        "update the database schema",
        "update the schema",
    ], "db"),

    // ── API ──────────────────────────────────────────────────────────────────
    (&[
        "create an api endpoint for",
        "create an endpoint for",
        "add an api endpoint for",
        "add an endpoint for",
        "design the api for",
        "design the api",
        "create an api for",
        "build an api for",
        "add a rest endpoint for",
        "add a graphql query for",
        "add a graphql mutation for",
    ], "api"),

    // ── CI/CD ────────────────────────────────────────────────────────────────
    (&[
        "set up ci cd for the",
        "set up ci cd for",
        "set up ci for the",
        "set up ci for",
        "set up cd for the",
        "set up cd for",
        "set up the ci pipeline for the",
        "set up the ci pipeline for",
        "set up the cd pipeline for the",
        "set up the cd pipeline for",
        "set up a ci pipeline for the",
        "set up a ci pipeline for",
        "create a ci cd pipeline for the",
        "create a ci cd pipeline for",
        "create a ci pipeline for the",
        "create a ci pipeline for",
        "create a cd pipeline for the",
        "create a cd pipeline for",
        "add a ci step for",
        "add a ci job for",
        "add a ci step",
        "add a ci job",
        "configure ci for the",
        "configure ci for",
        "configure cd for the",
        "configure cd for",
        "configure the pipeline for the",
        "configure the pipeline for",
        "add a github actions workflow for",
        "add a github action for",
        "create a github actions workflow for",
        "create a github action for",
    ], "ci"),

    // ── Error handling ───────────────────────────────────────────────────────
    (&[
        "add error handling to the",
        "add error handling to",
        "add error handling for the",
        "add error handling for",
        "add error handling in the",
        "add error handling in",
        "add error handling",
        "improve error handling in the",
        "improve error handling in",
        "improve error handling for the",
        "improve error handling for",
        "improve error handling",
        "handle errors in the",
        "handle errors in",
        "handle errors for the",
        "handle errors for",
        "handle the error in the",
        "handle the error in",
        "handle the error",
        "add validation to the",
        "add validation to",
        "add validation for the",
        "add validation for",
        "add input validation to the",
        "add input validation to",
        "add input validation for the",
        "add input validation for",
        "validate the input for the",
        "validate the input for",
        "validate the input",
    ], "err"),

    // ── Explain (treated as action since it's a command) ─────────────────────
    (&[
        "explain how the",
        "explain how to",
        "explain how",
        "explain the code in",
        "explain the code for",
        "explain the",
        "explain why the",
        "explain why",
        "explain what",
        "explain",
        "describe how the",
        "describe how to",
        "describe how",
        "describe the",
        "describe",
        "walk me through the",
        "walk me through",
        "walk through the",
        "walk through",
    ], "explain"),
];

// ─── Query trigger table ─────────────────────────────────────────────────────

const QUERY_PATTERNS: &[(&[&str], &str)] = &[
    // ── How ──────────────────────────────────────────────────────────────────
    (&[
        "how do i go about",
        "how do i properly",
        "how do i correctly",
        "how do i",
        "how do you",
        "how do we",
        "how can i properly",
        "how can i correctly",
        "how can i",
        "how can we",
        "how should i",
        "how should we",
        "how would i",
        "how would you",
        "how to properly",
        "how to correctly",
        "how to",
        "how does the",
        "how does a",
        "how does",
        "how is the",
        "how is a",
        "how is",
        "how are",
        "what is the way to",
        "what's the way to",
        "what is the process for",
        "what's the process for",
        "what are the steps to",
    ], "how"),

    // ── Why ──────────────────────────────────────────────────────────────────
    (&[
        "why does the",
        "why does my",
        "why does this",
        "why does",
        "why is the",
        "why is my",
        "why is this",
        "why is",
        "why do i need to",
        "why do i need",
        "why do i have to",
        "why do we need to",
        "why do we need",
        "why do we",
        "why do",
        "why are the",
        "why are my",
        "why are",
        "why did the",
        "why did my",
        "why did",
        "why was the",
        "why was",
        "why doesn't the",
        "why doesn't my",
        "why doesn't",
        "why isn't the",
        "why isn't my",
        "why isn't",
        "why can't i",
        "why can't",
        "why won't the",
        "why won't my",
        "why won't",
        "why would",
        "why should i",
        "why should we",
    ], "why"),

    // ── Best practice ────────────────────────────────────────────────────────
    (&[
        "what is the best approach to",
        "what is the best way to",
        "what is the best practice for",
        "what is the best method for",
        "what is the best strategy for",
        "what's the best approach to",
        "what's the best way to",
        "what's the best practice for",
        "what's the best method for",
        "what's the best strategy for",
        "what is the recommended way to",
        "what is the recommended approach to",
        "what's the recommended way to",
        "what's the recommended approach to",
        "best practice for",
        "best practices for",
        "best approach for",
        "best approach to",
        "best way to",
        "recommended way to",
        "recommended approach to",
        "recommended approach for",
        "what approach should i use for",
        "what approach should i use to",
        "what pattern should i use for",
        "what design pattern should i use for",
        "idiomatic way to",
    ], "best"),

    // ── Difference / Compare ─────────────────────────────────────────────────
    (&[
        "what is the difference between",
        "what's the difference between",
        "what are the differences between",
        "difference between",
        "differences between",
        "how does it differ from",
        "how do they differ",
        "how is it different from",
    ], "diff"),

    // ── What ─────────────────────────────────────────────────────────────────
    (&[
        "what is a",
        "what is an",
        "what is the",
        "what is",
        "what's a",
        "what's an",
        "what's the",
        "what's",
        "what are the",
        "what are",
        "what does the",
        "what does a",
        "what does",
        "what do the",
        "what do",
    ], "what"),

    // ── When ─────────────────────────────────────────────────────────────────
    (&[
        "when should i use a",
        "when should i use",
        "when should we use",
        "when to use a",
        "when to use",
        "when is it appropriate to use",
        "when is it appropriate to",
        "when is it best to",
        "when is the right time to",
        "when would i use",
        "when would you use",
    ], "when"),

    // ── Where ────────────────────────────────────────────────────────────────
    (&[
        "where is the code for",
        "where is the file for",
        "where is the definition of",
        "where is the implementation of",
        "where is the configuration for",
        "where is the",
        "where is",
        "where can i find the",
        "where can i find",
        "where does the",
        "where does",
        "where do i put",
        "where do i find",
        "where do i",
        "where should i put",
        "where should i place",
        "where should i define",
        "find the definition of",
        "find the implementation of",
        "find the",
        "find where",
        "locate the",
        "locate",
    ], "where"),

    // ── Can / Possible ───────────────────────────────────────────────────────
    (&[
        "is it possible to",
        "is there a way to",
        "is there any way to",
        "can i use",
        "can i",
        "can we",
        "can you",
        "would it be possible to",
        "is there support for",
        "does it support",
    ], "can"),

    // ── Compare ──────────────────────────────────────────────────────────────
    (&[
        "which is better for",
        "which is better",
        "which one should i use for",
        "which one should i use",
        "which should i use for",
        "which should i use",
        "should i use x or y for",
        "should i use",
        "compare the",
        "compare",
        "x vs y",
        "versus",
        "pros and cons of",
        "trade offs of",
        "tradeoffs of",
        "trade-offs of",
    ], "cmp"),

    // ── Alternatives ─────────────────────────────────────────────────────────
    (&[
        "what are the alternatives to",
        "what are the alternatives for",
        "what are alternatives to",
        "alternatives to",
        "alternative to",
        "what else can i use instead of",
        "what can i use instead of",
        "replacement for",
        "substitute for",
        "instead of",
    ], "alt"),

    // ── Error explanation ────────────────────────────────────────────────────
    (&[
        "what does this error mean",
        "what does the error mean",
        "what does this exception mean",
        "what does the error",
        "why am i getting this error",
        "why am i getting the error",
        "why am i getting an error",
        "why am i getting",
        "what causes the error",
        "what causes this error",
        "how to fix the error",
        "how to fix this error",
        "how to resolve the error",
        "how to resolve this error",
    ], "err"),

    // ── Performance ──────────────────────────────────────────────────────────
    (&[
        "why is this slow",
        "why is it slow",
        "why is the",
        "what is the performance of",
        "what's the performance of",
        "performance of",
        "how fast is",
        "how efficient is",
        "what is the time complexity of",
        "what's the time complexity of",
        "what is the space complexity of",
        "time complexity of",
        "space complexity of",
        "big o of",
    ], "perf"),
];

// ─── Noise phrases specific to code questions ────────────────────────────────

const CODE_NOISE: &[&str] = &[
    "could you please",
    "can you please",
    "would you please",
    "would you mind",
    "i need you to",
    "i want you to",
    "i'd like you to",
    "i would like you to",
    "i need to",
    "i want to",
    "i'd like to",
    "i would like to",
    "help me to",
    "help me",
    "assist me with",
    "assist me in",
    "assist with",
    "could you",
    "can you",
    "would you",
    "please go ahead and",
    "go ahead and",
    "i was wondering if you could",
    "i was wondering if you can",
    "do you think you could",
    "do you think you can",
    "it would be great if you could",
    "it would be nice if you could",
    "if possible",
    "if you can",
    "when you get a chance",
    "at your earliest convenience",
    "as soon as possible",
    "asap",
    "thanks in advance",
    "thank you in advance",
    "thank you",
    "thanks",
    "cheers",
    "please",
    "kindly",
    "hi",
    "hey",
    "hello",
    "good morning",
    "good afternoon",
];

// ─── Scope prepositions ──────────────────────────────────────────────────────

const SCOPE_PREPS: &[&str] = &[
    " in the ", " in my ", " in our ", " in this ", " in a ", " in an ", " in ",
    " for the ", " for my ", " for our ", " for this ", " for a ", " for an ", " for ",
    " of the ", " of my ", " of our ", " of this ", " of a ", " of an ", " of ",
    " on the ", " on my ", " on our ", " on this ", " on ",
    " within the ", " within my ", " within our ", " within ",
    " across the ", " across my ", " across our ", " across ",
    " inside the ", " inside my ", " inside ",
];

// ─── Transform prepositions ─────────────────────────────────────────────────

const TRANSFORM_PREPS: &[&str] = &[
    " to use ", " to ", " into ", " from ",
];

// ─── Articles and filler to strip from subjects ─────────────────────────────

const SUBJECT_NOISE: &[&str] = &[
    "the ", "a ", "an ", "my ", "our ", "this ", "that ", "these ", "those ",
    "some ", "any ", "all ", "new ", "existing ", "current ", "following ",
    "entire ", "whole ", "specific ", "particular ", "given ",
];

// ─── Structural operation tables ─────────────────────────────────────────────
//
// These handle fine-grained code modifications: field/method/parameter/type
// operations on structs, classes, functions, enums, etc.
//
// Output syntax:
//   @Target.member            field reference
//   @Target+.field            add field
//   @Target-.field            remove field
//   @Target.field→.newname    rename field
//   @Target.field:Type        set field type
//   @Target.field=$value      set field value
//   @Target+~method           add method
//   @Target-~method           remove method
//   ~func+$param              add parameter
//   ~func-$param              remove parameter
//   ~func→:RetType            change return type
//   @Type:impl(@Trait)        implement trait/interface
//   @Type<@Base               extend/inherit
//   +use(@Module)             add import
//   -use(@Module)             remove import
//   @Target.field+pub         change visibility
//   @Target~method+async      add modifier

/// Member-type keywords → canonical label
const MEMBER_KEYWORDS: &[(&[&str], &str)] = &[
    (&["return type", "return value"], "return"),
    (&["field", "property", "attribute", "member", "column", "key"], "field"),
    (&["method", "function", "func", "handler", "callback", "hook"], "method"),
    (&["parameter", "param", "argument", "arg"], "param"),
    (&["variant", "enum case", "case", "enum variant", "enum member"], "variant"),
    (&["constructor", "initializer", "init", "ctor"], "constructor"),
    (&["getter", "accessor"], "getter"),
    (&["setter", "mutator"], "setter"),
    (&["interface", "trait", "protocol", "contract"], "trait"),
    (&["import", "dependency", "package", "module", "require", "use statement"], "import"),
];

/// Target-type keywords — structural decoration stripped from type names.
/// Only includes words that are NEVER meaningful type names on their own.
const TARGET_KEYWORDS: &[&str] = &[
    "struct", "class", "type", "enum", "object",
    "function", "method", "func", "fn", "procedure", "routine",
    "variable", "var", "let", "const", "constant",
];

/// Visibility / modifier keywords
const MODIFIERS: &[(&str, &str)] = &[
    ("public", "pub"),
    ("private", "priv"),
    ("protected", "prot"),
    ("internal", "int"),
    ("async", "async"),
    ("synchronous", "sync"),
    ("static", "static"),
    ("const", "const"),
    ("constant", "const"),
    ("mutable", "mut"),
    ("immutable", "immut"),
    ("readonly", "readonly"),
    ("read-only", "readonly"),
    ("abstract", "abstract"),
    ("virtual", "virtual"),
    ("override", "override"),
    ("final", "final"),
    ("sealed", "sealed"),
    ("optional", "opt"),
    ("required", "req"),
    ("nullable", "nullable"),
    ("non-nullable", "non-null"),
    ("deprecated", "depr"),
    ("exported", "export"),
    ("generic", "generic"),
    ("lazy", "lazy"),
    ("volatile", "volatile"),
    ("transient", "transient"),
    ("inline", "inline"),
    ("recursive", "rec"),
    ("pure", "pure"),
    ("unsafe", "unsafe"),
];

/// Preposition patterns that separate member from target
const TARGET_PREPS: &[&str] = &[
    " on the ", " on my ", " on our ", " on this ", " on ",
    " in the ", " in my ", " in our ", " in this ", " in ",
    " of the ", " of my ", " of our ", " of this ", " of ",
    " from the ", " from my ", " from our ", " from this ", " from ",
    " to the ", " to my ", " to our ", " to this ", " to ",
    " for the ", " for my ", " for our ", " for this ", " for ",
    " within the ", " within ",
    " inside the ", " inside ",
];

// ─── Public types ────────────────────────────────────────────────────────────

pub struct CodeResult {
    pub axon: String,
    pub annotation: String,
    pub matched: bool,
}

// ─── CodeTranslator ──────────────────────────────────────────────────────────

pub struct CodeTranslator;

impl CodeTranslator {
    pub fn new() -> Self {
        Self
    }

    /// Attempt to translate a programming/project question into AXON-Code.
    /// Returns matched=false if the input does not look like a code question.
    pub fn translate(&self, input: &str) -> CodeResult {
        let cleaned = self.strip_noise(input);
        if cleaned.is_empty() {
            return CodeResult { axon: String::new(), annotation: String::new(), matched: false };
        }

        let lower = cleaned.to_lowercase();

        // Try structural code operations first (field/method/param/type ops)
        if let Some((axon, annotation)) = self.match_structural(&lower) {
            return CodeResult { axon, annotation, matched: true };
        }

        // Try action patterns (commands) — longest-match wins
        if let Some((verb, rest)) = self.match_action(&lower) {
            let (subject, scope, transform) = self.extract_parts(&rest);
            let subject = self.clean_subject(&subject);
            let axon = self.format_action(verb, &subject, &scope, &transform);
            let annotation = if subject.is_empty() {
                format!("code-opt · action({})", verb)
            } else {
                format!("code-opt · action({}) · subject({}){}{}", verb, subject,
                    if scope.is_empty() { String::new() } else { format!(" · scope({})", scope) },
                    if transform.is_empty() { String::new() } else { format!(" · target({})", transform) })
            };
            return CodeResult { axon, annotation, matched: true };
        }

        // Try query patterns — longest-match wins
        if let Some((qtype, rest)) = self.match_query(&lower) {
            let (subject, scope, transform) = self.extract_parts(&rest);
            let subject = self.clean_subject(&subject);
            if subject.is_empty() {
                return CodeResult { axon: String::new(), annotation: String::new(), matched: false };
            }
            let axon = self.format_query(qtype, &subject, &scope, &transform);
            let annotation = format!("code-opt · query({}) · subject({}){}{}", qtype, subject,
                if scope.is_empty() { String::new() } else { format!(" · scope({})", scope) },
                if transform.is_empty() { String::new() } else { format!(" · target({})", transform) });
            return CodeResult { axon, annotation, matched: true };
        }

        CodeResult { axon: String::new(), annotation: String::new(), matched: false }
    }

    // ── Noise stripping ──────────────────────────────────────────────────────

    fn strip_noise(&self, text: &str) -> String {
        let mut result = text.trim().to_string();

        // Remove trailing punctuation
        while result.ends_with('.') || result.ends_with('?') || result.ends_with('!') {
            result.pop();
        }
        result = result.trim().to_string();

        // Strip noise prefixes (iterate until stable)
        let mut changed = true;
        while changed {
            changed = false;
            let lower = result.to_lowercase();
            for phrase in CODE_NOISE {
                if lower.starts_with(phrase) {
                    result = result[phrase.len()..].trim().to_string();
                    changed = true;
                    break;
                }
            }
        }

        result.trim().to_string()
    }

    // ── Pattern matching ─────────────────────────────────────────────────────

    /// Find the action pattern with the longest matching trigger phrase.
    fn match_action(&self, lower: &str) -> Option<(&'static str, String)> {
        let mut best: Option<(&'static str, usize)> = None;
        for (triggers, verb) in ACTION_PATTERNS {
            for trigger in *triggers {
                if lower.starts_with(trigger) {
                    let len = trigger.len();
                    if best.is_none() || len > best.unwrap().1 {
                        best = Some((verb, len));
                    }
                }
            }
        }
        best.map(|(verb, len)| (verb, lower[len..].trim().to_string()))
    }

    /// Find the query pattern with the longest matching trigger phrase.
    fn match_query(&self, lower: &str) -> Option<(&'static str, String)> {
        let mut best: Option<(&'static str, usize)> = None;
        for (triggers, qtype) in QUERY_PATTERNS {
            for trigger in *triggers {
                if lower.starts_with(trigger) {
                    let len = trigger.len();
                    if best.is_none() || len > best.unwrap().1 {
                        best = Some((qtype, len));
                    }
                }
            }
        }
        best.map(|(qtype, len)| (qtype, lower[len..].trim().to_string()))
    }

    // ── Structural code operations ─────────────────────────────────────────

    /// Detect and translate fine-grained code operations.
    /// e.g. "add a field called email to the User struct" → "@User+.email"
    fn match_structural(&self, lower: &str) -> Option<(String, String)> {
        // 1. Try add-member pattern
        if let Some(r) = self.match_add_member(lower) { return Some(r); }
        // 2. Try remove-member pattern
        if let Some(r) = self.match_remove_member(lower) { return Some(r); }
        // 3. Try rename-member pattern
        if let Some(r) = self.match_rename_member(lower) { return Some(r); }
        // 4. Try change-type pattern (before update, since "change the type" is more specific)
        if let Some(r) = self.match_change_type(lower) { return Some(r); }
        // 5. Try change-return-type pattern
        if let Some(r) = self.match_change_return_type(lower) { return Some(r); }
        // 6. Try update/set/change member pattern
        if let Some(r) = self.match_update_member(lower) { return Some(r); }
        // 7. Try make-modifier pattern (make X public/async/static)
        if let Some(r) = self.match_make_modifier(lower) { return Some(r); }
        // 8. Try implement/extend pattern
        if let Some(r) = self.match_type_relation(lower) { return Some(r); }
        // 9. Try import pattern
        if let Some(r) = self.match_import_op(lower) { return Some(r); }
        None
    }

    /// "add a field called email to the User struct" → "@User+.email"
    /// "add a method validate to the Order class" → "@Order+~validate"
    /// "add a parameter timeout to the connect function" → "~connect+$timeout"
    /// "add a variant Pending to the Status enum" → "@Status+.Pending"
    /// "add a constructor to the User class" → "@User+~new"
    /// "add a getter for name on the User struct" → "@User+~get.name"
    fn match_add_member(&self, lower: &str) -> Option<(String, String)> {
        // Must start with "add a/an/the" or "add "
        let rest = if lower.starts_with("add a ") {
            &lower[6..]
        } else if lower.starts_with("add an ") {
            &lower[7..]
        } else if lower.starts_with("add the ") {
            &lower[8..]
        } else if lower.starts_with("add new ") {
            &lower[8..]
        } else if lower.starts_with("add ") {
            &lower[4..]
        } else if lower.starts_with("create a ") {
            &lower[9..]
        } else if lower.starts_with("create an ") {
            &lower[10..]
        } else if lower.starts_with("create new ") {
            &lower[11..]
        } else if lower.starts_with("insert a ") {
            &lower[9..]
        } else if lower.starts_with("insert an ") {
            &lower[10..]
        } else {
            return None;
        };

        // Detect member type keyword — skip import/trait (handled by dedicated matchers)
        let (member_type, after_type) = self.detect_member_keyword(rest)?;
        if member_type == "import" || member_type == "trait" {
            return None;
        }

        // Extract member name and target
        let (member_name, target) = self.extract_name_and_target(after_type, member_type);
        let member_name = self.clean_ident(&member_name);
        let target = self.clean_ident(&target);

        if target.is_empty() && member_name.is_empty() {
            return None;
        }

        let (sigil, mem_sigil) = self.member_sigils(member_type);
        let axon = if target.is_empty() {
            format!("{}+{}{}", sigil, mem_sigil, member_name)
        } else if member_name.is_empty() {
            // e.g. "add a constructor to User" → "@User+~new"
            let default_name = match member_type {
                "constructor" => "new",
                "getter" => "get",
                "setter" => "set",
                _ => return None,
            };
            format!("@{}+~{}", target, default_name)
        } else {
            match member_type {
                "field" | "variant" => format!("@{}+.{}", target, member_name),
                "method" | "constructor" => format!("@{}+~{}", target, member_name),
                "param" => format!("~{}+${}", target, member_name),
                "getter" => format!("@{}+~get.{}", target, member_name),
                "setter" => format!("@{}+~set.{}", target, member_name),
                _ => format!("@{}+{}{}", target, mem_sigil, member_name),
            }
        };

        let annotation = format!("code-opt · struct(add) · {}({}) · target({})", member_type, member_name, target);
        Some((axon, annotation))
    }

    /// "remove the field email from the User struct" → "@User-.email"
    /// "remove the method validate from the Order class" → "@Order-~validate"
    /// "remove the parameter timeout from the connect function" → "~connect-$timeout"
    /// "delete the age field from User" → "@User-.age"
    fn match_remove_member(&self, lower: &str) -> Option<(String, String)> {
        let rest = if lower.starts_with("remove the ") {
            &lower[11..]
        } else if lower.starts_with("remove a ") {
            &lower[9..]
        } else if lower.starts_with("remove ") {
            &lower[7..]
        } else if lower.starts_with("delete the ") {
            &lower[11..]
        } else if lower.starts_with("delete a ") {
            &lower[9..]
        } else if lower.starts_with("delete ") {
            &lower[7..]
        } else if lower.starts_with("drop the ") {
            &lower[9..]
        } else if lower.starts_with("drop ") {
            &lower[5..]
        } else {
            return None;
        };

        // Detect member type keyword — skip "import"/"dependency" (handled by match_import_op)
        let (member_type, after_type) = self.detect_member_keyword(rest)?;
        if member_type == "import" {
            return None; // Let match_import_op handle this
        }

        // Extract name and target
        let (member_name, target) = self.extract_name_and_target_from(after_type);
        let member_name = self.clean_ident(&member_name);
        let target = self.clean_ident(&target);

        if member_name.is_empty() {
            return None;
        }

        let axon = match member_type {
            "field" | "variant" => {
                if target.is_empty() {
                    format!("-.{}", member_name)
                } else {
                    format!("@{}-.{}", target, member_name)
                }
            }
            "method" | "constructor" => {
                if target.is_empty() {
                    format!("-~{}", member_name)
                } else {
                    format!("@{}-~{}", target, member_name)
                }
            }
            "param" => {
                if target.is_empty() {
                    format!("-${}", member_name)
                } else {
                    format!("~{}-${}", target, member_name)
                }
            }
            _ => {
                if target.is_empty() {
                    format!("-{}", member_name)
                } else {
                    format!("@{}-{}", target, member_name)
                }
            }
        };

        let annotation = format!("code-opt · struct(rm) · {}({}) · target({})", member_type, member_name, target);
        Some((axon, annotation))
    }

    /// "rename the field name to fullName on the User struct" → "@User.name→.fullName"
    /// "rename the method process to handle in the Worker class" → "@Worker~process→~handle"
    fn match_rename_member(&self, lower: &str) -> Option<(String, String)> {
        let rest = if lower.starts_with("rename the ") {
            &lower[11..]
        } else if lower.starts_with("rename a ") {
            &lower[9..]
        } else if lower.starts_with("rename ") {
            &lower[7..]
        } else {
            return None;
        };

        // Detect member type keyword
        let (member_type, after_type) = self.detect_member_keyword(rest)?;

        // Extract old name
        let (old_name, rest_after_name) = self.split_at_first_word(after_type);
        let old_name = self.clean_ident(&old_name);

        if old_name.is_empty() {
            return None;
        }

        // Find "to" keyword for new name — handle both " to " and leading "to "
        let after_to = if rest_after_name.starts_with("to ") {
            rest_after_name[3..].trim()
        } else if let Some(to_pos) = rest_after_name.find(" to ") {
            rest_after_name[to_pos + 4..].trim()
        } else {
            return None;
        };

        // Split: new_name might be followed by "on/in/of TARGET"
        let (new_name, target) = self.extract_name_and_target_on(after_to);
        let new_name = self.clean_ident(&new_name);
        let target = self.clean_ident(&target);

        if new_name.is_empty() {
            return None;
        }

        let mem_sigil = match member_type {
            "field" | "variant" => ".",
            "method" | "constructor" => "~",
            "param" => "$",
            _ => ".",
        };

        let axon = if target.is_empty() {
            format!("{}{}{}{}", mem_sigil, old_name, "→", format!("{}{}", mem_sigil, new_name))
        } else {
            format!("@{}{}{}{}{}{}", target, mem_sigil, old_name, "→", mem_sigil, new_name)
        };

        let annotation = format!("code-opt · struct(rename) · {}({}) → {}", member_type, old_name, new_name);
        Some((axon, annotation))
    }

    /// "update the name field on the User struct to Aaron" → "@User.name=$Aaron"
    /// "set the status field on the Order to shipped" → "@Order.status=$shipped"
    /// "change the value of count in the Counter to 0" → "@Counter.count=$0"
    fn match_update_member(&self, lower: &str) -> Option<(String, String)> {
        let rest = if lower.starts_with("update the ") {
            &lower[11..]
        } else if lower.starts_with("set the ") {
            &lower[8..]
        } else if lower.starts_with("change the value of ") {
            &lower[20..]
        } else if lower.starts_with("change the ") {
            // could be "change the X field on Y to Z" or "change the type of..."
            // Only handle if we can detect a field pattern
            &lower[11..]
        } else if lower.starts_with("modify the ") {
            &lower[11..]
        } else {
            return None;
        };

        // Look for field/member keyword, or assume field if we see "X on/in Y to Z"
        let (_member_type, after_type) = self.detect_member_keyword(rest)
            .unwrap_or(("field", rest));

        // We need to find both a target (on/in) and a value (to)
        // Pattern: "NAME (on/in TARGET) to VALUE"
        let to_pos = after_type.find(" to ");
        if to_pos.is_none() {
            return None;
        }
        let to_pos = to_pos.unwrap();
        let before_to = &after_type[..to_pos];
        let value = after_type[to_pos + 4..].trim();

        if value.is_empty() {
            return None;
        }

        // Extract member name and target from before_to
        let (member_name, target) = self.extract_name_and_target_on(before_to);
        let member_name = self.strip_member_words(&member_name);
        let member_name = self.clean_ident(&member_name);
        let target = self.strip_target_keywords(&target);
        let target = self.clean_ident(&target);
        let value = self.clean_ident(value);

        if member_name.is_empty() || value.is_empty() {
            return None;
        }

        let axon = if target.is_empty() {
            format!(".{}=${}", member_name, value)
        } else {
            format!("@{}.{}=${}", target, member_name, value)
        };

        let annotation = format!("code-opt · struct(set) · {}.{}={}", target, member_name, value);
        Some((axon, annotation))
    }

    /// "change the type of the name field on User to string" → "@User.name:string"
    /// "change the type of email from string to Email" → ".email:string→Email"
    fn match_change_type(&self, lower: &str) -> Option<(String, String)> {
        let rest = if lower.starts_with("change the type of the ") {
            &lower[23..]
        } else if lower.starts_with("change the type of ") {
            &lower[19..]
        } else if lower.starts_with("change the data type of the ") {
            &lower[28..]
        } else if lower.starts_with("change the data type of ") {
            &lower[24..]
        } else {
            return None;
        };

        // Pattern: "MEMBER (field/param) (on/in/of TARGET) (from OLD) to NEW"
        let to_pos = rest.rfind(" to ")?;
        let new_type = self.clean_ident(rest[to_pos + 4..].trim());
        let before_to = &rest[..to_pos];

        // Check for "from OLD" pattern
        let (before_from, old_type) = if let Some(from_pos) = before_to.rfind(" from ") {
            (&before_to[..from_pos], Some(self.clean_ident(before_to[from_pos + 6..].trim())))
        } else {
            (before_to, None)
        };

        // Try to detect member keyword
        let (member_type, after_kw) = self.detect_member_keyword(before_from)
            .unwrap_or(("field", before_from));

        let (member_name, target) = self.extract_name_and_target_on(after_kw);
        let member_name = self.strip_member_words(&member_name);
        let member_name = self.clean_ident(&member_name);
        let target = self.strip_target_keywords(&target);
        let target = self.clean_ident(&target);

        if member_name.is_empty() || new_type.is_empty() {
            return None;
        }

        let mem_sigil = match member_type {
            "field" | "variant" => ".",
            "method" => "~",
            "param" => "$",
            _ => ".",
        };

        let axon = if target.is_empty() {
            if let Some(old) = &old_type {
                format!("{}{}:{}→{}", mem_sigil, member_name, old, new_type)
            } else {
                format!("{}{}:{}", mem_sigil, member_name, new_type)
            }
        } else {
            if let Some(old) = &old_type {
                format!("@{}{}{}:{}→{}", target, mem_sigil, member_name, old, new_type)
            } else {
                format!("@{}{}{}:{}", target, mem_sigil, member_name, new_type)
            }
        };

        let annotation = format!("code-opt · struct(type) · {}.{}:{}", target, member_name, new_type);
        Some((axon, annotation))
    }

    /// "change the return type of the process function to Result" → "~process→:Result"
    /// "change the return type of validate to bool" → "~validate→:bool"
    fn match_change_return_type(&self, lower: &str) -> Option<(String, String)> {
        let rest = if lower.starts_with("change the return type of the ") {
            &lower[30..]
        } else if lower.starts_with("change the return type of ") {
            &lower[25..]
        } else if lower.starts_with("set the return type of the ") {
            &lower[27..]
        } else if lower.starts_with("set the return type of ") {
            &lower[23..]
        } else {
            return None;
        };

        let to_pos = rest.find(" to ")?;
        let func_part = rest[..to_pos].trim();
        let new_type = self.clean_ident(rest[to_pos + 4..].trim());

        if new_type.is_empty() {
            return None;
        }

        // Strip target-type and member keywords from func name
        let func_name = self.strip_target_keywords(func_part);
        let func_name = self.strip_member_words(&func_name);
        let func_name = self.clean_ident(&func_name);

        if func_name.is_empty() {
            return None;
        }

        let axon = format!("~{}→:{}", func_name, new_type);
        let annotation = format!("code-opt · struct(rettype) · ~{}→:{}", func_name, new_type);
        Some((axon, annotation))
    }

    /// "make the name field on User public" → "@User.name+pub"
    /// "make the process method async" → "~process+async"
    /// "make User serializable" → "@User+serial"
    /// "make the Handler class abstract" → "@Handler+abstract"
    fn match_make_modifier(&self, lower: &str) -> Option<(String, String)> {
        if !lower.starts_with("make ") {
            return None;
        }
        let rest = &lower[5..];
        // Strip "the "
        let rest = if rest.starts_with("the ") { &rest[4..] } else { rest };

        // The modifier is the last word — check it
        let words: Vec<&str> = rest.split_whitespace().collect();
        if words.is_empty() {
            return None;
        }

        // Check last word(s) against modifiers
        let last_word = words[words.len() - 1];
        let modifier = MODIFIERS.iter().find(|(kw, _)| kw == &last_word);
        if modifier.is_none() {
            return None;
        }
        let (_, mod_short) = modifier.unwrap();

        let before_mod = words[..words.len() - 1].join(" ");
        let before_mod = before_mod.trim();

        // Try to detect member keyword — both at start ("field name on X")
        // and in middle ("name field on X")
        if let Some((member_type, after_kw)) = self.detect_member_keyword(before_mod) {
            // Member keyword at start: "field name on User"
            let (member_name, target) = self.extract_name_and_target_on(after_kw);
            let member_name = self.clean_ident(&member_name);
            let target = self.strip_target_keywords(&target);
            let target = self.clean_ident(&target);

            // Only use this branch if we got a real member name
            if !member_name.is_empty() {
                let mem_sigil = match member_type {
                    "field" | "variant" => ".",
                    "method" | "constructor" => "~",
                    "param" => "$",
                    _ => ".",
                };

                let axon = if target.is_empty() {
                    format!("{}{}+{}", mem_sigil, member_name, mod_short)
                } else {
                    format!("@{}{}{}+{}", target, mem_sigil, member_name, mod_short)
                };
                let annotation = format!("code-opt · struct(mod) · {}+{}", member_name, mod_short);
                return Some((axon, annotation));
            }
        }

        // Try member keyword in middle: "name field on User"
        if let Some((member_type, member_name, target)) = self.detect_member_keyword_mid(before_mod) {
            let member_name = self.clean_ident(&member_name);
            let target = self.strip_target_keywords(&target);
            let target = self.clean_ident(&target);

            let mem_sigil = match member_type {
                "field" | "variant" => ".",
                "method" | "constructor" => "~",
                "param" => "$",
                _ => ".",
            };

            let axon = if target.is_empty() {
                format!("{}{}+{}", mem_sigil, member_name, mod_short)
            } else {
                format!("@{}{}{}+{}", target, mem_sigil, member_name, mod_short)
            };
            let annotation = format!("code-opt · struct(mod) · {}+{}", member_name, mod_short);
            return Some((axon, annotation));
        }

        // No member keyword — treat as type modifier
        // "make User public" → "@User+pub"
        // "make the handler async" → "@handler+async"
        let target = self.strip_target_keywords(before_mod);
        let target = self.clean_ident(&target);
        if target.is_empty() {
            return None;
        }

        let axon = format!("@{}+{}", target, mod_short);
        let annotation = format!("code-opt · struct(mod) · @{}+{}", target, mod_short);
        Some((axon, annotation))
    }

    /// "implement the Serializable interface on/for the User struct" → "@User:impl(@Serializable)"
    /// "extend the BaseController class with UserController" → "@UserController<@BaseController"
    /// "have User implement Display" → "@User:impl(@Display)"
    fn match_type_relation(&self, lower: &str) -> Option<(String, String)> {
        // Implement patterns — only match when there's a clear "on/for TARGET" structure
        // otherwise "implement caching" should fall through to action patterns
        if lower.starts_with("implement ") || lower.starts_with("impl ") {
            let rest = if lower.starts_with("implement ") { &lower[10..] } else { &lower[5..] };
            let rest = if rest.starts_with("the ") { &rest[4..] } else { rest };

            // Only match as structural if there's a clear trait/interface keyword,
            // OR the sentence uses "on" (not just "for", which is ambiguous with
            // "implement feature for project").
            let has_trait_keyword = ["interface", "trait", "protocol", "contract"]
                .iter().any(|kw| rest.contains(kw));
            let has_on_prep = [" on the ", " on "].iter().any(|p| rest.contains(p));
            if !has_trait_keyword && !has_on_prep {
                // Ambiguous — fall through to action patterns (">impl")
            } else {
                let (trait_part, target_part) = self.split_at_prep(rest,
                    &[" on the ", " on ", " for the ", " for "]);

                let trait_name = self.strip_target_keywords(trait_part.trim());
                let trait_name = self.strip_member_keywords(&trait_name);
                let trait_name = self.clean_ident(&trait_name);

                let target = self.strip_target_keywords(target_part.trim());
                let target = self.clean_ident(&target);

                if !trait_name.is_empty() && !target.is_empty() {
                    let axon = format!("@{}:impl(@{})", target, trait_name);
                    let annotation = format!("code-opt · struct(impl) · @{}:impl(@{})", target, trait_name);
                    return Some((axon, annotation));
                }
            }
        }

        // "have X implement Y"
        if lower.starts_with("have ") || lower.starts_with("make ") {
            let rest = &lower[5..];
            let rest = if rest.starts_with("the ") { &rest[4..] } else { rest };

            if let Some(impl_pos) = rest.find(" implement ") {
                let target = self.strip_target_keywords(&rest[..impl_pos]);
                let target = self.clean_ident(&target);
                let trait_name = self.strip_target_keywords(rest[impl_pos + 11..].trim());
                let trait_name = self.strip_member_keywords(&trait_name);
                let trait_name = self.clean_ident(&trait_name);

                if !target.is_empty() && !trait_name.is_empty() {
                    let axon = format!("@{}:impl(@{})", target, trait_name);
                    let annotation = format!("code-opt · struct(impl) · @{}:impl(@{})", target, trait_name);
                    return Some((axon, annotation));
                }
            }
        }

        // Extend patterns
        if lower.starts_with("extend ") {
            let rest = &lower[7..];
            let rest = if rest.starts_with("the ") { &rest[4..] } else { rest };

            let (base_part, child_part) = self.split_at_prep(rest,
                &[" with the ", " with ", " using the ", " using "]);

            let base = self.strip_target_keywords(base_part.trim());
            let base = self.clean_ident(&base);
            let child = self.strip_target_keywords(child_part.trim());
            let child = self.clean_ident(&child);

            if base.is_empty() {
                return None;
            }

            let axon = if child.is_empty() {
                format!("<@{}", base)
            } else {
                format!("@{}<@{}", child, base)
            };
            let annotation = format!("code-opt · struct(extend) · @{}<@{}", child, base);
            return Some((axon, annotation));
        }

        // "have X extend Y" / "make X extend Y"
        if lower.starts_with("have ") || lower.starts_with("make ") {
            let rest = &lower[5..];
            let rest = if rest.starts_with("the ") { &rest[4..] } else { rest };

            if let Some(ext_pos) = rest.find(" extend ") {
                let child = self.strip_target_keywords(&rest[..ext_pos]);
                let child = self.clean_ident(&child);
                let base = self.strip_target_keywords(rest[ext_pos + 8..].trim());
                let base = self.clean_ident(&base);

                if !child.is_empty() && !base.is_empty() {
                    let axon = format!("@{}<@{}", child, base);
                    let annotation = format!("code-opt · struct(extend) · @{}<@{}", child, base);
                    return Some((axon, annotation));
                }
            }
        }

        None
    }

    /// "add an import for React" → "+use(react)"
    /// "import the lodash module" → "+use(lodash)"
    /// "remove the import for axios" → "-use(axios)"
    fn match_import_op(&self, lower: &str) -> Option<(String, String)> {
        // Add import
        let is_add = lower.starts_with("add an import for ") ||
            lower.starts_with("add import for ") ||
            lower.starts_with("add the import for ") ||
            lower.starts_with("add import ") ||
            lower.starts_with("import the ") ||
            lower.starts_with("import ");
        let is_remove = lower.starts_with("remove the import for ") ||
            lower.starts_with("remove import for ") ||
            lower.starts_with("remove the import ") ||
            lower.starts_with("remove import ");

        if !is_add && !is_remove {
            return None;
        }

        let prefixes_add = &[
            "add an import for the ", "add an import for ",
            "add the import for the ", "add the import for ",
            "add import for the ", "add import for ",
            "add import ", "import the ", "import ",
        ];
        let prefixes_rm = &[
            "remove the import for the ", "remove the import for ",
            "remove import for the ", "remove import for ",
            "remove the import ", "remove import ",
        ];

        let (op, prefixes): (&str, &[&str]) = if is_add {
            ("+", prefixes_add)
        } else {
            ("-", prefixes_rm)
        };

        let mut module = "";
        for prefix in prefixes {
            if lower.starts_with(prefix) {
                module = lower[prefix.len()..].trim();
                break;
            }
        }

        let module = self.strip_target_keywords(module);
        let module = self.strip_member_words(&module);
        let module = self.clean_ident(&module);
        if module.is_empty() {
            return None;
        }

        let axon = format!("{}use({})", op, module);
        let annotation = format!("code-opt · struct({}) · import({})", if is_add { "import" } else { "unimport" }, module);
        Some((axon, annotation))
    }

    // ── Structural helpers ───────────────────────────────────────────────────

    /// Detect a member-type keyword at the start of text.
    /// Returns (canonical_type, rest_after_keyword).
    fn detect_member_keyword<'a>(&self, text: &'a str) -> Option<(&'static str, &'a str)> {
        let mut best: Option<(&'static str, usize)> = None;
        for (keywords, canonical) in MEMBER_KEYWORDS {
            for kw in *keywords {
                if text.starts_with(kw) {
                    let len = kw.len();
                    // Ensure word boundary after keyword
                    let next_char = text[len..].chars().next();
                    if next_char.is_none() || next_char == Some(' ') || next_char == Some('_') {
                        if best.is_none() || len > best.unwrap().1 {
                            best = Some((canonical, len));
                        }
                    }
                }
            }
        }
        best.map(|(canonical, len)| (canonical, text[len..].trim()))
    }

    /// Detect member keyword in the middle of text: "name field on User" →
    /// ("field", "name", "User"). Returns (member_type, name_before, target_after).
    fn detect_member_keyword_mid(&self, text: &str) -> Option<(&'static str, String, String)> {
        for (keywords, canonical) in MEMBER_KEYWORDS {
            for kw in *keywords {
                // Check "NAME kw REST"
                let pattern = format!(" {} ", kw);
                if let Some(pos) = text.find(&pattern) {
                    let before = text[..pos].trim();
                    let after = text[pos + pattern.len()..].trim();
                    if !before.is_empty() {
                        // Strip leading prepositions from after: "on User" → "User"
                        let target = self.strip_leading_preps(after);
                        let target = self.strip_target_keywords(&target);
                        return Some((canonical, before.to_string(), target));
                    }
                }
                // Check at end: "NAME kw"
                let pattern_end = format!(" {}", kw);
                if text.ends_with(&pattern_end) {
                    let before = text[..text.len() - pattern_end.len()].trim();
                    if !before.is_empty() {
                        return Some((canonical, before.to_string(), String::new()));
                    }
                }
            }
        }
        None
    }

    /// Strip leading prepositions: "on the User" → "User", "in my app" → "app"
    fn strip_leading_preps(&self, text: &str) -> String {
        let preps = [
            "on the ", "on my ", "on our ", "on this ", "on ",
            "in the ", "in my ", "in our ", "in this ", "in ",
            "of the ", "of my ", "of our ", "of this ", "of ",
            "from the ", "from my ", "from our ", "from this ", "from ",
            "for the ", "for my ", "for our ", "for this ", "for ",
        ];
        for prep in &preps {
            if text.starts_with(prep) {
                return text[prep.len()..].trim().to_string();
            }
        }
        text.to_string()
    }

    /// Extract member name and target from text like "email to the User struct"
    /// or "called email to the User struct" or "email on the User struct"
    fn extract_name_and_target(&self, text: &str, member_type: &str) -> (String, String) {
        let text = text.trim();
        // Strip "called" / "named"
        let text = if text.starts_with("called ") { &text[7..] }
            else if text.starts_with("named ") { &text[6..] }
            else { text };

        // For getter/setter, strip leading "for "
        let text = if (member_type == "getter" || member_type == "setter") && text.starts_with("for ") {
            &text[4..]
        } else {
            text
        };

        // Handle leading prepositions: "to the User struct" means name="" target="User"
        let leading_preps = ["to the ", "to my ", "to our ", "to ", "on the ", "on "];
        for prep in &leading_preps {
            if text.starts_with(prep) {
                let target = &text[prep.len()..];
                let target = self.strip_target_keywords(target);
                return (String::new(), target.trim().to_string());
            }
        }

        self.extract_name_and_target_on(text)
    }

    /// Extract "NAME from TARGET" where TARGET is after a prep like "from the X struct"
    fn extract_name_and_target_from(&self, text: &str) -> (String, String) {
        let text = text.trim();
        let text = if text.starts_with("called ") { &text[7..] }
            else if text.starts_with("named ") { &text[6..] }
            else { text };

        // Split at "from" preposition
        for prep in &[" from the ", " from my ", " from our ", " from this ", " from "] {
            if let Some(pos) = text.find(prep) {
                let name = &text[..pos];
                let target = &text[pos + prep.len()..];
                let target = self.strip_target_keywords(target);
                return (name.trim().to_string(), target.trim().to_string());
            }
        }

        (text.to_string(), String::new())
    }

    /// Split "NAME on/in/of TARGET" pattern
    fn extract_name_and_target_on(&self, text: &str) -> (String, String) {
        let text = text.trim();
        for prep in TARGET_PREPS {
            if let Some(pos) = text.find(prep) {
                let name = &text[..pos];
                let target = &text[pos + prep.len()..];
                let target = self.strip_target_keywords(target);
                return (name.trim().to_string(), target.trim().to_string());
            }
        }
        (text.to_string(), String::new())
    }

    /// Split text at the first occurrence of any of the given prepositions.
    fn split_at_prep<'a>(&self, text: &'a str, preps: &[&str]) -> (&'a str, &'a str) {
        for prep in preps {
            if let Some(pos) = text.find(prep) {
                return (&text[..pos], text[pos + prep.len()..].trim());
            }
        }
        (text, "")
    }

    /// Get the first whitespace-separated word and the rest.
    fn split_at_first_word<'a>(&self, text: &'a str) -> (String, &'a str) {
        let text = text.trim();
        if let Some(pos) = text.find(' ') {
            (text[..pos].to_string(), text[pos..].trim_start())
        } else {
            (text.to_string(), "")
        }
    }

    /// Strip target-type keywords ("struct", "class", "type", etc.)
    fn strip_target_keywords(&self, text: &str) -> String {
        let mut words: Vec<&str> = text.split_whitespace().collect();
        words.retain(|w| {
            let lower = w.to_lowercase();
            !TARGET_KEYWORDS.contains(&lower.as_str())
                && lower != "the" && lower != "a" && lower != "an"
                && lower != "my" && lower != "our" && lower != "this"
                && lower != "that" && lower != "new" && lower != "existing"
        });
        words.join(" ")
    }

    /// Strip member-type keywords from text
    fn strip_member_keywords(&self, text: &str) -> String {
        let mut result = text.to_string();
        for (keywords, _) in MEMBER_KEYWORDS {
            for kw in *keywords {
                // Replace keyword + space, handling word boundaries
                let pattern = format!("{} ", kw);
                if result.starts_with(&pattern) {
                    result = result[pattern.len()..].to_string();
                } else {
                    let pattern = format!(" {}", kw);
                    if result.ends_with(&pattern) {
                        result = result[..result.len() - pattern.len()].to_string();
                    }
                }
            }
        }
        result.trim().to_string()
    }

    /// Get the sigils for a member type: (target_sigil_prefix, member_sigil)
    fn member_sigils(&self, member_type: &str) -> (&str, &str) {
        match member_type {
            "field" | "variant" => ("", "."),
            "method" | "constructor" | "getter" | "setter" => ("", "~"),
            "param" => ("", "$"),
            "import" => ("", ""),
            _ => ("", "."),
        }
    }

    /// Strip member-type keywords from a name (e.g., "status field" → "status")
    fn strip_member_words(&self, text: &str) -> String {
        let mut words: Vec<&str> = text.split_whitespace().collect();
        words.retain(|w| {
            let lower = w.to_lowercase();
            // Check against all member keywords
            for (keywords, _) in MEMBER_KEYWORDS {
                for kw in *keywords {
                    if &lower == kw {
                        return false;
                    }
                }
            }
            true
        });
        words.join(" ")
    }

    /// Clean an identifier: lowercase, strip articles, hyphenate multi-word.
    fn clean_ident(&self, text: &str) -> String {
        let mut result = text.trim().to_string();

        while result.ends_with('.') || result.ends_with(',') || result.ends_with(';')
            || result.ends_with('?') || result.ends_with('!')
        {
            result.pop();
        }

        // Strip leading articles
        let lower = result.to_lowercase();
        for prefix in &["the ", "a ", "an ", "my ", "our ", "this ", "that "] {
            if lower.starts_with(prefix) {
                result = result[prefix.len()..].trim().to_string();
                break;
            }
        }

        // Preserve case for identifiers that look like they have intentional casing
        let has_mixed_case = result.chars().any(|c| c.is_uppercase())
            && result.chars().any(|c| c.is_lowercase())
            && !result.chars().all(|c| c.is_lowercase() || !c.is_alphabetic());

        if has_mixed_case {
            // Keep original casing (camelCase, PascalCase)
            result.split_whitespace().collect::<Vec<_>>().join("-")
        } else {
            result = result.to_lowercase();
            result.split_whitespace().collect::<Vec<_>>().join("-")
        }
    }

    // ── Part extraction ──────────────────────────────────────────────────────

    /// Extract subject, scope, and transform target from the remaining text.
    fn extract_parts(&self, text: &str) -> (String, String, String) {
        let mut subject = text.to_string();
        let mut scope = String::new();
        let mut transform = String::new();

        // Extract transform target ("to X", "into X", "from X")
        // But only if the transform prep is not part of a larger phrase
        for prep in TRANSFORM_PREPS {
            if let Some(pos) = subject.find(prep) {
                let before = &subject[..pos];
                let after = subject[pos + prep.len()..].trim();
                if !before.is_empty() && !after.is_empty() {
                    transform = self.clean_subject(after);
                    subject = before.trim().to_string();
                    break;
                }
            }
        }

        // Extract scope ("in X", "for X", "of X")
        for prep in SCOPE_PREPS {
            if let Some(pos) = subject.find(prep) {
                let before = &subject[..pos];
                let after = subject[pos + prep.len()..].trim();
                if !after.is_empty() {
                    scope = self.clean_subject(after);
                    subject = before.trim().to_string();
                    break;
                }
            }
        }

        (subject, scope, transform)
    }

    // ── Subject cleaning ─────────────────────────────────────────────────────

    fn clean_subject(&self, text: &str) -> String {
        let mut result = text.trim().to_string();

        // Strip trailing punctuation
        while result.ends_with('.') || result.ends_with('?') || result.ends_with('!')
            || result.ends_with(',') || result.ends_with(';')
        {
            result.pop();
        }

        // Strip leading articles/filler
        let lower = result.to_lowercase();
        for noise in SUBJECT_NOISE {
            if lower.starts_with(noise) {
                result = result[noise.len()..].trim().to_string();
                break;
            }
        }

        // Hyphenate multi-word subjects (replace spaces with hyphens)
        result = result.split_whitespace().collect::<Vec<_>>().join("-");

        // Lowercase the result
        result = result.to_lowercase();

        result
    }

    // ── Output formatting ────────────────────────────────────────────────────

    fn format_action(&self, verb: &str, subject: &str, scope: &str, transform: &str) -> String {
        let mut axon = if subject.is_empty() {
            format!(">{}", verb)
        } else {
            format!(">{} {}", verb, subject)
        };
        if !scope.is_empty() {
            axon.push(':');
            axon.push_str(scope);
        }
        if !transform.is_empty() {
            axon.push_str("→");
            axon.push_str(transform);
        }
        axon
    }

    fn format_query(&self, qtype: &str, subject: &str, scope: &str, transform: &str) -> String {
        let mut axon = format!("?{} {}", qtype, subject);
        if !scope.is_empty() {
            axon.push(':');
            axon.push_str(scope);
        }
        if !transform.is_empty() {
            axon.push_str("→");
            axon.push_str(transform);
        }
        axon
    }
}

impl Default for CodeTranslator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doc_basic() {
        let ct = CodeTranslator::new();
        let r = ct.translate("create documentation for the login component in the auth service");
        assert!(r.matched);
        assert_eq!(r.axon, ">doc login-component:auth-service");
    }

    #[test]
    fn test_query_best() {
        let ct = CodeTranslator::new();
        let r = ct.translate("what is the best approach to implement caching");
        assert!(r.matched);
        assert_eq!(r.axon, "?best implement-caching");
    }

    #[test]
    fn test_plan_feature() {
        let ct = CodeTranslator::new();
        let r = ct.translate("please plan the following feature: user authentication");
        assert!(r.matched);
        assert!(r.axon.starts_with(">plan"));
    }

    #[test]
    fn test_fix_with_scope() {
        let ct = CodeTranslator::new();
        let r = ct.translate("fix the bug in the payment processing module");
        assert!(r.matched);
        assert!(r.axon.starts_with(">fix"));
        assert!(r.axon.contains("payment-processing-module"));
    }

    #[test]
    fn test_noise_stripping() {
        let ct = CodeTranslator::new();
        let r = ct.translate("could you please create documentation for the API");
        assert!(r.matched);
        assert!(r.axon.starts_with(">doc"));
    }

    #[test]
    fn test_how_query() {
        let ct = CodeTranslator::new();
        let r = ct.translate("how do i implement authentication in my react app");
        assert!(r.matched);
        assert!(r.axon.starts_with("?how"));
    }

    #[test]
    fn test_transform() {
        let ct = CodeTranslator::new();
        let r = ct.translate("migrate the codebase from javascript to typescript");
        assert!(r.matched);
        assert!(r.axon.contains("→"));
    }

    #[test]
    fn test_non_code_passthrough() {
        let ct = CodeTranslator::new();
        let r = ct.translate("the sun emits ultraviolet radiation");
        assert!(!r.matched);
    }

    #[test]
    fn test_savings() {
        let ct = CodeTranslator::new();
        let r = ct.translate("could you please create documentation for the login component in the auth service");
        assert!(r.matched);
        let input_tokens = "could you please create documentation for the login component in the auth service".split_whitespace().count();
        let output_tokens = r.axon.split_whitespace().count();
        assert!(output_tokens < input_tokens);
    }
}
