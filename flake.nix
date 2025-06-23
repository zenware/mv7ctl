{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs";
  };
  outputs = {nixpkgs, ... }:
  let
    system = "x86_64-linux";
    pkgs = import nixpkgs { inherit system; };
  in
  {
    devShells.${system}.default = pkgs.mkShell {
      packages = [
        pkgs.cyme  # takes place of usbutils
        #pkgs.quickemu  # Windows Emulation
        # Building things
        pkgs.libusb1
        pkgs.pkg-config

        # Rust development stuff
        pkgs.cargo
        pkgs.clippy
        pkgs.rustfmt
        pkgs.rust-analyzer
        pkgs.lldb_20  # lldb-dap
      ];
      shellHook = ''
        alias lsusb='cyme --lsusb'
        echo "We're in."
      '';
    };
  };
}
