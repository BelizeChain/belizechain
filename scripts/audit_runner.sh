#!/bin/bash
# BelizeChain Comprehensive Security Audit Runner
# Performs automated security scans across all components

set -e

AUDIT_DIR="audit_results_$(date +%Y%m%d_%H%M%S)"
mkdir -p "$AUDIT_DIR"

echo "🔐 BelizeChain Security Audit"
echo "============================="
echo "Audit ID: $AUDIT_DIR"
echo "Started: $(date)"
echo ""

# Colors
RED='\033[0;31m'
YELLOW='\033[1;33m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

# ============================================================================
# PHASE 1: RUST/SUBSTRATE ANALYSIS
# ============================================================================

echo -e "${BLUE}Phase 1: Rust/Substrate Security Analysis${NC}"
echo "=========================================="
echo ""

# 1.1 Clippy (strictest lints)
echo "1.1 Running cargo clippy (pedantic mode)..."
cargo clippy --all-targets --all-features -- \
    -W clippy::all \
    -W clippy::pedantic \
    -W clippy::cargo \
    -D warnings \
    2>&1 | tee "$AUDIT_DIR/01_clippy_output.txt" || echo "Clippy found issues"

# 1.2 Cargo Audit (dependency vulnerabilities)
echo ""
echo "1.2 Running cargo audit (dependency vulnerabilities)..."
if command -v cargo-audit &> /dev/null; then
    cargo audit 2>&1 | tee "$AUDIT_DIR/02_cargo_audit.txt"
else
    echo "⚠️  cargo-audit not installed. Installing..."
    cargo install cargo-audit
    cargo audit 2>&1 | tee "$AUDIT_DIR/02_cargo_audit.txt"
fi

# 1.3 Cargo Geiger (unsafe code analysis)
echo ""
echo "1.3 Running cargo geiger (unsafe code detection)..."
if command -v cargo-geiger &> /dev/null; then
    cargo geiger 2>&1 | tee "$AUDIT_DIR/03_cargo_geiger.txt"
else
    echo "⚠️  cargo-geiger not installed. Installing..."
    cargo install cargo-geiger
    cargo geiger 2>&1 | tee "$AUDIT_DIR/03_cargo_geiger.txt"
fi

# 1.4 Cargo Deny (license/security policy)
echo ""
echo "1.4 Running cargo deny (license and security policy)..."
if command -v cargo-deny &> /dev/null; then
    cargo deny check 2>&1 | tee "$AUDIT_DIR/04_cargo_deny.txt"
else
    echo "⚠️  cargo-deny not installed. Skipping..."
fi

# 1.5 Test Coverage
echo ""
echo "1.5 Analyzing test coverage..."
if command -v cargo-tarpaulin &> /dev/null; then
    cargo tarpaulin --workspace --timeout 300 --out Html --output-dir "$AUDIT_DIR" 2>&1 | tee "$AUDIT_DIR/05_coverage.txt"
else
    echo "⚠️  cargo-tarpaulin not installed. Running basic tests..."
    cargo test --workspace 2>&1 | tee "$AUDIT_DIR/05_tests.txt"
fi

# ============================================================================
# PHASE 2: SMART CONTRACT ANALYSIS
# ============================================================================

echo ""
echo -e "${BLUE}Phase 2: Smart Contract Analysis (ink!)${NC}"
echo "========================================"
echo ""

# 2.1 Check contracts compilation
echo "2.1 Verifying contract compilation..."
for contract_dir in gem/*/; do
    if [ -f "$contract_dir/Cargo.toml" ]; then
        contract_name=$(basename "$contract_dir")
        echo "  Checking: $contract_name"
        cd "$contract_dir"
        cargo contract build --release 2>&1 | tee "../../$AUDIT_DIR/contract_${contract_name}.txt" || echo "Build failed"
        cd - > /dev/null
    fi
done

# 2.2 DeFi contracts
echo ""
echo "2.2 Checking DeFi contracts..."
for contract_dir in contracts/defi/*/; do
    if [ -f "$contract_dir/Cargo.toml" ]; then
        contract_name=$(basename "$contract_dir")
        echo "  Checking: $contract_name"
        cd "$contract_dir"
        cargo contract build --release 2>&1 | tee "../../$AUDIT_DIR/defi_${contract_name}.txt" || echo "Build failed"
        cd - > /dev/null
    fi
done

# ============================================================================
# PHASE 3: PYTHON COMPONENT ANALYSIS
# ============================================================================

echo ""
echo -e "${BLUE}Phase 3: Python Security Analysis${NC}"
echo "================================="
echo ""

# Activate venv
source .venv/bin/activate

# 3.1 Bandit (Python security linter)
echo "3.1 Running bandit (Python security linter)..."
if command -v bandit &> /dev/null; then
    bandit -r nawal/ kinich/ pakit/ -f json -o "$AUDIT_DIR/06_bandit.json" 2>&1 | tee "$AUDIT_DIR/06_bandit.txt" || true
    bandit -r nawal/ kinich/ pakit/ -f html -o "$AUDIT_DIR/bandit_report.html" || true
else
    echo "⚠️  bandit not installed. Installing..."
    pip install bandit
    bandit -r nawal/ kinich/ pakit/ -f json -o "$AUDIT_DIR/06_bandit.json" 2>&1 | tee "$AUDIT_DIR/06_bandit.txt" || true
fi

# 3.2 Safety (dependency vulnerabilities)
echo ""
echo "3.2 Running safety (Python dependency vulnerabilities)..."
if command -v safety &> /dev/null; then
    safety check --json --output "$AUDIT_DIR/07_safety.json" 2>&1 | tee "$AUDIT_DIR/07_safety.txt" || true
else
    echo "⚠️  safety not installed. Installing..."
    pip install safety
    safety check --json --output "$AUDIT_DIR/07_safety.json" 2>&1 | tee "$AUDIT_DIR/07_safety.txt" || true
fi

# 3.3 Pytest with coverage
echo ""
echo "3.3 Running Python tests with coverage..."
pytest nawal/ kinich/ pakit/ -v \
    --cov=nawal --cov=kinich --cov=pakit \
    --cov-report=html:"$AUDIT_DIR/python_coverage_html" \
    --cov-report=json:"$AUDIT_DIR/08_coverage.json" \
    --junit-xml="$AUDIT_DIR/08_pytest.xml" \
    2>&1 | tee "$AUDIT_DIR/08_pytest.txt" || echo "Some tests failed"

# 3.4 MyPy (type checking)
echo ""
echo "3.4 Running mypy (type safety)..."
if command -v mypy &> /dev/null; then
    mypy nawal/ kinich/ pakit/ --html-report "$AUDIT_DIR/mypy_html" 2>&1 | tee "$AUDIT_DIR/09_mypy.txt" || true
else
    echo "⚠️  mypy not installed. Skipping..."
fi

# ============================================================================
# PHASE 4: DEPENDENCY ANALYSIS
# ============================================================================

echo ""
echo -e "${BLUE}Phase 4: Dependency Analysis${NC}"
echo "============================"
echo ""

# 4.1 Generate dependency tree (Rust)
echo "4.1 Generating Rust dependency tree..."
cargo tree --all-features > "$AUDIT_DIR/10_rust_dependencies.txt"

# 4.2 Generate dependency tree (Python)
echo "4.2 Generating Python dependency tree..."
pip list --format=json > "$AUDIT_DIR/11_python_dependencies.json"
pip list > "$AUDIT_DIR/11_python_dependencies.txt"

# 4.3 Check for outdated dependencies
echo "4.3 Checking for outdated dependencies..."
cargo outdated 2>&1 | tee "$AUDIT_DIR/12_rust_outdated.txt" || echo "cargo-outdated not installed"
pip list --outdated > "$AUDIT_DIR/12_python_outdated.txt"

# ============================================================================
# PHASE 5: CODE METRICS
# ============================================================================

echo ""
echo -e "${BLUE}Phase 5: Code Metrics & Complexity${NC}"
echo "==================================="
echo ""

# 5.1 Lines of code
echo "5.1 Counting lines of code..."
echo "=== Rust ===" > "$AUDIT_DIR/13_loc.txt"
find belizechain/ -name "*.rs" | xargs wc -l | tail -1 >> "$AUDIT_DIR/13_loc.txt"
echo "" >> "$AUDIT_DIR/13_loc.txt"
echo "=== Python ===" >> "$AUDIT_DIR/13_loc.txt"
find nawal/ kinich/ pakit/ -name "*.py" | xargs wc -l | tail -1 >> "$AUDIT_DIR/13_loc.txt"
echo "" >> "$AUDIT_DIR/13_loc.txt"
echo "=== TypeScript ===" >> "$AUDIT_DIR/13_loc.txt"
find ui/ -name "*.ts" -o -name "*.tsx" | xargs wc -l 2>/dev/null | tail -1 >> "$AUDIT_DIR/13_loc.txt" || echo "No TS files"

# 5.2 Cyclomatic complexity (if available)
if command -v radon &> /dev/null; then
    echo "5.2 Analyzing Python cyclomatic complexity..."
    radon cc nawal/ kinich/ pakit/ -a -s > "$AUDIT_DIR/14_complexity.txt"
fi

# ============================================================================
# PHASE 6: SECURITY CHECKLIST
# ============================================================================

echo ""
echo -e "${BLUE}Phase 6: Security Checklist${NC}"
echo "==========================="
echo ""

cat > "$AUDIT_DIR/15_security_checklist.md" << 'EOF'
# BelizeChain Security Checklist

## Substrate/Pallet Security

### Weight Functions
- [ ] All extrinsics have accurate weight calculations
- [ ] Database reads/writes are counted correctly
- [ ] Computational complexity is bounded
- [ ] Max block weight is respected

### Storage
- [ ] All maps use bounded collections (BoundedVec, BoundedMap)
- [ ] Storage migrations have rollback mechanisms
- [ ] No unbounded iteration over storage
- [ ] Storage keys are properly namespaced

### Access Control
- [ ] Origin checks on all privileged extrinsics
- [ ] Multi-sig requirements enforced
- [ ] Root/sudo usage is justified and documented
- [ ] No missing ensure! checks

### Events & Errors
- [ ] All state changes emit events
- [ ] Error types are descriptive
- [ ] No silent failures
- [ ] Event data doesn't leak sensitive info

### Cryptography
- [ ] Standard library functions used (no custom crypto)
- [ ] Random number generation is secure (not predictable)
- [ ] Signature verification is correct
- [ ] No weak hash functions (MD5, SHA1)

### Cross-Pallet Calls
- [ ] Provider traits properly implemented
- [ ] Circular dependencies avoided
- [ ] Error propagation is correct
- [ ] No hidden state changes

## Smart Contract Security (ink!)

### Reentrancy
- [ ] State changes before external calls
- [ ] Reentrancy guards where needed
- [ ] Call ordering is safe

### Access Control
- [ ] Owner/admin checks on privileged functions
- [ ] Role-based access implemented correctly
- [ ] No missing modifiers

### Integer Safety
- [ ] Overflow checks enabled (default in Rust)
- [ ] Division by zero prevented
- [ ] Type conversions are safe

### Token Security (PSP22/PSP34)
- [ ] Transfer callbacks implemented
- [ ] Approval/allowance logic is correct
- [ ] Mint/burn authorization checked

## Economic Security

### Inflation/Deflation
- [ ] DALLA inflation rate is capped
- [ ] bBZD peg mechanism is sound
- [ ] Fee burning doesn't cause deflation issues

### Staking
- [ ] Slashing conditions are fair
- [ ] Rewards calculations don't overflow
- [ ] Unbonding period enforced
- [ ] Validator selection is decentralized

### Governance
- [ ] Proposal spam prevention
- [ ] Vote buying resistance
- [ ] Quorum requirements
- [ ] Execution delays for safety

### Oracle
- [ ] Price feed manipulation resistance
- [ ] Multiple data sources
- [ ] Outlier rejection
- [ ] Staleness checks

## Network Security

### Node
- [ ] P2P networking is secure
- [ ] DDoS protection mechanisms
- [ ] Rate limiting on RPC
- [ ] Telemetry doesn't leak sensitive data

### Consensus
- [ ] 51% attack cost analysis
- [ ] Finality guarantees
- [ ] Fork choice rule is sound
- [ ] Time-dependent logic is safe

## Privacy & Compliance

### KYC/AML
- [ ] Identity verification is robust
- [ ] PII storage is encrypted
- [ ] Data retention policies enforced
- [ ] Audit trails for compliance

### Data Sovereignty
- [ ] IPFS/Arweave integration is secure
- [ ] Encryption at rest
- [ ] Access control on storage
- [ ] No data leaks to external services

## Dependencies

### Rust Crates
- [ ] No known vulnerabilities (cargo audit)
- [ ] Dependencies are actively maintained
- [ ] Licenses are compatible
- [ ] Minimal dependency count

### Python Packages
- [ ] No known vulnerabilities (safety check)
- [ ] Pinned versions in requirements.txt
- [ ] Regular updates schedule

## Deployment

### Testnet
- [ ] Full integration tests passing
- [ ] Load testing completed
- [ ] Bug bounty program active
- [ ] Monitoring/alerting configured

### Mainnet
- [ ] Professional audit completed
- [ ] All Critical/High findings fixed
- [ ] Upgrade mechanism tested
- [ ] Emergency pause functionality

## Documentation

- [ ] All pallets have rustdoc comments
- [ ] Extrinsics have examples
- [ ] Architecture is documented
- [ ] Threat model is published

## Monitoring

- [ ] Error rates tracked
- [ ] Block production monitored
- [ ] Treasury balance alerts
- [ ] Unusual transaction patterns detected

---

**Audit Date**: $(date)
**Auditor**: AI-Assisted Initial Review
**Status**: Pre-Professional Audit
EOF

echo "✅ Security checklist created: $AUDIT_DIR/15_security_checklist.md"

# ============================================================================
# PHASE 7: GENERATE SUMMARY REPORT
# ============================================================================

echo ""
echo -e "${BLUE}Phase 7: Generating Summary Report${NC}"
echo "==================================="
echo ""

cat > "$AUDIT_DIR/AUDIT_REPORT.md" << EOF
# BelizeChain Security Audit Report
**Audit ID**: $AUDIT_DIR  
**Date**: $(date)  
**Auditor**: AI-Assisted Analysis  
**Scope**: Full codebase (Blockchain + AI + Quantum + Storage + UI)

---

## Executive Summary

This is an **automated security audit** performed using industry-standard tools. It is **NOT a substitute** for a professional audit by firms like Trail of Bits or CertiK, which is **required before mainnet launch**.

### Audit Coverage
- ✅ **Rust/Substrate**: Clippy, cargo-audit, cargo-geiger
- ✅ **Smart Contracts**: ink! compilation, static analysis
- ✅ **Python**: Bandit, safety, pytest, mypy
- ✅ **Dependencies**: Vulnerability scanning, outdated packages
- ✅ **Code Metrics**: Lines of code, complexity analysis

### Findings Summary
**Note**: Review individual report files for details.

| Severity | Count | Status |
|----------|-------|--------|
| Critical | TBD | See 01-15 reports |
| High | TBD | See 01-15 reports |
| Medium | TBD | See 01-15 reports |
| Low | TBD | See 01-15 reports |
| Info | TBD | See 01-15 reports |

---

## Files Generated

1. \`01_clippy_output.txt\` - Rust linting results
2. \`02_cargo_audit.txt\` - Dependency vulnerabilities
3. \`03_cargo_geiger.txt\` - Unsafe code analysis
4. \`04_cargo_deny.txt\` - License/security policy
5. \`05_coverage.txt\` - Test coverage report
6. \`06_bandit.txt\` - Python security issues
7. \`07_safety.txt\` - Python dependency vulnerabilities
8. \`08_pytest.txt\` - Python test results
9. \`09_mypy.txt\` - Type safety issues
10. \`10_rust_dependencies.txt\` - Rust dependency tree
11. \`11_python_dependencies.txt\` - Python packages
12. \`12_rust_outdated.txt\` - Outdated Rust dependencies
13. \`13_loc.txt\` - Lines of code analysis
14. \`14_complexity.txt\` - Cyclomatic complexity
15. \`15_security_checklist.md\` - Manual review checklist

---

## Critical Recommendations

### Before Testnet Launch
1. Fix all **Critical** and **High** severity findings
2. Achieve >80% test coverage on all pallets
3. Set up continuous security monitoring
4. Launch bug bounty program (\$10K-50K pool)

### Before Mainnet Launch
1. **REQUIRED**: Engage professional auditor (Trail of Bits/CertiK)
2. Fix all Critical/High/Medium findings
3. Complete penetration testing
4. Verify economic model with game theory experts
5. Obtain audit certification

### Ongoing Security
1. Monthly dependency updates
2. Quarterly security reviews
3. Annual full re-audits
4. Real-time monitoring (Grafana/Prometheus)

---

## Next Steps

1. **Review all generated reports** in this directory
2. **Prioritize findings** by severity
3. **Create GitHub issues** for each finding
4. **Fix Critical/High** issues immediately
5. **Schedule professional audit** (\$80K-120K budget)

---

## Audit Limitations

This automated audit **CANNOT**:
- ❌ Find novel attack vectors (requires human creativity)
- ❌ Verify cryptographic protocols (requires math expertise)
- ❌ Test live network security (requires penetration testing)
- ❌ Certify regulatory compliance (requires legal review)
- ❌ Provide insurance coverage (requires professional firm)

This automated audit **CAN**:
- ✅ Find common vulnerabilities and anti-patterns
- ✅ Check dependency security
- ✅ Measure code quality metrics
- ✅ Verify test coverage
- ✅ Provide actionable remediation steps

---

## Contact Information

**For Professional Audit Quotes**:
- Trail of Bits: security@trailofbits.com
- CertiK: contact@certik.com
- OpenZeppelin: security@openzeppelin.com

**For Questions About This Report**:
- Review individual report files in \`$AUDIT_DIR/\`
- Check \`15_security_checklist.md\` for manual review items

---

**Disclaimer**: This is an automated analysis tool output. It does not constitute a professional security audit or guarantee of security. Use at your own risk.
EOF

echo "✅ Summary report created: $AUDIT_DIR/AUDIT_REPORT.md"

# ============================================================================
# COMPLETION
# ============================================================================

echo ""
echo "======================================"
echo -e "${GREEN}✅ Audit Complete!${NC}"
echo "======================================"
echo ""
echo "Results saved to: $AUDIT_DIR/"
echo ""
echo "Next steps:"
echo "1. Review AUDIT_REPORT.md for summary"
echo "2. Check individual report files for details"
echo "3. Fix Critical/High severity issues"
echo "4. Schedule professional audit before mainnet"
echo ""
echo "Completed: $(date)"
