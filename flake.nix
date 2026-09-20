{
    description = "M0TH3R@3: retro terminal widgets, the Mothership ship interface and the car display";

    inputs = {
        nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
        rust-overlay = {
            url = "github:oxalica/rust-overlay";
            inputs.nixpkgs.follows = "nixpkgs";
        };
        naersk = {
            url = "github:nix-community/naersk";
            inputs.nixpkgs.follows = "nixpkgs";
        };
    };

    outputs = { self, nixpkgs, rust-overlay, naersk }:
    let
        system = "x86_64-linux";
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
            inherit system overlays;
        };
        rust-toolchain = pkgs.rust-bin.stable.latest.default;
        version = "0.1.0";
        naersk' = naersk.lib.${system}.override {
            cargo = rust-toolchain;
            rustc = rust-toolchain;
        };
        # One derivation per workspace binary. Cargo would happily build both
        # into one output, but a server only needs the mothership binary and
        # a car image only the car one.
        buildBin = name: naersk'.buildPackage {
            # naersk names the derivation after the workspace unless told
            # otherwise, and "rust-workspace" is not a useful store name.
            name = name;
            inherit version;
            src = ./.;
            cargoBuildOptions = opts: opts ++ [ "-p" name ];
            # The workspace tests are unit tests without external services;
            # the SSH round trip is covered by the VM test in checks instead.
            doCheck = true;
            cargoTestOptions = opts: opts ++ [ "-p" name ];
        };
    in {
        packages.${system} = {
            mothership = buildBin "mothership";
            car = buildBin "car";
            default = self.packages.${system}.mothership;
        };

        nixosModules = {
            mothership = import ./nix/module.nix { inherit self; };
            default = self.nixosModules.mothership;
        };

        checks.${system} = {
            mothership = self.packages.${system}.mothership;
            car = self.packages.${system}.car;
            mothership-service = pkgs.callPackage ./nix/test.nix { inherit self; };
        };

        devShells.${system}.default = with pkgs; mkShell {
            buildInputs = [
                rust-toolchain
                cargo-watch
                rust-analyzer
                rustfmt
                clippy
                
                python3

                cool-retro-term
            ];
            shellHook = ''
            '';
        };
    };
}
