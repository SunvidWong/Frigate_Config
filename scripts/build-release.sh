#!/bin/bash
# Complete release build script
# Builds agent + Tauri app for the current platform

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "🚀 Frigate Config Tool - Release Build"
echo "======================================"
echo ""

# Detect platform
PLATFORM=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case $ARCH in
    x86_64)
        GOARCH="amd64"
        ;;
    aarch64|arm64)
        GOARCH="arm64"
        ;;
    *)
        echo "❌ Unsupported architecture: $ARCH"
        exit 1
        ;;
esac

echo "📋 Build Information:"
echo "  Platform: $PLATFORM"
echo "  Architecture: $ARCH ($GOARCH)"
echo "  Version: ${VERSION:-dev}"
echo ""

# Step 1: Build agent for current platform
echo "Step 1/4: Building agent..."
cd "$PROJECT_ROOT/agent"

OUTPUT_NAME="agent"
if [ "$PLATFORM" = "windows" ]; then
    OUTPUT_NAME="agent.exe"
fi

CGO_ENABLED=0 GOOS=$PLATFORM GOARCH=$GOARCH go build \
    -v \
    -ldflags="-s -w -X main.Version=${VERSION:-dev}" \
    -o "../src-tauri/bin/$OUTPUT_NAME" \
    ./cmd/agent

echo "✅ Agent built: src-tauri/bin/$OUTPUT_NAME"
echo ""

# Step 2: Install frontend dependencies
echo "Step 2/4: Installing frontend dependencies..."
cd "$PROJECT_ROOT"
npm ci
echo "✅ Dependencies installed"
echo ""

# Step 3: Build frontend
echo "Step 3/4: Building frontend..."
cd "$PROJECT_ROOT/src-ui"
npm run build
echo "✅ Frontend built"
echo ""

# Step 4: Build Tauri app
echo "Step 4/4: Building Tauri application..."
cd "$PROJECT_ROOT"

case $PLATFORM in
    linux)
        npm run tauri build -- --bundles appimage,deb
        ;;
    darwin)
        npm run tauri build -- --bundles dmg,app
        ;;
    windows)
        npm run tauri build -- --bundles msi,nsis
        ;;
esac

echo ""
echo "🎉 Build complete!"
echo ""
echo "📦 Bundles created in: src-tauri/target/release/bundle/"

# List bundles
if [ -d "src-tauri/target/release/bundle" ]; then
    echo ""
    echo "📁 Bundle contents:"
    find src-tauri/target/release/bundle -type f -name "*.AppImage" -o -name "*.deb" -o -name "*.dmg" -o -name "*.app" -o -name "*.msi" -o -name "*.exe" | while read file; do
        SIZE=$(ls -lh "$file" | awk '{print $5}')
        echo "  - $(basename "$file") ($SIZE)"
    done
fi

echo ""
echo "✨ Release build complete!"
