#!/bin/bash
# BelizeChain Environment Validation Script
# Validates .env file and checks for required variables

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ENV_FILE="${SCRIPT_DIR}/.env"
ENV_TEMPLATE="${SCRIPT_DIR}/.env.template"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Counters
ERRORS=0
WARNINGS=0
SUCCESS=0

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}BelizeChain Environment Validator${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Check if .env file exists
if [ ! -f "$ENV_FILE" ]; then
    echo -e "${RED}✗ ERROR: .env file not found!${NC}"
    echo -e "  Copy .env.template to .env and configure your values:"
    echo -e "  ${YELLOW}cp .env.template .env${NC}"
    exit 1
else
    echo -e "${GREEN}✓ .env file exists${NC}"
    SUCCESS=$((SUCCESS + 1))
fi

# Source the .env file
set -a
source "$ENV_FILE"
set +a

echo ""
echo -e "${BLUE}Validating Critical Security Variables...${NC}"

# Function to check required variable
check_required() {
    local var_name=$1
    local var_value="${!var_name}"
    local min_length=${2:-8}
    
    if [ -z "$var_value" ]; then
        echo -e "${RED}✗ ERROR: $var_name is not set${NC}"
        ERRORS=$((ERRORS + 1))
        return 1
    elif [ "$var_value" = "CHANGE_ME_"* ] || [ "$var_value" = "changeme" ] || [ "$var_value" = "your_"* ]; then
        echo -e "${RED}✗ ERROR: $var_name still has default/placeholder value${NC}"
        ERRORS=$((ERRORS + 1))
        return 1
    elif [ ${#var_value} -lt $min_length ]; then
        echo -e "${YELLOW}⚠ WARNING: $var_name is too short (minimum $min_length characters)${NC}"
        WARNINGS=$((WARNINGS + 1))
        return 1
    else
        echo -e "${GREEN}✓ $var_name is configured${NC}"
        SUCCESS=$((SUCCESS + 1))
        return 0
    fi
}

# Function to check optional variable
check_optional() {
    local var_name=$1
    local var_value="${!var_name}"
    
    if [ -z "$var_value" ]; then
        echo -e "${YELLOW}⚠ INFO: $var_name is not set (optional)${NC}"
        return 1
    else
        echo -e "${GREEN}✓ $var_name is configured${NC}"
        SUCCESS=$((SUCCESS + 1))
        return 0
    fi
}

# Check critical security variables
check_required "POSTGRES_PASSWORD" 16
check_required "REDIS_PASSWORD" 16
check_required "GRAFANA_ADMIN_PASSWORD" 12

if [ "$ENVIRONMENT" = "production" ]; then
    echo ""
    echo -e "${BLUE}Production Environment Detected - Additional Checks...${NC}"
    check_required "JWT_SECRET" 32
    check_required "ENCRYPTION_SALT" 16
    
    # Check if DEBUG is false
    if [ "$DEBUG" = "true" ]; then
        echo -e "${RED}✗ ERROR: DEBUG should be false in production${NC}"
        ERRORS=$((ERRORS + 1))
    else
        echo -e "${GREEN}✓ DEBUG is disabled${NC}"
        SUCCESS=$((SUCCESS + 1))
    fi
    
    # Check CORS origins
    if [[ "$CORS_ORIGINS" == *"localhost"* ]]; then
        echo -e "${YELLOW}⚠ WARNING: CORS_ORIGINS contains localhost in production${NC}"
        WARNINGS=$((WARNINGS + 1))
    else
        echo -e "${GREEN}✓ CORS_ORIGINS configured for production${NC}"
        SUCCESS=$((SUCCESS + 1))
    fi
fi

echo ""
echo -e "${BLUE}Validating Service Configuration...${NC}"

# Blockchain
check_required "BLOCKCHAIN_RPC"
check_optional "NODE_NAME"

# Nawal
check_required "NAWAL_API_PORT"
check_optional "NAWAL_MIN_PARTICIPANTS"

# Kinich
check_required "KINICH_API_PORT"
if [ "$AZURE_QUANTUM_ENABLED" = "true" ]; then
    echo -e "${BLUE}Azure Quantum is enabled - checking credentials...${NC}"
    check_optional "AZURE_QUANTUM_SUBSCRIPTION_ID"
    check_optional "AZURE_QUANTUM_RESOURCE_GROUP"
    check_optional "AZURE_QUANTUM_WORKSPACE"
fi

# Pakit
check_required "PAKIT_API_PORT"
if [ "$IPFS_ENABLED" = "true" ]; then
    echo -e "${BLUE}IPFS is enabled - checking configuration...${NC}"
    check_optional "IPFS_API"
fi

# Database
check_required "POSTGRES_DB"
check_required "POSTGRES_USER"

echo ""
echo -e "${BLUE}Validating Port Configuration...${NC}"

# Check for port conflicts
declare -A PORTS
PORTS[P2P]=${P2P_PORT:-30333}
PORTS[RPC]=${RPC_PORT:-9933}
PORTS[WS]=${WS_PORT:-9944}
PORTS[NAWAL]=${NAWAL_API_PORT:-8080}
PORTS[KINICH]=${KINICH_API_PORT:-8888}
PORTS[PAKIT]=${PAKIT_API_PORT:-8001}
PORTS[POSTGRES]=${POSTGRES_PORT:-5432}
PORTS[REDIS]=${REDIS_PORT:-6379}
PORTS[IPFS_GATEWAY]=${IPFS_GATEWAY_PORT:-8082}

# Check for duplicates
seen_ports=()
port_conflicts=0
for service in "${!PORTS[@]}"; do
    port="${PORTS[$service]}"
    if [[ " ${seen_ports[@]} " =~ " ${port} " ]]; then
        echo -e "${RED}✗ ERROR: Port $port is used by multiple services${NC}"
        port_conflicts=$((port_conflicts + 1))
        ERRORS=$((ERRORS + 1))
    else
        seen_ports+=("$port")
    fi
done

if [ $port_conflicts -eq 0 ]; then
    echo -e "${GREEN}✓ No port conflicts detected${NC}"
    SUCCESS=$((SUCCESS + 1))
fi

# Check if ports are available (only for localhost services)
echo ""
echo -e "${BLUE}Checking Port Availability...${NC}"

check_port() {
    local port=$1
    local service=$2
    
    if command -v nc &> /dev/null; then
        if nc -z localhost "$port" 2>/dev/null; then
            echo -e "${YELLOW}⚠ WARNING: Port $port ($service) is already in use${NC}"
            WARNINGS=$((WARNINGS + 1))
        else
            echo -e "${GREEN}✓ Port $port ($service) is available${NC}"
            SUCCESS=$((SUCCESS + 1))
        fi
    else
        echo -e "${YELLOW}⚠ INFO: 'nc' command not found - skipping port checks${NC}"
        return
    fi
}

for service in "${!PORTS[@]}"; do
    check_port "${PORTS[$service]}" "$service"
done

# Check Docker
echo ""
echo -e "${BLUE}Validating Docker Environment...${NC}"

if command -v docker &> /dev/null; then
    echo -e "${GREEN}✓ Docker is installed${NC}"
    SUCCESS=$((SUCCESS + 1))
    
    if docker info &> /dev/null; then
        echo -e "${GREEN}✓ Docker daemon is running${NC}"
        SUCCESS=$((SUCCESS + 1))
    else
        echo -e "${RED}✗ ERROR: Docker daemon is not running${NC}"
        ERRORS=$((ERRORS + 1))
    fi
else
    echo -e "${RED}✗ ERROR: Docker is not installed${NC}"
    ERRORS=$((ERRORS + 1))
fi

if command -v docker-compose &> /dev/null; then
    echo -e "${GREEN}✓ Docker Compose is installed${NC}"
    SUCCESS=$((SUCCESS + 1))
else
    echo -e "${RED}✗ ERROR: Docker Compose is not installed${NC}"
    ERRORS=$((ERRORS + 1))
fi

# Validate docker-compose.yml syntax
echo ""
echo -e "${BLUE}Validating Docker Compose Configuration...${NC}"

if docker-compose -f "${SCRIPT_DIR}/infra/docker-compose.yml" --env-file "$ENV_FILE" config &> /dev/null; then
    echo -e "${GREEN}✓ docker-compose.yml is valid${NC}"
    SUCCESS=$((SUCCESS + 1))
else
    echo -e "${RED}✗ ERROR: docker-compose.yml validation failed${NC}"
    echo -e "  Run: ${YELLOW}docker-compose config${NC} for details"
    ERRORS=$((ERRORS + 1))
fi

# Summary
echo ""
echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}Validation Summary${NC}"
echo -e "${BLUE}========================================${NC}"
echo -e "${GREEN}Successes: $SUCCESS${NC}"
echo -e "${YELLOW}Warnings:  $WARNINGS${NC}"
echo -e "${RED}Errors:    $ERRORS${NC}"
echo ""

if [ $ERRORS -gt 0 ]; then
    echo -e "${RED}✗ Validation FAILED - Please fix errors before proceeding${NC}"
    echo ""
    echo -e "Common fixes:"
    echo -e "  1. Copy template: ${YELLOW}cp .env.template .env${NC}"
    echo -e "  2. Generate passwords: ${YELLOW}openssl rand -base64 32${NC}"
    echo -e "  3. Edit .env file with secure values"
    echo -e "  4. Ensure Docker is running: ${YELLOW}sudo systemctl start docker${NC}"
    exit 1
elif [ $WARNINGS -gt 0 ]; then
    echo -e "${YELLOW}⚠ Validation PASSED with warnings${NC}"
    echo -e "  Review warnings above and fix if needed"
    exit 0
else
    echo -e "${GREEN}✓ Validation PASSED - Environment is properly configured!${NC}"
    echo ""
    echo -e "Next steps:"
    echo -e "  1. Install Python dependencies: ${YELLOW}pip install -r requirements.txt${NC}"
    echo -e "  2. Start services: ${YELLOW}docker-compose up -d${NC}"
    echo -e "  3. Check logs: ${YELLOW}docker-compose logs -f${NC}"
    exit 0
fi
