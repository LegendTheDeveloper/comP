// relevance.rs - Unified relevance scoring for run_pipeline
//
// Pure, DB-free scoring logic. Per-engine evidence (symbol LIKE quality,
// TF-IDF cosine, BM25) is accumulated per file into an Evidence record,
// combined into one normalized score, sorted, cut by a relevance threshold,
// and capped per file. Keeping this module free of DB/IO access makes the
// unit tests parallel-safe (they never touch the shared test database).
//
// Score-scale note: final scores are max-normalized PER QUERY, so they are
// never comparable across run_pipeline calls. Weak-result detection therefore
// uses RAW engine signals (RawSignals), not the normalized scores.

use serde_json::Value;
use std::collections::HashSet;

// ---------------------------------------------------------------------------
// Scoring weights and raw-signal thresholds
// ---------------------------------------------------------------------------

/// Weight of the symbol-LIKE component (highest precision signal).
pub const W_SYMBOL: f32 = 0.40;
/// Weight of the TF-IDF cosine component (multi-token code semantics).
pub const W_TFIDF: f32 = 0.35;
/// Weight of the BM25 component (docs only; docs were the observed noise source).
pub const W_BM25: f32 = 0.25;
/// Additive boost for files in `git diff HEAD` (actively edited).
pub const GIT_DIFF_BOOST: f32 = 0.25;

/// Raw-signal thresholds for confidence assessment. Heuristics validated
/// against the GameLauncherCloud workspace; tune here if smoke tests show
/// false weak/strong classifications.
pub const WEAK_TFIDF_RAW: f32 = 0.15;
pub const WEAK_BM25_RAW: f64 = 1.0;
pub const STRONG_TFIDF_RAW: f32 = 0.4;
pub const STRONG_BM25_RAW: f64 = 3.0;
/// Symbol quality at or above this counts as a real (non-coincidence) hit.
pub const STRONG_SYM_QUALITY: f32 = 0.7;
/// Pivot cap when results are weak: return a short honest list, not 20 noise files.
pub const WEAK_MAX_PIVOTS: usize = 5;
/// Keywords whose document frequency exceeds this share of all nodes are
/// skipped for LIKE search (their top rows are arbitrary); they still
/// participate in TF-IDF and BM25, whose own IDF handles common terms.
pub const COMMON_KEYWORD_DF_SHARE: f64 = 0.10;
/// Keywords at or above this rarity weight are "defining" for the task:
/// confidence degrades when they produce no code evidence.
pub const DEFINING_KW_WEIGHT: f32 = 0.5;
/// A keyword counts as covered when some symbol/filename hit reached this
/// match quality (substring 0.4 counts; zero-hit keywords do not).
pub const COVERED_QUALITY: f32 = 0.4;
/// Only keywords at or above this rarity weight count toward the
/// distinct-keyword bonus: a launcher+player brand-word pair is not coverage.
pub const BONUS_KW_WEIGHT: f32 = 0.4;

// ---------------------------------------------------------------------------
// Stopwords
// ---------------------------------------------------------------------------

/// English function words plus generic task verbs that produce garbage LIKE
/// hits ("add", "with", "time"...). Deliberately excludes domain-plausible
/// nouns ("user", "dashboard", "file"): corpus IDF weighting downweights
/// those instead of a brittle blocklist. Keywords shorter than 3 chars are
/// already dropped by the caller's split.
pub const STOPWORDS: &[&str] = &[
    // Function words
    "the", "and", "for", "with", "from", "into", "onto", "that", "this",
    "these", "those", "when", "where", "which", "while", "are", "was",
    "were", "can", "will", "shall", "should", "would", "could", "may",
    "might", "must", "have", "has", "had", "not", "but", "all", "any",
    "some", "one", "two", "our", "out", "your", "their", "its", "about",
    "after", "before", "also", "only", "just", "very", "more", "most",
    "each", "been", "being", "does", "doing", "done", "how", "why",
    "what", "who", "then", "than", "there", "here", "over", "under",
    "between", "during", "via", "per", "upon", "own", "same", "such",
    "both", "few", "other", "another", "again", "once", "now", "lets",
    // Generic task verbs and fillers
    "add", "adds", "adding", "fix", "fixes", "fixing", "implement",
    "implements", "implementing", "create", "creates", "creating",
    "update", "updates", "updating", "change", "changes", "changing",
    "make", "makes", "making", "get", "gets", "getting", "set", "sets",
    "setting", "use", "uses", "using", "used", "new", "need", "needs",
    "needed", "want", "wants", "time", "times", "way", "ways", "please",
    "help", "like", "etc",
];

/// Drop stopwords from the keyword list. Falls back to the unfiltered list
/// when filtering would leave nothing (never run with zero keywords).
pub fn filter_keywords<'a>(keywords: &[&'a str]) -> Vec<&'a str> {
    let filtered: Vec<&'a str> = keywords
        .iter()
        .copied()
        .filter(|k| !STOPWORDS.contains(&k.to_lowercase().as_str()))
        .collect();
    if filtered.is_empty() {
        keywords.to_vec()
    } else {
        filtered
    }
}

/// Workspace-specific noise keywords: the tokens that make up the registered
/// repo aliases (GameLauncherCloud-Backend -> game, launcher, cloud, backend)
/// are ambient in that workspace by definition. Matching them proves nothing,
/// so they are skipped in the LIKE and filename channels (TF-IDF and BM25
/// keep them; their own IDF copes). Merged with the `noise_keywords` config.
pub fn derive_noise_keywords(aliases: &[String], configured: &[String]) -> HashSet<String> {
    let mut noise: HashSet<String> = configured.iter().map(|s| s.to_lowercase()).collect();
    for alias in aliases {
        for token in split_symbol_tokens(alias) {
            if token.len() >= 3 {
                noise.insert(token);
            }
        }
    }
    noise
}

// ---------------------------------------------------------------------------
// Symbol match quality and keyword weighting
// ---------------------------------------------------------------------------

/// Split a symbol name into lowercase tokens on camelCase, snake_case and
/// dotted boundaries ("getAuthToken" -> ["get", "auth", "token"]).
fn split_symbol_tokens(symbol: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let chars: Vec<char> = symbol.chars().collect();
    for (i, &ch) in chars.iter().enumerate() {
        if ch == '_' || ch == '-' || ch == '.' {
            if !current.is_empty() {
                tokens.push(current.to_lowercase());
                current.clear();
            }
        } else if ch.is_uppercase() {
            if !current.is_empty() && (i == 0 || !chars[i - 1].is_uppercase()) {
                tokens.push(current.to_lowercase());
                current.clear();
            }
            current.push(ch);
        } else {
            current.push(ch);
        }
    }
    if !current.is_empty() {
        tokens.push(current.to_lowercase());
    }
    tokens
}

/// How well a symbol name matches a keyword:
///   1.0   exact name match
///   0.85  keyword equals a whole camelCase/snake token, or is a prefix
///   0.4   plain substring anywhere
///   0.0   no match
///
/// The exact/token gap is deliberately narrow: compound names are the norm in
/// code (OnboardingSurveyComponent for "survey"), so a token match must not be
/// heavily outranked by a trivial one-word property that happens to equal a
/// generic keyword ("Progress", "Question").
pub fn symbol_match_quality(symbol: &str, keyword: &str) -> f32 {
    let kw = keyword.to_lowercase();
    let sym_lower = symbol.to_lowercase();
    if sym_lower == kw {
        return 1.0;
    }
    if sym_lower.starts_with(&kw) || split_symbol_tokens(symbol).iter().any(|t| *t == kw) {
        return 0.85;
    }
    if sym_lower.contains(&kw) {
        return 0.4;
    }
    0.0
}

/// IDF-style keyword weight in (0, 1]: rare keywords approach 1, keywords
/// matching most of the corpus approach 0.
///
///   weight = ln(1 + N / (1 + df)) / ln(1 + N)
pub fn keyword_weight(df: i64, total_nodes: i64) -> f32 {
    if total_nodes <= 0 {
        return 1.0;
    }
    let n = total_nodes as f64;
    let w = (1.0 + n / (1.0 + df.max(0) as f64)).ln() / (1.0 + n).ln();
    w.clamp(0.0, 1.0) as f32
}

// ---------------------------------------------------------------------------
// Per-file evidence and raw signals
// ---------------------------------------------------------------------------

/// Engine evidence accumulated for one file before scoring.
#[derive(Debug, Clone, Default)]
pub struct Evidence {
    /// Best (quality x softened keyword_weight) among symbol/filename hits.
    /// Max, not sum: many junk substring hits must not beat one exact match.
    pub sym_best: f32,
    /// Distinct keywords that produced a symbol/filename hit (for reporting).
    pub sym_keywords: HashSet<String>,
    /// Subset of sym_keywords rare enough (weight >= BONUS_KW_WEIGHT) to
    /// count toward the diversity bonus: brand-word pairs earn nothing.
    pub bonus_keywords: HashSet<String>,
    /// True when the file NAME (stem) matched a task keyword.
    pub filename_hit: bool,
    /// Raw TF-IDF cosine (or the fixed 0.3 fallback score); max over hits.
    pub tfidf_raw: f32,
    /// Raw BM25 score; max over hits.
    pub bm25_raw: f64,
}

impl Evidence {
    pub fn add_symbol_hit(&mut self, keyword: &str, quality: f32, kw_weight: f32) {
        if quality <= 0.0 {
            return;
        }
        // Soften the rarity weight into [0.5, 1.0] so a legitimate match on a
        // mid-frequency keyword is not diluted twice (once by quality, once by
        // rarity). Rarity still orders matches, it just cannot halve them.
        let weighted = quality * (0.5 + 0.5 * kw_weight);
        if weighted > self.sym_best {
            self.sym_best = weighted;
        }
        self.sym_keywords.insert(keyword.to_lowercase());
        // Only rare-enough keywords count toward the diversity bonus:
        // matching two brand words (launcher+player) is not extra coverage.
        if kw_weight >= BONUS_KW_WEIGHT {
            self.bonus_keywords.insert(keyword.to_lowercase());
        }
    }

    /// Filename (stem) match: same math as a symbol hit, plus a marker so
    /// match_reasons can report it honestly.
    pub fn add_filename_hit(&mut self, keyword: &str, quality: f32, kw_weight: f32) {
        if quality <= 0.0 {
            return;
        }
        self.filename_hit = true;
        self.add_symbol_hit(keyword, quality, kw_weight);
    }

    pub fn add_tfidf(&mut self, raw: f32) {
        if raw > self.tfidf_raw {
            self.tfidf_raw = raw;
        }
    }

    pub fn add_bm25(&mut self, raw: f64) {
        if raw > self.bm25_raw {
            self.bm25_raw = raw;
        }
    }

    /// Symbol component with the distinct-keyword bonus, capped at 1.0.
    /// The bonus rewards multi-keyword coverage: a file matching "survey" AND
    /// "feedback" should outrank one exact-matching a single generic keyword.
    /// Only rare-enough keywords count (bonus_keywords), so files matching
    /// launcher+player brand pairs earn no bonus.
    pub fn symbol_component(&self) -> f32 {
        let bonus = 0.15 * (self.bonus_keywords.len().saturating_sub(1)) as f32;
        (self.sym_best + bonus).min(1.0)
    }
}

/// Query-wide raw engine signals used for confidence assessment.
/// Deliberately raw: per-query max normalization always makes SOME candidate
/// score well, even when everything is garbage.
#[derive(Debug, Clone, Copy, Default)]
pub struct RawSignals {
    /// Best unweighted symbol match quality across all hits.
    pub max_sym_quality: f32,
    /// True when some exact symbol match had keyword weight >= 0.5.
    pub strong_exact: bool,
    /// Highest raw TF-IDF cosine over REAL (non-fallback) hits.
    pub max_tfidf_raw: f32,
    /// True when the substring fallback path produced hits.
    pub had_tfidf_fallback: bool,
    /// Highest raw BM25 score over doc hits.
    pub max_bm25_raw: f64,
}

impl RawSignals {
    pub fn note_symbol(&mut self, quality: f32, kw_weight: f32) {
        if quality > self.max_sym_quality {
            self.max_sym_quality = quality;
        }
        if quality >= 1.0 && kw_weight >= 0.5 {
            self.strong_exact = true;
        }
    }

    pub fn note_tfidf(&mut self, raw: f32, is_fallback: bool) {
        if is_fallback {
            self.had_tfidf_fallback = true;
        } else if raw > self.max_tfidf_raw {
            self.max_tfidf_raw = raw;
        }
    }

    pub fn note_bm25(&mut self, raw: f64) {
        if raw > self.max_bm25_raw {
            self.max_bm25_raw = raw;
        }
    }
}

/// Classify overall result strength from raw signals.
/// Returns (confidence, weak_results).
pub fn assess_confidence(sig: &RawSignals) -> (&'static str, bool) {
    let weak = sig.max_sym_quality < STRONG_SYM_QUALITY
        && sig.max_tfidf_raw < WEAK_TFIDF_RAW
        && sig.max_bm25_raw < WEAK_BM25_RAW;
    if weak {
        return ("low", true);
    }
    let high = sig.strong_exact
        || sig.max_tfidf_raw >= STRONG_TFIDF_RAW
        || sig.max_bm25_raw >= STRONG_BM25_RAW;
    (if high { "high" } else { "medium" }, false)
}

/// Per-keyword search outcome: corpus rarity plus the best symbol/filename
/// match quality any file achieved for it.
#[derive(Debug, Clone)]
pub struct KeywordCoverage {
    pub keyword: String,
    pub df: i64,
    pub weight: f32,
    pub best_quality: f32,
}

/// Degrade confidence when the DEFINING (rarest) task keywords found nothing.
///
/// WHY: exact matches on generic words ("progress", "question") can push raw
/// signals to "high" even though the keyword that names the feature
/// ("achievements") matched nothing anywhere. That is exactly the greenfield
/// case where the agent must be told the feature does not exist yet.
///
/// Rules, applied on top of assess_confidence's verdict:
/// - defining keywords = coverage entries with weight >= DEFINING_KW_WEIGHT
/// - a keyword is uncovered when best_quality < COVERED_QUALITY
/// - ALL defining uncovered -> confidence "low", weak_results, weak_reason
/// - the single rarest defining keyword uncovered -> cap confidence at "medium"
///
/// Returns (confidence, weak_results, uncovered_keywords, weak_reason).
pub fn apply_keyword_coverage(
    confidence: &'static str,
    weak_results: bool,
    coverage: &[KeywordCoverage],
) -> (&'static str, bool, Vec<String>, Option<String>) {
    let defining: Vec<&KeywordCoverage> = coverage
        .iter()
        .filter(|c| c.weight >= DEFINING_KW_WEIGHT)
        .collect();
    let uncovered: Vec<String> = defining
        .iter()
        .filter(|c| c.best_quality < COVERED_QUALITY)
        .map(|c| c.keyword.clone())
        .collect();
    if defining.is_empty() || uncovered.is_empty() {
        return (confidence, weak_results, uncovered, None);
    }
    if uncovered.len() == defining.len() {
        let reason = format!(
            "defining keywords found no code evidence: {}",
            uncovered.join(", ")
        );
        return ("low", true, uncovered, Some(reason));
    }
    let rarest_uncovered = defining
        .iter()
        .max_by(|a, b| {
            a.weight
                .partial_cmp(&b.weight)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|c| c.best_quality < COVERED_QUALITY)
        .unwrap_or(false);
    let conf = if rarest_uncovered && confidence == "high" {
        "medium"
    } else {
        confidence
    };
    (conf, weak_results, uncovered, None)
}

/// Drop generated twin files (`X.Designer.cs`) when their base `X.cs` is also
/// a candidate: the Designer twin duplicates the migration snapshot and eats
/// budget. Git-diff files are exempt (actively edited).
pub fn drop_designer_twins(candidates: Vec<Candidate>) -> Vec<Candidate> {
    let paths: HashSet<String> = candidates.iter().map(|c| c.path.clone()).collect();
    candidates
        .into_iter()
        .filter(|c| {
            if c.git_diff {
                return true;
            }
            match c.path.strip_suffix(".Designer.cs") {
                Some(base) => !paths.contains(&format!("{}.cs", base)),
                None => true,
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Candidate, combination, cutoff
// ---------------------------------------------------------------------------

/// A pivot candidate with its unified relevance score.
#[derive(Debug, Clone)]
pub struct Candidate {
    pub path: String,
    pub sym_count: usize,
    pub base_tokens: usize,
    pub score: f32,
    pub reasons: Vec<String>,
    pub git_diff: bool,
}

/// Combine per-engine evidence into the final score:
///   final = 0.40 * sym + 0.35 * tfidf_norm + 0.25 * bm25_norm (+ 0.25 git diff)
/// TF-IDF and BM25 are max-normalized over this query's result set (the
/// standard fix for BM25's unbounded scale vs cosine's [0, 1]).
pub fn combine_score(ev: &Evidence, max_tfidf_raw: f32, max_bm25_raw: f64, git_diff: bool) -> f32 {
    let tfidf_norm = if max_tfidf_raw > 0.0 {
        ev.tfidf_raw / max_tfidf_raw
    } else {
        0.0
    };
    let bm25_norm = if max_bm25_raw > 0.0 {
        (ev.bm25_raw / max_bm25_raw) as f32
    } else {
        0.0
    };
    let mut score = W_SYMBOL * ev.symbol_component() + W_TFIDF * tfidf_norm + W_BM25 * bm25_norm;
    if git_diff {
        score += GIT_DIFF_BOOST;
    }
    score
}

/// Human/agent-readable list of which engines matched this file.
pub fn match_reasons(ev: &Evidence, git_diff: bool) -> Vec<String> {
    let mut reasons = Vec::new();
    if ev.sym_best > 0.0 {
        let mut kws: Vec<&str> = ev.sym_keywords.iter().map(|s| s.as_str()).collect();
        kws.sort();
        if kws.is_empty() {
            // Hits existed but only on below-bonus-threshold (common) keywords.
            reasons.push("symbol".to_string());
        } else {
            reasons.push(format!("symbol:{}", kws.join("+")));
        }
    }
    if ev.filename_hit {
        reasons.push("filename".to_string());
    }
    if ev.tfidf_raw > 0.0 {
        reasons.push("tfidf".to_string());
    }
    if ev.bm25_raw > 0.0 {
        reasons.push("bm25".to_string());
    }
    if git_diff {
        reasons.push("git_diff".to_string());
    }
    reasons
}

/// Sort by score descending; ties broken by path ascending for determinism.
/// partial_cmp with an Equal fallback: never unwrap on a NaN.
pub fn sort_by_score(candidates: &mut [Candidate]) {
    candidates.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.path.cmp(&b.path))
    });
}

/// Sort, then drop the weak tail: candidates scoring below the absolute
/// floor or below ratio x top_score. Git-diff files are exempt (a file the
/// agent is actively editing must never be threshold-dropped).
/// Returns (survivors, dropped_count).
pub fn apply_cutoff(mut candidates: Vec<Candidate>, cfg: &RelevanceConfig) -> (Vec<Candidate>, usize) {
    sort_by_score(&mut candidates);
    let top = candidates.first().map(|c| c.score).unwrap_or(0.0);
    let threshold = cfg.min_score_abs.max(cfg.min_score_ratio * top);
    let before = candidates.len();
    let survivors: Vec<Candidate> = candidates
        .into_iter()
        .filter(|c| c.git_diff || c.score >= threshold)
        .collect();
    let dropped = before - survivors.len();
    (survivors, dropped)
}

// ---------------------------------------------------------------------------
// Per-file token cap
// ---------------------------------------------------------------------------

/// Doc languages get a stricter cap: docs were the observed budget hogs.
/// `sql` counts as a doc: it has no symbol grammar, so its relevance channel
/// is BM25 over the raw text (schema files, RLS policies).
pub fn is_doc_language(lang: &str) -> bool {
    matches!(lang, "markdown" | "docx" | "pptx" | "xlsx" | "pdf" | "sql")
}

/// Structured-data languages (i18n bundles, configs, manifests). Their
/// "symbols" are data keys, so an exact key match carries no code-relevance
/// signal: a 5000-key translation file exactly matches almost any UI word.
/// They are excluded from the symbol/TF-IDF evidence channels (filename
/// evidence still applies) and share the tight doc token cap.
pub fn is_data_language(lang: &str) -> bool {
    matches!(lang, "json" | "yaml" | "xml" | "toml")
}

/// Max tokens any single pivot may consume: a share of the budget, further
/// tightened by doc_token_cap for doc/data files. Size alone is not a sin;
/// only budget domination is.
pub fn file_cap(budget: usize, cfg: &RelevanceConfig, tight_cap: bool) -> usize {
    let mut cap = ((budget as f64) * (cfg.max_file_budget_share as f64)) as usize;
    if tight_cap {
        cap = cap.min(cfg.doc_token_cap);
    }
    cap.max(1)
}

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

/// Tunables for cutoff and per-file caps. Precedence: param > config > default.
#[derive(Debug, Clone, PartialEq)]
pub struct RelevanceConfig {
    /// Absolute score floor (config-only knob).
    pub min_score_abs: f32,
    /// Drop candidates scoring below this fraction of the top score.
    pub min_score_ratio: f32,
    /// Hard cap on returned pivots (tightened to WEAK_MAX_PIVOTS when weak).
    pub max_pivots: usize,
    /// Max share of the token budget one pivot may consume.
    pub max_file_budget_share: f32,
    /// Additional absolute token cap for doc files (markdown/office/pdf/sql).
    pub doc_token_cap: usize,
    /// Extra workspace-specific noise keywords (config-only), merged with the
    /// tokens derived from repo aliases; skipped in LIKE/filename channels.
    pub noise_keywords: Vec<String>,
}

impl Default for RelevanceConfig {
    fn default() -> Self {
        RelevanceConfig {
            min_score_abs: 0.05,
            min_score_ratio: 0.30,
            max_pivots: 20,
            max_file_budget_share: 0.25,
            doc_token_cap: 1500,
            noise_keywords: Vec::new(),
        }
    }
}

impl RelevanceConfig {
    /// Merge defaults <- `.comp/config.json` values <- run_pipeline params.
    /// `min_score_abs` is config-only; the other four are also params.
    pub fn from_sources(config: &Value, params: &Value) -> Self {
        let mut cfg = RelevanceConfig::default();
        if let Some(v) = config["min_score_abs"].as_f64() {
            cfg.min_score_abs = v as f32;
        }
        if let Some(v) = config["min_score_ratio"].as_f64() {
            cfg.min_score_ratio = v as f32;
        }
        if let Some(v) = config["max_pivots"].as_u64() {
            cfg.max_pivots = v as usize;
        }
        if let Some(v) = config["max_file_budget_share"].as_f64() {
            cfg.max_file_budget_share = v as f32;
        }
        if let Some(v) = config["doc_token_cap"].as_u64() {
            cfg.doc_token_cap = v as usize;
        }
        if let Some(arr) = config["noise_keywords"].as_array() {
            cfg.noise_keywords = arr
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_lowercase()))
                .collect();
        }
        if let Some(v) = params["min_score_ratio"].as_f64() {
            cfg.min_score_ratio = v as f32;
        }
        if let Some(v) = params["max_pivots"].as_u64() {
            cfg.max_pivots = v as usize;
        }
        if let Some(v) = params["max_file_budget_share"].as_f64() {
            cfg.max_file_budget_share = v as f32;
        }
        if let Some(v) = params["doc_token_cap"].as_u64() {
            cfg.doc_token_cap = v as usize;
        }
        // Clamp to sane ranges so a bad config cannot zero out results.
        cfg.min_score_abs = cfg.min_score_abs.clamp(0.0, 1.0);
        cfg.min_score_ratio = cfg.min_score_ratio.clamp(0.0, 1.0);
        cfg.max_file_budget_share = cfg.max_file_budget_share.clamp(0.05, 1.0);
        if cfg.max_pivots == 0 {
            cfg.max_pivots = 1;
        }
        if cfg.doc_token_cap == 0 {
            cfg.doc_token_cap = 1;
        }
        cfg
    }
}

// ---------------------------------------------------------------------------
// Tests (pure, no DB, parallel-safe)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn cand(path: &str, score: f32, git_diff: bool) -> Candidate {
        Candidate {
            path: path.to_string(),
            sym_count: 1,
            base_tokens: 100,
            score,
            reasons: Vec::new(),
            git_diff,
        }
    }

    #[test]
    fn test_filter_keywords_drops_stopwords() {
        let kws = vec!["add", "user", "feedback", "surveys", "with", "time"];
        let filtered = filter_keywords(&kws);
        assert_eq!(filtered, vec!["user", "feedback", "surveys"]);
    }

    #[test]
    fn test_filter_keywords_all_stopwords_falls_back() {
        let kws = vec!["add", "the", "with"];
        let filtered = filter_keywords(&kws);
        assert_eq!(filtered, kws);
    }

    #[test]
    fn test_symbol_match_quality_ordering() {
        let exact = symbol_match_quality("survey", "survey");
        let token = symbol_match_quality("OnboardingSurveyDto", "survey");
        let prefix = symbol_match_quality("surveyResults", "survey");
        let substr = symbol_match_quality("DashboardDto", "board");
        let none = symbol_match_quality("LoginController", "survey");
        assert_eq!(exact, 1.0);
        assert_eq!(token, 0.85);
        assert_eq!(prefix, 0.85);
        assert_eq!(substr, 0.4);
        assert_eq!(none, 0.0);
        assert!(exact > token && token > substr && substr > none);
    }

    #[test]
    fn test_symbol_match_quality_snake_case_token() {
        assert_eq!(symbol_match_quality("get_survey_results", "survey"), 0.85);
    }

    #[test]
    fn test_keyword_weight_bounds_and_monotonicity() {
        let n = 100_000;
        let rare = keyword_weight(3, n);
        let mid = keyword_weight(1_000, n);
        let common = keyword_weight(50_000, n);
        assert!(rare > mid && mid > common, "{} {} {}", rare, mid, common);
        assert!(rare <= 1.0 && common > 0.0);
        // df = 0 is maximally rare
        assert!((keyword_weight(0, n) - 1.0).abs() < 1e-6);
        // Degenerate corpus: neutral weight
        assert_eq!(keyword_weight(5, 0), 1.0);
    }

    #[test]
    fn test_evidence_max_not_sum() {
        let mut ev = Evidence::default();
        // Many junk substring hits on the same keyword...
        for _ in 0..10 {
            ev.add_symbol_hit("board", 0.4, 0.5);
        }
        let mut exact = Evidence::default();
        exact.add_symbol_hit("survey", 1.0, 0.8);
        assert!(exact.symbol_component() > ev.symbol_component());
    }

    #[test]
    fn test_evidence_distinct_keyword_bonus() {
        let mut one = Evidence::default();
        one.add_symbol_hit("survey", 0.7, 0.8);
        let mut two = Evidence::default();
        two.add_symbol_hit("survey", 0.7, 0.8);
        two.add_symbol_hit("feedback", 0.7, 0.8);
        assert!(two.symbol_component() > one.symbol_component());
        // Bonus is capped at 1.0
        let mut many = Evidence::default();
        for kw in ["a1", "b2", "c3", "d4", "e5", "f6", "g7"] {
            many.add_symbol_hit(kw, 1.0, 1.0);
        }
        assert_eq!(many.symbol_component(), 1.0);
    }

    #[test]
    fn test_combine_two_engines_beat_one() {
        let mut both = Evidence::default();
        both.add_symbol_hit("survey", 0.7, 0.8);
        both.add_tfidf(0.5);
        let mut single = Evidence::default();
        single.add_symbol_hit("survey", 0.7, 0.8);
        let s_both = combine_score(&both, 0.5, 0.0, false);
        let s_single = combine_score(&single, 0.5, 0.0, false);
        assert!(s_both > s_single);
    }

    #[test]
    fn test_combine_git_diff_boost() {
        let ev = Evidence::default();
        let boosted = combine_score(&ev, 0.0, 0.0, true);
        let plain = combine_score(&ev, 0.0, 0.0, false);
        assert_eq!(boosted, GIT_DIFF_BOOST);
        assert_eq!(plain, 0.0);
    }

    #[test]
    fn test_sort_by_score_desc_with_path_tiebreak() {
        let mut cands = vec![cand("b.rs", 0.5, false), cand("a.rs", 0.5, false), cand("c.rs", 0.9, false)];
        sort_by_score(&mut cands);
        let paths: Vec<&str> = cands.iter().map(|c| c.path.as_str()).collect();
        assert_eq!(paths, vec!["c.rs", "a.rs", "b.rs"]);
    }

    #[test]
    fn test_cutoff_relative_threshold() {
        let cfg = RelevanceConfig::default();
        let cands = vec![
            cand("top.rs", 0.8, false),
            cand("mid.rs", 0.5, false),
            cand("weak.rs", 0.1, false), // below 0.30 * 0.8 = 0.24
        ];
        let (kept, dropped) = apply_cutoff(cands, &cfg);
        assert_eq!(dropped, 1);
        assert!(kept.iter().all(|c| c.path != "weak.rs"));
    }

    #[test]
    fn test_cutoff_absolute_floor() {
        let cfg = RelevanceConfig::default();
        // All feeble: top = 0.04, ratio threshold = 0.012, but abs floor = 0.05
        let cands = vec![cand("a.rs", 0.04, false), cand("b.rs", 0.02, false)];
        let (kept, dropped) = apply_cutoff(cands, &cfg);
        assert_eq!(kept.len(), 0);
        assert_eq!(dropped, 2);
    }

    #[test]
    fn test_cutoff_git_diff_exempt() {
        let cfg = RelevanceConfig::default();
        let cands = vec![
            cand("top.rs", 0.9, false),
            cand("editing.rs", 0.01, true), // would fail both thresholds
        ];
        let (kept, dropped) = apply_cutoff(cands, &cfg);
        assert_eq!(dropped, 0);
        assert!(kept.iter().any(|c| c.path == "editing.rs"));
    }

    #[test]
    fn test_assess_confidence_weak() {
        let sig = RawSignals {
            max_sym_quality: 0.4, // substring coincidences only
            strong_exact: false,
            max_tfidf_raw: 0.0,
            had_tfidf_fallback: true,
            max_bm25_raw: 0.5,
        };
        let (conf, weak) = assess_confidence(&sig);
        assert_eq!(conf, "low");
        assert!(weak);
    }

    #[test]
    fn test_assess_confidence_high_on_strong_exact() {
        let sig = RawSignals {
            max_sym_quality: 1.0,
            strong_exact: true,
            max_tfidf_raw: 0.1,
            had_tfidf_fallback: false,
            max_bm25_raw: 0.0,
        };
        let (conf, weak) = assess_confidence(&sig);
        assert_eq!(conf, "high");
        assert!(!weak);
    }

    #[test]
    fn test_assess_confidence_medium() {
        let sig = RawSignals {
            max_sym_quality: 0.7, // real token hit, but nothing strong
            strong_exact: false,
            max_tfidf_raw: 0.2,
            had_tfidf_fallback: false,
            max_bm25_raw: 1.5,
        };
        let (conf, weak) = assess_confidence(&sig);
        assert_eq!(conf, "medium");
        assert!(!weak);
    }

    #[test]
    fn test_file_cap_share_and_doc_cap() {
        let cfg = RelevanceConfig::default();
        // Code file: 25% of 8000 = 2000
        assert_eq!(file_cap(8000, &cfg, false), 2000);
        // Doc file: min(2000, 1500) = 1500
        assert_eq!(file_cap(8000, &cfg, true), 1500);
        // Small budget: doc cap not binding
        assert_eq!(file_cap(4000, &cfg, true), 1000);
        // Never zero
        assert_eq!(file_cap(1, &cfg, true), 1);
    }

    #[test]
    fn test_config_from_sources_precedence() {
        let config = json!({
            "min_score_ratio": 0.5,
            "max_pivots": 10,
            "min_score_abs": 0.1
        });
        let params = json!({
            "max_pivots": 3
        });
        let cfg = RelevanceConfig::from_sources(&config, &params);
        assert_eq!(cfg.min_score_ratio, 0.5); // from config
        assert_eq!(cfg.max_pivots, 3); // param wins over config
        assert!((cfg.min_score_abs - 0.1).abs() < 1e-6); // config-only knob
        assert_eq!(cfg.doc_token_cap, 1500); // untouched default
    }

    #[test]
    fn test_config_clamps_bad_values() {
        let config = json!({
            "min_score_ratio": 5.0,
            "max_file_budget_share": 0.0,
            "max_pivots": 0
        });
        let cfg = RelevanceConfig::from_sources(&config, &Value::Null);
        assert_eq!(cfg.min_score_ratio, 1.0);
        assert_eq!(cfg.max_file_budget_share, 0.05);
        assert_eq!(cfg.max_pivots, 1);
    }

    #[test]
    fn test_match_reasons() {
        let mut ev = Evidence::default();
        ev.add_symbol_hit("survey", 0.7, 0.8);
        ev.add_symbol_hit("feedback", 0.4, 0.5);
        ev.add_bm25(2.0);
        let reasons = match_reasons(&ev, true);
        assert_eq!(reasons, vec!["symbol:feedback+survey", "bm25", "git_diff"]);
    }

    #[test]
    fn test_derive_noise_keywords_from_aliases() {
        let aliases = vec![
            "GameLauncherCloud-Backend".to_string(),
            "GameLauncherCore".to_string(),
        ];
        let configured = vec!["Custom".to_string()];
        let noise = derive_noise_keywords(&aliases, &configured);
        for kw in ["game", "launcher", "cloud", "backend", "core", "custom"] {
            assert!(noise.contains(kw), "noise must contain {}", kw);
        }
        assert!(!noise.contains("survey"));
    }

    #[test]
    fn test_bonus_only_counts_rare_keywords() {
        let mut ev = Evidence::default();
        // Two distinct but very common keywords: reported in reasons, but
        // no diversity bonus (bonus_keywords stays empty).
        ev.add_symbol_hit("launcher", 0.85, 0.2);
        ev.add_symbol_hit("player", 0.85, 0.3);
        assert_eq!(ev.sym_keywords.len(), 2, "reporting keeps all matched keywords");
        assert!(ev.bonus_keywords.is_empty(), "common keywords earn no bonus");
        assert_eq!(ev.symbol_component(), ev.sym_best);
        // Reasons still show what matched (needed to diagnose brand noise).
        assert_eq!(match_reasons(&ev, false), vec!["symbol:launcher+player"]);
        // A rare keyword does count toward the bonus.
        let mut rare = Evidence::default();
        rare.add_symbol_hit("survey", 0.85, 0.8);
        rare.add_symbol_hit("feedback", 0.85, 0.8);
        assert_eq!(rare.bonus_keywords.len(), 2);
        assert!(rare.symbol_component() > rare.sym_best);
    }

    #[test]
    fn test_apply_keyword_coverage_all_uncovered() {
        let coverage = vec![
            KeywordCoverage { keyword: "achievements".into(), df: 0, weight: 0.9, best_quality: 0.0 },
            KeywordCoverage { keyword: "trophies".into(), df: 2, weight: 0.8, best_quality: 0.0 },
        ];
        let (conf, weak, uncovered, reason) = apply_keyword_coverage("high", false, &coverage);
        assert_eq!(conf, "low");
        assert!(weak);
        assert_eq!(uncovered.len(), 2);
        let reason = reason.expect("weak_reason must be set");
        assert!(reason.contains("achievements"), "{}", reason);
    }

    #[test]
    fn test_apply_keyword_coverage_rarest_uncovered_caps_at_medium() {
        let coverage = vec![
            KeywordCoverage { keyword: "achievements".into(), df: 0, weight: 0.9, best_quality: 0.0 },
            KeywordCoverage { keyword: "unlock".into(), df: 50, weight: 0.6, best_quality: 1.0 },
        ];
        let (conf, weak, uncovered, reason) = apply_keyword_coverage("high", false, &coverage);
        assert_eq!(conf, "medium");
        assert!(!weak);
        assert_eq!(uncovered, vec!["achievements".to_string()]);
        assert!(reason.is_none());
    }

    #[test]
    fn test_apply_keyword_coverage_covered_stays_high() {
        let coverage = vec![
            KeywordCoverage { keyword: "license".into(), df: 100, weight: 0.7, best_quality: 1.0 },
            KeywordCoverage { keyword: "entitlement".into(), df: 40, weight: 0.8, best_quality: 0.85 },
            // A common keyword with no hits must not degrade anything.
            KeywordCoverage { keyword: "backend".into(), df: 9000, weight: 0.3, best_quality: 0.0 },
        ];
        let (conf, weak, uncovered, reason) = apply_keyword_coverage("high", false, &coverage);
        assert_eq!(conf, "high");
        assert!(!weak);
        assert!(uncovered.is_empty());
        assert!(reason.is_none());
    }

    #[test]
    fn test_drop_designer_twins() {
        let cands = vec![
            cand("Backend/Migrations/AddThing.cs", 0.5, false),
            cand("Backend/Migrations/AddThing.Designer.cs", 0.5, false),
            // Lone Designer without its base stays.
            cand("Backend/Migrations/Orphan.Designer.cs", 0.4, false),
            // Git-diff Designer is exempt even with the base present.
            cand("Backend/Migrations/Edited.cs", 0.4, false),
            cand("Backend/Migrations/Edited.Designer.cs", 0.4, true),
        ];
        let kept = drop_designer_twins(cands);
        let paths: Vec<&str> = kept.iter().map(|c| c.path.as_str()).collect();
        assert!(paths.contains(&"Backend/Migrations/AddThing.cs"));
        assert!(!paths.contains(&"Backend/Migrations/AddThing.Designer.cs"));
        assert!(paths.contains(&"Backend/Migrations/Orphan.Designer.cs"));
        assert!(paths.contains(&"Backend/Migrations/Edited.Designer.cs"));
    }

    #[test]
    fn test_config_noise_keywords_parse() {
        let config = json!({ "noise_keywords": ["Foo", "bar"] });
        let cfg = RelevanceConfig::from_sources(&config, &Value::Null);
        assert_eq!(cfg.noise_keywords, vec!["foo".to_string(), "bar".to_string()]);
    }

    #[test]
    fn test_sql_is_doc_not_data() {
        assert!(is_doc_language("sql"));
        assert!(!is_data_language("sql"));
    }

    #[test]
    fn test_filename_hit_marks_reason_and_scores() {
        let mut ev = Evidence::default();
        // "onboarding-survey.component" stem matching keyword "survey"
        let quality = symbol_match_quality("onboarding-survey.component", "survey");
        assert_eq!(quality, 0.85);
        ev.add_filename_hit("survey", quality, 0.8);
        assert!(ev.filename_hit);
        assert!(ev.symbol_component() > 0.0);
        let reasons = match_reasons(&ev, false);
        assert_eq!(reasons, vec!["symbol:survey", "filename"]);
    }
}
