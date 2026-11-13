//! Checks for the repository's development-shell contract.

/// Whether a flake's `devShells` package list contains the named package.
#[must_use]
pub fn dev_shell_has_package(text: &str, package: &str) -> bool {
    let wanted = format!("pkgs.{package}");
    text.lines()
        .scan((0_i8, false), |state, line| scan_line(state, line, &wanted))
        .any(|found| found)
}

fn scan_line(state: &mut (i8, bool), line: &str, wanted: &str) -> Option<bool> {
    let trimmed = line.trim();
    if state.0 == 0 && trimmed.starts_with("devShells") {
        state.0 = brace_delta(trimmed);
    }
    if state.0 > 0 && trimmed == "packages = [" {
        state.1 = true;
    }
    let found = state.1 && trimmed == wanted;
    if state.1 && trimmed == "];" {
        state.1 = false;
    }
    if state.0 > 0 && !trimmed.starts_with("devShells") {
        state.0 = state.0.saturating_add(brace_delta(trimmed));
    }
    Some(found)
}

fn brace_delta(line: &str) -> i8 {
    line.chars().fold(0, |delta, c| match c {
        '{' => delta.saturating_add(1),
        '}' => delta.saturating_sub(1),
        _ => delta,
    })
}

#[cfg(test)]
mod tests {
    use super::dev_shell_has_package;

    #[test]
    fn planted_missing_package_is_rejected() {
        let flake = "devShells = forAll (pkgs: { default = pkgs.mkShell { packages = [ pkgs.rustc ]; }; });";
        assert!(!dev_shell_has_package(flake, "cargo-release"));
    }

    #[test]
    fn package_entry_is_accepted() {
        let flake = "devShells = forAll (pkgs: {\n  default = pkgs.mkShell {\n    packages = [\n      pkgs.cargo-release\n    ];\n  };\n});";
        assert!(dev_shell_has_package(flake, "cargo-release"));
    }

    #[test]
    fn package_outside_dev_shell_is_rejected() {
        let flake = "devShells = {};
packages = [
  pkgs.cargo-release
];";
        assert!(!dev_shell_has_package(flake, "cargo-release"));
    }
}
