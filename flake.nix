{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
    }:
    let
      overlays = [
        rust-overlay.overlays.default
        (final: prev: {
          rustToolchain = prev.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        })
      ];
      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      eachSystem =
        f:
        nixpkgs.lib.genAttrs supportedSystems (
          system:
          f {
            pkgs = import nixpkgs { inherit overlays system; };
          }
        );
    in
    {
      devShells = eachSystem (
        { pkgs }:
        {
          default = pkgs.mkShell (
            with pkgs;
            rec {
              packages = [
                rustToolchain
              ];

              nativeBuildInputs = [
                pkg-config
              ];

              buildInputs = [
                openssl
                fontconfig
                libxkbcommon
                xorg.libxcb
                wayland
                vulkan-loader
                freetype
                libpulseaudio
              ];

              LD_LIBRARY_PATH = "${lib.makeLibraryPath buildInputs}";
              RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
            }
          );
        }
      );
    };
}
