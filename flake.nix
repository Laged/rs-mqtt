{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = { self, flake-utils, naersk, nixpkgs }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = (import nixpkgs) {
          inherit system;
        };

        naersk' = pkgs.callPackage naersk { };

      in
      {
        # A default package for nix build .
        packages.default= naersk'.buildPackage {
          pname = "broker";
          src = ./.;
        };
        # For `nix build .#broker` and `nix run .#broker`:
        packages.broker = naersk'.buildPackage {
          pname = "broker";
          src = ./.;
        };

        # For `nix build .#client` and `nix run .#client`:
        packages.client = naersk'.buildPackage {
          pname = "client";
          src = ./.;
        };

        # For `nix develop`:
        devShell = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustc
            cargo
            rustfmt
            pre-commit
            rustPackages.clippy
          ];
          RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
        };
      }
    );
}
