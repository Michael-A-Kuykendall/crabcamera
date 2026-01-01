#!/bin/bash

# Script to create GitHub releases for existing tags
# Run this locally after setting up the release workflow

set -e

# Get all tags
TAGS=$(git tag --list | sort -V)

for TAG in $TAGS; do
    # Remove 'v' prefix for version
    VERSION=${TAG#v}

    echo "Processing $TAG (version $VERSION)..."

    # Check if release already exists (basic check - you might need gh CLI)
    # For now, we'll assume we need to create them

    # Extract release notes from CHANGELOG.md
    awk -v version="$VERSION" '
        BEGIN { in_section = 0; found = 0 }
        /^## \['\''?'version/ { in_section = 1; found = 1; next }
        /^## \[/ && in_section { in_section = 0; exit }
        in_section { print }
    ' CHANGELOG.md > /tmp/release_notes_$VERSION.md

    if [ -s /tmp/release_notes_$VERSION.md ]; then
        echo "Found changelog for $VERSION"
        echo "Release notes preview:"
        head -10 /tmp/release_notes_$VERSION.md
        echo "..."

        # Use GitHub CLI to create release (if available)
        if command -v gh &> /dev/null; then
            echo "Creating release for $TAG..."
            gh release create "$TAG" \
                --title "Release $TAG" \
                --notes-file /tmp/release_notes_$VERSION.md \
                --generate-notes
        else
            echo "GitHub CLI not found. Please install and run:"
            echo "gh release create $TAG --title \"Release $TAG\" --notes-file /tmp/release_notes_$VERSION.md"
        fi
    else
        echo "No changelog found for $VERSION"
    fi

    echo "---"
done

echo "Done processing tags. If using gh CLI, releases should be created."
echo "Otherwise, use the commands above manually."