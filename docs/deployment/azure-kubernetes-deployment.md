# Azure Kubernetes Deployment

**Cloud Infrastructure • Container Orchestration • Monitoring • Scaling**

Complete deployment guide for BelizeChain on Azure Kubernetes Service (AKS).

---

## Architecture Overview

```
┌──────────────────────────────────────────────────────┐
│ Azure Front Door (Global load balancing)            │
└─────────────┬────────────────────────────────────────┘
              │
┌─────────────▼────────────────────────────────────────┐
│ AKS Cluster (3+ nodes, multi-zone)                   │
│                                                       │
│  ┌──────────────┐  ┌──────────────┐  ┌────────────┐ │
│  │ Blockchain   │  │ Nawal AI     │  │ Kinich     │ │
│  │ Nodes (3)    │  │ Server       │  │ Quantum    │ │
│  └──────────────┘  └──────────────┘  └────────────┘ │
│                                                       │
│  ┌──────────────┐  ┌──────────────┐  ┌────────────┐ │
│  │ Pakit        │  │ UI Portals   │  │ Monitoring │ │
│  │ Storage      │  │ (Maya + BHP) │  │ Stack      │ │
│  └──────────────┘  └──────────────┘  └────────────┘ │
└───────────────────────────┬───────────────────────────┘
                            │
┌───────────────────────────▼───────────────────────────┐
│ Azure Cosmos DB (metadata) + Blob Storage (DAG data)  │
└───────────────────────────────────────────────────────┘
```

---

## Prerequisites

### 1. Azure CLI & kubectl

```bash
# Install Azure CLI
curl -sL https://aka.ms/InstallAzureCLIDeb | sudo bash

# Login to Azure
az login

# Set subscription
az account set --subscription "BelizeChain Production"

# Install kubectl
az aks install-cli

# Install Helm
curl https://raw.githubusercontent.com/helm/helm/main/scripts/get-helm-3 | bash
```

### 2. Create Resource Group

```bash
# Create resource group in US South Central (closest to Belize)
az group create \
    --name belizechain-production \
    --location southcentralus \
    --tags environment=production project=belizechain
```

---

## AKS Cluster Setup

### Create Cluster

```bash
# Create AKS cluster (3 nodes, Standard_D8s_v3 = 8 vCPU, 32 GB RAM)
az aks create \
    --resource-group belizechain-production \
    --name belizechain-aks \
    --node-count 5 \
    --node-vm-size Standard_D8s_v3 \
    --enable-addons monitoring \
    --generate-ssh-keys \
    --zones 1 2 3 \
    --network-plugin azure \
    --enable-managed-identity \
    --kubernetes-version 1.28.5

# Get credentials
az aks get-credentials \
    --resource-group belizechain-production \
    --name belizechain-aks

# Verify connection
kubectl get nodes
# NAME                                STATUS   ROLES   AGE   VERSION
# aks-nodepool1-12345678-vmss000000   Ready    agent   5m    v1.28.5
# aks-nodepool1-12345678-vmss000001   Ready    agent   5m    v1.28.5
# aks-nodepool1-12345678-vmss000002   Ready    agent   5m    v1.28.5
```

### Configure Node Pool for Validators

```bash
# Add dedicated node pool for blockchain validators (16 vCPU, 64 GB RAM)
az aks nodepool add \
    --resource-group belizechain-production \
    --cluster-name belizechain-aks \
    --name validators \
    --node-count 3 \
    --node-vm-size Standard_D16s_v3 \
    --node-taints validator=true:NoSchedule \
    --labels role=validator \
    --zones 1 2 3
```

---

## Database Setup

### Azure Cosmos DB (Metadata)

```bash
# Create Cosmos DB account
az cosmosdb create \
    --name belizechain-metadata \
    --resource-group belizechain-production \
    --kind MongoDB \
    --locations regionName=southcentralus failoverPriority=0 \
    --default-consistency-level Session \
    --enable-automatic-failover true

# Create database
az cosmosdb mongodb database create \
    --account-name belizechain-metadata \
    --resource-group belizechain-production \
    --name belizechain

# Get connection string
az cosmosdb keys list \
    --name belizechain-metadata \
    --resource-group belizechain-production \
    --type connection-strings \
    --query "connectionStrings[0].connectionString" -o tsv
```

### Azure Blob Storage (Pakit DAG)

```bash
# Create storage account
az storage account create \
    --name belizechainpakit \
    --resource-group belizechain-production \
    --location southcentralus \
    --sku Standard_LRS \
    --kind StorageV2 \
    --access-tier Hot

# Create container for DAG blocks
az storage container create \
    --name dag-blocks \
    --account-name belizechainpakit \
    --public-access off

# Get access key
az storage account keys list \
    --resource-group belizechain-production \
    --account-name belizechainpakit \
    --query "[0].value" -o tsv
```

---

## Kubernetes Configurations

### 1. Blockchain Node Deployment

**File: `infra/k8s/blockchain-node.yaml`**

```yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: belizechain-validator
  namespace: belizechain
spec:
  serviceName: validator
  replicas: 3
  selector:
    matchLabels:
      app: belizechain-node
      role: validator
  template:
    metadata:
      labels:
        app: belizechain-node
        role: validator
    spec:
      nodeSelector:
        role: validator
      tolerations:
      - key: validator
        operator: Equal
        value: "true"
        effect: NoSchedule
      containers:
      - name: belizechain
        image: ghcr.io/belizechain/belizechain-node:v4.0.0-stable2512
        ports:
        - containerPort: 9944  # WebSocket RPC
          name: ws-rpc
        - containerPort: 30333  # P2P
          name: p2p
        - containerPort: 9615  # Prometheus metrics
          name: metrics
        env:
        - name: NODE_NAME
          valueFrom:
            fieldRef:
              fieldPath: metadata.name
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: belizechain-secrets
              key: database-url
        command:
        - /usr/local/bin/belizechain-node
        - --validator
        - --name=$(NODE_NAME)
        - --base-path=/data
        - --chain=mainnet
        - --rpc-cors=all
        - --prometheus-external
        volumeMounts:
        - name: data
          mountPath: /data
        resources:
          requests:
            memory: "32Gi"
            cpu: "8"
          limits:
            memory: "64Gi"
            cpu: "16"
  volumeClaimTemplates:
  - metadata:
      name: data
    spec:
      accessModes: [ "ReadWriteOnce" ]
      storageClassName: managed-premium
      resources:
        requests:
          storage: 500Gi
```

### 2. Nawal AI Server

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: nawal-server
  namespace: belizechain
spec:
  replicas: 2
  selector:
    matchLabels:
      app: nawal
  template:
    metadata:
      labels:
        app: nawal
    spec:
      containers:
      - name: nawal
        image: ghcr.io/belizechain/nawal-ai:v1.0.0
        ports:
        - containerPort: 8080
          name: grpc
        env:
        - name: REDIS_URL
          value: "redis://redis-service:6379"
        - name: FL_PORT
          value: "8080"
        resources:
          requests:
            memory: "16Gi"
            cpu: "4"
          limits:
            memory: "32Gi"
            cpu: "8"
            nvidia.com/gpu: 1  # Require 1 GPU
```

### 3. UI Portals (Maya Wallet + Blue Hole)

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: maya-wallet
  namespace: belizechain
spec:
  replicas: 3
  selector:
    matchLabels:
      app: maya-wallet
  template:
    metadata:
      labels:
        app: maya-wallet
    spec:
      containers:
      - name: maya
        image: ghcr.io/belizechain/maya-wallet:v1.0.0
        ports:
        - containerPort: 3000
        env:
        - name: NEXT_PUBLIC_RPC_URL
          value: "wss://rpc.belizechain.org"
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "1Gi"
            cpu: "1"
---
apiVersion: v1
kind: Service
metadata:
  name: maya-wallet
  namespace: belizechain
spec:
  type: LoadBalancer
  selector:
    app: maya-wallet
  ports:
  - port: 443
    targetPort: 3000
    protocol: TCP
```

---

## Monitoring with Prometheus & Grafana

### Install Prometheus Stack

```bash
# Add Helm repo
helm repo add prometheus-community https://prometheus-community.github.io/helm-charts
helm repo update

# Install kube-prometheus-stack
helm install prometheus prometheus-community/kube-prometheus-stack \
    --namespace monitoring \
    --create-namespace \
    --set prometheus.prometheusSpec.retention=30d \
    --set prometheus.prometheusSpec.storageSpec.volumeClaimTemplate.spec.resources.requests.storage=100Gi \
    --set grafana.adminPassword=CHANGE_ME_IN_PRODUCTION

# Port-forward Grafana
kubectl port-forward -n monitoring svc/prometheus-grafana 3000:80

# Access: http://localhost:3000 (admin / CHANGE_ME_IN_PRODUCTION)
```

### Blockchain Metrics Dashboard

Import dashboard JSON from `infra/monitoring/grafana-blockchain-dashboard.json`:

**Key metrics:**
- Block production rate (10 blocks/minute target)
- Finality lag (should be <6 seconds)
- Peer count (>50 peers healthy)
- Memory usage (<80% of allocated)
- Transaction pool size (<1000 pending txs)

---

## Logging with Loki

### Install Loki Stack

```bash
helm install loki prometheus-community/loki-stack \
    --namespace monitoring \
    --set loki.persistence.enabled=true \
    --set loki.persistence.size=100Gi \
    --set promtail.enabled=true

# Configure Grafana data source
kubectl get secret -n monitoring loki -o jsonpath="{.data.loki\.yaml}" | base64 --decode
```

### Query Logs

```promql
# View validator logs
{namespace="belizechain", app="belizechain-node", role="validator"}

# Filter errors only
{namespace="belizechain"} |= "ERROR"

# Nawal federated learning logs
{app="nawal"} |= "training_completed"
```

---

## Secrets Management (Azure Key Vault)

### Create Key Vault

```bash
az keyvault create \
    --name belizechain-keyvault \
    --resource-group belizechain-production \
    --location southcentralus \
    --enable-rbac-authorization

# Store secrets
az keyvault secret set \
    --vault-name belizechain-keyvault \
    --name database-url \
    --value "postgresql://user:pass@host/db"

az keyvault secret set \
    --vault-name belizechain-keyvault \
    --name validator-session-key \
    --value "0xabc123..."  # Generated via subkey
```

### Use Secrets in Kubernetes

```bash
# Install CSI driver
helm repo add csi-secrets-store-provider-azure https://azure.github.io/secrets-store-csi-driver-provider-azure/charts
helm install csi csi-secrets-store-provider-azure/csi-secrets-store-provider-azure \
    --namespace kube-system

# Create SecretProviderClass
kubectl apply -f - <<EOF
apiVersion: secrets-store.csi.x-k8s.io/v1
kind: SecretProviderClass
metadata:
  name: belizechain-secrets
  namespace: belizechain
spec:
  provider: azure
  parameters:
    usePodIdentity: "false"
    useVMManagedIdentity: "true"
    userAssignedIdentityID: "YOUR_MANAGED_IDENTITY_CLIENT_ID"
    keyvaultName: "belizechain-keyvault"
    tenantId: "YOUR_TENANT_ID"
    objects: |
      array:
        - objectName: "database-url"
          objectType: "secret"
        - objectName: "validator-session-key"
          objectType: "secret"
EOF
```

---

## Autoscaling

### Horizontal Pod Autoscaler (HPA)

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: maya-wallet-hpa
  namespace: belizechain
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: maya-wallet
  minReplicas: 3
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
```

---

## Backup & Disaster Recovery

### Blockchain Data Backup

```bash
# Create Azure Backup Vault
az backup vault create \
    --resource-group belizechain-production \
    --name belizechain-backup \
    --location southcentralus

# Automated snapshots every 6 hours
kubectl apply -f - <<EOF
apiVersion: snapshot.storage.k8s.io/v1
kind: VolumeSnapshotClass
metadata:
  name: belizechain-snapshots
driver: disk.csi.azure.com
deletionPolicy: Retain
parameters:
  incremental: "true"
---
apiVersion: batch/v1
kind: CronJob
metadata:
  name: blockchain-snapshot
  namespace: belizechain
spec:
  schedule: "0 */6 * * *"  # Every 6 hours
  jobTemplate:
    spec:
      template:
        spec:
          containers:
          - name: snapshot
            image: bitnami/kubectl:latest
            command:
            - /bin/sh
            - -c
            - |
              kubectl create volumesnapshot validator-snapshot-$(date +%Y%m%d%H%M) \
                --from-pvc=data-belizechain-validator-0 \
                --volume-snapshot-class=belizechain-snapshots
EOF
```

---

## Related Documentation

- [Validator Setup Guide](../validators/validator-setup.md)
- [Security Hardening](../security/security-audit-results.md)
- [Monitoring Guide](../operations/monitoring.md)
- [Troubleshooting](../operations/troubleshooting.md)
