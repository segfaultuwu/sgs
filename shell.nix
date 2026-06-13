{
  pkgs ? import <nixpkgs> { },
}:

pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    pkg-config
  ];

  buildInputs = with pkgs; [
    rustc
    cargo
    rustfmt
    clippy

    glib
    gtk4
    gtk4-layer-shell
    gobject-introspection

    lua5_4

    jq
    socat
  ];

  shellHook = ''
    export PKG_CONFIG_PATH="${pkgs.glib.dev}/lib/pkgconfig:${pkgs.gtk4.dev}/lib/pkgconfig:${pkgs.gtk4-layer-shell}/lib/pkgconfig:$PKG_CONFIG_PATH"
    echo "SGS dev shell loaded"
  '';
}
