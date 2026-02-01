#!/bin/bash
# BelizeChain Security & Configuration Verification Script
# Ensures sensitive files are properly gitignored and configurations are valid

set -e

echo "🔒 BelizeChain Security Verification"
echo "===================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

ISSUES=0

# 1. Verify .env is gitignored
echo "1️⃣  Checking .env protection..."
if git check-ignore .env > /dev/null 2>&1; then
    echo -e "   ${GREEN}✅ .env is gitignored${NC}"
else
    echo -e "   ${RED}❌ .env is NOT gitignored - SECURITY RISK!${NC}"
    ISSUES=$((ISSUES + 1))
fi

# 2. Verify .env exists
if [ -f .env ]; then
    echo -e "   ${GREEN}✅ .env file exists${NC}"
    
    # Check for placeholder values
    if grep -q "CHANGE_ME" .env; then
        echo -e "   ${YELLOW}⚠️  .env contains CHANGE_ME placeholders${NC}"
        echo "      Update these before production deployment:"
        grep "CHANGE_ME" .env | head -3
    fi
else
    echo -e "   ${YELLOW}⚠️  .env file not found${NC}"
    echo "      Copy .env.template to .env and configure"
fi

echo ""

# 3. Verify .coverage is gitignored
echo "2️⃣  Checking test coverage artifacts..."
if git check-ignore .coverage > /dev/null 2>&1; then
    echo -e "   ${GREEN}✅ .coverage is gitignored${NC}"
else
    echo -e "   ${RED}❌ .coverage is NOT gitignored${NC}"
    ISSUES=$((ISSUES + 1))
fi

if git check-ignore htmlcov/ > /dev/null 2>&1; then
    echo -e "   ${GREEN}✅ htmlcov/ is gitignored${NC}"
else
    echo -e "   ${RED}❌ htmlcov/ is NOT gitignored${NC}"
    ISSUES=$((ISSUES + 1))
fi

echo ""

# 4. Verify logs are gitignored
echo "3️⃣  Checking log file protection..."
if git check-ignore logs/test.log > /dev/null 2>&1; then
    echo -e "   ${GREEN}✅ logs/*.log is gitignored${NC}"
else
    echo -e "   ${RED}❌ logs/*.log is NOT gitignored${NC}"
    ISSUES=$((ISSUES + 1))
fi

# Check for any .log files in root
LOG_FILES=$(find . -maxdepth 1 -name "*.log" 2>/dev/null | wc -l)
if [ "$LOG_FILES" -gt 0 ]; then
    echo -e "   ${YELLOW}⚠️  Found $LOG_FILES .log files in root directory${NC}"
    echo "      Move these to logs/ directory:"
    find . -maxdepth 1 -name "*.log" -exec basename {} \;
fi

echo ""

# 5. Verify Python build artifacts are gitignored
echo "4️⃣  Checking Python build artifacts..."
if git check-ignore pakit.egg-info/ > /dev/null 2>&1; then
    echo -e "   ${GREEN}✅ *.egg-info/ is gitignored${NC}"
else
    echo -e "   ${RED}❌ *.egg-info/ is NOT gitignored${NC}"
    ISSUES=$((ISSUES + 1))
fi

# Check for any egg-info directories
EGG_INFO=$(find . -name "*.egg-info" -type d 2>/dev/null | wc -l)
if [ "$EGG_INFO" -gt 0 ]; then
    echo -e "   ${YELLOW}⚠️  Found $EGG_INFO .egg-info directories${NC}"
    echo "      These can be deleted (auto-regenerated):"
    find . -name "*.egg-info" -type d
fi

echo ""

# 6. Verify node_modules is gitignored
echo "5️⃣  Checking Node.js artifacts..."
if git check-ignore node_modules/ > /dev/null 2>&1; then
    echo -e "   ${GREEN}✅ node_modules/ is gitignored${NC}"
else
    echo -e "   ${RED}❌ node_modules/ is NOT gitignored${NC}"
    ISSUES=$((ISSUES + 1))
fi

echo ""

# 7. Verify Rust build artifacts are gitignored
echo "6️⃣  Checking Rust build artifacts..."
if git check-ignore target/ > /dev/null 2>&1; then
    echo -e "   ${GREEN}✅ target/ is gitignored${NC}"
else
    echo -e "   ${RED}❌ target/ is NOT gitignored${NC}"
    ISSUES=$((ISSUES + 1))
fi

if git check-ignore Cargo.lock > /dev/null 2>&1; then
    echo -e "   ${GREEN}✅ Cargo.lock is gitignored${NC}"
else
    echo -e "   ${YELLOW}⚠️  Cargo.lock is tracked (expected for workspace root)${NC}"
fi

echo ""

# 8. Check for secrets in tracked files
echo "7️⃣  Scanning for potential secrets in tracked files..."
SECRETS_FOUND=0

# Check for common secret patterns (API keys, passwords, tokens)
if git grep -E "(API_KEY|PASSWORD|SECRET|TOKEN).*=.*['\"][^CHANGE_ME]" 2>/dev/null | grep -v ".env.template" | grep -q .; then
    echo -e "   ${YELLOW}⚠️  Found potential secrets in tracked files:${NC}"
    git grep -E "(API_KEY|PASSWORD|SECRET|TOKEN).*=.*['\"]" 2>/dev/null | grep -v ".env.template" | head -5
    SECRETS_FOUND=1
else
    echo -e "   ${GREEN}✅ No obvious secrets found in tracked files${NC}"
fi

echo ""

# 9. Verify Python package structure
echo "8️⃣  Checking Python package structure..."
SETUP_FILES=0
[ -f "nawal/setup.py" ] && SETUP_FILES=$((SETUP_FILES + 1)) && echo -e "   ${GREEN}✅ nawal/setup.py exists${NC}"
[ -f "kinich/setup.py" ] && SETUP_FILES=$((SETUP_FILES + 1)) && echo -e "   ${GREEN}✅ kinich/setup.py exists${NC}"
[ -f "pakit/setup.py" ] && SETUP_FILES=$((SETUP_FILES + 1)) && echo -e "   ${GREEN}✅ pakit/setup.py exists${NC}"

if [ "$SETUP_FILES" -eq 3 ]; then
    echo -e "   ${GREEN}✅ All Python components have setup.py${NC}"
else
    echo -e "   ${YELLOW}⚠️  Missing setup.py files ($SETUP_FILES/3 found)${NC}"
fi

# Check for root setup.py (should NOT exist)
if [ -f "setup.py" ]; then
    echo -e "   ${YELLOW}⚠️  Root setup.py exists (should be moved to pakit/)${NC}"
fi

echo ""

# 10. Summary
echo "===================================="
if [ $ISSUES -eq 0 ] && [ $SECRETS_FOUND -eq 0 ]; then
    echo -e "${GREEN}✅ All security checks passed!${NC}"
    exit 0
else
    echo -e "${RED}❌ Found $ISSUES configuration issues${NC}"
    if [ $SECRETS_FOUND -eq 1 ]; then
        echo -e "${YELLOW}⚠️  Potential secrets detected - review manually${NC}"
    fi
    echo ""
    echo "Fix these issues before committing sensitive changes."
    exit 1
fi
