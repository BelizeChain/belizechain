# 🛠️ Setting Up Your Development Environment

**Complete guide to setting up your development environment for BelizeChain**

---

## 📋 What You'll Learn

By the end of this guide, you'll have:
- ✅ All required tools installed (Node.js, Rust, Docker)
- ✅ BelizeChain development environment configured
- ✅ IDE set up with helpful extensions
- ✅ Test blockchain running locally
- ✅ Example dApp built and tested

**Time Required**: 30-60 minutes  
**Skill Level**: Beginner to Intermediate  
**Prerequisites**: Basic terminal/command line knowledge

---

## 🎯 Why This Matters

A properly configured development environment:
- **Saves time**: No fighting with missing dependencies
- **Prevents errors**: Correct versions avoid compatibility issues
- **Improves productivity**: IDE extensions catch bugs early
- **Enables testing**: Local blockchain for rapid iteration
- **Matches production**: Same tools validators use

---

## 🖥️ System Requirements

### **Minimum Requirements**

| Component | Minimum | Recommended |
|-----------|---------|-------------|
| **CPU** | 2 cores | 4+ cores |
| **RAM** | 4 GB | 8+ GB |
| **Storage** | 20 GB free | 50+ GB SSD |
| **OS** | Ubuntu 20.04, macOS 11, Windows 10 | Ubuntu 22.04, macOS 13, Windows 11 |
| **Internet** | 10 Mbps | 50+ Mbps |

### **Supported Operating Systems**

✅ **Ubuntu/Debian Linux** (recommended)  
✅ **macOS** (Intel and Apple Silicon)  
✅ **Windows 10/11** (via WSL2)  
❌ **Windows native** (not recommended - use WSL2)

> **💡 Tip**: Most BelizeChain validators run Ubuntu 22.04 LTS. If you develop on Ubuntu, you'll have fewer deployment surprises.

---

## 📦 Quick Start (15 Minutes)

### **Step 1: Check Current Setup** (2 minutes)

Open a terminal and run:

```bash
# Check if Node.js is installed
node --version
# Should show: v20.x.x or higher

# Check if npm is installed
npm --version
# Should show: 10.x.x or higher

# Check if Rust is installed
rustc --version
# Should show: rustc 1.75.0 or higher

# Check if Docker is installed
docker --version
# Should show: Docker version 24.x.x or higher
```

**If any command shows "command not found"**, follow the installation steps below.

---

### **Step 2: Install Node.js** (3 minutes)

#### **Ubuntu/Debian Linux**

```bash
# Install Node.js 20 (LTS)
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt-get install -y nodejs

# Verify installation
node --version
npm --version
```

#### **macOS**

```bash
# Install Homebrew if not already installed
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install Node.js
brew install node@20

# Verify installation
node --version
npm --version
```

#### **Windows (WSL2)**

```bash
# First, install WSL2 if you haven't:
# Open PowerShell as Administrator and run:
# wsl --install

# Then inside WSL2 Ubuntu terminal:
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt-get install -y nodejs

# Verify installation
node --version
npm --version
```

---

### **Step 3: Install Rust** (5 minutes)

#### **All Operating Systems**

```bash
# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Choose option 1 (default installation)
# Press Enter when prompted

# Load Rust environment (or restart terminal)
source $HOME/.cargo/env

# Verify installation
rustc --version
cargo --version

# Install nightly toolchain (required for some Substrate features)
rustup install nightly
rustup target add wasm32-unknown-unknown --toolchain nightly
```

**Expected output**:
```
rustc 1.75.0 (82e1608df 2023-12-21)
cargo 1.75.0 (1d8b05cdd 2023-11-20)
```

---

### **Step 4: Install Docker** (5 minutes)

#### **Ubuntu/Debian Linux**

```bash
# Install Docker
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh

# Add your user to docker group (avoid sudo)
sudo usermod -aG docker $USER

# Log out and back in for group change to take effect
# Or run: newgrp docker

# Verify installation
docker --version
docker run hello-world
```

#### **macOS**

```bash
# Install Docker Desktop
brew install --cask docker

# Start Docker Desktop from Applications folder
# Wait for Docker to fully start (whale icon in menu bar)

# Verify installation
docker --version
docker run hello-world
```

#### **Windows (WSL2)**

1. Download **Docker Desktop for Windows** from docker.com
2. Install and **enable WSL2 backend** during setup
3. Start Docker Desktop
4. In WSL2 terminal, verify:

```bash
docker --version
docker run hello-world
```

---

### **Step 5: Verify Everything Works** (1 minute)

Run this verification script:

```bash
# Check all tools
echo "🔍 Checking development environment..."
echo ""

echo "Node.js: $(node --version)"
echo "npm: $(npm --version)"
echo "Rust: $(rustc --version | cut -d' ' -f1-2)"
echo "Cargo: $(cargo --version | cut -d' ' -f1-2)"
echo "Docker: $(docker --version | cut -d' ' -f1-3)"

echo ""
echo "✅ All tools installed successfully!"
```

**Expected output**:
```
🔍 Checking development environment...

Node.js: v20.11.0
npm: 10.2.4
Rust: rustc 1.75.0
Cargo: cargo 1.75.0
Docker: Docker version 24.0.7

✅ All tools installed successfully!
```

---

## 🚀 Complete Setup (45 Minutes)

### **Part 1: Clone BelizeChain Repository** (2 minutes)

```bash
# Navigate to your projects directory
cd ~/projects  # or wherever you keep code

# Clone the repository
git clone https://github.com/BelizeChain/belizechain.git
cd belizechain

# Check repository structure
ls -la
```

**You should see**:
```
belizechain/
├── Cargo.toml          # Rust workspace
├── package.json        # JavaScript/UI workspace
├── pallets/            # Blockchain pallets (Rust)
├── runtime/            # Blockchain runtime (Rust)
├── node/               # Blockchain node (Rust)
├── ui/                 # User interfaces (TypeScript)
├── scripts/            # Development scripts
├── docs/               # Documentation
└── README.md
```

---

### **Part 2: Install Blockchain Dependencies** (10 minutes)

```bash
# Navigate to blockchain directory
cd ~/projects/belizechain

# Install additional system dependencies
# Ubuntu/Debian:
sudo apt-get update
sudo apt-get install -y \
  build-essential \
  git \
  clang \
  curl \
  libssl-dev \
  llvm \
  libudev-dev \
  pkg-config \
  protobuf-compiler

# macOS (with Homebrew):
brew install openssl cmake llvm protobuf

# Install Substrate dependencies
rustup default stable
rustup update
rustup update nightly
rustup target add wasm32-unknown-unknown
rustup target add wasm32-unknown-unknown --toolchain nightly

# Verify WASM target installed
rustup target list | grep wasm32-unknown-unknown
```

**Expected output**:
```
wasm32-unknown-unknown (installed)
```

---

### **Part 3: Build BelizeChain** (20 minutes)

> **⚠️ Warning**: First build takes 15-30 minutes. Subsequent builds are much faster (2-5 minutes).

```bash
# Build the blockchain (release mode)
cargo build --release

# Or use the convenience script
./scripts/build_chain.sh
```

**What's happening?**
1. **Downloading dependencies**: ~5 minutes (150+ crates)
2. **Compiling Substrate**: ~10 minutes (large framework)
3. **Compiling pallets**: ~3 minutes (9 custom pallets)
4. **Building runtime**: ~2 minutes (WASM compilation)
5. **Linking node**: ~1 minute (final executable)

**Expected output** (final lines):
```
   Compiling belizechain-node (current workspace build) (/home/you/projects/belizechain/node)
    Finished release [optimized] target(s) in 18m 32s
✅ BelizeChain built successfully!
```

**Build artifacts location**:
```
target/release/belizechain-node  # Main blockchain node executable
target/release/wbuild/           # WASM runtime
```

---

### **Part 4: Install UI Dependencies** (5 minutes)

```bash
# Navigate to UI directory
cd ~/projects/belizechain/ui

# Install Node.js dependencies (using pnpm for monorepo)
npm install -g pnpm
pnpm install

# This installs dependencies for all UI apps:
# - maya-wallet (citizen wallet)
# - pek-business (business dashboard)
# - winik-governance (governance portal)
# - gubida-validator (validator dashboard)
# - kijka-explorer (blockchain explorer)
```

**Expected output**:
```
Progress: resolved 1247, reused 1189, downloaded 58, added 1247, done

+ @belizechain/maya-wallet 1.0.0
+ @belizechain/pek-business 1.0.0
+ @belizechain/winik-governance 1.0.0
+ @belizechain/gubida-validator 1.0.0
+ @belizechain/kijka-explorer 1.0.0

Done in 45.2s
```

---

### **Part 5: Install Python Dependencies** (3 minutes)

```bash
# Navigate to Python AI/quantum directories
cd ~/projects/belizechain

# Create Python virtual environment
python3 -m venv venv
source venv/bin/activate

# Install requirements
pip install --upgrade pip
pip install -r requirements.txt

# This installs:
# - numpy, scipy (scientific computing)
# - torch (federated learning)
# - qiskit (quantum computing)
# - fastapi, uvicorn (API servers)
# - flower (federated learning framework)
```

**Expected output**:
```
Successfully installed numpy-1.24.3 scipy-1.11.2 torch-2.1.0 
qiskit-0.45.0 fastapi-0.104.1 uvicorn-0.24.0 flower-1.5.0
```

---

### **Part 6: Start Development Environment** (5 minutes)

```bash
# Navigate to project root
cd ~/projects/belizechain

# Start full development environment
./scripts/start_dev.sh
```

**This starts**:
1. **Blockchain node** (localhost:9933 RPC, localhost:9944 WebSocket)
2. **Federated learning server** (localhost:8080)
3. **Quantum orchestrator** (localhost:8765)
4. **All UI applications**:
   - Maya Wallet (localhost:3000)
   - Pek Business (localhost:3001)
   - Winik Governance (localhost:3002)
   - Gubida Validator (localhost:3003)
   - Kijka Explorer (localhost:3004)

**Expected output**:
```
🚀 Starting BelizeChain Development Environment...

✅ Blockchain node started (PID: 12345)
   RPC:       http://localhost:9933
   WebSocket: ws://localhost:9944

✅ Federated Learning server started (PID: 12346)
   API: http://localhost:8080

✅ Quantum Orchestrator started (PID: 12347)
   API: http://localhost:8765

✅ UI applications starting...
   Maya Wallet:       http://localhost:3000
   Pek Business:      http://localhost:3001
   Winik Governance:  http://localhost:3002
   Gubida Validator:  http://localhost:3003
   Kijka Explorer:    http://localhost:3004

🎉 Development environment ready!
```

---

## 🎨 IDE Setup

### **Visual Studio Code (Recommended)**

#### **Install VS Code**

- **Ubuntu/Debian**: 
  ```bash
  sudo snap install code --classic
  ```
- **macOS**: 
  ```bash
  brew install --cask visual-studio-code
  ```
- **Windows**: Download from code.visualstudio.com

#### **Essential Extensions**

Open VS Code and install these extensions:

**Rust Development**:
```
ext install rust-lang.rust-analyzer
ext install vadimcn.vscode-lldb
ext install serayuzgur.crates
```

**JavaScript/TypeScript**:
```
ext install dbaeumer.vscode-eslint
ext install esbenp.prettier-vscode
ext install bradlc.vscode-tailwindcss
```

**Substrate/Blockchain**:
```
ext install paritytech.vscode-substrate
```

**General Productivity**:
```
ext install eamodio.gitlens
ext install streetsidesoftware.code-spell-checker
ext install wayou.vscode-todo-highlight
```

#### **Configure VS Code Settings**

Create `.vscode/settings.json` in your project:

```json
{
  "rust-analyzer.cargo.features": "all",
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.procMacro.enable": true,
  "rust-analyzer.cargo.allTargets": false,
  
  "editor.formatOnSave": true,
  "editor.defaultFormatter": "esbenp.prettier-vscode",
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  },
  
  "files.watcherExclude": {
    "**/target/**": true,
    "**/node_modules/**": true
  }
}
```

#### **Recommended Workspace Layout**

```
VS Code Layout:
├── Explorer (left sidebar)
│   ├── pallets/           # Rust blockchain code
│   ├── ui/                # TypeScript UI code
│   └── scripts/           # Development scripts
├── Editor (center)
│   ├── Split 1: Rust file
│   └── Split 2: TypeScript file
├── Terminal (bottom)
│   ├── Tab 1: Blockchain logs
│   ├── Tab 2: UI dev server
│   └── Tab 3: Commands
└── Debug (left sidebar)
```

---

### **Alternative IDEs**

#### **IntelliJ IDEA / CLion** (JetBrains)

**Pros**:
- Excellent Rust support
- Built-in debugger
- Advanced refactoring

**Cons**:
- Heavy resource usage
- Paid (though free for students)

**Install Plugins**:
- Rust
- TOML
- Substrate

#### **Vim/Neovim**

**Pros**:
- Lightweight
- Extremely fast
- Highly customizable

**Cons**:
- Steep learning curve

**Install Plugins**:
```vim
" Using vim-plug
Plug 'rust-lang/rust.vim'
Plug 'dense-analysis/ale'
Plug 'neoclide/coc.nvim'
```

---

## 🧪 Testing Your Setup

### **Test 1: Build a Pallet** (3 minutes)

```bash
# Build just the economy pallet
cd ~/projects/belizechain
cargo build -p pallet-belize-economy

# Should complete in ~30 seconds (after initial build)
```

**Expected output**:
```
   Compiling pallet-belize-economy (current workspace build)
    Finished dev [unoptimized + debuginfo] target(s) in 27.3s
```

---

### **Test 2: Run Pallet Tests** (2 minutes)

```bash
# Run tests for economy pallet
cargo test -p pallet-belize-economy

# Should pass all tests
```

**Expected output**:
```
running 15 tests
test tests::create_account_works ... ok
test tests::transfer_works ... ok
test tests::multi_sig_approval_works ... ok
...
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured
```

---

### **Test 3: Start Local Blockchain** (2 minutes)

```bash
# Start development blockchain
./target/release/belizechain-node --dev --tmp

# Or use the script
./scripts/start_blockchain.sh
```

**Expected output**:
```
2025-10-14 10:30:00  BelizeChain Node
2025-10-14 10:30:00  ✌️  version 1.0.0-dev
2025-10-14 10:30:00  ❤️  by BelizeChain Team
2025-10-14 10:30:00  📋 Chain specification: Development
2025-10-14 10:30:00  🏷  Node name: grateful-rain-1234
2025-10-14 10:30:00  👤 Role: AUTHORITY
2025-10-14 10:30:00  💾 Database: RocksDb at /tmp/substrate...
2025-10-14 10:30:00  ⛓  Native runtime: belizechain-1
2025-10-14 10:30:01  🔨 Initializing Genesis block/state
2025-10-14 10:30:01  👶 Creating empty BABE epoch changes
2025-10-14 10:30:01  🏷  Local node identity is: 12D3KooW...
2025-10-14 10:30:01  💤 Idle (0 peers), best: #0
2025-10-14 10:30:06  ✨ Imported #1 (0x1a2b…3c4d)
2025-10-14 10:30:12  ✨ Imported #2 (0x2b3c…4d5e)
```

**Test blockchain is working**:
```bash
# In another terminal, check node is responding
curl -H "Content-Type: application/json" \
  -d '{"id":1, "jsonrpc":"2.0", "method": "system_health"}' \
  http://localhost:9933
```

**Expected response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "peers": 0,
    "isSyncing": false,
    "shouldHavePeers": false
  },
  "id": 1
}
```

✅ **Blockchain is running!**

Stop the node with `Ctrl+C`.

---

### **Test 4: Build UI Application** (3 minutes)

```bash
# Navigate to wallet UI
cd ~/projects/belizechain/ui/maya-wallet

# Start development server
pnpm dev
```

**Expected output**:
```
> @belizechain/maya-wallet@1.0.0 dev
> next dev

  ▲ Next.js 14.0.3
  - Local:        http://localhost:3000
  - Network:      http://192.168.1.100:3000

 ✓ Ready in 2.1s
```

**Open browser**: Navigate to http://localhost:3000

**You should see**: Maya Wallet welcome screen with "Create Wallet" button.

✅ **UI is working!**

Stop the server with `Ctrl+C`.

---

### **Test 5: Run Python Scripts** (2 minutes)

```bash
# Activate virtual environment
cd ~/projects/belizechain
source venv/bin/activate

# Test federated learning client
python nawal/client/train.py --help
```

**Expected output**:
```
usage: train.py [-h] [--server SERVER] [--dataset DATASET]

BelizeChain Federated Learning Client

options:
  -h, --help         show this help message
  --server SERVER    FL server address (default: localhost:8080)
  --dataset DATASET  Path to local dataset
```

✅ **Python environment is working!**

---

## 🛠️ Development Workflow

### **Daily Workflow** (Typical Day)

```bash
# 1. Start development environment (morning)
cd ~/projects/belizechain
./scripts/start_dev.sh

# 2. Make changes to code (throughout day)
#    - Edit Rust pallets in pallets/
#    - Edit TypeScript UI in ui/
#    - Test changes continuously

# 3. Build and test (after changes)
cargo build -p pallet-belize-economy  # Build single pallet
cargo test -p pallet-belize-economy   # Test single pallet
cd ui/maya-wallet && pnpm build       # Build UI

# 4. Commit changes (end of day)
git add .
git commit -m "feat: add new feature"
git push

# 5. Stop development environment (end of day)
./scripts/stop_dev.sh
```

---

### **Hot Reload Setup**

**Rust** (requires manual reload):
```bash
# Install cargo-watch for auto-rebuild
cargo install cargo-watch

# Auto-rebuild on file changes
cargo watch -x 'build -p pallet-belize-economy'
```

**TypeScript** (automatic hot reload):
```bash
# Next.js automatically reloads on file changes
cd ui/maya-wallet
pnpm dev
# Edit files in src/ - browser auto-refreshes!
```

---

### **Debugging Setup**

#### **Rust Debugging (VS Code)**

Create `.vscode/launch.json`:

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug BelizeChain Node",
      "cargo": {
        "args": [
          "build",
          "--bin=belizechain-node",
          "--package=belizechain-node"
        ],
        "filter": {
          "name": "belizechain-node",
          "kind": "bin"
        }
      },
      "args": ["--dev", "--tmp"],
      "cwd": "${workspaceFolder}"
    },
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug Pallet Tests",
      "cargo": {
        "args": [
          "test",
          "--no-run",
          "--package=pallet-belize-economy"
        ]
      },
      "args": [],
      "cwd": "${workspaceFolder}"
    }
  ]
}
```

**Usage**:
1. Set breakpoint in Rust code (click left of line number)
2. Press `F5` to start debugging
3. Code pauses at breakpoint
4. Inspect variables, step through code

---

#### **TypeScript Debugging (VS Code)**

Built-in with Next.js dev server:

1. Start dev server: `pnpm dev`
2. Open browser DevTools (`F12`)
3. Set breakpoints in Sources tab
4. Inspect variables in console

**Or use VS Code debugger**:

Add to `.vscode/launch.json`:

```json
{
  "type": "node",
  "request": "launch",
  "name": "Debug Next.js",
  "runtimeExecutable": "pnpm",
  "runtimeArgs": ["dev"],
  "cwd": "${workspaceFolder}/ui/maya-wallet",
  "port": 9229,
  "serverReadyAction": {
    "pattern": "started server on .+, url: (https?://.+)",
    "uriFormat": "%s",
    "action": "debugWithChrome"
  }
}
```

---

## 🔧 Useful Development Scripts

### **Build Scripts**

```bash
# Build everything (blockchain + UI)
./scripts/build_all.sh

# Build only blockchain
./scripts/build_chain.sh

# Build only UI applications
cd ui && pnpm build

# Build specific pallet
cargo build -p pallet-belize-economy

# Build with optimizations (slower, smaller binary)
cargo build --release

# Build WASM runtime only
cargo build -p belizechain-runtime --target wasm32-unknown-unknown
```

---

### **Test Scripts**

```bash
# Run all tests (blockchain + UI)
./scripts/test_all.sh

# Run blockchain tests only
cargo test --workspace

# Run specific pallet tests
cargo test -p pallet-belize-economy

# Run UI tests
cd ui && pnpm test

# Run tests with output
cargo test -- --nocapture

# Run specific test by name
cargo test -p pallet-belize-economy create_account_works
```

---

### **Lint and Format Scripts**

```bash
# Lint Rust code
cargo clippy --workspace -- -D warnings

# Fix Rust linting issues automatically
cargo clippy --fix --workspace

# Format Rust code
cargo fmt --all

# Check Rust formatting (CI)
cargo fmt --all -- --check

# Lint TypeScript code
cd ui && pnpm lint

# Fix TypeScript linting issues
cd ui && pnpm lint:fix

# Format TypeScript code
cd ui && pnpm format
```

---

### **Clean Scripts**

```bash
# Clean Rust build artifacts
cargo clean

# Clean Node.js dependencies
cd ui && rm -rf node_modules
cd ui && pnpm install

# Clean everything (start fresh)
./scripts/clean_all.sh

# Clear blockchain data
rm -rf /tmp/substrate*
```

---

## 📊 Performance Optimization

### **Speed Up Rust Compilation**

#### **Method 1: Use `sccache` (Shared Compilation Cache)**

```bash
# Install sccache
cargo install sccache

# Configure Cargo to use sccache
export RUSTC_WRAPPER=sccache

# Add to ~/.bashrc or ~/.zshrc for persistence
echo 'export RUSTC_WRAPPER=sccache' >> ~/.bashrc
```

**Result**: 30-50% faster recompilation after first build.

---

#### **Method 2: Use `mold` Linker (Linux only)**

```bash
# Install mold (faster linker)
sudo apt install mold  # Ubuntu 22.04+

# Configure Cargo to use mold
mkdir -p ~/.cargo
cat >> ~/.cargo/config.toml << EOF
[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=mold"]
EOF
```

**Result**: 50-70% faster linking (last build step).

---

#### **Method 3: Increase Parallel Jobs**

```bash
# Build with more parallel jobs (default: # of CPUs)
cargo build -j 8

# Or set in ~/.cargo/config.toml
cat >> ~/.cargo/config.toml << EOF
[build]
jobs = 8
EOF
```

**Result**: Better CPU utilization on multi-core systems.

---

### **Speed Up Node.js Build**

```bash
# Use pnpm instead of npm (already configured)
# pnpm is 2-3x faster and uses less disk space

# Enable caching in Next.js (automatic)
# Build cache stored in ui/*/. next/cache

# Use Turbo for monorepo builds (already configured)
cd ui && pnpm turbo build
# Turbo caches and parallelizes builds
```

---

## 🐛 Troubleshooting

### **Issue 1: "cargo: command not found"**

**Problem**: Rust not installed or not in PATH.

**Solution**:
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add to PATH (or restart terminal)
source $HOME/.cargo/env

# Verify
cargo --version
```

---

### **Issue 2: "wasm32-unknown-unknown target not installed"**

**Problem**: WASM compilation target missing.

**Solution**:
```bash
# Install WASM target
rustup target add wasm32-unknown-unknown
rustup target add wasm32-unknown-unknown --toolchain nightly

# Verify
rustup target list | grep wasm32
```

---

### **Issue 3: "error: linking with `cc` failed"**

**Problem**: Missing C compiler or linker.

**Solution**:
```bash
# Ubuntu/Debian
sudo apt-get install build-essential clang

# macOS
xcode-select --install

# Verify
cc --version
clang --version
```

---

### **Issue 4: "Could not find `protoc` installation"**

**Problem**: Protocol Buffers compiler not installed.

**Solution**:
```bash
# Ubuntu/Debian
sudo apt-get install protobuf-compiler

# macOS
brew install protobuf

# Verify
protoc --version
```

---

### **Issue 5: Compilation runs out of memory**

**Problem**: Not enough RAM for parallel compilation.

**Solution**:
```bash
# Reduce parallel jobs
cargo build -j 2

# Or use release mode (less memory, slower)
cargo build --release
```

---

### **Issue 6: "Address already in use" (port conflicts)**

**Problem**: Another process using port 9944 (blockchain) or 3000 (UI).

**Solution**:
```bash
# Find process using port
lsof -i :9944
# or
netstat -tulpn | grep 9944

# Kill process
kill -9 <PID>

# Or use different ports
./target/release/belizechain-node --dev --tmp --ws-port 9945 --rpc-port 9934
```

---

### **Issue 7: Node.js version mismatch**

**Problem**: Project requires Node.js 20, but you have 18 or 19.

**Solution**:
```bash
# Install Node Version Manager (nvm)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash

# Install Node.js 20
nvm install 20
nvm use 20
nvm alias default 20

# Verify
node --version  # Should show v20.x.x
```

---

### **Issue 8: Docker permission denied**

**Problem**: User not in `docker` group.

**Solution**:
```bash
# Add user to docker group
sudo usermod -aG docker $USER

# Log out and back in (or run)
newgrp docker

# Verify
docker run hello-world
```

---

## 📚 Additional Resources

### **Documentation**

- **BelizeChain Docs**: `docs/` directory in repository
- **Substrate Docs**: https://docs.substrate.io
- **Rust Book**: https://doc.rust-lang.org/book/
- **Next.js Docs**: https://nextjs.org/docs

### **Community**

- **BelizeChain Discord**: https://discord.gg/belizechain
- **Forum**: https://forum.belizechain.org
- **Stack Overflow**: Tag `belizechain`
- **GitHub Discussions**: https://github.com/BelizeChain/belizechain/discussions

### **Video Tutorials**

- **Environment Setup**: https://youtube.com/belizechain/setup
- **First Pallet**: https://youtube.com/belizechain/first-pallet
- **Building dApps**: https://youtube.com/belizechain/dapp-tutorial

---

## ✅ Success Checklist

Before proceeding to building your first dApp, verify:

- [ ] Node.js 20+ installed (`node --version`)
- [ ] Rust 1.75+ installed (`rustc --version`)
- [ ] Docker 24+ installed (`docker --version`)
- [ ] BelizeChain repository cloned
- [ ] Blockchain builds successfully (`cargo build --release`)
- [ ] UI dependencies installed (`cd ui && pnpm install`)
- [ ] Python environment set up (`source venv/bin/activate`)
- [ ] Local blockchain starts (`./scripts/start_blockchain.sh`)
- [ ] UI dev server starts (`cd ui/maya-wallet && pnpm dev`)
- [ ] All tests pass (`cargo test --workspace`)
- [ ] IDE configured with extensions
- [ ] Development workflow understood

**🎉 Congratulations!** You're ready to build on BelizeChain!

---

## 🚀 What's Next?

Now that your environment is set up, you can:

1. **[Build Your First dApp →](build-dapp.md)**  
   Create a complete decentralized application from scratch

2. **[Develop Custom Pallets →](smart-pallets.md)**  
   Build blockchain business logic in Rust

3. **[API Reference →](api-reference.md)**  
   Explore all available RPC methods and APIs

4. **[Testing Guide →](testing.md)**  
   Write comprehensive tests for your code

---

**Questions?** Join our [Discord](https://discord.gg/belizechain) or check [GitHub Discussions](https://github.com/BelizeChain/belizechain/discussions).

**Happy coding!** 💻🇧🇿
