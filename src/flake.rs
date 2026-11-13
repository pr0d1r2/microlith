//! Checks for the repository's development-shell contract.

/// Whether a flake's `devShells` package list contains the named package.
#[must_use]
pub fn dev_shell_has_package(text: &str, package: &str) -> bool {
    let wanted = format!("pkgs.{package}");
    text.lines()
        .scan((false, false), |(in_dev, in_packages), line| {
            let trimmed = line.trim();
            *in_dev |= trimmed.starts_with("devShells");
            if *in_dev && trimmed == "packages = [" {
                *in_packages = true;
            }
            let found = *in_packages && trimmed == wanted;
            if *in_packages && trimmed == "];" {
                *in_packages = false;
            }
            Some(found)
        })
        .any(|found| found)
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
}
