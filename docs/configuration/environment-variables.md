# BelizeChain Environment Configuration Guide

## Quick Start

### 1. Create Your Environment File

```bash
# Copy the template
cp .env.template .env

# Edit with your values
nano .env  # or use your preferred editor
```

### 2. Minimum Required Configuration

At a minimum, you MUST set these security-critical variables:

```bash
# Required for security
POSTGRES_PASSWORD=your_secure_postgres_password_here
REDIS_PASSWORD=your_secure_redis_password_here
GRAFANA_ADMIN_PASSWORD=your_secure_grafana_password_here

# Required for production deployment
JWT_SECRET=minimum_32_character_random_string_here
ENCRYPTION_SALT=another_random_salt_string_here
```

### 3. Generate Secure Passwords

Use these commands to generate cryptographically secure passwords:

```bash
# PostgreSQL password (32 characters)
openssl rand -base64 32

# Redis password (32 characters)
openssl rand -base64 32

# Grafana password (16 characters)
openssl rand -base64 16

# JWT secret (64 characters)
openssl rand -base64 64

# Encryption salt (32 characters)
openssl rand -base64 32
```

## Configuration by Component

### Blockchain Node

```bash
# Network ports
P2P_PORT=30333          # Peer-to-peer communication
RPC_PORT=9933           # HTTP RPC endpoint
WS_PORT=9944            # WebSocket endpoint

# Node configuration
NODE_NAME=BelizeChain-Node-1
CHAIN=dev               # Options: dev, local, mainnet
VALIDATOR=true          # Run as validator node
LOG_LEVEL=info          # Options: error, warn, info, debug, trace
```

### Nawal Federated Learning

```bash
# API configuration
NAWAL_API_HOST=0.0.0.0
NAWAL_API_PORT=8080

# Federated learning parameters
NAWAL_MIN_PARTICIPANTS=3     # Minimum clients needed for training round
NAWAL_MAX_PARTICIPANTS=100   # Maximum allowed participants

# Integration
BLOCKCHAIN_RPC=ws://localhost:9944
IPFS_API=http://127.0.0.1:5001
```

### Kinich Quantum Computing

```bash
# API configuration
KINICH_API_HOST=0.0.0.0
KINICH_API_PORT=8888
KINICH_MAX_CONCURRENT_JOBS=10

# Quantum backend (provider-agnostic)
AZURE_QUANTUM_ENABLED=true
AZURE_QUANTUM_SUBSCRIPTION_ID=provider-account-id
AZURE_QUANTUM_RESOURCE_GROUP=provider-project-or-group
AZURE_QUANTUM_WORKSPACE=provider-workspace
AZURE_QUANTUM_LOCATION=provider-region

# Optional secondary backend
IBM_QUANTUM_ENABLED=false
IBM_QUANTUM_TOKEN=your-secondary-backend-token
```

**Getting Quantum Backend Credentials:**
1. Create an account with your selected provider
2. Create a workspace/project in the provider console
3. Copy account ID, project/group, workspace, and region values

### Pakit Decentralized Storage

```bash
# API configuration
PAKIT_API_HOST=0.0.0.0
PAKIT_API_PORT=8001
PAKIT_STORAGE_DIR=./pakit_storage

# DAG (primary sovereign backend - IPFS is legacy fallback)
IPFS_ENABLED=true
IPFS_API=http://127.0.0.1:5001
IPFS_GATEWAY_PORT=8082

# Arweave (permanent storage)
ARWEAVE_ENABLED=false
ARWEAVE_WALLET_PATH=/path/to/arweave-wallet.json
```

**Getting Arweave Wallet:**
1. Install Arweave CLI: `npm install -g arweave`
2. Generate wallet: `arweave key-create wallet.json`
3. Fund wallet with AR tokens

### Meshtastic Mesh Network

```bash
# Mesh network configuration
MESH_ENABLED=true
MESH_LORA_REGION=US915                    # ISM band (US915 for Belize/Americas)
MESH_CHANNEL_PRESET=LongFast             # LoRa preset (LongFast, LongSlow, ShortFast)
MESH_MAX_HOPS=7                           # Maximum mesh relay hops per message
MESH_GATEWAY_ENABLED=false                # Enable gateway mode (bridges mesh ↔ internet)
MESH_RELAY_MINING_ENABLED=true            # Enable relay mining rewards
MESH_EMERGENCY_SYSTEM_ENABLED=true        # Enable emergency broadcast system
MESH_VALIDATOR_RELAY_ENABLED=true         # Enable validator block header relay via mesh
MESH_SERIAL_PORT=/dev/ttyUSB0             # Serial port for Meshtastic radio (gateway nodes)
MESH_BLE_ENABLED=true                     # Enable Bluetooth LE for phone ↔ radio communication
```

### Database Configuration

```bash
# PostgreSQL
POSTGRES_DB=belizechain
POSTGRES_USER=belizechain
POSTGRES_PASSWORD=CHANGE_ME_POSTGRES_PASSWORD  # REQUIRED
POSTGRES_HOST=postgres  # Use 'localhost' for local dev
POSTGRES_PORT=5432

# Redis
REDIS_HOST=redis  # Use 'localhost' for local dev
REDIS_PORT=6379
REDIS_PASSWORD=CHANGE_ME_REDIS_PASSWORD  # REQUIRED
```

### UI Portals

Both production portals (Maya Wallet, Blue Hole Portal) use these common environment variables:

```bash
# Blockchain connectivity
NEXT_PUBLIC_MAYA_BLOCKCHAIN_RPC=http://localhost:9933
NEXT_PUBLIC_MAYA_BLOCKCHAIN_WS=ws://localhost:9944

# Service endpoints
NEXT_PUBLIC_IPFS_GATEWAY=http://localhost:8082
NEXT_PUBLIC_NAWAL_API=http://localhost:8080
NEXT_PUBLIC_KINICH_API=http://localhost:8888
NEXT_PUBLIC_PAKIT_API=http://localhost:8001

# CORS (comma-separated origins)
CORS_ORIGINS=http://localhost:3000,http://localhost:3001,http://localhost:3002
```

## Environment-Specific Configurations

### Development Environment

```bash
ENVIRONMENT=dev
DEBUG=true
LOG_LEVEL=debug
RELOAD=true           # Auto-reload on code changes
WORKERS=1             # Single worker for debugging

# Use localhost for services
POSTGRES_HOST=localhost
REDIS_HOST=localhost
BLOCKCHAIN_RPC=ws://localhost:9944
```

### Staging Environment

```bash
ENVIRONMENT=staging
DEBUG=false
LOG_LEVEL=info
RELOAD=false
WORKERS=4

# Use Docker service names
POSTGRES_HOST=postgres
REDIS_HOST=redis
BLOCKCHAIN_RPC=ws://belizechain-node:9944
```

### Production Environment

```bash
ENVIRONMENT=production
DEBUG=false
LOG_LEVEL=warn
RELOAD=false
WORKERS=8            # Scale based on CPU cores

# Production domains
POSTGRES_HOST=your-postgres-server.amazonaws.com
REDIS_HOST=your-redis-host.example.com
BLOCKCHAIN_RPC=wss://mainnet.belizechain.org:9944

# Enable all security features
GRAFANA_USERS_ALLOW_SIGN_UP=false
CORS_ORIGINS=https://wallet.belizechain.org,https://explorer.belizechain.org
```

## Docker Compose Usage

### Starting Services

```bash
# Start all services with environment file
docker-compose --env-file .env up -d

# Start specific service
docker-compose up -d belizechain-node

# View logs
docker-compose logs -f nawal

# Check service status
docker-compose ps
```

### Stopping Services

```bash
# Stop all services
docker-compose down

# Stop and remove volumes (WARNING: deletes all data)
docker-compose down -v

# Stop specific service
docker-compose stop nawal
```

## Validation & Testing

### Verify Environment Variables

```bash
# Check if required variables are set
docker-compose config

# Test database connection
docker-compose exec postgres psql -U $POSTGRES_USER -d $POSTGRES_DB

# Test Redis connection
docker-compose exec redis redis-cli -a $REDIS_PASSWORD ping
```

### Health Checks

```bash
# Blockchain node
curl http://localhost:9933 -X POST -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"system_health","params":[],"id":1}'

# Nawal API
curl http://localhost:8080/health

# Kinich API
curl http://localhost:8888/health

# Pakit API
curl http://localhost:8001/health

# IPFS
curl http://localhost:5001/api/v0/id
```

## Security Best Practices

### 1. Never Commit .env File

```bash
# Add to .gitignore (already done)
echo ".env" >> .gitignore

# Verify it's not tracked
git status
```

### 2. Rotate Secrets Regularly

```bash
# Update passwords every 90 days
# Generate new secrets
NEW_PASSWORD=$(openssl rand -base64 32)

# Update .env file
sed -i "s/POSTGRES_PASSWORD=.*/POSTGRES_PASSWORD=$NEW_PASSWORD/" .env

# Recreate services
docker-compose up -d --force-recreate postgres
```

### 3. Use Secret Management (Production)

For production deployments, use proper secret management:

- **AWS**: AWS Secrets Manager
- **External secret manager**: provider of your choice
- **GCP**: Google Secret Manager
- **Kubernetes**: Sealed Secrets or External Secrets Operator

Example with AWS Secrets Manager:

```bash
# Store secret
aws secretsmanager create-secret \
  --name belizechain/postgres-password \
  --secret-string "your-secure-password"

# Retrieve in application
aws secretsmanager get-secret-value \
  --secret-id belizechain/postgres-password \
  --query SecretString --output text
```

### 4. Environment-Specific Files

Maintain separate environment files:

```bash
.env.dev           # Development
.env.staging       # Staging
.env.production    # Production (NEVER commit)
```

Use with Docker Compose:

```bash
docker-compose --env-file .env.production up -d
```

## Troubleshooting

### Service Won't Start

```bash
# Check environment validation
docker-compose config

# View detailed logs
docker-compose logs nawal

# Verify environment variable is set
echo $POSTGRES_PASSWORD
```

### Port Conflicts

```bash
# Change ports in .env
NAWAL_API_PORT=8090  # Instead of 8080
IPFS_GATEWAY_PORT=8082  # Instead of 8080

# Restart services
docker-compose up -d
```

### Database Connection Failed

```bash
# Verify PostgreSQL is running
docker-compose ps postgres

# Check password is correct
docker-compose exec postgres psql -U $POSTGRES_USER -d $POSTGRES_DB

# Reset database (WARNING: deletes all data)
docker-compose down -v postgres
docker-compose up -d postgres
```

## Production Deployment Checklist

Before deploying to production:

- [ ] All passwords changed from defaults
- [ ] JWT_SECRET is at least 32 characters
- [ ] ENCRYPTION_SALT is unique and random
- [ ] DEBUG=false
- [ ] ENVIRONMENT=production
- [ ] GRAFANA_USERS_ALLOW_SIGN_UP=false
- [ ] CORS_ORIGINS limited to production domains
- [ ] SSL/TLS certificates configured
- [ ] Firewall rules configured
- [ ] Backup strategy implemented
- [ ] Monitoring alerts configured
- [ ] Rate limiting enabled
- [ ] .env file NOT in version control
- [ ] Secret rotation schedule established
- [ ] Quantum backend credentials configured
- [ ] IPFS/Arweave API keys secured

## Additional Resources

- **BelizeChain Documentation**: `/docs/README.md`
- **Development Guide**: `/DEVELOPMENT_GUIDE.md`
- **Docker Compose Reference**: https://docs.docker.com/compose/
- **Quantum backend docs**: use your selected provider documentation
- **IPFS**: https://docs.ipfs.tech/
- **Substrate**: https://docs.substrate.io/

## Support

For issues or questions:
- GitHub Issues: https://github.com/belizechain/belizechain/issues
- Documentation: `/docs/`
- Community Discord: [To be added]
