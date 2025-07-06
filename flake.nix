{
    description = "Mothership terminal";

    inputs = {
        nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
        rust-overlay.url = "github:oxalica/rust-overlay";
    };

    outputs = { self, nixpkgs, rust-overlay }:
    let
        system = "x86_64-linux";
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
            inherit system overlays;
        };
        rust-toolchain = pkgs.rust-bin.stable.latest.default;
    in {
        devShells.${system}.default = with pkgs; mkShell {
            buildInputs = [
                rust-toolchain
                cargo-watch
                rust-analyzer
                rustfmt
                clippy

                cool-retro-term
            ];
            shellHook = ''
            '';
        };
    };
}
