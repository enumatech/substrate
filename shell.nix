let
  moz_overlay = import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz);
  nixpkgs = import <nixpkgs> { overlays = [ moz_overlay ]; };
in
  with nixpkgs;
let
  myrust = ((rustChannelOf { date = "2019-05-22"; channel = "nightly"; }).rust.override { extensions = [ "rust-src" "rls-preview" "rust-analysis" "rustfmt-preview" "clippy-preview" ]; targets = [ "wasm32-unknown-unknown" ]; });
  wasm-gc = rustPlatform.buildRustPackage rec {
    name = "wasm-gc-${version}";
    version = "0.1.11";
    src = fetchFromGitHub {
      owner = "alexcrichton";
      repo = "wasm-gc";
      rev = "wasm-gc-api-${version}";
      sha256 = "0nw6yhgn3d38s1dmh4jkiccsrks5i1cpjgb7r52l2n8cnry4h1zs";
    };
    cargoSha256 = "10ici7qfb1gxfc08ag6j7dn3b91c0irqr32pyrc6nz9q4aywc3b2";
  };
in
mkShell {
  name = "substrate";
  buildInputs = [
    myrust
    wasm-gc
    openssl
    pkgconfig
    cmake
    python3
    llvmPackages.clang-unwrapped
    gnuplot
    wabt
    rustup # for vscode
  ] ++ lib.optionals stdenv.isDarwin [
    darwin.apple_sdk.frameworks.IOKit
    darwin.apple_sdk.frameworks.Security
    darwin.apple_sdk.frameworks.CoreServices
  ];
  LIBCLANG_PATH="${llvmPackages.libclang}/lib";
  RUST_SRC_PATH="${myrust}/lib/rustlib/src/rust/src";
  shellHook = ''
    export NIX_CXXSTDLIB_LINK=""
  '';
}
