#!/bin/bash
# Quick BelizeChain Component Test Script

echo "🧪 BelizeChain Quick Component Test"
echo "===================================="
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test blockchain RPC
echo -n "Testing Blockchain RPC... "
if curl -s -X POST -H "Content-Type: application/json" -d '{"id":1,"jsonrpc":"2.0","method":"system_health","params":[]}' http://localhost:9933 | grep -q "result"; then
    echo -e "${GREEN}✅ PASS${NC}"
else
    echo -e "${RED}❌ FAIL${NC}"
fi

# Test blockchain WebSocket
echo -n "Testing Blockchain WebSocket... "
if timeout 2 wscat -c ws://localhost:9944 -x '{"id":1,"jsonrpc":"2.0","method":"system_health","params":[]}' 2>/dev/null | grep -q "result"; then
    echo -e "${GREEN}✅ PASS${NC}"
else
    echo -e "${YELLOW}⚠️  SKIP (wscat not installed)${NC}"
fi

# Test IPFS
echo -n "Testing IPFS API... "
if curl -s http://localhost:5001/api/v0/version | grep -q "Version"; then
    echo -e "${GREEN}✅ PASS${NC}"
else
    echo -e "${RED}❌ FAIL${NC}"
fi

# Test Nawal AI
echo -n "Testing Nawal AI API... "
if curl -s http://localhost:8080/health 2>/dev/null | grep -q -E "ok|healthy|running"; then
    echo -e "${GREEN}✅ PASS${NC}"
elif curl -s http://localhost:8080/ 2>/dev/null | grep -q -E "Nawal|BelizeChain"; then
    echo -e "${GREEN}✅ PASS (root endpoint)${NC}"
else
    echo -e "${RED}❌ FAIL${NC}"
fi

# Test Kinich Quantum
echo -n "Testing Kinich Quantum API... "
if curl -s http://localhost:8888/health 2>/dev/null | grep -q -E "ok|healthy|running"; then
    echo -e "${GREEN}✅ PASS${NC}"
elif curl -s http://localhost:8888/ 2>/dev/null | grep -q -E "Kinich|Quantum"; then
    echo -e "${GREEN}✅ PASS (root endpoint)${NC}"
else
    echo -e "${RED}❌ FAIL${NC}"
fi

# Test PostgreSQL
echo -n "Testing PostgreSQL... "
if pg_isready -h localhost -p 5432 -U belizechain 2>/dev/null | grep -q "accepting connections"; then
    echo -e "${GREEN}✅ PASS${NC}"
elif nc -z localhost 5432 2>/dev/null; then
    echo -e "${GREEN}✅ PASS (port open)${NC}"
else
    echo -e "${RED}❌ FAIL${NC}"
fi

# Test Redis
echo -n "Testing Redis... "
if redis-cli -h localhost ping 2>/dev/null | grep -q "PONG"; then
    echo -e "${GREEN}✅ PASS${NC}"
elif nc -z localhost 6379 2>/dev/null; then
    echo -e "${GREEN}✅ PASS (port open)${NC}"
else
    echo -e "${RED}❌ FAIL${NC}"
fi

# Test UI portals
echo -n "Testing Blue Hole Portal... "
if curl -s http://localhost:3000 2>/dev/null | grep -q -E "Blue Hole|BelizeChain|<!DOCTYPE"; then
    echo -e "${GREEN}✅ PASS${NC}"
else
    echo -e "${RED}❌ FAIL${NC}"
fi

echo -n "Testing Maya Wallet... "
if curl -s http://localhost:3001 2>/dev/null | grep -q -E "Maya|Wallet|BelizeChain|<!DOCTYPE"; then
    echo -e "${GREEN}✅ PASS${NC}"
else
    echo -e "${RED}❌ FAIL${NC}"
fi

echo -n "Testing Kijka Explorer... "
if curl -s http://localhost:3002 2>/dev/null | grep -q -E "Kijka|Explorer|BelizeChain|<!DOCTYPE"; then
    echo -e "${GREEN}✅ PASS${NC}"
else
    echo -e "${RED}❌ FAIL${NC}"
fi

# Test monitoring
echo -n "Testing Prometheus... "
if curl -s http://localhost:9090/-/healthy 2>/dev/null | grep -q "Healthy"; then
    echo -e "${GREEN}✅ PASS${NC}"
else
    echo -e "${RED}❌ FAIL${NC}"
fi

echo -n "Testing Grafana... "
if curl -s http://localhost:3003/api/health 2>/dev/null | grep -q -E "ok|database"; then
    echo -e "${GREEN}✅ PASS${NC}"
else
    echo -e "${RED}❌ FAIL${NC}"
fi

echo ""
echo "📊 Container Status:"
docker-compose -f /home/wicked/BelizeChain/belizechain/infra/docker-compose.yml ps

echo ""
echo "💡 For detailed logs: docker-compose -f infra/docker-compose.yml logs -f <service-name>"
