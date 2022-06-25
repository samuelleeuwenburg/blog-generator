{
  description = "blog generator tool";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-21.11";
  };

  outputs = { self, nixpkgs }:
    let
      supportedSystems = [ "x86_64-linux" "x86_64-darwin" "aarch64-linux" "aarch64-darwin" ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
      nixpkgsFor = forAllSystems (system: import nixpkgs { inherit system; });
    in
    {
      defaultPackage = forAllSystems (system:
        let
          pkgs = nixpkgsFor.${system};
        in
          pkgs.rustPlatform.buildRustPackage {
            pname = "blog-generator";
            version = "1.0.0";

            src = self;

            cargoLock = {
              lockFile = ./Cargo.lock;
            };
          });

    };
}
