{
  description = "Rust development environment";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.05";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ rust-overlay.overlays.default ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        
        # Define rust toolchain from rust-toolchain.toml or use default
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustToolchain
            
            # Common cargo tools
            cargo-edit
            cargo-watch
            cargo-audit
            cargo-outdated
            cargo-nextest
            
            # Build dependencies
            pkg-config
            openssl
            
            # Optional: useful dev tools
            just
            bacon
          ];

          # Environment variables
          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";

          shellHook = ''
            printf "\n    \033[1;35m🦀 Rust DevShell\033[0m\n\n"
            echo "Rust: $(rustc --version)"
            echo "Cargo: $(cargo --version)"
            echo "Rust-analyzer: $(rust-analyzer --version)"
            printf "\n"
          '';
        };
      }
    );
}

