{
  description = "Rust dev env";
  
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in {
        devShells.default = pkgs.mkShell {
					packages = with pkgs; [
						rustc
						cargo
						rust-analyzer
						ffmpeg
						viu
						wayland
						libxkbcommon
					];
					LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (with pkgs; [
						wayland
						libxkbcommon
						libGL
					]) + ":/run/opengl-driver/lib";
				};
      }
    );
}
