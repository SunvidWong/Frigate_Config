#!/bin/bash
# Build hardware detection agent for all platforms
# T204: Multi-platform agent binary builder

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
AGENT_DIR="$PROJECT_ROOT/agent"
OUTPUT_DIR="$PROJECT_ROOT/target/release"

echo "🔨 Building Frigate Config Agent for all platforms..."

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Build configuration
PLATFORMS=(
    "linux/amd64"
    "linux/arm64"
    "darwin/amd64"
    "darwin/arm64"
    "windows/amd64"
)

# Build for each platform
for platform in "${PLATFORMS[@]}"; do
    IFS='/' read -r GOOS GOARCH <<< "$platform"

    echo ""
    echo "📦 Building for $GOOS/$GOARCH..."

    OUTPUT_NAME="agent-${GOOS}-${GOARCH}"
    if [ "$GOOS" = "windows" ]; then
        OUTPUT_NAME="${OUTPUT_NAME}.exe"
    fi

    cd "$AGENT_DIR"

    CGO_ENABLED=0 GOOS=$GOOS GOARCH=$GOARCH go build \
        -v \
        -ldflags="-s -w -X main.Version=${VERSION:-dev} -X main.BuildTime=$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
        -o "$OUTPUT_DIR/$OUTPUT_NAME" \
        ./cmd/agent

    # Show file info
    if [ -f "$OUTPUT_DIR/$OUTPUT_NAME" ]; then
        SIZE=$(ls -lh "$OUTPUT_DIR/$OUTPUT_NAME" | awk '{print $5}')
        echo "✅ Built: $OUTPUT_NAME ($SIZE)"
    else
        echo "❌ Failed to build: $OUTPUT_NAME"
        exit 1
    fi
done

echo ""
echo "🎉 All agent binaries built successfully!"
echo ""
echo "📁 Output directory: $OUTPUT_DIR"
ls -lh "$OUTPUT_DIR"/agent-*

echo ""
echo "📝 Checksums:"
cd "$OUTPUT_DIR"
shasum -a 256 agent-* > agent-checksums.txt
cat agent-checksums.txt

echo ""
echo "✨ Done!"
