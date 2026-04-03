#!/bin/sh
# Aether Programming Language Installer
# Usage: curl -sSf https://aether-lang.org/install | sh
#    or: curl -sSf https://raw.githubusercontent.com/aether-lang/aether/main/install.sh | sh
#
# Options (via env vars):
#   AETHER_VERSION=v0.1.0  Install a specific version (default: latest)
#   AETHER_DIR=~/.aether   Installation directory (default: ~/.aether)

set -e

BOLD="\033[1m"
GREEN="\033[32m"
YELLOW="\033[33m"
RED="\033[31m"
CYAN="\033[36m"
RESET="\033[0m"

REPO="aether-lang/aether"
INSTALL_DIR="${AETHER_DIR:-$HOME/.aether}"
BIN_DIR="$INSTALL_DIR/bin"

info()  { printf "${CYAN}info${RESET}: %s\n" "$1"; }
ok()    { printf "${GREEN}  ok${RESET}: %s\n" "$1"; }
warn()  { printf "${YELLOW}warn${RESET}: %s\n" "$1"; }
err()   { printf "${RED}error${RESET}: %s\n" "$1" >&2; }

# ─── Detect platform ─────────────────────────────────────────────

detect_platform() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"

    case "$OS" in
        Linux)  OS_NAME="linux" ;;
        Darwin) OS_NAME="macos" ;;
        MINGW*|MSYS*|CYGWIN*)
            err "Windows detected. Use install.ps1 instead:"
            err "  irm https://aether-lang.org/install.ps1 | iex"
            exit 1
            ;;
        *)
            err "Unsupported OS: $OS"
            exit 1
            ;;
    esac

    case "$ARCH" in
        x86_64|amd64)   ARCH_NAME="x86_64" ;;
        aarch64|arm64)   ARCH_NAME="aarch64" ;;
        *)
            err "Unsupported architecture: $ARCH"
            exit 1
            ;;
    esac

    PLATFORM="${OS_NAME}-${ARCH_NAME}"
    ARTIFACT="aether-${PLATFORM}"
    info "Detected platform: $PLATFORM"
}

# ─── Determine version ───────────────────────────────────────────

get_version() {
    if [ -n "$AETHER_VERSION" ]; then
        VERSION="$AETHER_VERSION"
        info "Installing version: $VERSION"
        return
    fi

    info "Fetching latest version..."
    if command -v curl >/dev/null 2>&1; then
        VERSION=$(curl -sSf "https://api.github.com/repos/$REPO/releases/latest" 2>/dev/null \
            | grep '"tag_name"' | head -1 | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/')
    elif command -v wget >/dev/null 2>&1; then
        VERSION=$(wget -qO- "https://api.github.com/repos/$REPO/releases/latest" 2>/dev/null \
            | grep '"tag_name"' | head -1 | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/')
    fi

    if [ -z "$VERSION" ]; then
        warn "Could not determine latest version from GitHub"
        warn "Falling back to building from source..."
        build_from_source
        exit 0
    fi
    info "Latest version: $VERSION"
}

# ─── Download ─────────────────────────────────────────────────────

download() {
    URL="https://github.com/$REPO/releases/download/$VERSION/${ARTIFACT}.tar.gz"
    SHA_URL="${URL}.sha256"

    info "Downloading $ARTIFACT..."

    TMPDIR=$(mktemp -d)
    trap 'rm -rf "$TMPDIR"' EXIT

    if command -v curl >/dev/null 2>&1; then
        HTTP_CODE=$(curl -sSL -w "%{http_code}" -o "$TMPDIR/aether.tar.gz" "$URL" 2>/dev/null || echo "000")
    elif command -v wget >/dev/null 2>&1; then
        wget -q -O "$TMPDIR/aether.tar.gz" "$URL" 2>/dev/null && HTTP_CODE="200" || HTTP_CODE="000"
    else
        err "Neither curl nor wget found. Install one and retry."
        exit 1
    fi

    if [ "$HTTP_CODE" != "200" ]; then
        warn "Pre-built binary not available for $PLATFORM ($HTTP_CODE)"
        warn "Falling back to building from source..."
        build_from_source
        return
    fi

    # Verify checksum if available
    if command -v curl >/dev/null 2>&1; then
        EXPECTED_SHA=$(curl -sSL "$SHA_URL" 2>/dev/null | awk '{print $1}')
    elif command -v wget >/dev/null 2>&1; then
        EXPECTED_SHA=$(wget -qO- "$SHA_URL" 2>/dev/null | awk '{print $1}')
    fi

    if [ -n "$EXPECTED_SHA" ]; then
        if command -v sha256sum >/dev/null 2>&1; then
            ACTUAL_SHA=$(sha256sum "$TMPDIR/aether.tar.gz" | awk '{print $1}')
        elif command -v shasum >/dev/null 2>&1; then
            ACTUAL_SHA=$(shasum -a 256 "$TMPDIR/aether.tar.gz" | awk '{print $1}')
        fi

        if [ -n "$ACTUAL_SHA" ] && [ "$ACTUAL_SHA" != "$EXPECTED_SHA" ]; then
            err "SHA256 checksum mismatch!"
            err "  Expected: $EXPECTED_SHA"
            err "  Actual:   $ACTUAL_SHA"
            exit 1
        fi
        ok "Checksum verified"
    fi

    # Extract
    tar xzf "$TMPDIR/aether.tar.gz" -C "$TMPDIR"

    if [ ! -f "$TMPDIR/aether" ]; then
        err "Binary not found in archive"
        exit 1
    fi

    install_binary "$TMPDIR/aether"
}

# ─── Build from source ───────────────────────────────────────────

build_from_source() {
    info "Building from source..."

    if ! command -v cargo >/dev/null 2>&1; then
        err "Rust is not installed. Install it first:"
        err "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        err ""
        err "Or wait for a pre-built binary release."
        exit 1
    fi

    if ! command -v git >/dev/null 2>&1; then
        err "git is not installed."
        exit 1
    fi

    TMPDIR=$(mktemp -d)
    trap 'rm -rf "$TMPDIR"' EXIT

    info "Cloning repository..."
    git clone --depth 1 "https://github.com/$REPO.git" "$TMPDIR/aether" 2>/dev/null

    info "Compiling (this may take a few minutes)..."
    cd "$TMPDIR/aether"
    cargo build --release 2>/dev/null

    install_binary "$TMPDIR/aether/target/release/aether"
}

# ─── Install binary ──────────────────────────────────────────────

install_binary() {
    BINARY="$1"

    mkdir -p "$BIN_DIR"
    cp "$BINARY" "$BIN_DIR/aether"
    chmod +x "$BIN_DIR/aether"

    ok "Installed to $BIN_DIR/aether"

    # Add to PATH
    add_to_path

    # Verify
    if "$BIN_DIR/aether" --version >/dev/null 2>&1; then
        INSTALLED_VER=$("$BIN_DIR/aether" --version 2>/dev/null)
        ok "Aether $INSTALLED_VER installed successfully!"
    else
        ok "Binary installed to $BIN_DIR/aether"
    fi

    echo ""
    printf "${BOLD}${GREEN}Aether has been installed!${RESET}\n"
    echo ""
    echo "  To get started, run:"
    echo ""
    printf "    ${CYAN}aether --version${RESET}\n"
    printf "    ${CYAN}aether repl${RESET}\n"
    printf "    ${CYAN}aether run hello.ae${RESET}\n"
    echo ""

    if [ -n "$PATH_UPDATED" ]; then
        printf "  ${YELLOW}Restart your shell or run:${RESET}\n"
        printf "    ${CYAN}source ~/.bashrc${RESET}  (or ~/.zshrc)\n"
        echo ""
    fi
}

# ─── Add to PATH ─────────────────────────────────────────────────

add_to_path() {
    PATH_LINE="export PATH=\"$BIN_DIR:\$PATH\""

    # Check if already in PATH
    case ":$PATH:" in
        *":$BIN_DIR:"*) return ;;
    esac

    PATH_UPDATED=1

    # Detect shell config file
    SHELL_NAME="$(basename "${SHELL:-/bin/sh}")"
    case "$SHELL_NAME" in
        zsh)
            PROFILE="$HOME/.zshrc"
            ;;
        bash)
            if [ -f "$HOME/.bashrc" ]; then
                PROFILE="$HOME/.bashrc"
            elif [ -f "$HOME/.bash_profile" ]; then
                PROFILE="$HOME/.bash_profile"
            else
                PROFILE="$HOME/.profile"
            fi
            ;;
        fish)
            FISH_DIR="$HOME/.config/fish"
            mkdir -p "$FISH_DIR"
            PROFILE="$FISH_DIR/config.fish"
            PATH_LINE="set -gx PATH $BIN_DIR \$PATH"
            ;;
        *)
            PROFILE="$HOME/.profile"
            ;;
    esac

    # Don't duplicate
    if [ -f "$PROFILE" ] && grep -q "$BIN_DIR" "$PROFILE" 2>/dev/null; then
        return
    fi

    info "Adding $BIN_DIR to PATH in $PROFILE"
    echo "" >> "$PROFILE"
    echo "# Aether Programming Language" >> "$PROFILE"
    echo "$PATH_LINE" >> "$PROFILE"
}

# ─── Uninstall ────────────────────────────────────────────────────

uninstall() {
    info "Uninstalling Aether..."
    rm -rf "$INSTALL_DIR"
    ok "Removed $INSTALL_DIR"
    warn "You may want to remove the PATH entry from your shell config."
    exit 0
}

# ─── Main ─────────────────────────────────────────────────────────

main() {
    echo ""
    printf "${BOLD}${CYAN}  Aether Installer${RESET}\n"
    echo ""

    # Handle flags
    for arg in "$@"; do
        case "$arg" in
            --uninstall) uninstall ;;
            --help|-h)
                echo "Usage: install.sh [options]"
                echo ""
                echo "Options:"
                echo "  --uninstall    Remove Aether"
                echo "  --help         Show this help"
                echo ""
                echo "Environment variables:"
                echo "  AETHER_VERSION   Version to install (default: latest)"
                echo "  AETHER_DIR       Install directory (default: ~/.aether)"
                exit 0
                ;;
        esac
    done

    # Check if already installed
    if [ -f "$BIN_DIR/aether" ]; then
        CURRENT=$("$BIN_DIR/aether" --version 2>/dev/null || echo "unknown")
        warn "Aether is already installed: $CURRENT"
        info "Reinstalling..."
    fi

    detect_platform
    get_version
    download
}

main "$@"
