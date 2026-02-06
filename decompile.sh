#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MINECRAFT_SRC_DIR="$SCRIPT_DIR/.minecraft"

# create temp directory on same filesystem to avoid cross-device link errors
TEMP_DIR="$SCRIPT_DIR/.gitcraft"
rm -rf "$TEMP_DIR"
mkdir -p "$TEMP_DIR"
echo "Cloning GitCraft into $TEMP_DIR..."

# cleanup on exit
trap "rm -rf $TEMP_DIR" EXIT

# clone GitCraft
git clone https://github.com/WinPlay02/GitCraft "$TEMP_DIR/GitCraft"

# patch Groovy version (use specific version instead of dynamic 5.0.+)
sed -i 's/groovy_version = 5\.0\.+/groovy_version = 5.0.0/' "$TEMP_DIR/GitCraft/gradle.properties"
echo "Patched gradle.properties:"
grep groovy "$TEMP_DIR/GitCraft/gradle.properties"

# run GitCraft
cd "$TEMP_DIR/GitCraft"
echo "Running GitCraft..."
./gradlew run -Dorg.gradle.jvmargs="-Xmx8G" --args="--override-repo-target=$MINECRAFT_SRC_DIR --only-unobfuscated --only-stable --mappings=identity_unmapped --min-version=1.21.11"

echo "Done! .minecraft has been updated."