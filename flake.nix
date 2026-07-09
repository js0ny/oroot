{
  description = "Command-line tool to manipluate impermanance old roots in btrfs";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs =
    { nixpkgs, ... }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];

      forAllSystems = nixpkgs.lib.genAttrs systems;
    in
    {
      packages = forAllSystems (
        system:
        let
          pkgs = import nixpkgs { inherit system; };
          lib = pkgs.lib;
        in
        {
          default = pkgs.rustPlatform.buildRustPackage {
            pname = "oroot";
            version = "0.1.0";

            src = lib.fileset.toSource {
              root = ./.;
              fileset = lib.fileset.unions [
                ./Cargo.toml
                ./Cargo.lock
                ./LICENSE
                ./src
              ];
            };

            cargoLock.lockFile = ./Cargo.lock;

            meta = {
              description = "Command-line tool to manipluate impermanance old roots in btrfs";
              license = lib.licenses.mit;
              mainProgram = "oroot";
              platforms = lib.platforms.linux;
            };
          };
        }
      );

      devShells = forAllSystems (
        system:
        let
          pkgs = import nixpkgs { inherit system; };
          ciDeps = with pkgs; [
            cargo
            clippy
            rustc
          ];
          devDeps = with pkgs; [
            rust-analyzer
            rustfmt
          ];
        in
        {
          default = pkgs.mkShell {
            buildInputs = ciDeps ++ devDeps;
            shellHook = /* bash */ ''
              export RUST_BACKTRACE=1
            '';
          };
          ci = pkgs.mkShell {
            buildInputs = ciDeps;
          };
        }
      );
    };
}
