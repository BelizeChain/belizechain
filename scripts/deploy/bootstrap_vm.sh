#!/bin/bash
# ─────────────────────────────────────────────────────────
# BelizeChain — Azure VM Bootstrap
# Run this ONCE on each VM before the first CI/CD deploy.
# It creates the shared Docker network and pulls base images.
# ─────────────────────────────────────────────────────────
set -euo pipefail

echo "🔧 BelizeChain VM Bootstrap"
echo "==========================="

# ── 1. Docker network shared by all BelizeChain containers ──
NETWORK="belizechain-net"
if docker network inspect "$NETWORK" >/dev/null 2>&1; then
  echo "✅ Docker network '$NETWORK' already exists"
else
  echo "🌐 Creating Docker network '$NETWORK'…"
  docker network create "$NETWORK"
  echo "✅ Network created"
fi

# ── 2. Login to ACR ─────────────────────────────────────────
REGISTRY="belizechainregistry.azurecr.io"
ACR_USER="${ACR_USERNAME:-wicked}"
echo ""
echo "🔑 Logging in to ACR ($REGISTRY)…"
if [ -n "${ACR_PASSWORD:-}" ]; then
  echo "$ACR_PASSWORD" | docker login "$REGISTRY" -u "$ACR_USER" --password-stdin
else
  echo "   Enter ACR password when prompted."
  docker login "$REGISTRY" -u "$ACR_USER"
fi

# ── 3. Create persistent volumes ────────────────────────────
echo ""
echo "📦 Ensuring Docker volumes exist…"
for vol in blockchain-data pakit-data quantum-results nawal-models; do
  docker volume create "$vol" 2>/dev/null && echo "   ✅ $vol" || echo "   ✅ $vol (exists)"
done

# ── 4. Firewall / NSG reminder ──────────────────────────────
echo ""
echo "🔒 Verify Azure NSG allows inbound traffic on:"
echo "   • 30333  (P2P)"
echo "   • 9944   (RPC / WebSocket)"
echo "   • 9615   (Prometheus)"
echo "   • 8001   (Pakit API)"
echo "   • 8081   (Kinich API)"
echo "   • 8002   (Nawal API)"
echo "   • 3000-3002 (UI apps)"
echo ""
echo "🎉 Bootstrap complete. CI/CD deploys can now target this VM."
