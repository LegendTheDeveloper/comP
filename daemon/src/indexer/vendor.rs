// vendor.rs — first-party vs third-party classification of indexed files
//
// WHY this exists: a Unity workspace carries whole asset-store packages inside
// Assets/ (PostProcessing, Modern UI Pack, Photon, TextMesh Pro...) and a .NET
// one commits NuGet packages. Their symbols are real, so they matched
// run_pipeline keywords exactly like the project's own code: on the Lynium
// workspace ~30% of returned pivots came from folders the user had never
// edited, and "game settings model" answered with eleven PostProcessing
// `*Model.cs` files above `GameSettings.cs`. Excluding them would break
// legitimate lookups (Steamworks, UniTask), so they stay indexed and are
// merely demoted: each file carries a `vendor` flag that scoring multiplies
// by a factor and that the response reports.
//
// Three sources decide, in this order:
//   1. `first_party_paths` / `vendor_paths` in `.comp/config.json`, plus the
//      lines of `.comp/vendor` (gitignore syntax) — the user's word wins.
//   2. Git activity: a folder the project has committed to repeatedly is
//      first-party even if it lives under a built-in vendor path (LoginPro
//      and TigerForge are bought assets that Lynium patches constantly).
//   3. Built-in patterns for Unity/.NET/JS package folders, then the
//      inactive-folder heuristic: a big folder nobody has touched in the
//      activity window is a package that was dropped in once.
//
// Measured on Lynium (distinct commits per top-level Assets folder over 18
// months): Scripts 452, Editor 247, LoginPro 42, TigerForge 27,
// Implementation 23 ... Plugins 5, Photon 5, TextMesh Pro 2, MHLab 2,
// Frost UI 2, PostProcessing 1, Modern UI Pack 0, WorldFlags 0.

use ignore::gitignore::{Gitignore, GitignoreBuilder};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Package folders that are third-party in practically every project.
/// gitignore semantics: a pattern with an inner slash is anchored at the repo
/// root, a bare `name/` matches that directory anywhere.
pub const BUILTIN_VENDOR_PATTERNS: &[&str] = &[
    "Assets/Plugins/",
    "Assets/Standard Assets/",
    "Assets/TextMesh Pro/",
    "Assets/PostProcessing/",
    "Assets/Photon/",
    "/Packages/",
    "/Library/",
    "packages/",
    "bin/",
    "obj/",
    "node_modules/",
    "vendor/",
    "dist/",
    "wwwroot/lib/",
    "ThirdParty/",
    "third_party/",
    "3rdParty/",
    "External/",
    "Externals/",
    "phpmailer/",
    "*.min.js",
];

/// Repos with fewer commits than this in the activity window carry too little
/// history for the inactive-folder heuristic to mean anything.
pub const MIN_REPO_COMMITS_FOR_AUTO: usize = 50;

/// Tunables, read from `.comp/config.json` (all optional).
#[derive(Debug, Clone, PartialEq)]
pub struct VendorConfig {
    /// gitignore-style patterns forced to vendor (`vendor_paths` + `.comp/vendor`).
    pub vendor_paths: Vec<String>,
    /// gitignore-style patterns forced to first-party (`first_party_paths`).
    pub first_party_paths: Vec<String>,
    /// Enable the git inactive-folder heuristic (`vendor_auto`, default true).
    pub auto: bool,
    /// Folder commit count at or above which git activity overrides the
    /// built-in patterns (`vendor_active_min_commits`, default 6).
    pub active_min_commits: usize,
    /// Folder commit count at or below which a large folder is treated as a
    /// dropped-in package (`vendor_auto_max_commits`, default 3).
    pub auto_max_commits: usize,
    /// Minimum indexed files in a folder for the heuristic to apply
    /// (`vendor_auto_min_files`, default 20).
    pub auto_min_files: usize,
    /// How far back `git log` looks (`vendor_activity_months`, default 18).
    pub activity_months: u32,
    /// How long the cached activity stays valid (`vendor_activity_ttl_hours`, default 24).
    pub activity_ttl_hours: u64,
}

impl Default for VendorConfig {
    fn default() -> Self {
        VendorConfig {
            vendor_paths: Vec::new(),
            first_party_paths: Vec::new(),
            auto: true,
            active_min_commits: 6,
            auto_max_commits: 3,
            auto_min_files: 20,
            activity_months: 18,
            activity_ttl_hours: 24,
        }
    }
}

impl VendorConfig {
    /// Read `.comp/config.json` and `.comp/vendor` under `repo_root`.
    pub fn load(repo_root: &str) -> Self {
        let comp = Path::new(repo_root).join(".comp");
        let content = std::fs::read_to_string(comp.join("config.json")).unwrap_or_default();
        let json: serde_json::Value =
            serde_json::from_str(&content).unwrap_or(serde_json::Value::Null);
        let mut cfg = Self::from_json(&json);
        if let Ok(lines) = std::fs::read_to_string(comp.join("vendor")) {
            cfg.vendor_paths.extend(
                lines
                    .lines()
                    .map(|l| l.trim())
                    .filter(|l| !l.is_empty() && !l.starts_with('#'))
                    .map(|l| l.to_string()),
            );
        }
        cfg
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        let strings = |key: &str| -> Vec<String> {
            json[key]
                .as_array()
                .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default()
        };
        let d = VendorConfig::default();
        VendorConfig {
            vendor_paths: strings("vendor_paths"),
            first_party_paths: strings("first_party_paths"),
            auto: json["vendor_auto"].as_bool().unwrap_or(d.auto),
            active_min_commits: json["vendor_active_min_commits"].as_u64().map(|v| v as usize).unwrap_or(d.active_min_commits),
            auto_max_commits: json["vendor_auto_max_commits"].as_u64().map(|v| v as usize).unwrap_or(d.auto_max_commits),
            auto_min_files: json["vendor_auto_min_files"].as_u64().map(|v| v as usize).unwrap_or(d.auto_min_files),
            activity_months: json["vendor_activity_months"].as_u64().map(|v| v as u32).unwrap_or(d.activity_months),
            activity_ttl_hours: json["vendor_activity_ttl_hours"].as_u64().unwrap_or(d.activity_ttl_hours),
        }
    }
}

/// Distinct commits per folder over the activity window, cached in the DB
/// `metadata` table as JSON under `vendor_activity:<alias>`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct GitActivity {
    /// Unix seconds when `git log` ran.
    pub computed_at: u64,
    /// Commits in the window, repo-wide.
    pub repo_commits: usize,
    /// Distinct commits touching each folder key (`a` and `a/b`, see `folder_key`).
    pub folders: HashMap<String, usize>,
}

impl GitActivity {
    /// Run `git log` in `repo_root`. None when git is missing, the folder is
    /// not a repository, or the command fails: callers then skip every
    /// activity-based rule instead of guessing.
    pub fn compute(repo_root: &str, months: u32) -> Option<Self> {
        let since = format!("{} months ago", months.max(1));
        let output = std::process::Command::new("git")
            .args([
                "-c",
                "core.quotepath=false",
                "log",
                "--since",
                &since,
                "--name-only",
                "--pretty=format:@@%h",
            ])
            .current_dir(repo_root)
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let mut activity = Self::parse(&String::from_utf8_lossy(&output.stdout));
        activity.computed_at = now_secs();
        Some(activity)
    }

    /// Parse `git log --name-only --pretty=format:@@%h` output.
    pub fn parse(log: &str) -> Self {
        let mut folders: HashMap<String, usize> = HashMap::new();
        let mut repo_commits = 0usize;
        let mut touched: HashSet<String> = HashSet::new();
        let flush = |touched: &mut HashSet<String>, folders: &mut HashMap<String, usize>| {
            for key in touched.drain() {
                *folders.entry(key).or_insert(0) += 1;
            }
        };
        for line in log.lines() {
            let line = line.trim();
            if line.starts_with("@@") {
                flush(&mut touched, &mut folders);
                repo_commits += 1;
                continue;
            }
            if line.is_empty() {
                continue;
            }
            let segments: Vec<&str> = line.split('/').collect();
            if segments.len() >= 2 {
                touched.insert(segments[0].to_string());
            }
            if segments.len() >= 3 {
                touched.insert(format!("{}/{}", segments[0], segments[1]));
            }
        }
        flush(&mut touched, &mut folders);
        GitActivity { computed_at: 0, repo_commits, folders }
    }

    pub fn is_fresh(&self, ttl_hours: u64) -> bool {
        now_secs().saturating_sub(self.computed_at) < ttl_hours.saturating_mul(3600)
    }

    pub fn commits_for(&self, key: &str) -> usize {
        self.folders.get(key).copied().unwrap_or(0)
    }
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// The folder a file is judged by: `a/b` for `a/b/c/...`, `a` for `a/c`,
/// nothing for a file at the repo root (root files are always first-party).
pub fn folder_key(rel_path: &str) -> Option<String> {
    let segments: Vec<&str> = rel_path.split('/').filter(|s| !s.is_empty()).collect();
    match segments.len() {
        0 | 1 => None,
        2 => Some(segments[0].to_string()),
        _ => Some(format!("{}/{}", segments[0], segments[1])),
    }
}

/// Count indexed files per folder key; feeds the inactive-folder heuristic.
pub fn count_files_per_folder<'a>(paths: impl Iterator<Item = &'a str>) -> HashMap<String, usize> {
    let mut out = HashMap::new();
    for p in paths {
        if let Some(key) = folder_key(p) {
            *out.entry(key).or_insert(0) += 1;
        }
    }
    out
}

pub struct VendorClassifier {
    config_vendor: Gitignore,
    config_first_party: Gitignore,
    builtin: Gitignore,
    activity: Option<GitActivity>,
    files_per_folder: HashMap<String, usize>,
    cfg: VendorConfig,
}

impl VendorClassifier {
    pub fn new(
        repo_root: &str,
        cfg: VendorConfig,
        activity: Option<GitActivity>,
        files_per_folder: HashMap<String, usize>,
    ) -> Self {
        let build = |patterns: &[String]| -> Gitignore {
            let mut b = GitignoreBuilder::new(repo_root);
            for p in patterns {
                if let Err(e) = b.add_line(None, p) {
                    log::warn!("vendor pattern {:?} ignored: {}", p, e);
                }
            }
            b.build().unwrap_or_else(|_| Gitignore::empty())
        };
        let builtin: Vec<String> = BUILTIN_VENDOR_PATTERNS.iter().map(|s| s.to_string()).collect();
        VendorClassifier {
            config_vendor: build(&cfg.vendor_paths),
            config_first_party: build(&cfg.first_party_paths),
            builtin: build(&builtin),
            activity,
            files_per_folder,
            cfg,
        }
    }

    /// Classify a repo-relative path. Returns (is_vendor, reason); the reason
    /// is stored beside the flag so `get_stats` can show why a folder was
    /// demoted and the user can correct it in config.
    pub fn classify(&self, rel_path: &str) -> (bool, &'static str) {
        let path = Path::new(rel_path);
        if self.config_first_party.matched_path_or_any_parents(path, false).is_ignore() {
            return (false, "first-party:config");
        }
        if self.config_vendor.matched_path_or_any_parents(path, false).is_ignore() {
            return (true, "config");
        }
        let key = folder_key(rel_path);
        let commits = match (&self.activity, &key) {
            (Some(a), Some(k)) => Some(a.commits_for(k)),
            _ => None,
        };
        if commits.is_some_and(|c| c >= self.cfg.active_min_commits) {
            return (false, "first-party:git");
        }
        if self.builtin.matched_path_or_any_parents(path, false).is_ignore() {
            return (true, "builtin");
        }
        if self.cfg.auto {
            if let (Some(a), Some(k), Some(c)) = (&self.activity, &key, commits) {
                let files = self.files_per_folder.get(k).copied().unwrap_or(0);
                if a.repo_commits >= MIN_REPO_COMMITS_FOR_AUTO
                    && files >= self.cfg.auto_min_files
                    && c <= self.cfg.auto_max_commits
                {
                    return (true, "git-inactive");
                }
            }
        }
        (false, "")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn activity(pairs: &[(&str, usize)], repo_commits: usize) -> GitActivity {
        GitActivity {
            computed_at: now_secs(),
            repo_commits,
            folders: pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
        }
    }

    fn classifier(cfg: VendorConfig, act: Option<GitActivity>, files: &[(&str, usize)]) -> VendorClassifier {
        VendorClassifier::new(
            ".",
            cfg,
            act,
            files.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
        )
    }

    #[test]
    fn test_parse_git_log_counts_distinct_commits_per_folder() {
        let log = "@@aaa\nAssets/Scripts/A.cs\nAssets/Scripts/Sub/B.cs\n\n@@bbb\nAssets/Scripts/A.cs\nAssets/Photon/P.cs\n\n@@ccc\nREADME.md\n";
        let a = GitActivity::parse(log);
        assert_eq!(a.repo_commits, 3);
        assert_eq!(a.commits_for("Assets/Scripts"), 2, "same folder twice in one commit counts once");
        assert_eq!(a.commits_for("Assets"), 2);
        assert_eq!(a.commits_for("Assets/Photon"), 1);
        assert_eq!(a.commits_for("README.md"), 0, "root files have no folder key");
    }

    #[test]
    fn test_folder_key_depths() {
        assert_eq!(folder_key("Assets/Scripts/Framework/X.cs").as_deref(), Some("Assets/Scripts"));
        assert_eq!(folder_key("Assets/X.cs").as_deref(), Some("Assets"));
        assert_eq!(folder_key("README.md"), None);
    }

    #[test]
    fn test_builtin_patterns_and_git_active_override() {
        let act = activity(&[("Assets/LoginPro", 42), ("Assets/Plugins", 5), ("Assets/Scripts", 400)], 900);
        let c = classifier(VendorConfig::default(), Some(act), &[]);
        assert_eq!(c.classify("Assets/Plugins/CodeStage/Obscured.cs"), (true, "builtin"));
        assert_eq!(c.classify("Assets/PostProcessing/Runtime/Models/BloomModel.cs"), (true, "builtin"));
        assert_eq!(c.classify("Assets/Scripts/Framework/GameSettings.cs"), (false, "first-party:git"));
        // 42 commits: an actively patched bought asset stays first-party even
        // though nothing in config says so.
        assert_eq!(c.classify("Assets/LoginPro/System/Session.cs"), (false, "first-party:git"));
        assert_eq!(c.classify("src/node_modules/left-pad/index.js"), (true, "builtin"));
        assert_eq!(c.classify("Packages/System.Runtime.4.3.0/lib/x.xml"), (true, "builtin"));
        assert_eq!(c.classify("Packages/manifest.json").0, true, "root Packages/ is the built-in default; override with first_party_paths");
    }

    #[test]
    fn test_git_inactive_folder_needs_size_and_history() {
        let act = activity(&[("Assets/Modern UI Pack", 0), ("Assets/Tiny", 0)], 900);
        let c = classifier(
            VendorConfig::default(),
            Some(act),
            &[("Assets/Modern UI Pack", 300), ("Assets/Tiny", 3)],
        );
        assert_eq!(c.classify("Assets/Modern UI Pack/Scripts/ButtonManager.cs"), (true, "git-inactive"));
        assert_eq!(c.classify("Assets/Tiny/One.cs"), (false, ""), "small folders are never auto-vendor");
        // A repo with almost no history cannot tell active from inactive.
        let young = activity(&[("Assets/Modern UI Pack", 0)], 12);
        let c2 = classifier(VendorConfig::default(), Some(young), &[("Assets/Modern UI Pack", 300)]);
        assert_eq!(c2.classify("Assets/Modern UI Pack/Scripts/ButtonManager.cs"), (false, ""));
        // No git at all: only config and built-ins decide.
        let c3 = classifier(VendorConfig::default(), None, &[("Assets/Modern UI Pack", 300)]);
        assert_eq!(c3.classify("Assets/Modern UI Pack/Scripts/ButtonManager.cs"), (false, ""));
    }

    #[test]
    fn test_config_wins_over_everything() {
        let act = activity(&[("Assets/Photon", 40), ("Assets/Scripts", 400)], 900);
        let cfg = VendorConfig {
            vendor_paths: vec!["Assets/Photon/".into(), "**/Examples to study/".into()],
            first_party_paths: vec!["Assets/Plugins/Ours/".into()],
            ..VendorConfig::default()
        };
        let c = classifier(cfg, Some(act), &[]);
        assert_eq!(c.classify("Assets/Photon/PhotonUnityNetworking/Code/PhotonNetwork.cs"), (true, "config"));
        assert_eq!(c.classify("Assets/Plugins/Ours/Helper.cs"), (false, "first-party:config"));
        assert_eq!(c.classify("Assets/LoginPro/Use my content/Examples to study/X.cs"), (true, "config"));
        assert_eq!(c.classify("Assets/Scripts/A.cs"), (false, "first-party:git"));
    }

    #[test]
    fn test_root_files_are_first_party() {
        let c = classifier(VendorConfig::default(), None, &[]);
        assert_eq!(c.classify("README.md"), (false, ""));
        assert_eq!(c.classify("bundle.min.js"), (true, "builtin"));
    }

    #[test]
    fn test_config_from_json_defaults_and_overrides() {
        let json = serde_json::json!({
            "vendor_paths": ["Assets/Photon/"],
            "vendor_auto": false,
            "vendor_active_min_commits": 10
        });
        let cfg = VendorConfig::from_json(&json);
        assert_eq!(cfg.vendor_paths, vec!["Assets/Photon/".to_string()]);
        assert!(!cfg.auto);
        assert_eq!(cfg.active_min_commits, 10);
        assert_eq!(cfg.auto_max_commits, 3);
        assert_eq!(cfg.activity_months, 18);
    }

    #[test]
    fn test_activity_freshness() {
        let mut a = GitActivity::default();
        assert!(!a.is_fresh(24), "computed_at 0 is stale");
        a.computed_at = now_secs();
        assert!(a.is_fresh(24));
    }
}
