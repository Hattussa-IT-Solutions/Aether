#!/bin/bash
# Build a .deb package for Aether
# Usage: ./packaging/build-deb.sh [version]
set -e

VERSION="${1:-0.1.0}"
ARCH=$(dpkg --print-architecture 2>/dev/null || echo "amd64")
PKG_NAME="aether_${VERSION}_${ARCH}"
PKG_DIR="/tmp/$PKG_NAME"

echo "Building Aether $VERSION .deb package for $ARCH..."

# Build release binary
cargo build --release

# Create package structure
rm -rf "$PKG_DIR"
mkdir -p "$PKG_DIR/DEBIAN"
mkdir -p "$PKG_DIR/usr/bin"
mkdir -p "$PKG_DIR/usr/share/doc/aether"

# Copy binary
cp target/release/aether "$PKG_DIR/usr/bin/aether"
chmod 755 "$PKG_DIR/usr/bin/aether"

# Copy docs
cp README.md "$PKG_DIR/usr/share/doc/aether/"
cp LICENSE "$PKG_DIR/usr/share/doc/aether/copyright"

# Control file
cat > "$PKG_DIR/DEBIAN/control" << EOF
Package: aether
Version: $VERSION
Section: devel
Priority: optional
Architecture: $ARCH
Maintainer: Aether Language Team <team@aether-lang.org>
Description: Modern programming language with parallelism and genetic evolution
 Aether is a modern, expressive programming language with built-in
 parallelism, pattern matching, genetic evolution, GPU compute, and
 novel OOP concepts.
Homepage: https://aether-lang.org
EOF

# Build .deb
dpkg-deb --build "$PKG_DIR"

echo ""
echo "Package built: /tmp/${PKG_NAME}.deb"
echo "Install with: sudo dpkg -i /tmp/${PKG_NAME}.deb"
