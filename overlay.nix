self: super:
let rust-stable = super.latest.rustChannels.stable.rust.override {
    		extensions = [ "rust-src" "rust-analysis" "rustfmt-preview" "rls-preview" ];
  		};
in {

	# make our package installable easily to minimize the default.nix file
	scriptr = self.callPackage ./derivation.nix { };

	# this assumes the rust overlay is present
	# we override the rustPlatform to our specific rust stable version
	# https://www.breakds.org/post/build-rust-package/
	rustPlatform = super.makeRustPlatform {
		rustc = rust-stable;
  		cargo = rust-stable;
	};
}
