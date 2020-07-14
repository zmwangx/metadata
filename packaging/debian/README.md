# Ubuntu PPA packaging instructions

When a new release is tagged:

1. Create archive `packaging/debian/metadata_$version.orig.tar.xz`;

2. Add entry to `packaging/debian/debian/changelog`;

3. If there's a need to add/update/remove patches, unpack the orig tarball into `packaging/debain/metadata-$version`, and use `quilt` in the working tree to manage patches;

4. **bionic**:
   - Copy the orig tarball into `packaging/debian/bionic/`;
   - Unpack the tarball into `packaging/debian/bionic/metadata-$version`;
   - Copy `packaging/debian/debian` into `packaging/debian/bionic/metadata-$version`;
   - Update version number and distribution in `changelog`:

         sed -ri 's/^metadata \((.*)-([0-9])\) unstable;/metadata (\1-bionic\2) bionic;/' packaging/debian/bionic/metadata-$version/debian/changelog

   - Make sure `debhelper-compat` is pegged to `= 11` in `control`;
   - Run

         debuild -S -sa

     from the `packaging/debian/bionic/metadata-$version` directory. (This builds the source package. Run `dpkg-buildpackage -B -tc` to build the binary package instead.)
   - Run

         dput ppa:zmwangx/metadata metadata_$version-bionicN_source.changes

     from the `packaging/debian/bionic` directory.

5. **focal**: Basically the same as bionic, except
   - Update version version number and distribution in `changelog`:

         sed -ri 's/^metadata \((.*)-([0-9])\) unstable;/metadata (\1-focal\2) focal;/' packaging/debian/focal/metadata-$version/debian/changelog

   - Make sure `debhelper-compat` is pegged to `= 12` in `control`.
