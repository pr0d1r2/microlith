use super::*;

#[test]
fn deps_are_counted_per_key_not_per_line() {
    let toml = "[dependencies]\n# a comment\nserde = { version = \"1\", features = [\n  \"derive\",\n] }\n\nregex = \"1\"\n\n[dependencies.clap]\nversion = \"4\"\n\n[dev-dependencies]\nproptest = \"1\"\n";
    assert_eq!(dependency_count(toml), 3);
}

#[test]
fn deps_are_zero_for_an_empty_table() {
    assert_eq!(
        dependency_count("[package]\nname = \"x\"\n\n[dependencies]\n"),
        0
    );
}

/// The decoy sits FIRST, as a prefix-named node with its own `nixpkgs` input
/// and a channel of its own. A first-match read badges 1.01.
const DECOY_LOCK: &str = r#"{
  "nodes": {
    "nixpkgs-lock": {
      "inputs": { "nixpkgs": "nixpkgs" },
      "original": { "ref": "nixos-1.01", "type": "github" }
    },
    "nix-hk": {
      "inputs": { "nixpkgs": { "original": { "ref": "nixos-99.99" } } },
      "original": { "ref": "nixos-2.02" }
    },
    "nixpkgs": {
      "locked": { "narHash": "sha256-\"quoted\"" },
      "original": { "owner": "NixOS", "ref": "nixos-26.05" }
    },
    "root": {
      "inputs": { "nix-hk": "nix-hk", "nixpkgs": ["nixpkgs-lock", "nixpkgs"],
        "nixpkgs-lock": "nixpkgs-lock" }
    }
  },
  "root": "root",
  "version": 7
}"#;

#[test]
fn channel_follows_the_root_input_past_a_decoy() {
    assert_eq!(nixpkgs_channel(DECOY_LOCK), Ok("26.05".to_string()));
}

#[test]
fn channel_reads_a_direct_input() {
    let lock = r#"{"nodes":{"np":{"original":{"ref":"nixos-25.11"}},"root":{"inputs":{"nixpkgs":"np"}}}}"#;
    assert_eq!(nixpkgs_channel(lock), Ok("25.11".to_string()));
}

#[test]
fn channel_refuses_what_it_cannot_read() {
    let unstable = r#"{"nodes":{"np":{"original":{"ref":"nixpkgs-unstable"}},"root":{"inputs":{"nixpkgs":"np"}}}}"#;
    assert!(
        nixpkgs_channel(unstable)
            .is_err_and(|e| e.contains("nixpkgs-unstable"))
    );
    assert!(nixpkgs_channel(r#"{"nodes":{"root":{"inputs":{}}}}"#).is_err());
    assert!(nixpkgs_channel("{}").is_err());
    assert!(
        nixpkgs_channel(r#"{"nodes":{"root":{"inputs":{"nixpkgs":"np"}}"#)
            .is_err()
    );
}

#[test]
fn platforms_come_from_the_ci_matrix() {
    let flow = "jobs:\n  t:\n    strategy:\n      matrix:\n        os: [ubuntu-latest, ubuntu-24.04-arm, 'macos-latest']\n";
    let block = "matrix:\n  os:\n    - macos-latest\n    - \"ubuntu-24.04-arm\"\nsteps:\n  - run: x\n";
    assert_eq!(
        platforms(flow),
        Ok(vec!["intel linux", "amd linux", "arm linux", "arm macos"])
    );
    assert_eq!(platforms(block), Ok(vec!["arm macos", "arm linux"]));
}

#[test]
fn an_unknown_runner_is_an_error_not_a_guess() {
    assert!(
        platforms("matrix:\n  os: [windows-latest]\n")
            .is_err_and(|e| e.contains("windows-latest"))
    );
    assert!(platforms("os: [ubuntu-latest]\n").is_err());
}

#[test]
fn splice_replaces_only_the_block_and_is_idempotent() {
    let readme = "a\n<!-- BEGIN badges -->\nold\n<!-- END badges -->\nz\n";
    let once = splice(readme, "new\n");
    assert_eq!(
        once.as_deref(),
        Ok("a\n<!-- BEGIN badges -->\nnew\n<!-- END badges -->\nz\n")
    );
    assert_eq!(
        once.and_then(|r| splice(&r, "new\n")).as_deref(),
        Ok(readme.replace("old", "new").as_str())
    );
}

#[test]
fn splice_refuses_missing_or_doubled_markers() {
    let pair = "<!-- BEGIN badges -->\n<!-- END badges -->\n";
    assert!(splice("x\n", "b").is_err());
    assert!(splice("<!-- BEGIN badges -->\nx\n", "b").is_err());
    assert!(splice(&format!("{pair}{pair}"), "b").is_err());
}

#[test]
fn diff_names_the_moved_badge() {
    assert_eq!(diff("a\nb\n", "a\nc\n"), "- b\n+ c\n");
}

#[test]
fn scope_runs_only_when_an_input_changed() {
    assert!(in_scope(&[]));
    assert!(in_scope(&["src/main.rs", "flake.lock"]));
    assert!(in_scope(&["dev/src/lib.rs"]));
    assert!(!in_scope(&["src/main.rs", "SPEC.md"]));
}

/// The two-layer selection holds only if hk hands this crate every change
/// to an input: the scoped step's glob IS the input list.
#[test]
fn hk_glob_is_the_input_list() {
    let hk = include_str!("../../hk.pkl");
    let glob = hk
        .split("[\"readme-badges\"]")
        .nth(1)
        .and_then(|step| step.split("glob = List(").nth(1))
        .and_then(|list| list.split(')').next())
        .map(|list| {
            list.split(',')
                .map(|s| s.trim().trim_matches('"'))
                .collect::<Vec<_>>()
        });
    assert_eq!(glob, Some(INPUTS.to_vec()));
    assert!(hk.contains("[\"readme-badges-all\"]"));
}

#[test]
fn usage_is_exit_two_and_help_is_zero() {
    let root = Path::new("/nonexistent");
    let call = |args: &[&str]| {
        let args: Vec<String> = args.iter().map(|a| (*a).to_string()).collect();
        run(&args, root, &mut Vec::new())
    };
    assert_eq!(call(&[]), 2);
    assert_eq!(call(&["badges"]), 2);
    assert_eq!(call(&["readme", "--bogus"]), 2);
    assert_eq!(call(&["readme", "--help"]), 0);
    assert_eq!(call(&["readme", "--check", "SPEC.md"]), 0);
    assert_eq!(call(&["readme", "--check"]), 1);
}

/// V54: the dev crate never ships. Either flag alone leaves a path out --
/// `publish` guards crates.io, `release` guards cargo-release's tag.
#[test]
fn this_crate_never_ships() {
    let manifest = include_str!("../Cargo.toml");
    assert!(manifest.contains("\npublish = false\n"));
    assert!(manifest.contains("[package.metadata.release]\nrelease = false\n"));
}
