{
  description = "bevy flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
    use-mold.url = "github:campbellcole/use-mold";
  };

  outputs =
    {
      nixpkgs,
      rust-overlay,
      flake-utils,
      use-mold,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        dlopenLibraries = with pkgs; [
          # for Linux
          # Audio (Linux only)
          alsa-lib
          pipewire
          # Cross Platform 3D Graphics API
          vulkan-loader
          # For debugging around vulkan
          vulkan-tools
          # Window system
          wayland
          # Other dependencies
          udev
          libudev-zero
          libX11
          libXcursor
          libXi
          libXrandr
          libxkbcommon
          libxcb
        ];

        moldHook = use-mold.useMoldHook { };
      in
      {
        devShells.default =
          with pkgs;
          mkShell {
            buildInputs = [
              # Rust dependencies
              (rust-bin.stable.latest.default.override {
                extensions = [
                  "rust-src"
                  "rust-analyzer"
                  "clippy"
                ];
              })
              pkg-config
              clang
            ]
            ++ lib.optionals (lib.strings.hasInfix "linux" system) dlopenLibraries;
            RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
            LD_LIBRARY_PATH = "${lib.makeLibraryPath dlopenLibraries}";

            shellHook = ''
              # Keep .zed/debug.env in sync with the current flake environment
              {
                echo "LD_LIBRARY_PATH=$LD_LIBRARY_PATH"
              } > .zed/debug.env
            ''
            + moldHook mold;
          };
      }
    );
}
