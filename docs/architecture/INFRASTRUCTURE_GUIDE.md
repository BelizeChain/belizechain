# BelizeChain Infrastructure & Deployment Guide

## 📋 Table of Contents

1. [Overview](#overview)
2. [Infrastructure Architecture](#infrastructure-architecture)
3. [Kubernetes Deployment](#kubernetes-deployment)
4. [Helm Charts](#helm-charts)
5. [CI/CD Pipelines](#cicd-pipelines)
6. [ArgoCD GitOps](#argocd-gitops)
7. [Secret Management](#secret-management)
8. [Monitoring & Observability](#monitoring--observability)
9. [Troubleshooting](#troubleshooting)
10. [Production Checklist](#production-checklist)

---

## Overview

BelizeChain infrastructure is designed for multi-environment deployment (dev/staging/production) using modern cloud-native technologies:

- **Container Orchestration**: Kubernetes 1.28+
- **Package Manager**: Helm 3.13+
- **GitOps**: ArgoCD 2.8+
- **CI/CD**: GitHub Actions
- **Secret Management**: HashiCorp Vault / Azure Key Vault
- **Monitoring**: Prometheus + Grafana
- **Service Mesh** (optional): Istio

---

## Infrastructure Architecture

### Component Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    BelizeChain Platform                      │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │ Blockchain   │  │   Nawal FL   │  │    Kinich    │      │
│  │     Node     │  │   (AI/ML)    │  │   (Quantum)  │      │
│  │ StatefulSet  │  │  Deployment  │  │  Deployment  │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
│                                                               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │    Pakit     │  │  PostgreSQL  │  │    Redis     │      │
│  │  (Storage)   │  │  StatefulSet │  │ StatefulSet  │      │
│  │  Deployment  │  │              │  │              │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
│                                                               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │     IPFS     │  │  Prometheus  │  │   Grafana    │      │
│  │ StatefulSet  │  │ StatefulSet  │  │ StatefulSet  │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

### Resource Requirements by Environment

| Component | Dev | Staging | Production |
|-----------|-----|---------|------------|
| **Blockchain Node** | 4Gi / 2 CPU | 8Gi / 4 CPU | 16Gi / 8 CPU |
| **Nawal** | 2Gi / 1 CPU | 8Gi / 4 CPU | 16Gi / 8 CPU |
| **Kinich** | 2Gi / 1 CPU | 8Gi / 4 CPU | 16Gi / 8 CPU |
| **Pakit** | 1Gi / 0.5 CPU | 4Gi / 2 CPU | 8Gi / 4 CPU |
| **PostgreSQL** | 2Gi / 1 CPU | 4Gi / 2 CPU | 16Gi / 8 CPU |
| **Redis** | 512Mi / 0.25 CPU | 2Gi / 1 CPU | 8Gi / 4 CPU |
| **IPFS** | 1Gi / 0.5 CPU | 2Gi / 1 CPU | 8Gi / 4 CPU |
| **Prometheus** | 2Gi / 1 CPU | 4Gi / 2 CPU | 16Gi / 8 CPU |
| **Grafana** | 512Mi / 0.25 CPU | 1Gi / 0.5 CPU | 4Gi / 2 CPU |
| **Total** | ~15Gi / ~9 CPU | ~45Gi / ~21 CPU | ~120Gi / ~56 CPU |

---

## Kubernetes Deployment

### Prerequisites

```bash
# Install kubectl
curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl"
chmod +x kubectl && sudo mv kubectl /usr/local/bin/

# Install Helm
curl https://raw.githubusercontent.com/helm/helm/main/scripts/get-helm-3 | bash

# Verify installations
kubectl version --client
helm version
```

### Using Kustomize (Alternative to Helm)

```bash
# Deploy base manifests
kubectl apply -k infra/k8s/base

# Deploy environment-specific overlays
kubectl apply -k infra/k8s/overlays/dev
kubectl apply -k infra/k8s/overlays/staging
kubectl apply -k infra/k8s/overlays/production

# Verify deployment
kubectl get pods -n belizechain
kubectl get svc -n belizechain
kubectl get pvc -n belizechain
```

### Base Kubernetes Files

Located in `infra/k8s/base/`:

- **namespace.yaml** - BelizeChain namespace
- **configmap.yaml** - Non-sensitive configuration
- **secret.yaml** - Sensitive credentials (template only)
- **blockchain-node.yaml** - Substrate node StatefulSet
- **postgres.yaml** - PostgreSQL database
- **redis.yaml** - Redis cache
- **ipfs.yaml** - IPFS storage
- **nawal.yaml** - Federated learning API
- **kinich.yaml** - Quantum computing API
- **pakit.yaml** - Decentralized storage API
- **monitoring.yaml** - Prometheus + Grafana

---

## Helm Charts

### Quick Start

```bash
# Development deployment
helm install belizechain ./infra/helm/belizechain \
  --namespace belizechain \
  --create-namespace \
  --values ./infra/helm/belizechain/values-dev.yaml

# Staging deployment
helm install belizechain ./infra/helm/belizechain \
  --namespace belizechain \
  --create-namespace \
  --values ./infra/helm/belizechain/values-staging.yaml

# Production deployment (with secrets from Vault)
helm install belizechain ./infra/helm/belizechain \
  --namespace belizechain \
  --create-namespace \
  --values ./infra/helm/belizechain/values-production.yaml \
  --set secrets.useExternalSecrets=true \
  --set secrets.vaultAddress=https://vault.belizechain.org
```

### Helm Chart Structure

```
infra/helm/belizechain/
├── Chart.yaml                 # Chart metadata
├── values.yaml                # Default values
├── values-dev.yaml            # Development overrides
├── values-staging.yaml        # Staging overrides
├── values-production.yaml     # Production overrides
└── templates/                 # Kubernetes manifests templates
    ├── namespace.yaml
    ├── configmap.yaml
    ├── secret.yaml
    ├── blockchain-node.yaml
    ├── nawal.yaml
    ├── kinich.yaml
    ├── pakit.yaml
    ├── postgres.yaml
    ├── redis.yaml
    ├── ipfs.yaml
    └── monitoring.yaml
```

### Customizing Deployments

Override values via command line:

```bash
helm upgrade belizechain ./infra/helm/belizechain \
  --namespace belizechain \
  --set nawal.replicaCount=5 \
  --set kinich.resources.limits.memory=16Gi \
  --set global.domain=belizechain.org
```

Or create custom values file:

```yaml
# my-values.yaml
global:
  environment: production
  domain: my-custom-domain.com

nawal:
  replicaCount: 10
  resources:
    limits:
      memory: 32Gi
      cpu: 16000m
```

```bash
helm upgrade belizechain ./infra/helm/belizechain \
  --values my-values.yaml
```

---

## CI/CD Pipelines

### GitHub Actions Workflows

Located in `.github/workflows/`:

#### 1. **blockchain.yml** - Blockchain Build Pipeline

Triggers:
- Push to `main`, `develop`, `belizechain` branches
- Pull requests modifying `belizechain/` or `Cargo.toml`

Jobs:
1. **check** - Code formatting (rustfmt) + linting (clippy)
2. **test** - Unit tests + integration tests
3. **build** - Release binary compilation
4. **docker** - Build and push Docker image

```bash
# Manual trigger (if enabled)
gh workflow run blockchain.yml
```

#### 2. **python.yml** - Python Components Pipeline

Triggers:
- Push to main branches modifying `nawal/`, `kinich/`, `pakit/`
- Pull requests

Jobs:
1. **lint** - black, isort, flake8
2. **test** - pytest with PostgreSQL + Redis services
3. **build-nawal** - Build Nawal Docker image
4. **build-kinich** - Build Kinich Docker image
5. **build-pakit** - Build Pakit Docker image

```bash
# Manual trigger
gh workflow run python.yml
```

#### 3. **deploy.yml** - Kubernetes Deployment Pipeline

Triggers:
- Push to `main` (production) or `develop` (staging)
- Manual workflow dispatch

Jobs:
1. **deploy** - Helm deployment to Kubernetes
2. **verify** - Rollout status checks
3. **smoke-test** - Health endpoint validation

```bash
# Manual deployment to specific environment
gh workflow run deploy.yml -f environment=staging
```

### Required GitHub Secrets

Configure these in GitHub repository settings:

```bash
# Docker Hub
DOCKER_USERNAME
DOCKER_PASSWORD

# Kubernetes
KUBECONFIG  # Base64-encoded kubeconfig file

# Application secrets
POSTGRES_PASSWORD
REDIS_PASSWORD
GRAFANA_ADMIN_PASSWORD
JWT_SECRET
ENCRYPTION_SALT
```

### Setting up Secrets

```bash
# Encode kubeconfig
cat ~/.kube/config | base64 -w 0

# Add to GitHub via CLI
gh secret set DOCKER_USERNAME -b "your_username"
gh secret set DOCKER_PASSWORD -b "your_password"
gh secret set KUBECONFIG -b "$(cat ~/.kube/config | base64 -w 0)"
```

---

## ArgoCD GitOps

### Installation

```bash
# Install ArgoCD
kubectl create namespace argocd
kubectl apply -n argocd -f https://raw.githubusercontent.com/argoproj/argo-cd/stable/manifests/install.yaml

# Expose ArgoCD UI
kubectl port-forward svc/argocd-server -n argocd 8080:443

# Get admin password
kubectl -n argocd get secret argocd-initial-admin-secret -o jsonpath="{.data.password}" | base64 -d
```

### Deploy BelizeChain Project

```bash
# Create ArgoCD project
kubectl apply -f infra/argocd/project.yaml

# Deploy development environment
kubectl apply -f infra/argocd/application-dev.yaml

# Deploy staging environment
kubectl apply -f infra/argocd/application-staging.yaml

# Deploy production (manual sync required)
kubectl apply -f infra/argocd/application-production.yaml
```

### ArgoCD Applications

| Application | Environment | Branch | Sync Policy |
|-------------|-------------|--------|-------------|
| `belizechain-dev` | Development | `develop` | Automated |
| `belizechain-staging` | Staging | `develop` | Automated |
| `belizechain-production` | Production | `main` | **Manual** |

### Syncing Applications

```bash
# Via CLI
argocd app sync belizechain-dev

# Via UI
# Navigate to http://localhost:8080
# Login with admin credentials
# Click on application → Sync
```

### Auto-sync Behavior

**Development/Staging**:
- Automatically syncs on git push
- Self-heals on drift detection
- Prunes deleted resources

**Production**:
- Requires manual sync approval
- No auto-heal to prevent accidental changes
- Extended revision history (20 revisions)

---

## Secret Management

### Option 1: HashiCorp Vault (Recommended)

#### Setup Vault

```bash
# Install Vault
helm repo add hashicorp https://helm.releases.hashicorp.com
helm install vault hashicorp/vault \
  --namespace vault \
  --create-namespace

# Initialize and unseal
kubectl exec -n vault vault-0 -- vault operator init
kubectl exec -n vault vault-0 -- vault operator unseal

# Enable Kubernetes auth
kubectl exec -n vault vault-0 -- vault auth enable kubernetes
kubectl exec -n vault vault-0 -- vault write auth/kubernetes/config \
    kubernetes_host="https://$KUBERNETES_PORT_443_TCP_ADDR:443"
```

#### Store Secrets

```bash
# Write secrets to Vault
vault kv put secret/belizechain/postgres \
    password="your_secure_password"

vault kv put secret/belizechain/redis \
    password="your_secure_password"

vault kv put secret/belizechain/grafana \
    admin_password="your_secure_password"

vault kv put secret/belizechain/jwt \
    secret="your_jwt_secret"
```

#### Install External Secrets Operator

```bash
helm repo add external-secrets https://charts.external-secrets.io
helm install external-secrets \
  external-secrets/external-secrets \
  --namespace external-secrets-system \
  --create-namespace
```

#### Create ExternalSecret

```yaml
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: belizechain-secrets
  namespace: belizechain
spec:
  secretStoreRef:
    name: vault-backend
    kind: SecretStore
  target:
    name: belizechain-secrets
  data:
    - secretKey: POSTGRES_PASSWORD
      remoteRef:
        key: secret/belizechain/postgres
        property: password
    - secretKey: REDIS_PASSWORD
      remoteRef:
        key: secret/belizechain/redis
        property: password
```

### Option 2: Azure Key Vault

```bash
# Install Azure Key Vault Provider
helm repo add csi-secrets-store-provider-azure \
    https://azure.github.io/secrets-store-csi-driver-provider-azure/charts
    
helm install csi-secrets-store-provider-azure/csi-secrets-store-provider-azure \
    --generate-name \
    --namespace kube-system

# Create SecretProviderClass
kubectl apply -f - <<EOF
apiVersion: secrets-store.csi.x-k8s.io/v1
kind: SecretProviderClass
metadata:
  name: azure-belizechain
spec:
  provider: azure
  parameters:
    keyvaultName: "belizechain-kv"
    objects: |
      array:
        - |
          objectName: postgres-password
          objectType: secret
        - |
          objectName: redis-password
          objectType: secret
EOF
```

---

## Monitoring & Observability

### Accessing Grafana

```bash
# Port forward Grafana
kubectl port-forward -n belizechain svc/grafana 3000:3000

# Open browser
xdg-open http://localhost:3000

# Login with admin credentials
# Username: admin
# Password: (from secrets)
```

### Pre-configured Dashboards

Located in `infra/grafana/dashboards/`:

1. **Blockchain Node Metrics** - Block production, peer count, sync status
2. **Nawal FL Metrics** - Training rounds, participant count, model accuracy
3. **Kinich Quantum Metrics** - Job queue, QPU utilization, success rate
4. **Pakit Storage Metrics** - Storage usage, IPFS pins, compression ratio
5. **System Overview** - CPU, memory, disk, network across all services

### Prometheus Queries

```promql
# Blockchain sync status
substrate_block_height{job="belizechain-node"}

# Nawal training rounds
nawal_training_rounds_total

# Kinich job queue length
kinich_job_queue_length

# Pakit storage usage
pakit_storage_bytes_total
```

---

## Troubleshooting

### Common Issues

#### 1. Pod CrashLoopBackOff

```bash
# Check pod logs
kubectl logs -n belizechain <pod-name>

# Describe pod for events
kubectl describe pod -n belizechain <pod-name>

# Common causes:
# - Missing secrets
# - Incorrect configuration
# - Resource limits too low
```

#### 2. PersistentVolumeClaim Pending

```bash
# Check PVC status
kubectl get pvc -n belizechain

# Describe PVC
kubectl describe pvc -n belizechain <pvc-name>

# Solution: Ensure StorageClass exists
kubectl get storageclass
```

#### 3. Service Unavailable

```bash
# Check service endpoints
kubectl get endpoints -n belizechain

# Test service connectivity
kubectl run -it --rm debug --image=curlimages/curl --restart=Never -- \
    curl http://nawal:8080/health
```

#### 4. ArgoCD Sync Failures

```bash
# View application status
argocd app get belizechain-dev

# Check sync logs
argocd app sync belizechain-dev --dry-run

# Force sync
argocd app sync belizechain-dev --force
```

### Debug Commands

```bash
# Get all resources in namespace
kubectl get all -n belizechain

# Check resource usage
kubectl top pods -n belizechain
kubectl top nodes

# View events
kubectl get events -n belizechain --sort-by='.lastTimestamp'

# Execute shell in pod
kubectl exec -it -n belizechain <pod-name> -- /bin/bash

# Port forward for local testing
kubectl port-forward -n belizechain svc/nawal 8080:8080
```

---

## Production Checklist

### Pre-Deployment

- [ ] All secrets stored in Vault/Key Vault (not in Git)
- [ ] Resource limits configured for all pods
- [ ] Persistent volumes sized appropriately
- [ ] Network policies enabled
- [ ] RBAC configured for least privilege
- [ ] TLS certificates configured (cert-manager)
- [ ] Ingress controller installed (nginx/traefik)
- [ ] Monitoring dashboards configured
- [ ] Alerting rules configured (PagerDuty/Slack)
- [ ] Backup strategy implemented
- [ ] Disaster recovery plan documented

### Post-Deployment

- [ ] Health checks passing for all services
- [ ] Metrics flowing to Prometheus
- [ ] Grafana dashboards displaying data
- [ ] Log aggregation configured (ELK/Loki)
- [ ] ArgoCD sync successful
- [ ] Smoke tests passed
- [ ] Load testing completed
- [ ] Security scan completed (Trivy/Snyk)
- [ ] Documentation updated
- [ ] Team trained on deployment procedures

### Scaling Checklist

- [ ] HorizontalPodAutoscaler configured
- [ ] PodDisruptionBudget set
- [ ] Resource quotas defined
- [ ] Cluster autoscaling enabled
- [ ] Multi-zone deployment for HA
- [ ] Database replication configured
- [ ] Redis clustering enabled
- [ ] IPFS cluster configured

---

## Quick Reference

### Deployment Commands

```bash
# Deploy development
helm upgrade --install belizechain ./infra/helm/belizechain \
  --namespace belizechain --create-namespace \
  --values ./infra/helm/belizechain/values-dev.yaml

# Deploy staging
helm upgrade --install belizechain ./infra/helm/belizechain \
  --namespace belizechain --create-namespace \
  --values ./infra/helm/belizechain/values-staging.yaml

# Deploy production
helm upgrade --install belizechain ./infra/helm/belizechain \
  --namespace belizechain --create-namespace \
  --values ./infra/helm/belizechain/values-production.yaml

# Rollback
helm rollback belizechain -n belizechain

# Uninstall
helm uninstall belizechain -n belizechain
```

### Useful Links

- **Kubernetes Docs**: https://kubernetes.io/docs/
- **Helm Docs**: https://helm.sh/docs/
- **ArgoCD Docs**: https://argo-cd.readthedocs.io/
- **Prometheus**: https://prometheus.io/docs/
- **Grafana**: https://grafana.com/docs/

---

**Last Updated**: October 2025  
**Version**: 1.0.0  
**Maintainer**: BelizeChain DevOps Team
