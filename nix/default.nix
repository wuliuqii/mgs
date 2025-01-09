{
  rustPlatform,
  wayland,
  libxkbcommon,
  pkg-config,
  fontconfig,
  xorg,
  openssl,
  vulkan-loader,
  freetype,
}:
let
  cargoToml = builtins.fromTOML (builtins.readFile ../Cargo.toml);
in
rustPlatform.buildRustPackage {
  pname = cargoToml.package.name;
  version = cargoToml.package.version;

  src = ../.;

  cargoLock = {
    lockFile = ../Cargo.lock;
    outputHashes = {
      "blade-graphics-0.6.0" = "sha256-j/JI34ZPD7RAHNHu3krgDLnIq4QmmZaZaU1FwD7f2FM=";
      "collections-0.1.0" = "sha256-4kSXDjT40PNJmGm5HB+hOws8BZNlhdCn3qB8aX3lvwY=";
      "cosmic-text-0.11.2" = "sha256-TLPDnqixuW+aPAhiBhSvuZIa69vgV3xLcw32OlkdCcM=";
      "font-kit-0.14.1" = "sha256-qUKvmi+RDoyhMrZ7T6SoVAyMc/aasQ9Y/okzre4SzXo=";
      "xim-0.4.0" = "sha256-BXyaIBoqMNbzaSJqMadmofdjtlEVSoU6iogF66YP6a4=";
      "xkbcommon-0.7.0" = "sha256-2RjZWiAaz8apYTrZ82qqH4Gv20WyCtPT+ldOzm0GWMo=";
    };
  };

  buildInputs = [
    wayland
    libxkbcommon
    fontconfig
    xorg.libxcb
    openssl
    freetype
  ];

  nativeBuildInputs = [
    pkg-config
  ];

  postFixup = ''
    patchelf --add-rpath ${vulkan-loader}/lib $out/bin/* 
  '';

  doCheck = false;
}
