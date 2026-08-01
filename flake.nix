{
  # nanokit's pinned toolchain. Lives at the repo ROOT, which is not
  # decoration: a flake's source root is its own directory, so a flake in a
  # subdirectory cannot see `Cargo.toml`/`src/` and can offer no package at
  # all. Root is also every Rust project's shape, so it needs no teaching.
  #
  # Pinned to the same nixpkgs revision as itok, the first consumer, so the
  # two repos cannot drift into different rustc/clippy versions and
  # disagree about a verdict.
  description = "nanokit -- the cavekit SPEC format, enforced (dev shell)";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/241313f4e8e508cb9b13278c2b0fa25b9ca27163";

  outputs =
    { nixpkgs, ... }:
    let
      systems = [
        "aarch64-darwin"
        "x86_64-linux"
        "aarch64-linux"
      ];
      forAll = f: nixpkgs.lib.genAttrs systems (s: f nixpkgs.legacyPackages.${s});

      # nanokit formats its own SPEC.md (V10), so the dev shell must PROVIDE
      # `nanokit`, not merely the toolchain to build it.
      #
      # Deliberately a SHIM rather than a package in the shell's closure: a
      # package has to BUILD the crate in order to enter the shell, so one
      # compile error locks you out of the shell you need to fix it. And a
      # built package is stale against the working tree, while `cargo run`
      # is a no-op when fresh and rebuilds only on change -- so the shim
      # always matches the source you are editing, which is the whole point
      # of dogfooding.
      nanokitShim =
        pkgs:
        pkgs.writeShellScriptBin "nanokit" ''
          set -eu
          manifest="''${NANOKIT_MANIFEST:-}"
          if [ -z "$manifest" ] || [ ! -f "$manifest" ]; then
            echo "nanokit(shim): NANOKIT_MANIFEST unset or missing -- re-enter the dev shell from the repo root" >&2
            exit 2
          fi
          profile=""
          [ "''${NANOKIT_PROFILE:-debug}" = "release" ] && profile="--release"
          exec cargo run --quiet $profile \
            --manifest-path "$manifest" --bin nanokit -- "$@"
        '';

      # The reproducible build. Possible because the flake sits at the root
      # and `Cargo.lock` is tracked -- flakes copy git-TRACKED files into
      # the store, and the build sandbox has no network, so the lock is what
      # lets nix vendor crates offline. nanokit has zero dependencies, so
      # there is nothing to vendor today; the lock still matters, because
      # the guarantee should not depend on staying dependency-free.
      nanokitPkg =
        pkgs:
        pkgs.rustPlatform.buildRustPackage {
          pname = "nanokit";
          # Read from Cargo.toml so there is ONE version, never two that can
          # disagree.
          version = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package.version;
          # Only what the BUILD reads. With `src = ./.` every tracked file is
          # an input, so editing SPEC.md would rebuild the crate from
          # scratch -- which for a tool whose job is editing SPEC.md would be
          # a rebuild on nearly every commit.
          src = pkgs.lib.fileset.toSource {
            root = ./.;
            fileset = pkgs.lib.fileset.unions [
              ./src
              ./Cargo.toml
              ./Cargo.lock
            ];
          };
          cargoLock.lockFile = ./Cargo.lock;
          # `tests/dogfood.rs` reads SPEC.md, which is deliberately NOT in
          # the fileset above. So the dogfood cannot run here; it runs in the
          # dev shell and in CI, where the whole tree exists. Excluding the
          # spec from the build inputs and running its test elsewhere is the
          # same tradeoff seen from both sides.
          doCheck = false;
          meta = {
            description = "enforce the cavekit SPEC format: lossless minify, structural check, CPU only";
            homepage = "https://github.com/pr0d1r2/nanokit";
            license = pkgs.lib.licenses.mit;
            mainProgram = "nanokit";
          };
        };
    in
    {
      packages = forAll (pkgs: {
        default = nanokitPkg pkgs;
      });

      devShells = forAll (pkgs: {
        default = pkgs.mkShell {
          packages = [
            (nanokitShim pkgs)
            pkgs.rustc
            pkgs.cargo
            pkgs.clippy
            pkgs.rustfmt
            pkgs.cargo-nextest
            pkgs.cargo-llvm-cov
            pkgs.llvmPackages.llvm
            pkgs.git
            # The gate runner. Present now so the shell is ready for it;
            # the ops it runs (`hk.pkl`) arrive with T6.
            pkgs.hk
            # Gate steps that need a real binary, pinned here so the dev
            # shell and CI run identical versions rather than whatever each
            # happened to install.
            pkgs.typos
            pkgs.taplo
            # `nixfmt --check` gates this very file: the flake decides what
            # every other step runs with, so drift here is drift everywhere.
            pkgs.nixfmt
          ];
          # Pin locale so tool output is deterministic across machines.
          LANG = "C.UTF-8";
          LLVM_COV = "${pkgs.llvmPackages.llvm}/bin/llvm-cov";
          LLVM_PROFDATA = "${pkgs.llvmPackages.llvm}/bin/llvm-profdata";
          # Resolved at ENTRY rather than baked in: `.envrc` sits beside
          # `Cargo.toml` and direnv enters with PWD set to that directory,
          # so this works wherever the repo is checked out. Derive, never
          # hardcode a path.
          #
          # NO git-hook installation here yet, on purpose. The hook command
          # would invoke `hk run`, and `hk.pkl` does not exist until T6 --
          # so installing now would fail every commit rather than gate it.
          # A hook whose runner cannot succeed does not gate the repo, it
          # blocks it. Hooks land in the same commit as the ops they run.
          shellHook = ''
            if [ -f "$PWD/Cargo.toml" ]; then
              export NANOKIT_MANIFEST="$PWD/Cargo.toml"
            else
              echo "nanokit(shell): no Cargo.toml in $PWD -- \`nanokit\` shim disabled" >&2
            fi
          '';
        };
      });
    };
}
