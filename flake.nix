{
  description = "sgs - segfault's gtk shell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs =
    { self, nixpkgs }:
    let
      system = "x86_64-linux";

      pkgs = import nixpkgs {
        inherit system;
      };
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        nativeBuildInputs = with pkgs; [
          pkg-config
          gobject-introspection
        ];

        buildInputs = with pkgs; [
          rustc
          cargo
          rustfmt
          clippy
          rust-analyzer

          glib
          gtk4
          gtk4-layer-shell

          lua5_4
          lua-language-server

          jq
          socat
        ];

        shellHook = ''
          export PKG_CONFIG_PATH="${pkgs.glib.dev}/lib/pkgconfig:${pkgs.gtk4.dev}/lib/pkgconfig:${pkgs.gtk4-layer-shell}/lib/pkgconfig:$PKG_CONFIG_PATH"
          export RUST_BACKTRACE=1
          export GDK_BACKEND=wayland

          echo "SGS dev shell loaded"
        '';
      };
    };
}
