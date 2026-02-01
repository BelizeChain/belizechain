#!/bin/bash
# Monitor Polkadot SDK for releases that might include ring/lru fixes
# Run this weekly or add to cron: 0 9 * * 1 (Mondays at 9 AM)

set -e

CURRENT_SDK_VERSION="polkadot-stable2509"
GITHUB_API="https://api.github.com/repos/paritytech/polkadot-sdk/releases/latest"
NOTIFY_EMAIL="${NOTIFY_EMAIL:-}" # Optional: set to email for notifications

echo "🔍 Checking for Polkadot SDK updates..."
echo "Current version: $CURRENT_SDK_VERSION"
echo ""

# Fetch latest release
LATEST_VERSION=$(curl -s $GITHUB_API | jq -r .tag_name)
RELEASE_DATE=$(curl -s $GITHUB_API | jq -r .published_at)
RELEASE_URL=$(curl -s $GITHUB_API | jq -r .html_url)

echo "Latest release: $LATEST_VERSION"
echo "Published: $RELEASE_DATE"
echo "URL: $RELEASE_URL"
echo ""

if [ "$LATEST_VERSION" != "$CURRENT_SDK_VERSION" ]; then
    echo "🚨 NEW POLKADOT SDK RELEASE DETECTED: $LATEST_VERSION"
    echo ""
    echo "ACTION REQUIRED:"
    echo "1. Check release notes: $RELEASE_URL"
    echo "2. Verify if it includes ring >=0.17.12 fix"
    echo "3. Verify if it includes lru memory safety fix"
    echo "4. If fixes included, update Cargo.toml dependencies"
    echo "5. Run: cargo update && cargo audit"
    echo "6. Run full test suite: cargo test --workspace"
    echo ""
    
    # Check if release notes mention our vulnerabilities
    RELEASE_NOTES=$(curl -s $GITHUB_API | jq -r .body)
    
    if echo "$RELEASE_NOTES" | grep -qi "ring\|crypto\|0.17"; then
        echo "✅ Release notes mention 'ring' or 'crypto' - LIKELY CONTAINS FIX!"
    fi
    
    if echo "$RELEASE_NOTES" | grep -qi "lru\|cache\|memory"; then
        echo "✅ Release notes mention 'lru' or 'cache' - LIKELY CONTAINS FIX!"
    fi
    
    # Optional: Send email notification
    if [ -n "$NOTIFY_EMAIL" ]; then
        echo "Sending notification to $NOTIFY_EMAIL..."
        echo "New Polkadot SDK release: $LATEST_VERSION - Check for ring/lru fixes!" | \
            mail -s "🚨 Polkadot SDK Update Available" "$NOTIFY_EMAIL"
    fi
    
    exit 1  # Exit with error to trigger CI notifications
else
    echo "✅ Already on latest version: $CURRENT_SDK_VERSION"
    echo "Next check: Run this script weekly or subscribe to:"
    echo "https://github.com/paritytech/polkadot-sdk/releases.atom"
fi

echo ""
echo "📋 Current dependency status:"
echo "Run: cargo audit | grep -E 'RUSTSEC-2025-0009|RUSTSEC-2025-0010|RUSTSEC-2026-0002'"
