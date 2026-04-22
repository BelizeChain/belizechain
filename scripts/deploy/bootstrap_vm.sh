#!/bin/bash
# BelizeChain host bootstrap (Ceiba/self-hosted)
# Run this once on each host before first deployment.
set -euo pipefail

echo "🔧 BelizeChain Host Bootstrap"
echo "============================="

# ── 1. Docker network shared by all BelizeChain containers ──
NETWORK="belizechain-net"
if docker network inspect "$NETWORK" >/dev/null 2>&1; then
  echo "✅ Docker network '$NETWORK' already exists"
else
  echo "🌐 Creating Docker network '$NETWORK'…"
  docker network create "$NETWORK"
  echo "✅ Network created"
fi

# ── 2. Optional container registry login ─────────────────────
echo ""
if [ -n "${CONTAINER_REGISTRY:-}" ] && [ -n "${CONTAINER_REGISTRY_USER:-}" ] && [ -n "${CONTAINER_REGISTRY_PASSWORD:-}" ]; then
  echo "🔑 Logging in to container registry ($CONTAINER_REGISTRY)…"
  echo "$CONTAINER_REGISTRY_PASSWORD" | docker login "$CONTAINER_REGISTRY" -u "$CONTAINER_REGISTRY_USER" --password-stdin
  echo "✅ Registry login complete"
else
  echo "ℹ️  Skipping registry login (CONTAINER_REGISTRY* not set)"
fi

# ── 3. Create persistent volumes ────────────────────────────
echo ""
echo "📦 Ensuring Docker volumes exist…"
for vol in blockchain-data pakit-data quantum-results nawal-models; do
  docker volume create "$vol" 2>/dev/null && echo "   ✅ $vol" || echo "   ✅ $vol (exists)"
done

# ── 4. Firewall reminder ─────────────────────────────────────
echo ""
echo "🔒 Verify host firewall allows inbound traffic on:"
echo "   • 30333  (P2P)"
echo "   • 9944   (RPC / WebSocket)"
echo "   • 9615   (Prometheus)"
echo "   • 8001   (Pakit API)"
echo "   • 8081   (Kinich API)"
echo "   • 8002   (Nawal API)"
echo "   • 3000-3002 (UI apps)"
echo ""
echo "🎉 Bootstrap complete. Host is ready for Ceiba/self-hosted deployments."
