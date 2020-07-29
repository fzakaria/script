self: super:
{

	# make our package installable easily to minimize the default.nix file
	scriptr = self.callPackage ./derivation.nix { };
}
