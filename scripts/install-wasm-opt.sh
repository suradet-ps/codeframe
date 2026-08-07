#!/bin/sh
# Install a wasm-opt CLI on hosts without a package manager.
#
# Vercel's container-optimized build image has no apt-get (build fails with
# "apt-get: command not found"), so instead of native packages we install
# binaryen's official Node.js build - a drop-in wasm-opt replacement that
# runs on any Node 18+ environment. ~2 MB download vs ~100 MB for the
# native x86_64-linux tarball.
#
# If a native wasm-opt is already on PATH (e.g. GitHub Actions installs
# binaryen via apt), it is used as-is.
#
# Usage: bash scripts/install-wasm-opt.sh
# After running, add "$HOME/.codeframe-bin" to PATH.
set -eu

BINARYEN_VERSION=130

if command -v wasm-opt >/dev/null 2>&1; then
  echo "wasm-opt already available: $(command -v wasm-opt)"
  exit 0
fi

if ! command -v node >/dev/null 2>&1; then
  echo "error: node is required to install wasm-opt" >&2
  exit 1
fi

INSTALL_DIR="$HOME/.codeframe-bin"
mkdir -p "$INSTALL_DIR"
cd "$INSTALL_DIR"

curl -fsSL "https://github.com/WebAssembly/binaryen/releases/download/version_${BINARYEN_VERSION}/binaryen-version_${BINARYEN_VERSION}-node.tar.gz" \
  -o "binaryen-version_${BINARYEN_VERSION}.tar.gz"
rm -rf "binaryen-version_${BINARYEN_VERSION}"
tar xzf "binaryen-version_${BINARYEN_VERSION}.tar.gz"

# Shim: trunk (and any other tool) spawns `wasm-opt <args>`; the Node build
# is invoked as `node wasm-opt.js <args>`.
printf '#!/bin/sh\nexec node "%s/binaryen-version_%s/wasm-opt.js" "$@"\n' \
  "$INSTALL_DIR" "$BINARYEN_VERSION" > wasm-opt
chmod +x wasm-opt

echo "wasm-opt installed to $INSTALL_DIR (binaryen version_${BINARYEN_VERSION})"
