{
  description = "Flake for the itempool crate";

  inputs = {
    harbor-rs.url = "git+https://github.com/caniko/harbor-rs.git?ref=trunk&rev=891cf7c2827e61ed4b07e3caef6efe0543b0d5d0";
    nixpkgs.follows = "harbor-rs/nixpkgs";
    rust-overlay.follows = "harbor-rs/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    harbor-rs,
    flake-utils,
    rust-overlay,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [(import rust-overlay)];
      };

      inherit (pkgs) lib;

      toolchain = harbor-rs.lib.mkToolchain {
        inherit pkgs;
        toolchainProfile = "stable";
      };
      inherit (toolchain) rustToolchain craneLib;
      buildCache = harbor-rs.lib.mkBuildCachePolicy {
        inherit pkgs;
        sccachePackage = harbor-rs.packages.${system}.sccache;
        cacheRoot = null;
        namespaceScope = "canix-rust";
        namespaceGeneration = 5;
      };
      src = craneLib.cleanCargoSource ./.;

      # Common arguments can be set here to avoid repeating them later
      commonArgs = {
        inherit src;
        strictDeps = true;

        buildInputs = lib.optionals pkgs.stdenv.isDarwin [
          pkgs.libiconv
        ];
      };

      # Build *just* the cargo dependencies, so we can reuse
      # all of that work (e.g. via cachix) when running in CI
      cargoArtifacts = craneLib.buildDepsOnly commonArgs;

      # Build the actual crate itself, reusing the dependency
      # artifacts from above.
      my-crate = buildCache.withRustCache {
        package = craneLib.buildPackage (commonArgs
          // {
            inherit cargoArtifacts;
          });
      };
    in {
      checks = {
        # Build the crate as part of `nix flake check` for convenience
        inherit my-crate;

        # Run clippy (and deny all warnings) on the crate source,
        # again, reusing the dependency artifacts from above.
        my-crate-clippy = craneLib.cargoClippy (commonArgs
          // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          });

        # Check formatting
        my-crate-fmt = craneLib.cargoFmt {
          inherit src;
        };

        # Run tests
        my-crate-nextest = craneLib.cargoNextest (commonArgs
          // {
            inherit cargoArtifacts;
            partitions = 1;
            partitionType = "count";
          });
      };

      packages = {
        default = my-crate;
      };

      apps.default = flake-utils.lib.mkApp {
        drv = my-crate;
      };

      devShells.default = craneLib.devShell {
        # Inherit inputs from checks.
        checks = self.checks.${system};

        # Extra inputs can be added here; cargo and rustc are provided by default.
        packages = [];
      };
    });
}
