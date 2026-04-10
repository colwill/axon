# AXON v1.1 Specification

**AI eXchange Optimized Notation** — a compact symbolic language that compresses natural language into token-efficient expressions for AI communication. Uses ASCII-only operators for optimal BPE tokenization.

## Type Sigils

Sigils are applied **conditionally** — only to tokens found in the known entity/concept/verb databases. Unknown tokens are emitted bare (no sigil) to save BPE tokens. Consecutive bare tokens are merged with hyphens into compounds.

| Sigil | Name              | Definition                                              |
|-------|-------------------|---------------------------------------------------------|
| @     | Entity / Agent    | A named actor, system, or proper noun: @sun, @OpenAI    |
| #     | Concept / Abstract| An idea, category, or domain: #gravity, #justice        |
| ~     | Process / Action  | A verb, transformation, or operation: ~emit, ~learn     |
| ?     | Query / Unknown   | An open question or unresolved value: ?cause, ?result   |
| !     | Negation          | Negated token: !evidence, !data                         |
| ^     | Temporal          | A time reference or duration: ^now, ^T-2d, ^T+1mo      |
| $     | Scalar            | A measurable value or magnitude: $high, $3.14, $low     |

## Logical & Relational Operators (ASCII)

| Op   | Name                       | Reads as          |
|------|----------------------------|-------------------|
| ->   | Causes                     | "leads to"        |
| <-   | Result of                  | "caused by"       |
| :.   | Therefore                  | "conclusion"      |
| bc   | Because                    | "premise/reason"  |
| &&   | And                        | "conjunction"     |
| \|\| | Or                         | "disjunction"     |
| A.   | For all                    | "universal"       |
| E.   | Exists                     | "existential"     |
| :    | Type/impl                  | type annotation   |
| =    | Set value                  | assignment        |
| +    | Add                        | add member        |
| -    | Remove                     | remove member     |
| <    | Inherits                   | extends/inherits  |

## Epistemic Confidence Markers

| Marker | Level       | Meaning                       |
|--------|-------------|-------------------------------|
| !!     | Certain     | Verified fact, no doubt       |
| !      | High        | Strong supporting evidence    |
| ~      | Moderate    | Plausible, some uncertainty   |
| *      | Low         | Weak evidence or guess        |
| **     | Speculative | Hypothetical or extrapolated  |
| ?      | Unknown     | Insufficient data to assess   |

## Temporal Markers

- `^now` — current moment
- `^T-Nd` — N days past (e.g. `^T-7d` = one week ago)
- `^T+Nd` — N days future (e.g. `^T+30d` = next month)
- `^A.t` — all time / always true
- `^span[A,B]` — time range from A to B

## Programming Commands

Command verbs use the `>` prefix:

```
>doc >impl >fix >test >rev >ref >opt >plan >dep >add >rm >up >mv >cfg >mig
>db >api >ci >sec >err >log >bench >lint >merge >explain
```

Syntax: `>verb subject[:scope][->target]`

## Programming Queries

Query types use the `?` prefix:

```
?how ?why ?best ?what ?diff ?when ?where ?can ?cmp ?alt ?err ?perf
```

Syntax: `?type subject[:scope][->target]`

## Structural Operations

| Syntax              | Meaning            |
|---------------------|--------------------|
| @Type+.field        | Add field          |
| @Type-.field        | Remove field       |
| @Type.x=$v          | Set field value    |
| @Type.x:T           | Set field type     |
| @Type:impl(@Trait)  | Implement trait    |
| @Child<@Parent      | Inherit / extend   |
| +use(module)        | Add import         |
| -use(module)        | Remove import      |

## Abbreviation Tiers

AXON v1.1 introduces **tiered abbreviation levels**. Each level is inclusive of all lower levels. The default is L0.

### L0 — Default (85+ terms)

The baseline abbreviation dictionary, universally understood by developers.

| Full | Abbr | Full | Abbr | Full | Abbr |
|------|------|------|------|------|------|
| object | obj | function | fn | component | comp |
| documentation | docs | implementation | impl | authentication | auth |
| application | app | configuration | cfg | environment | env |
| development | dev | production | prod | repository | repo |
| parameter | param | argument | arg | reference | ref |
| variable | var | constant | const | directory | dir |
| dependency | dep | dependencies | deps | performance | perf |
| optimization | opt | database | db | message | msg |
| request | req | response | res | middleware | mw |
| specification | spec | information | info | property | prop |
| attribute | attr | expression | expr | condition | cond |
| temporary | tmp | template | tpl | maximum | max |
| minimum | min | number | num | string | str |
| boolean | bool | integer | int | character | char |
| subscription | sub | notification | notif | generation | gen |
| operation | op | administrator | admin | previous | prev |
| current | curr | source | src | destination | dest |
| original | orig | description | desc | infrastructure | infra |
| transaction | txn | synchronous | sync | asynchronous | async |
| memory | mem | iteration | iter | executable | exec |
| execution | exec | navigation | nav | initialization | init |
| connection | conn | communication | comm | certificate | cert |
| technology | tech | management | mgmt | calculate | calc |
| callback | cb | render | rnd | inline | inl |

### L1 — Extended Technical

Broader vocabulary for networking, OS, data, DevOps, and general development.

| Full | Abbr | Full | Abbr | Full | Abbr |
|------|------|------|------|------|------|
| protocol | proto | address | addr | network | net |
| socket | sock | bandwidth | bw | latency | lat |
| throughput | tput | packet | pkt | process | proc |
| thread | thd | scheduler | sched | filesystem | fs |
| allocation | alloc | register | reg | permission | perm |
| collection | coll | element | elem | index | idx |
| length | len | count | cnt | buffer | buf |
| sequence | seq | frequency | freq | column | col |
| record | rec | aggregate | agg | accumulator | acc |
| container | ctr | kubernetes | k8s | namespace | ns |
| instance | inst | deployment | deploy | monitoring | mon |
| observability | obs | replication | repl | endpoint | ep |
| controller | ctrl | serialization | ser | pagination | page |
| validation | val | constructor | ctor | destructor | dtor |
| exception | exc | library | lib | package | pkg |
| interface | iface | algorithm | algo | architecture | arch |
| version | ver | revision | rev | extension | ext |
| definition | def | declaration | decl | statement | stmt |
| annotation | annot | utility | util | debug | dbg |
| profiling | prof | benchmark | bench | migration | migr |
| integration | integ | refactoring | refac |

### L2 — Aggressive

Common English words shortened for maximum compression in technical discourse.

| Full | Abbr | Full | Abbr | Full | Abbr |
|------|------|------|------|------|------|
| between | btw | without | w/o | because | bc |
| example | ex | different | diff | difference | diff |
| important | imp | multiple | mult | available | avail |
| required | reqd | optional | opt | alternative | alt |
| comparison | cmp | approximately | approx | especially | esp |
| including | incl | regarding | re | typically | typ |
| specific | spec | generally | gen | additional | addl |
| automatic | auto | independent | indep | separate | sep |
| immediately | immed | distributed | distrib | concurrent | concur |
| horizontal | horiz | vertical | vert | duplicate | dup |
| strategy | strat | mechanism | mech |

### L3 — Maximum

Single-character and extreme shortenings for expert users. Overrides lower-tier abbreviations for maximum compression.

| Full | Abbr | Full | Abbr | Full | Abbr |
|------|------|------|------|------|------|
| function | f | variable | v | object | o |
| string | s | number | n | integer | i |
| boolean | b | return | ret | value | val |
| error | err | result | res | context | ctx |
| message | m | data | d | type | t |
| event | evt | handler | h | service | svc |
| server | srv | client | cli | command | cmd |
| channel | ch | signal | sig | method | m |
| module | mod | resource | rsc | identifier | id |
| manager | mgr | factory | fac | provider | prov |
| consumer | cons | producer | prod | subscriber | sub |
| publisher | pub | listener | lsnr | observer | obs |

### Selecting a Level

When using AXON in a system prompt, specify the level:

```
Use AXON v1.1 with abbreviation level L1.
```

When no level is specified, L0 (default) is assumed.

## Grammar Pattern

```
[QUANTIFIER] [SUBJECT sigil+name] [OPERATOR] [OBJECT sigil+name] [CONFIDENCE] [TEMPORAL]
```

Parentheses group sub-expressions. Multi-word tokens use hyphens. Bare (unsigiled) consecutive tokens merge into hyphenated compounds.

## Encoding Rules (Natural Language -> AXON)

1. Named entities, people, systems, orgs -> `@` prefix
2. Known abstract concepts -> `#` prefix (unknown words get NO sigil)
3. Known verbs, actions, processes -> `~` prefix (unknown verbs get NO sigil)
4. Numeric values, measurements -> `$` prefix
5. Causal relationships -> `->` or `<-`
6. Logical connectives (and/or/therefore/because) -> `&&` `||` `:.` `bc`
7. Negation (not, no evidence, absence) -> `!` prefix
8. Universal/existential quantifiers (all, every, some) -> `A.` `E.`
9. Extract confidence from hedge words -> append marker
10. Extract time references -> append temporal marker
11. Strip filler words, articles, copulas, pleasantries
12. Abbreviate common terms using the active abbreviation tier
13. Merge consecutive bare tokens into hyphenated compounds

## Decoding Rules (AXON -> Natural Language)

1. `@` tokens -> named entities
2. `#` tokens -> concepts or abstract nouns
3. `~` tokens -> verbs (conjugate naturally)
4. `$` tokens -> numeric values or scalar descriptors
5. `->` -> "causes" / "leads to"
6. `<-` -> "is caused by" / "results from"
7. `:.` -> "therefore" / `bc` -> "because"
8. `!` prefix -> "not" / "no [noun]" / "absence of"
9. `A.` -> "all" / "every" / `E.` -> "there exists"
10. `&&` -> "and" / `||` -> "or"
11. Bare tokens (no sigil) -> context-dependent nouns/adjectives
12. Hyphenated compounds -> multi-word phrases
13. Confidence markers -> hedge language
14. Temporal markers -> time phrases

## Response Compression

AXON can also be used to compress LLM responses. Include this directive in the system prompt:

```
Respond using AXON notation where applicable: sigils (@entity #concept ~verb),
operators (-> <- :. bc), abbreviations (database->db, component->comp, function->fn).
Strip filler. Be terse.
```

This is complementary to prompt-side compression — AXON compresses the input, response compression reduces the output.

## Examples

| Natural Language | AXON |
|---|---|
| The sun probably emits ultraviolet radiation. | `@sun ~emit* #uv-radiation` |
| All living things require energy to survive. | `A.@organism #energy -> #survival!!` |
| Climate change caused by CO2 leads to temperature rise. | `@co2 #emission -> #climate-change -> #temperature ~rise!!` |
| There is no evidence that this treatment works. | `!#evidence :. !~work #treatment` |
| The model will probably predict the outcome in 30 days. | `@model ~predict* #outcome ^T+30d` |
| fix the bug in the authentication service | `>fix bug:auth-service` |
| what is the best way to implement caching | `?best impl-caching` |
| add a field email to User | `@user+.email` |
| explain database connection pooling | `>explain db-conn-pooling` |

## Version History

- **v1.0** — Initial release. ASCII operators, conditional sigils, L0 abbreviation dictionary (85+ terms), compound merging, BPE-aware token estimation.
- **v1.1** — Tiered abbreviation levels (L0-L3), stop-word stripping in code queries, response compression directive, expanded technical vocabulary.
