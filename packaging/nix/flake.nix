{
  description = "TermExplorer - Inspection-first terminal developer workstation in Rust";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "termexplorer";
          version = "1.1.0";

          src = ./../..;

          cargoLock = {
            lockFile = ./../../Cargo.lock;
          };

          nativeBuildInputs = with pkgs; [
            pkg-config
            installShellFiles
          ];

          buildInputs = with pkgs; [
            openssl
          ];

          postInstall = ''
            installManPage packaging/termexplorer.1
            install -Dm644 packaging/termexplorer.desktop $out/share/applications/termexplorer.desktop
            install -Dm644 packaging/termexplorer.svg $out/share/icons/hicolor/scalable/apps/termexplorer.svg
            installShellCompletion --cmd termexplorer \
              --bash packaging/completions/termexplorer.bash \
              --zsh packaging/completions/_termexplorer \
              --fish packaging/completions/termexplorer.fish
          '';

          meta = with pkgs.lib; {
            description = "Inspection-first terminal developer workstation combining file exploration, preview, and developer workflows";
            homepage = "https://github.com/jyothiprajith1323/TermExplorer";
            license = licenses.mit;
            maintainers = [ ];
            platforms = platforms.linux;
          };
        };
      }
    );
}
