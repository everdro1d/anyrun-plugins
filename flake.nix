{
  description = "Anyrun plugin development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github: oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs @ { self, nixpkgs, flake-utils, rust-overlay, ...  }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ rust-overlay.overlays.default ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        lockFile = ./Cargo.lock;

        # Rust toolchain with overlay
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };
      in
      {
        formatter = pkgs.nixpkgs-fmt;

        # Development shell for plugin development
        devShells.default = pkgs.mkShell {
          name = "anyrun-plugin-dev";

          buildInputs = [
            # Rust toolchain
            rustToolchain

            # Required libraries for anyrun plugins
            pkgs.glib
            pkgs.atk
            pkgs.gtk3
            pkgs.gtk-layer-shell
            pkgs.librsvg
            pkgs.pango
            pkgs.cairo
            pkgs.gdk-pixbuf
          ];

          nativeBuildInputs = [
            pkgs.pkg-config
            pkgs.makeWrapper
          ];

          # Environment variables for building
          RUST_BACKTRACE = "1";
          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";

          shellHook = ''
            echo "Anyrun plugin development environment"
            echo "Rust version:  $(rustc --version)"
            echo ""
            echo "Available commands:"
            echo "  cargo build --release  - Build plugins"
            echo "  cargo check            - Check for errors"
            echo "  cargo clippy           - Run linter"
            echo ""
          '';
        };

        packages = {
          # Example:  expose each plugin as a package
          # Uncomment and modify based on your plugins
          #
          # my-plugin = pkgs.callPackage ./plugin.nix {
          #   inherit inputs lockFile;
          #   name = "my-plugin";
          # };
        };
      }
    );
}
