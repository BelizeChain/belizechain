#!/bin/bash
# BelizeChain Quick Security Audit
# Focuses on critical tools that work reliably

set -e

AUDIT_DIR="audit_results_$(date +%Y%m%d_%H%M%S)"
mkdir -p "$AUDIT_DIR"

echo "🔐 BelizeChain Quick Security Audit"
echo "===================================="
echo "Audit ID: $AUDIT_DIR"
echo ""

RED='\033[0;31m'
YELLOW='\033[1;33m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

# ============================================================================
# PHASE 1: CRITICAL RUST ANALYSIS
# ============================================================================

echo -e "${BLUE}Phase 1: Critical Rust Security${NC}"
echo "================================"
echo ""

echo "1.1 Cargo Audit (Dependency Vulnerabilities)..."
cargo audit --json > "$AUDIT_DIR/01_cargo_audit.json" 2>&1 || true
cargo audit 2>&1 | tee "$AUDIT_DIR/01_cargo_audit.txt"

echo ""
echo "1.2 Cargo Clippy (Code Quality)..."
cargo clippy --workspace --all-targets --message-format=json 2>&1 | tee "$AUDIT_DIR/02_clippy.json" || true
echo "Detailed warnings saved to $AUDIT_DIR/02_clippy.json"

# ============================================================================
# PHASE 2: PYTHON SECURITY
# ============================================================================

echo ""
echo -e "${BLUE}Phase 2: Python Security Analysis${NC}"
echo "=================================="
echo ""

source .venv/bin/activate

echo "2.1 Bandit (Python Security Linter)..."
bandit -r nawal/ kinich/ pakit/ -f json -o "$AUDIT_DIR/03_bandit.json" 2>&1 | tee "$AUDIT_DIR/03_bandit.txt" || true

echo ""
echo "2.2 Safety Check (Python Dependencies)..."
safety check --json --output "$AUDIT_DIR/04_safety.json" 2>&1 | tee "$AUDIT_DIR/04_safety.txt" || true

echo ""
echo "2.3 Python Test Coverage..."
pytest nawal/ kinich/ pakit/ -v \
    --cov=nawal --cov=kinich --cov=pakit \
    --cov-report=json:"$AUDIT_DIR/05_coverage.json" \
    --cov-report=term \
    2>&1 | tee "$AUDIT_DIR/05_pytest.txt" || true

# ============================================================================
# PHASE 3: DEPENDENCY ANALYSIS  
# ============================================================================

echo ""
echo -e "${BLUE}Phase 3: Dependency Analysis${NC}"
echo "============================="
echo ""

echo "3.1 Rust Dependencies..."
cargo tree --all-features > "$AUDIT_DIR/06_rust_deps.txt"

echo "3.2 Python Dependencies..."
pip list --format=json > "$AUDIT_DIR/07_python_deps.json"
pip list > "$AUDIT_DIR/07_python_deps.txt"

# ============================================================================
# PHASE 4: CODE METRICS
# ============================================================================

echo ""
echo -e "${BLUE}Phase 4: Code Metrics${NC}"
echo "===================="
echo ""

cat > "$AUDIT_DIR/08_code_metrics.txt" << EOF
BelizeChain Code Metrics
$(date)
=======================

=== Rust Code (Blockchain) ===
EOF

find belizechain/ -name "*.rs" | xargs wc -l | tail -1 >> "$AUDIT_DIR/08_code_metrics.txt"

cat >> "$AUDIT_DIR/08_code_metrics.txt" << EOF

=== Python Code (AI/Quantum/Storage) ===
EOF

find nawal/ kinich/ pakit/ -name "*.py" | xargs wc -l | tail -1 >> "$AUDIT_DIR/08_code_metrics.txt"

cat >> "$AUDIT_DIR/08_code_metrics.txt" << EOF

=== TypeScript Code (UI) ===
EOF

find ui/ -name "*.ts" -o -name "*.tsx" 2>/dev/null | xargs wc -l 2>/dev/null | tail -1 >> "$AUDIT_DIR/08_code_metrics.txt" || echo "No TS files found" >> "$AUDIT_DIR/08_code_metrics.txt"

cat "$AUDIT_DIR/08_code_metrics.txt"

# ============================================================================
# GENERATE SUMMARY
# ============================================================================

echo ""
echo -e "${BLUE}Generating Final Report...${NC}"
echo ""

python3 << 'PYTHON_SCRIPT' > "$AUDIT_DIR/AUDIT_SUMMARY.md"
import json
import os
from datetime import datetime

audit_dir = [d for d in os.listdir('.') if d.startswith('audit_results_')][-1]

print("# 🔐 BelizeChain Security Audit Summary")
print(f"**Date**: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
print(f"**Audit ID**: {audit_dir}")
print("\n---\n")

print("## 🚨 Critical Findings\n")

# Parse cargo audit
try:
    with open(f"{audit_dir}/01_cargo_audit.json") as f:
        cargo_audit = json.load(f)
        vulns = cargo_audit.get('vulnerabilities', {}).get('list', [])
        warnings = cargo_audit.get('warnings', [])
        
        print(f"### Rust Dependencies")
        print(f"- **Vulnerabilities**: {len(vulns)}")
        print(f"- **Warnings**: {len(warnings)}\n")
        
        if vulns:
            print("#### Vulnerabilities:")
            for v in vulns:
                adv = v.get('advisory', {})
                print(f"- **{adv.get('id')}**: {adv.get('title')}")
                print(f"  - Package: `{v.get('package', {}).get('name')}`")
                print(f"  - Severity: {adv.get('severity', 'Unknown').upper()}")
                print()
        
        if warnings:
            print("#### Warnings:")
            for w in warnings:
                kind = w.get('kind', 'unknown')
                if kind == 'unmaintained':
                    pkg = w.get('package', {})
                    adv = w.get('advisory', {})
                    print(f"- **UNMAINTAINED**: `{pkg.get('name')} {pkg.get('version')}`")
                    print(f"  - {adv.get('title')}")
                    print()
except Exception as e:
    print(f"Error parsing cargo audit: {e}\n")

# Parse bandit
try:
    with open(f"{audit_dir}/03_bandit.json") as f:
        bandit = json.load(f)
        results = bandit.get('results', [])
        
        print(f"### Python Security (Bandit)")
        print(f"- **Issues Found**: {len(results)}\n")
        
        by_severity = {}
        for r in results:
            sev = r.get('issue_severity', 'UNKNOWN')
            by_severity[sev] = by_severity.get(sev, 0) + 1
        
        for sev in ['HIGH', 'MEDIUM', 'LOW']:
            if sev in by_severity:
                print(f"- {sev}: {by_severity[sev]}")
        print()
        
except Exception as e:
    print(f"Error parsing bandit: {e}\n")

# Parse coverage
try:
    with open(f"{audit_dir}/05_coverage.json") as f:
        cov = json.load(f)
        total_cov = cov.get('totals', {}).get('percent_covered', 0)
        
        print(f"### Python Test Coverage")
        print(f"- **Overall Coverage**: {total_cov:.1f}%\n")
        
        if total_cov < 80:
            print("⚠️ **WARNING**: Coverage below 80% threshold\n")
        else:
            print("✅ Coverage meets 80% threshold\n")
            
except Exception as e:
    print(f"Error parsing coverage: {e}\n")

print("---\n")
print("## 📊 Summary")
print("- ✅ Automated security scans completed")
print("- ✅ Dependency vulnerabilities identified")
print("- ✅ Code quality metrics collected")
print("- ⏳ Manual review recommended")
print("- ⏳ Professional audit required before mainnet\n")

print("## 📋 Next Steps")
print("1. Review all findings in `{}`".format(audit_dir))
print("2. Fix Critical/High severity issues")
print("3. Improve test coverage where needed")
print("4. Schedule professional audit ($80K-120K)")
print("5. Set up continuous security monitoring\n")

print("---")
print("**Note**: This is an automated analysis. Professional audit required for mainnet deployment.")

PYTHON_SCRIPT

cat "$AUDIT_DIR/AUDIT_SUMMARY.md"

echo ""
echo "======================================"
echo -e "${GREEN}✅ Quick Audit Complete!${NC}"
echo "======================================"
echo ""
echo "Results: $AUDIT_DIR/"
echo ""
echo "Review files:"
echo "  - AUDIT_SUMMARY.md (this report)"
echo "  - 01_cargo_audit.txt (Rust vulnerabilities)"
echo "  - 03_bandit.txt (Python security)"
echo "  - 05_pytest.txt (test results)"
echo ""
