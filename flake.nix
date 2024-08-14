{
  description = "Scalable push server for XMPP";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    crane.inputs.nixpkgs.follows = "nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, crane }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        craneLib = crane.mkLib pkgs;

        commonArgs = {
          src = craneLib.cleanCargoSource ./.;

          buildInputs = [
            pkgs.openssl
          ];

          nativeBuildInputs = [
            pkgs.pkg-config
          ];
        };
      in
      {
        packages.default = craneLib.buildPackage ({
          meta = { mainProgram = "fpush"; };
          cargoExtraArgs = "--all-features";
         } // commonArgs);

         devShells = {
           default = pkgs.mkShell {
             buildInputs = [ ] ++ commonArgs.buildInputs;
             nativeBuildInputs = builtins.attrValues
               {
                 inherit (pkgs) cargo rustc fmt cargo-udeps cargo-outdated cargo-audit;
               } ++ [
               # This is required to prevent a mangled bash shell in nix develop
               # see: https://discourse.nixos.org/t/interactive-bash-with-nix-develop-flake/15486
               (pkgs.hiPrio pkgs.bashInteractive)
             ] ++ commonArgs.nativeBuildInputs;
           };
         };
      }
    );
}
