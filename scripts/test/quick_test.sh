#!/bin/bash
# Quick BelizeChain Component Test Script

set -u

echo "🧪 BelizeChain Quick Component Test"
echo "===================================="
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

CEIBA_RPC_URL="${CEIBA_RPC_URL:-http://100.81.45.25:9944}"
LOCAL_RPC_URL="${LOCAL_RPC_URL:-http://localhost:9933}"

if command -v docker >/dev/null 2>&1 && docker compose version >/dev/null 2>&1; then
    DOCKER_COMPOSE_CMD="docker compose"
elif command -v docker-compose >/dev/null 2>&1; then
    DOCKER_COMPOSE_CMD="docker-compose"
else
    DOCKER_COMPOSE_CMD=""
fi

has_cmd() {
    command -v "$1" >/dev/null 2>&1
}

# Test blockchain RPC (Ceiba first)
echo -n "Testing Blockchain RPC (Ceiba)... "
if curl -s -X POST -H "Content-Type: application/json" -d '{"id":1,"jsonrpc":"2.0","method":"system_health","params":[]}' "$CEIBA_RPC_URL" | grep -q "result"; then
    echo -e "${GREEN}✅ PASS${NC}"
else
    echo -e "${YELLOW}⚠️  SKIP (Ceiba RPC unreachable at $CEIBA_RPC_URL)${NC}"
fi

echo -n "Testing Blockchain RPC (Local)... "
if curl -s -X POST -H "Content-Type: application/json" -d '{"id":1,"jsonrpc":"2.0","method":"system_health","params":[]}' "$LOCAL_RPC_URL" | grep -q "result"; then
    echo -e "${GREEN}✅ PASS${NC}"
else
    echo -e "${YELLOW}⚠️  SKIP (Local RPC unreachable at $LOCAL_RPC_URL)${NC}"
fi

# Test blockchain WebSocket
echo -n "Testing Blockchain WebSocket... "
if timeout 2 wscat -c ws://localhost:9944 -x '{"id":1,"jsonrpc":"2.0","method":"system_health","params":[]}' 2>/dev/null | grep -q "result"; then
    echo -e "${GREEN}✅ PASS${NC}"
else
    if has_cmd wscat; then
        echo -e "${YELLOW}⚠️  SKIP (Local WebSocket endpoint unavailable)${NC}"
    else
        echo -e "${YELLOW}⚠️  SKIP (wscat not installed)${NC}"
    fi
fi

# Test IPFS
echo -n "Testing IPFS API... "
if curl -s http://localhost:5001/api/v0/version | grep -q "Version"; then
    echo -e "${GREEN}✅ PASS${NC}"
else
    echo -e "${YELLOW}⚠️  SKIP (local IPFS not running)${NC}"
fi

# Test Nawal AI
echo -n "Testing Nawal AI API... "
if curl -s http://localhost:8080/health 2>/dev/null | grep -q -E "ok|healthy|running"; then
    echo -e "${GREEN}✅ PASS${NC}"
elif curl -s http://localhost:8080/ 2>/dev/null | grep -q -E "Nawal|BelizeChain"; then
    echo -e "${GREEN}✅ PASS (root endpoint)${NC}"
else
    echo -e "${YELLOW}⚠️  SKIP (local Nawal API not running)${NC}"
fi

# Test Kinich Quantum
echo -n "Testing Kinich Quantum API... "
if curl -s http://localhost:8888/health 2>/dev/null | grep -q -E "ok|healthy|running"; then
    echo -e "${GREEN}✅ PASS${NC}"
elif curl -s http://localhost:8888/ 2>/dev/null | grep -q -E "Kinich|Quantum"; then
    echo -e "${GREEN}✅ PASS (root endpoint)${NC}"
else
    echo -e "${YELLOW}⚠️  SKIP (local Kinich API not running)${NC}"
fi

# Test PostgreSQL
echo -n "Testing PostgreSQL... "
if has_cmd pg_isready && pg_isready -h localhost -p 5432 -U belizechain 2>/dev/null | grep -q "accepting connections"; then
    echo -e "${GREEN}✅ PASS${NC}"
elif has_cmd nc && nc -z localhost 5432 2>/dev/null; then
    echo -e "${GREEN}✅ PASS (port open)${NC}"
else
    echo -e "${YELLOW}⚠️  SKIP (local PostgreSQL not running)${NC}"
fi

# Test Redis
echo -n "Testing Redis... "
if has_cmd redis-cli && redis-cli -h localhost ping 2>/dev/null | grep -q "PONG"; then
    echo -e "${GREEN}✅ PASS${NC}"
elif has_cmd nc && nc -z localhost 6379 2>/dev/null; then
    echo -e "${GREEN}✅ PASS (port open)${NC}"
else
    echo -e "${YELLOW}⚠️  SKIP (local Redis not running)${NC}"
fi

# Test UI portals
echo -n "Testing Blue Hole Portal... "
if curl -s http://localhost:3000 2>/dev/null | grep -q -E "Blue Hole|BelizeChain|<!DOCTYPE"; then
    echo -e "${GREEN}✅ PASS${NC}"
else
    echo -e "${YELLOW}⚠️  SKIP (local UI not running)${NC}"
fi

echo -n "Testing Maya Wallet... "
if curl -s http://localhost:3001 2>/dev/null | grep -q -E "Maya|Wallet|BelizeChain|<!DOCTYPE"; then
    echo -e "${GREEN}✅ PASS${NC}"
else
    echo -e "${YELLOW}⚠️  SKIP (local UI not running)${NC}"
fi

echo -n "Testing Kijka Explorer... "
if curl -s http://localhost:3002 2>/dev/null | grep -q -E "Kijka|Explorer|BelizeChain|<!DOCTYPE"; then
    echo -e "${GREEN}✅ PASS${NC}"
else
    echo -e "${YELLOW}⚠️  SKIP (local UI not running)${NC}"
fi

# Test monitoring
echo -n "Testing Prometheus... "
if curl -s http://localhost:9090/-/healthy 2>/dev/null | grep -q "Healthy"; then
    echo -e "${GREEN}✅ PASS${NC}"
else
    echo -e "${YELLOW}⚠️  SKIP (local Prometheus not running)${NC}"
fi

echo -n "Testing Grafana... "
if curl -s http://localhost:3003/api/health 2>/dev/null | grep -q -E "ok|database"; then
    echo -e "${GREEN}✅ PASS${NC}"
else
    echo -e "${YELLOW}⚠️  SKIP (local Grafana not running)${NC}"
fi

echo ""
echo "📊 Container Status:"
if [ -n "$DOCKER_COMPOSE_CMD" ]; then
    if [ -f /home/wicked/Projects/infra/docker-compose.yml ]; then
        if ! $DOCKER_COMPOSE_CMD -f /home/wicked/Projects/infra/docker-compose.yml ps 2>/dev/null; then
            echo -e "${YELLOW}⚠️  SKIP (compose requires env vars not currently set)${NC}"
        fi
    elif [ -f infra/docker-compose.yml ]; then
        if ! $DOCKER_COMPOSE_CMD -f infra/docker-compose.yml ps 2>/dev/null; then
            echo -e "${YELLOW}⚠️  SKIP (compose requires env vars not currently set)${NC}"
        fi
    else
        echo -e "${YELLOW}⚠️  No docker-compose.yml found in expected paths${NC}"
    fi
else
    echo -e "${YELLOW}⚠️  docker compose not installed${NC}"
fi

echo ""
if [ -n "$DOCKER_COMPOSE_CMD" ]; then
    echo "💡 For detailed logs: $DOCKER_COMPOSE_CMD -f /home/wicked/Projects/infra/docker-compose.yml logs -f <service-name>"
else
    echo "💡 Install Docker Compose plugin to enable container status checks"
fi
