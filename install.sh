#!/usr/bin/env bash
# install.sh - Install the rw CLI
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/RoundingWell/app-cli/main/install.sh | bash
#
# Options (via environment):
#   RW_BIN_DIR    Where to install binary (default: ~/.local/bin)
#   RW_VERSION    Specific version to install (default: latest)
#
# To pass environment variables, use:
#   RW_VERSION=1.2.3 bash -c "$(curl -fsSL https://raw.githubusercontent.com/RoundingWell/app-cli/main/install.sh)"

set -euo pipefail

REPO="RoundingWell/app-cli"
BIN_DIR="${RW_BIN_DIR:-$HOME/.local/bin}"
VERSION="${RW_VERSION:-}"

# Color helpers — respect NO_COLOR (https://no-color.org)
if [[ -z "${NO_COLOR:-}" ]] && [[ -t 1 ]]; then
  bold()  { printf '\033[1m%s\033[0m' "$1"; }
  green() { printf '\033[32m%s\033[0m' "$1"; }
  red()   { printf '\033[31m%s\033[0m' "$1"; }
else
  bold()  { printf '%s' "$1"; }
  green() { printf '%s' "$1"; }
  red()   { printf '%s' "$1"; }
fi

info()  { echo "  $(green "✓") $1"; }
step()  { echo "  $(bold "→") $1"; }
error() { echo "  $(red "✗ ERROR:") $1" >&2; exit 1; }

find_sha256_cmd() {
  if command -v sha256sum &>/dev/null; then
    echo "sha256sum"
  elif command -v shasum &>/dev/null; then
    echo "shasum -a 256"
  else
    error "No SHA256 tool found (need sha256sum or shasum)"
  fi
}

detect_platform() {
  local os arch

  os=$(uname -s | tr '[:upper:]' '[:lower:]')
  case "$os" in
    darwin) os="darwin" ;;
    linux)  os="linux" ;;
    mingw*|msys*|cygwin*) error "Windows is not supported" ;;
    *) error "Unsupported OS: $os" ;;
  esac

  arch=$(uname -m)
  case "$arch" in
    x86_64|amd64)  arch="amd64" ;;
    aarch64|arm64) arch="arm64" ;;
    *) error "Unsupported architecture: $arch" ;;
  esac

  echo "${os}_${arch}"
}

get_latest_version() {
  local url version
  url=$(curl -fsSL -o /dev/null -w '%{url_effective}' "https://github.com/${REPO}/releases/latest" 2>/dev/null) || true
  version="${url##*/}"
  if [[ ! $version =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    error "Could not determine latest version (resolved '${version:-<empty>}' from '${url:-<no URL>}'). Check your network connection or repository tags."
  fi
  echo "$version"
}

verify_checksums() {
  local version="$1"
  local tmp_dir="$2"
  local archive_name="$3"
  local base_url="https://github.com/${REPO}/releases/download/${version}"

  step "Verifying checksums..."

  if ! curl -fsSL "${base_url}/checksums.txt" -o "${tmp_dir}/checksums.txt"; then
    error "Failed to download checksums.txt"
  fi

  local expected actual
  expected=$(awk -v f="$archive_name" '$2 == f || $2 == ("*" f) {print $1; exit}' "${tmp_dir}/checksums.txt")
  actual=$(cd "$tmp_dir" && $(find_sha256_cmd) "$archive_name" | awk '{print $1}')
  [[ -n "$expected" && "$expected" == "$actual" ]] \
    || error "Checksum verification failed for $archive_name"

  info "Checksum verified"
}

download_binary() {
  local version="$1"
  local platform="$2"
  local tmp_dir="$3"
  local url archive_name

  archive_name="rw_${version}_${platform}.tar.gz"
  url="https://github.com/${REPO}/releases/download/${version}/${archive_name}"

  step "Downloading rw v${version} for ${platform}..."

  if ! curl -fsSL "$url" -o "${tmp_dir}/${archive_name}"; then
    error "Failed to download from $url"
  fi

  verify_checksums "$version" "$tmp_dir" "$archive_name"

  step "Extracting..."
  tar -xzf "${tmp_dir}/${archive_name}" -C "$tmp_dir"

  if [[ ! -f "${tmp_dir}/rw" ]]; then
    error "Binary not found in archive"
  fi

  mkdir -p "$BIN_DIR"
  mv "${tmp_dir}/rw" "$BIN_DIR/"
  chmod +x "$BIN_DIR/rw"

  info "Installed rw to $BIN_DIR/rw"
}

setup_path() {
  if [[ ":$PATH:" == *":$BIN_DIR:"* ]]; then
    return 0
  fi

  step "Adding $BIN_DIR to PATH"

  local shell_rc=""
  case "${SHELL:-}" in
    */zsh)  shell_rc="$HOME/.zshrc" ;;
    */bash) shell_rc="$HOME/.bashrc" ;;
    *)      shell_rc="$HOME/.profile" ;;
  esac

  local path_line="export PATH=\"$BIN_DIR:\$PATH\""

  if [[ -f "$shell_rc" ]] && grep -qF "$BIN_DIR" "$shell_rc" 2>/dev/null; then
    info "PATH already configured in $shell_rc"
  else
    echo "" >> "$shell_rc"
    echo "# Added by rw installer" >> "$shell_rc"
    echo "$path_line" >> "$shell_rc"
    info "Added to $shell_rc"
    info "Run: source $shell_rc"
  fi
}

verify_install() {
  local installed_version
  if installed_version=$("$BIN_DIR/rw" --version 2>/dev/null); then
    info "${installed_version} installed"
    return 0
  fi

  error "Installation failed — rw not working"
}

main() {
  if ! command -v curl &>/dev/null; then
    error "curl is required but not installed"
  fi

  local platform version tmp_dir
  platform=$(detect_platform)

  if [[ -n "$VERSION" ]]; then
    version="$VERSION"
    if [[ ! $version =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$ ]]; then
      error "Invalid version '${version}'. Expected semver format (e.g. 1.2.3 or 1.2.3-rc.1)."
    fi
  else
    version=$(get_latest_version)
  fi

  tmp_dir=$(mktemp -d)
  trap "rm -rf '${tmp_dir}'" EXIT

  download_binary "$version" "$platform" "$tmp_dir"
  setup_path
  verify_install

  echo ""
  echo "  Next steps:"
  echo "    $(bold "rw auth login")    Authenticate with RoundingWell"
  echo "    $(bold "rw --help")        Show available commands"
  echo ""
}

main "$@"
