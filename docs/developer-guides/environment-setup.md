# Developer Environment Setup

**Prerequisites • Tool Installation • Multi-Repository Configuration • IDE Setup**

Complete guide to setting up your BelizeChain development environment.

---

## System Requirements

### Minimum Requirements
- **CPU:** 4 cores (x86_64 or ARM64)
- **RAM:** 16 GB
- **Storage:** 100 GB SSD
- **OS:** Ubuntu 22.04 LTS, macOS 13+, or Windows 11 (WSL2)
- **Internet:** Stable connection for blockchain sync

### Recommended for Full Development
- **CPU:** 8+ cores
- **RAM:** 32 GB
- **Storage:** 500 GB NVMe SSD
- **GPU:** NVIDIA GPU with CUDA support (for Nawal AI training)

---

## Core Dependencies

### 1. Rust Toolchain (Blockchain Development)

```bash
# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Configure for current session
source $HOME/.cargo/env

# Verify installation
rustc --version  # Expected: rustc 1.82.0 or newer
cargo --version  # Expected: cargo 1.82.0 or newer

# Install nightly toolchain (required for some features)
rustup toolchain install nightly-2024-01-01
rustup target add wasm32-unknown-unknown --toolchain nightly-2024-01-01

# Install stable toolchain (default)
rustup default stable
rustup update stable
```

**BelizeChain uses specific toolchain version:**
```bash
# The rust-toolchain.toml file pins the version
cat rust-toolchain.toml
# [toolchain]
# channel = "stable"
# components = ["rustfmt", "clippy"]
# targets = ["wasm32-unknown-unknown"]
```

### 2. Python 3.11+ (AI/Quantum Components)

```bash
# Ubuntu/Debian
sudo apt update
sudo apt install -y python3.11 python3.11-venv python3.11-dev python3-pip

# macOS (via Homebrew)
brew install python@3.11

# Verify installation
python3 --version  # Expected: Python 3.11.x or 3.12.x
```

**Create virtual environment:**
```bash
cd belizechain-belizechain/
python3 -m venv .venv
source .venv/bin/activate  # On Windows: .venv\Scripts\activate

# Upgrade pip
pip install --upgrade pip setuptools wheel

# Install dependencies
pip install -r requirements.txt  # Core dependencies
pip install -r requirements-ml.txt  # ML dependencies (PyTorch, Transformers)
```

### 3. Node.js 20+ (UI Development)

```bash
# Using nvm (recommended)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
source ~/.bashrc

nvm install 20
nvm use 20
nvm alias default 20

# Verify installation
node --version  # Expected: v20.x.x
npm --version   # Expected: 10.x.x

# Install pnpm (faster alternative to npm)
npm install -g pnpm
```

### 4. Build Tools

```bash
# Ubuntu/Debian
sudo apt install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    git \
    clang \
    curl \
    libclang-dev \
    protobuf-compiler

# macOS
xcode-select --install
brew install openssl pkg-config protobuf

# Verify clang
clang --version  # Expected: 14.0+
```

---

## Repository Setup

### Clone All Repositories

BelizeChain uses a **multi-repository architecture**:

```bash
# Create workspace directory
mkdir -p ~/belizechain-workspace
cd ~/belizechain-workspace

# 1. Main blockchain repository
git clone https://github.com/BelizeChain/belizechain.git
cd belizechain/

# 2. Nawal AI (federated learning)
git clone https://github.com/BelizeChain/nawal-ai.git nawal/

# 3. Kinich Quantum (quantum computing)
git clone https://github.com/BelizeChain/kinich-quantum.git kinich/

# 4. Pakit Storage (sovereign DAG storage)
git clone https://github.com/BelizeChain/pakit-storage.git pakit/

# 5. GEM Platform (smart contracts)
git clone https://github.com/BelizeChain/gem.git gem/

# 6. UI Portals (Maya Wallet + Blue Hole Portal)
git clone https://github.com/BelizeChain/ui.git ui/

# 7. Infrastructure (deployment configs)
git clone https://github.com/BelizeChain/infra.git infra/
```

**Alternative: Use extraction scripts** (if working from monorepo):
```bash
cd belizechain-belizechain/

# Extract each component to its own repository
./EXTRACT_NAWAL.sh    # Creates nawal/ directory
./EXTRACT_KINICH.sh   # Creates kinich/ directory
./EXTRACT_PAKIT.sh    # Creates pakit/ directory
./EXTRACT_GEM.sh      # Creates gem/ directory
./EXTRACT_UI.sh       # Creates ui/ directory
./EXTRACT_INFRA.sh    # Creates infra/ directory
```

### Configure Git

```bash
# Set your identity (required for commits)
git config --global user.name "Your Name"
git config --global user.email "your.email@example.com"

# Set default branch name
git config --global init.defaultBranch main

# Enable commit signing (recommended)
git config --global commit.gpgsign true
git config --global user.signingkey YOUR_GPG_KEY_ID

# Helpful aliases
git config --global alias.co checkout
git config --global alias.br branch
git config --global alias.ci commit
git config --global alias.st status
```

---

## Service Dependencies

### PostgreSQL (Metadata Storage)

```bash
# Ubuntu/Debian
sudo apt install -y postgresql postgresql-contrib

# Start service
sudo systemctl start postgresql
sudo systemctl enable postgresql

# Create database user
sudo -u postgres createuser -s belizechain
sudo -u postgres createdb -O belizechain belizechain_dev

# Set password
sudo -u postgres psql -c "ALTER USER belizechain WITH PASSWORD 'dev_password_change_me';"

# Test connection
psql -U belizechain -d belizechain_dev -h localhost
# \q to exit
```

**Configure connection string:**
```bash
# Add to ~/.bashrc or ~/.zshrc
export DATABASE_URL="postgresql://belizechain:dev_password_change_me@localhost/belizechain_dev"
```

### Redis (Caching)

```bash
# Ubuntu/Debian
sudo apt install -y redis-server

# Start service
sudo systemctl start redis-server
sudo systemctl enable redis-server

# Test connection
redis-cli ping  # Expected: PONG
```

### Docker (Optional, for containerized development)

```bash
# Ubuntu/Debian
curl -fsSL https://get.docker.com | sh
sudo usermod -aG docker $USER  # Add your user to docker group
newgrp docker  # Apply group changes

# Verify installation
docker --version
docker compose --version

# Start services via Docker Compose
cd belizechain-belizechain/
docker compose up -d postgres redis
```

---

## Build BelizeChain

### Full Release Build

```bash
cd belizechain-belizechain/

# Build all components (takes ~8 minutes on 8-core CPU)
cargo build --release

# Binaries created at:
# ./target/release/belizechain-node (blockchain node)
```

**Build output:**
```
   Compiling belizechain-runtime v4.0.0
   Compiling belizechain-node v4.0.0
    Finished release [optimized] target(s) in 8m 32s
```

### Development Build (Faster)

```bash
# Skip optimizations (takes ~2 minutes)
cargo build

# Binary at: ./target/debug/belizechain-node
```

### Build Individual Pallets

```bash
# Check compilation of single pallet
cargo check -p pallet-belize-economy
cargo check -p pallet-belize-identity
cargo check -p pallet-bns

# Build specific pallet
cargo build -p pallet-belize-economy --release
```

### Build WASM Runtime Only

```bash
# Build WebAssembly runtime (for runtime upgrades)
cargo build --release --features runtime-benchmarks

# WASM runtime location:
# ./target/release/wbuild/belizechain-runtime/belizechain_runtime.compact.compressed.wasm
```

---

## Verify Installation

### Run Automated Checks

```bash
# Full environment verification script
./scripts/check_build_status.sh

# Expected output:
# ✅ Rust toolchain: 1.82.0
# ✅ Python: 3.11.7
# ✅ Node.js: 20.11.0
# ✅ PostgreSQL: 14.10
# ✅ Redis: 7.0.11
# ✅ Blockchain build: SUCCESS
# ✅ Python tests: PASSED
# ✅ UI build: SUCCESS
```

### Manual Verification

```bash
# 1. Test blockchain node
./target/release/belizechain-node --version
# Expected: a BelizeChain node version string for the current stable2603-based build

# 2. Test Python environment
source .venv/bin/activate
python -c "import torch; print(f'PyTorch: {torch.__version__}')"
# Expected: PyTorch: 2.5.x

# 3. Test UI build
cd ui/maya-wallet/
npm install
npm run build
# Expected: Build completed successfully

# 4. Run quick tests
cd ../../
cargo test --workspace --lib  # Rust unit tests
pytest tests/ -v  # Python tests
```

---

## IDE Configuration

### Visual Studio Code (Recommended)

**Install extensions:**
```bash
code --install-extension rust-lang.rust-analyzer  # Rust support
code --install-extension ms-python.python  # Python support
code --install-extension dbaeumer.vscode-eslint  # TypeScript linting
code --install-extension bradlc.vscode-tailwindcss  # Tailwind CSS
code --install-extension eamodio.gitlens  # Git integration
```

**Workspace settings (.vscode/settings.json):**
```json
{
  "rust-analyzer.cargo.features": ["runtime-benchmarks"],
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.checkOnSave.extraArgs": ["--", "-W", "clippy::all"],
  
  "python.defaultInterpreterPath": "${workspaceFolder}/.venv/bin/python",
  "python.linting.enabled": true,
  "python.linting.pylintEnabled": true,
  "python.formatting.provider": "black",
  
  "editor.formatOnSave": true,
  "editor.rulers": [100, 120],
  "files.trimTrailingWhitespace": true,
  
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer",
    "editor.formatOnSave": true
  },
  "[python]": {
    "editor.defaultFormatter": "ms-python.python",
    "editor.formatOnSave": true
  },
  "[typescript]": {
    "editor.defaultFormatter": "esbenp.prettier-vscode"
  }
}
```

### JetBrains IDEs

**IntelliJ IDEA / CLion:**
- Install **Rust Plugin**
- Install **Python Plugin**
- Configure Rust toolchain: Settings → Languages & Frameworks → Rust
- Configure Python interpreter: Settings → Project → Python Interpreter → Add (.venv)

---

## Troubleshooting

### Common Issues

#### 1. Rust Compilation Errors

**Issue:** `error: linking with 'cc' failed`

**Solution:**
```bash
# Install missing linker
sudo apt install -y clang lld

# Set LLVM linker
export RUSTFLAGS="-C link-arg=-fuse-ld=lld"
```

#### 2. WASM Build Fails

**Issue:** `wasm32-unknown-unknown target not found`

**Solution:**
```bash
rustup target add wasm32-unknown-unknown
```

#### 3. Python Module Import Errors

**Issue:** `ModuleNotFoundError: No module named 'torch'`

**Solution:**
```bash
# Ensure virtual environment is activated
source .venv/bin/activate

# Reinstall dependencies
pip install --force-reinstall -r requirements-ml.txt
```

#### 4. PostgreSQL Connection Refused

**Issue:** `connection refused` when connecting to database

**Solution:**
```bash
# Check if PostgreSQL is running
sudo systemctl status postgresql

# Restart service
sudo systemctl restart postgresql

# Verify port 5432 is listening
sudo netstat -plnt | grep 5432
```

#### 5. Out of Memory During Build

**Issue:** `SIGKILL` during cargo build

**Solution:**
```bash
# Limit parallel compilation jobs
cargo build --release -j 2  # Use only 2 cores

# Or increase swap space
sudo fallocate -l 8G /swapfile
sudo chmod 600 /swapfile
sudo mkswap /swapfile
sudo swapon /swapfile
```

---

## Next Steps

After successful environment setup:

1. **Build and run local node:** [Blockchain Development Guide](./blockchain-development.md)
2. **Deploy smart contracts:** [Smart Contract Development](./smart-contract-development.md)
3. **Run integration tests:** [Testing Guide](../testing/integration-tests.md)
4. **Set up validator:** [Validator Guide](../validators/validator-setup.md)

---

## Related Documentation

- [Blockchain Development Guide](./blockchain-development.md)
- [Smart Contract Development](./smart-contract-development.md)
- [Multi-Repo Architecture](../architecture/multi-repo-overview.md)
- [Deployment Guide](../deployment/TESTNET_DEPLOYMENT.md)
- [Troubleshooting Guide](../operations/troubleshooting.md)
