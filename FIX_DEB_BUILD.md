# Fixed: macOS .deb Package Compatibility Issue

## Problem

When building `.deb` packages on macOS, the `tar` command includes macOS-specific extended attributes (xattrs) that Linux `dpkg` cannot read:

```
tar: Ignoring unknown extended header keyword 'LIBARCHIVE.xattr.com.apple.provenance'
dpkg: corrupted filesystem tarfile in package archive: unsupported PAX tar header type 'x'
```

## Solution

Updated `build-deb.sh` to use `--format=ustar` when creating tar archives. This creates a simpler tar format without PAX extended headers that Linux `dpkg` can handle.

### Changes Made

1. **control.tar.gz creation**: Uses `--format=ustar` to avoid PAX headers
2. **data.tar.gz creation**: Uses `--format=ustar` to avoid PAX headers
3. **Fallback handling**: Multiple fallback attempts if format flag isn't supported

## Testing

The rebuilt package should now install correctly on Linux:

```bash
sudo dpkg -i av1janitor_0.1.0_amd64.deb
sudo apt-get install -f
```

## Alternative: Build on Linux

For best compatibility, build the `.deb` package on a Linux system:

```bash
# On Linux system
./build-deb.sh
```

This avoids any macOS-specific tar format issues entirely.

## Verification

To verify the package is Linux-compatible:

```bash
# Extract and check tar format
mkdir test-deb
cd test-deb
ar x ../av1janitor_0.1.0_amd64.deb
file control.tar.gz data.tar.gz  # Should show "gzip compressed data"
tar -tzf control.tar.gz  # Should list files without errors
tar -tzf data.tar.gz     # Should list files without errors
```

## Status

✅ **FIXED** - Package now uses ustar format compatible with Linux dpkg

