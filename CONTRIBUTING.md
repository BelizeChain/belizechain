# Contributing to BelizeChain

First off, thank you for considering contributing to BelizeChain! This is Belize's sovereign blockchain infrastructure, and community contributions are essential to building a system that serves all Belizeans.

## 🇧🇿 Mission & Vision

BelizeChain aims to provide:
- **Economic Sovereignty**: National digital currency (DALLA) and stablecoin (bBZD)
- **Data Sovereignty**: All Belizean data stays within national borders
- **Democratic Governance**: District-based representation with ethics oversight
- **AI-Powered Consensus**: Proof of Useful Work through federated learning
- **Legal Compliance**: Full integration with Financial Services Commission (FSC)

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Environment](#development-environment)
- [How to Contribute](#how-to-contribute)
- [Coding Standards](#coding-standards)
- [Testing Requirements](#testing-requirements)
- [Pull Request Process](#pull-request-process)
- [Architecture Guidelines](#architecture-guidelines)
- [Security & Compliance](#security--compliance)

## Code of Conduct

### Our Standards

- **Respectful Communication**: Treat all contributors with respect regardless of background
- **Constructive Feedback**: Focus on improving code, not criticizing people
- **Belizean Values**: Uphold integrity, transparency, and service to the nation
- **Data Sovereignty**: Never compromise on data protection or privacy
- **Inclusive Development**: Welcome contributions from all skill levels

### Unacceptable Behavior

- Harassment, discrimination, or offensive comments
- Compromising security or data sovereignty
- Submitting malicious code or backdoors
- Violating Belizean laws or FSC regulations

## Getting Started

### Prerequisites

**For Blockchain Development (Rust)**:
```bash
# Rust toolchain (stable)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-unknown-unknown

# Substrate dependencies (Ubuntu/Debian)
sudo apt install build-essential git clang curl libssl-dev llvm libudev-dev protobuf-compiler
```

**For AI Development (Python)**:
```bash
# Python 3.11+ with PyTorch
python3 -m venv .venv
source .venv/bin/activate  # or `.venv\Scripts\activate` on Windows
pip install -r requirements.txt
```

**For UI Development (TypeScript)**:
```bash
# Node.js 18+ and npm
nvm install 18
npm install
```

### Clone the Repository

First, fork the repository and clone it locally:

```bash
git clone https://github.com/BelizeChain/belizechain.git
cd belizechain
```

### Build the Project

```bash
# Full blockchain build (includes WASM runtime)
./scripts/build_chain.sh

# Development environment (blockchain + AI + quantum + UI)
./scripts/start_dev.sh

# Individual pallet compilation check
cargo check -p pallet-belize-economy
```

## Development Environment

### Project Structure

```
belizechain/
├── belizechain/          # Substrate blockchain runtime
│   ├── node/             # Node implementation
│   ├── pallets/          # 12 custom pallets
│   └── runtime/          # Runtime configuration
├── nawal/                # Federated AI evolution system
│   ├── orchestrator.py   # Evolution orchestrator
│   ├── genome/           # Genome operators & models
│   ├── client/           # Training clients
│   └── server/           # Aggregation server
├── ui/                   # Multi-language TypeScript interfaces
├── scripts/              # Development automation
├── docs/                 # Comprehensive documentation
└── tests/                # Integration tests
```

### Running Tests

**Python Tests**:
```bash
source .venv/bin/activate
pytest nawal/ -v --cov=nawal --cov-report=html

# Run specific test
pytest nawal/tests/test_orchestrator.py::test_evolution_cycle -v
```

**Rust Tests**:
```bash
# All pallet tests
cargo test --workspace

# Specific pallet
cargo test -p pallet-belize-economy

# With benchmarks
cargo test --features runtime-benchmarks
```

## How to Contribute

### Types of Contributions

1. **Bug Fixes**: Fix issues in existing code
2. **New Features**: Add functionality aligned with roadmap
3. **Documentation**: Improve guides, API docs, or tutorials
4. **Testing**: Add test coverage or improve test quality
5. **Performance**: Optimize blockchain or AI performance
6. **Security Audits**: Review code for vulnerabilities
7. **Translations**: Localize UI for Belizean languages

### Finding Work

- Check **Issues** tab for tasks labeled `good first issue`
- Review **TODO comments** in source code (6 security enhancements planned)
- Read **CHANGELOG.md** for planned v0.2.0 features
- Ask in **Discussions** for guidance on where to contribute

### Creating an Issue

Before creating an issue:
1. Search existing issues to avoid duplicates
2. Verify the bug is reproducible on latest `belizechain` branch
3. Include relevant logs, error messages, and environment details

**Issue Template**:
```markdown
### Description
[Clear description of the issue or feature request]

### Environment
- OS: [Ubuntu 22.04, macOS 13, Windows 11, etc.]
- Rust version: [output of `rustc --version`]
- Python version: [output of `python3 --version`]
- Branch: [belizechain, develop, etc.]

### Steps to Reproduce (for bugs)
1. [First step]
2. [Second step]
3. [And so on...]

### Expected Behavior
[What you expected to happen]

### Actual Behavior
[What actually happened]

### Logs/Screenshots
[Include relevant output]
```

## Coding Standards

### Rust (Substrate Pallets)

**Follow Substrate v42+ Best Practices**:

```rust
// ✅ CORRECT: Use BoundedVec for MaxEncodedLen compliance
pub name: BoundedVec<u8, ConstU32<64>>,

// ❌ WRONG: Vec doesn't implement MaxEncodedLen
pub name: Vec<u8>,

// ✅ CORRECT: Use expect() with descriptive messages
let value = option.expect("Value must exist after validation");

// ❌ WRONG: unwrap() provides no context on failure
let value = option.unwrap();

// ✅ CORRECT: All custom types must derive full trait set
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum MyType { ... }
```

**Weight Functions Required**:
```rust
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn my_extrinsic() -> Weight {
        Weight::from_parts(25_000_000, 0)
            .saturating_add(RocksDbWeight::get().reads(2))
            .saturating_add(RocksDbWeight::get().writes(1))
    }
}
```

**Error Handling**:
```rust
// Use ensure! for input validation
ensure!(amount > 0, Error::<T>::InvalidAmount);

// Propagate errors with ?
let account = Self::get_account(&who)?;

// Never panic in production code paths
```

### Python (Federated AI)

**Type Hints Required** (target: >90% coverage):
```python
# ✅ CORRECT: Full type annotations
async def evaluate_genome(self, genome: Genome, generation: int) -> float:
    """Evaluate genome fitness through federated training."""
    ...

# ❌ WRONG: Missing type hints
async def evaluate_genome(self, genome, generation):
    ...
```

**Async/Await Patterns**:
```python
# Use async def for I/O operations
async def train_model(self, model: nn.Module) -> dict[str, float]:
    await self.federated_client.connect()
    metrics = await self.federated_client.train(model)
    return metrics

# Use asyncio.gather() for parallel operations
results = await asyncio.gather(
    self.train_participant_1(),
    self.train_participant_2(),
    self.train_participant_3(),
)
```

**Validation with Pydantic**:
```python
from pydantic import BaseModel, Field, field_validator

class GenomeConfig(BaseModel):
    """Configuration for genome evolution."""
    population_size: int = Field(ge=2, le=100)
    mutation_rate: float = Field(ge=0.0, le=1.0)
    
    @field_validator('population_size')
    def validate_even_population(cls, v: int) -> int:
        if v % 2 != 0:
            raise ValueError('Population size must be even')
        return v
```

### TypeScript (UI)

**Strict Mode Required**:
```typescript
// tsconfig.json
{
  "compilerOptions": {
    "strict": true,
    "noImplicitAny": true,
    "strictNullChecks": true
  }
}
```

**React Component Patterns**:
```typescript
// Use functional components with hooks
export const WalletConnect: React.FC = () => {
  const [account, setAccount] = useState<string | null>(null);
  
  useEffect(() => {
    // Async logic
  }, []);
  
  return <div>...</div>;
};
```

### Formatting

**Rust**: Use `rustfmt`
```bash
cargo fmt --all
```

**Python**: Use `black` and `isort`
```bash
black nawal/
isort nawal/
```

**TypeScript**: Use `prettier`
```bash
npm run format
```

## Testing Requirements

### Coverage Standards

- **Python**: Maintain >95% test coverage for critical modules
- **Rust**: Unit tests for all public functions and extrinsics
- **Integration**: End-to-end tests for multi-pallet workflows

### Test Structure

**Python (pytest)**:
```python
@pytest.mark.asyncio
async def test_evolution_cycle():
    """Test complete evolution cycle with selection and mutation."""
    config = EvolutionConfig(population_size=10, generations=5)
    orchestrator = EvolutionOrchestrator(config)
    
    await orchestrator.initialize()
    best_genome = await orchestrator.run_evolution()
    
    assert best_genome is not None
    assert 0.0 <= best_genome.fitness <= 100.0
```

**Rust (Substrate)**:
```rust
#[test]
fn create_proposal_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        assert_ok!(Community::create_proposal(
            RuntimeOrigin::signed(ALICE),
            ProposalType::Grant,
            BOB,
            1000,
            b"Test Proposal".to_vec().try_into().unwrap(),
            b"Description".to_vec(),
        ));
        
        assert_eq!(ProposalCount::<Test>::get(), 1);
    });
}
```

### Running Tests Before PR

```bash
# Python: All tests must pass
pytest nawal/ -v --cov=nawal

# Rust: All tests must pass
cargo test --workspace

# Clippy: No warnings allowed
cargo clippy --workspace -- -D warnings

# Format: Code must be formatted
cargo fmt --all -- --check
black --check nawal/
```

## Pull Request Process

### Before Submitting

1. **Create a feature branch**: `git checkout -b feature/your-feature-name`
2. **Write tests**: Ensure new code has test coverage
3. **Run all tests**: Verify nothing broke
4. **Format code**: Apply formatting tools
5. **Update docs**: Add/update documentation as needed
6. **Update CHANGELOG**: Add entry under `[Unreleased]`

### PR Title Format

Use conventional commits:
```
feat: Add gradient verification to federated training
fix: Prevent fitness overflow in evolution orchestrator
docs: Update installation guide for Windows users
test: Add integration tests for economy pallet
refactor: Optimize genome mutation performance
```

### PR Description Template

```markdown
## Description
[Brief description of changes]

## Motivation
[Why is this change needed?]

## Changes
- [Change 1]
- [Change 2]
- [Change 3]

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests pass
- [ ] Manual testing completed

## Checklist
- [ ] Code follows project style guidelines
- [ ] All tests pass locally
- [ ] Documentation updated
- [ ] CHANGELOG.md updated
- [ ] No new compiler warnings
- [ ] Commits are signed (optional but recommended)

## Related Issues
Closes #[issue number]
```

### Review Process

1. **Automated Checks**: CI must pass (tests, linting, formatting)
2. **Code Review**: At least 1 maintainer approval required
3. **Security Review**: For sensitive changes (cryptography, consensus, etc.)
4. **Testing**: Manual testing by reviewers if applicable
5. **Merge**: Squash and merge to maintain clean history

### Review Time

- **Bug fixes**: 1-2 days
- **Small features**: 3-5 days
- **Major features**: 1-2 weeks
- **Breaking changes**: Requires architecture discussion first

## Architecture Guidelines

### Substrate Pallet Design

**Single Responsibility**: Each pallet has one clear purpose
```
✅ pallet-belize-economy: Handles currency and treasury
✅ pallet-belize-staking: Manages validator staking only
❌ pallet-everything: Does too many things
```

**Cross-Pallet Communication**:
```rust
// Use traits for loose coupling
pub trait EconomyInterface<AccountId, Balance> {
    fn transfer(from: &AccountId, to: &AccountId, amount: Balance) -> DispatchResult;
}

// Implement in dependent pallet
impl<T: Config> EconomyInterface<T::AccountId, BalanceOf<T>> for Pallet<T> {
    fn transfer(from: &AccountId, to: &AccountId, amount: Balance) -> DispatchResult {
        // Implementation
    }
}
```

### Federated AI Design

**Separation of Concerns**:
- **Orchestrator**: High-level evolution coordination
- **Operators**: Genetic algorithms (mutation, crossover)
- **Population**: Genome storage and management
- **Client**: Local training execution
- **Server**: Secure aggregation

**Async First**: All I/O operations must be async
```python
# Network calls
async def connect_to_blockchain(self) -> None: ...

# Model training
async def train_local_model(self) -> dict[str, float]: ...

# File I/O
async def save_checkpoint(self, path: Path) -> None: ...
```

## Security & Compliance

### Critical Security Rules

1. **No unwrap() in production**: Use `expect()` or `?` operator
2. **No unsafe blocks**: Unless absolutely necessary with safety proof
3. **Input validation**: All user inputs must be validated
4. **Rate limiting**: Extrinsics must have weight limits
5. **Access control**: Check permissions before state changes

### Data Sovereignty Requirements

All features must respect:
- **Geographic restrictions**: Belizean data stays in Belize
- **KYC/AML compliance**: Financial operations require identity verification
- **FSC oversight**: Regulatory reporting mechanisms in place
- **PII protection**: Personally identifiable information encrypted at rest
- **Audit trails**: All financial transactions logged immutably

### Reporting Security Issues

**DO NOT** open public issues for security vulnerabilities.

Instead:
1. Email security concerns to: [security@belizechain.org]
2. Use PGP encryption if possible (key available on website)
3. Include detailed steps to reproduce
4. Allow reasonable time for fix before public disclosure (90 days)

## Recognition

Contributors will be recognized in:
- **CHANGELOG.md**: Listed under each release
- **docs/CONTRIBUTORS.md**: Hall of fame for significant contributions
- **GitHub Insights**: Automatic contribution tracking
- **Annual Report**: Outstanding contributors highlighted

## Questions?

- **GitHub Discussions**: For general questions and ideas
- **GitHub Issues**: For bug reports and feature requests
- **Discord**: [Coming soon] Real-time chat with maintainers
- **Email**: [dev@belizechain.org] for private inquiries

---

Thank you for contributing to Belize's digital future! 🇧🇿

*"Building sovereignty through technology, one commit at a time."*
