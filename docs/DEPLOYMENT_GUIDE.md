# Deployment Guide

This guide covers the deployment of the Cross-Chain Bridge in various environments.

## Prerequisites

### System Requirements

- **CPU**: 4+ cores
- **RAM**: 8GB+ 
- **Storage**: 100GB+ SSD
- **Network**: Stable internet connection with low latency
- **OS**: Ubuntu 20.04+ or similar Linux distribution

### Software Dependencies

- **Rust**: 1.70+
- **Node.js**: 18+
- **PostgreSQL**: 13+
- **Docker**: 20.10+ (optional)
- **Foundry**: Latest (for Ethereum contracts)

## Environment Setup

### 1. Development Environment

```bash
# Clone repository
git clone <repository-url>
cd cross-chain-bridge

# Install Rust dependencies
cargo build

# Install Node.js dependencies (for contracts)
cd contracts/ethereum
npm install
cd ../..

# Set up database
createdb bridge_dev
export DATABASE_URL="postgresql://user:password@localhost:5432/bridge_dev"

# Run migrations
cargo run --bin relayer -- migrate

# Start development services
cargo run --bin relayer &
cargo run --bin api-server &
```

### 2. Testnet Environment

```bash
# Set environment variables
export ETHEREUM_RPC_URL="https://goerli.infura.io/v3/YOUR_KEY"
export POLKADOT_WS_URL="wss://westend-rpc.polkadot.io"
export DATABASE_URL="postgresql://user:password@localhost:5432/bridge_testnet"

# Deploy contracts to testnet
cd contracts/ethereum
forge script script/Deploy.s.sol --rpc-url goerli --broadcast --verify

# Start services
cargo run --release --bin relayer
cargo run --release --bin api-server
```

### 3. Production Environment

See detailed production deployment section below.

## Smart Contract Deployment

### Ethereum Contracts

1. **Prepare deployment configuration**
   ```bash
   cd contracts/ethereum
   cp .env.example .env
   # Edit .env with your configuration
   ```

2. **Deploy to testnet**
   ```bash
   # Deploy to Goerli
   forge script script/Deploy.s.sol \
     --rpc-url goerli \
     --broadcast \
     --verify \
     --etherscan-api-key $ETHERSCAN_API_KEY
   ```

3. **Deploy to mainnet**
   ```bash
   # Deploy to Ethereum mainnet
   forge script script/Deploy.s.sol \
     --rpc-url mainnet \
     --broadcast \
     --verify \
     --etherscan-api-key $ETHERSCAN_API_KEY
   ```

### Polkadot Pallet

1. **Integration with runtime**
   ```rust
   // In runtime/src/lib.rs
   impl pallet_cross_chain_bridge::Config for Runtime {
       type RuntimeEvent = RuntimeEvent;
       type WeightInfo = ();
       type PalletId = BridgePalletId;
       type MaxValidators = MaxValidators;
       type MaxSignatureLength = MaxSignatureLength;
   }
   
   construct_runtime!(
       pub enum Runtime where
           Block = Block,
           NodeBlock = opaque::Block,
           UncheckedExtrinsic = UncheckedExtrinsic
       {
           // ... other pallets
           Bridge: pallet_cross_chain_bridge,
       }
   );
   ```

2. **Build and deploy runtime**
   ```bash
   cargo build --release
   # Deploy runtime upgrade through governance
   ```

## Production Deployment

### Infrastructure Setup

1. **Server Provisioning**
   ```bash
   # Example with AWS EC2
   aws ec2 run-instances \
     --image-id ami-0c02fb55956c7d316 \
     --instance-type t3.large \
     --key-name bridge-keypair \
     --security-group-ids sg-12345678 \
     --subnet-id subnet-12345678
   ```

2. **Database Setup**
   ```bash
   # Set up PostgreSQL cluster
   sudo apt update
   sudo apt install postgresql-13 postgresql-contrib
   
   # Create database and user
   sudo -u postgres createuser bridge_user
   sudo -u postgres createdb bridge_prod
   sudo -u postgres psql -c "ALTER USER bridge_user WITH PASSWORD 'secure_password';"
   sudo -u postgres psql -c "GRANT ALL PRIVILEGES ON DATABASE bridge_prod TO bridge_user;"
   ```

3. **Security Configuration**
   ```bash
   # Set up firewall
   sudo ufw enable
   sudo ufw allow 22/tcp    # SSH
   sudo ufw allow 3001/tcp  # API server
   sudo ufw allow 9001/tcp  # Metrics
   
   # Set up SSL certificates
   sudo certbot --nginx -d bridge-api.yourdomain.com
   ```

### Service Deployment

1. **Build release binaries**
   ```bash
   cargo build --release
   
   # Copy binaries to production server
   scp target/release/relayer user@server:/opt/bridge/
   scp target/release/api-server user@server:/opt/bridge/
   ```

2. **Create systemd services**
   ```ini
   # /etc/systemd/system/bridge-relayer.service
   [Unit]
   Description=Cross-Chain Bridge Relayer
   After=network.target postgresql.service
   
   [Service]
   Type=simple
   User=bridge
   WorkingDirectory=/opt/bridge
   ExecStart=/opt/bridge/relayer
   Restart=always
   RestartSec=10
   Environment=DATABASE_URL=postgresql://bridge_user:password@localhost:5432/bridge_prod
   Environment=LOG_LEVEL=info
   
   [Install]
   WantedBy=multi-user.target
   ```

   ```ini
   # /etc/systemd/system/bridge-api.service
   [Unit]
   Description=Cross-Chain Bridge API
   After=network.target bridge-relayer.service
   
   [Service]
   Type=simple
   User=bridge
   WorkingDirectory=/opt/bridge
   ExecStart=/opt/bridge/api-server
   Restart=always
   RestartSec=10
   Environment=API_HOST=0.0.0.0
   Environment=API_PORT=3001
   
   [Install]
   WantedBy=multi-user.target
   ```

3. **Start services**
   ```bash
   sudo systemctl daemon-reload
   sudo systemctl enable bridge-relayer bridge-api
   sudo systemctl start bridge-relayer bridge-api
   ```

### Monitoring Setup

1. **Prometheus configuration**
   ```yaml
   # prometheus.yml
   global:
     scrape_interval: 15s
   
   scrape_configs:
     - job_name: 'bridge'
       static_configs:
         - targets: ['localhost:9001']
   ```

2. **Grafana dashboard**
   ```json
   {
     "dashboard": {
       "title": "Cross-Chain Bridge",
       "panels": [
         {
           "title": "Processed Transactions",
           "type": "graph",
           "targets": [
             {
               "expr": "bridge_processed_transactions_total"
             }
           ]
         }
       ]
     }
   }
   ```

3. **Alerting rules**
   ```yaml
   # alerts.yml
   groups:
     - name: bridge
       rules:
         - alert: BridgeDown
           expr: up{job="bridge"} == 0
           for: 1m
           annotations:
             summary: "Bridge service is down"
   ```

## Docker Deployment

### Docker Compose

```yaml
# docker-compose.yml
version: '3.8'

services:
  postgres:
    image: postgres:13
    environment:
      POSTGRES_DB: bridge
      POSTGRES_USER: bridge_user
      POSTGRES_PASSWORD: bridge_pass
    volumes:
      - postgres_data:/var/lib/postgresql/data
    ports:
      - "5432:5432"

  relayer:
    build: .
    command: ["./target/release/relayer"]
    environment:
      DATABASE_URL: postgresql://bridge_user:bridge_pass@postgres:5432/bridge
      ETHEREUM_RPC_URL: ${ETHEREUM_RPC_URL}
      POLKADOT_WS_URL: ${POLKADOT_WS_URL}
    depends_on:
      - postgres
    restart: unless-stopped

  api:
    build: .
    command: ["./target/release/api-server"]
    environment:
      DATABASE_URL: postgresql://bridge_user:bridge_pass@postgres:5432/bridge
    ports:
      - "3001:3001"
    depends_on:
      - relayer
    restart: unless-stopped

  prometheus:
    image: prom/prometheus
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml

  grafana:
    image: grafana/grafana
    ports:
      - "3000:3000"
    environment:
      GF_SECURITY_ADMIN_PASSWORD: admin

volumes:
  postgres_data:
```

### Kubernetes Deployment

```yaml
# k8s/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: bridge-relayer
spec:
  replicas: 3
  selector:
    matchLabels:
      app: bridge-relayer
  template:
    metadata:
      labels:
        app: bridge-relayer
    spec:
      containers:
      - name: relayer
        image: bridge:latest
        command: ["./relayer"]
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: bridge-secrets
              key: database-url
        resources:
          requests:
            memory: "1Gi"
            cpu: "500m"
          limits:
            memory: "2Gi"
            cpu: "1000m"
```

## Configuration Management

### Environment Variables

```bash
# Production environment variables
export ETHEREUM_RPC_URL="https://mainnet.infura.io/v3/YOUR_KEY"
export ETHEREUM_WS_URL="wss://mainnet.infura.io/ws/v3/YOUR_KEY"
export ETHEREUM_CHAIN_ID=1
export ETHEREUM_BRIDGE_CONTRACT="0x..."
export ETHEREUM_PRIVATE_KEY="0x..."

export POLKADOT_WS_URL="wss://rpc.polkadot.io"
export POLKADOT_ACCOUNT_SEED="//YourProductionSeed"

export DATABASE_URL="postgresql://user:pass@localhost:5432/bridge_prod"

export VALIDATOR_ID="validator_prod_1"
export VALIDATOR_ENABLED=true
export VALIDATOR_PRIVATE_KEY="0x..."

export LOG_LEVEL=info
export METRICS_PORT=9001
```

### Configuration File

```toml
# config/production.toml
[ethereum]
rpc_url = "https://mainnet.infura.io/v3/YOUR_KEY"
ws_url = "wss://mainnet.infura.io/ws/v3/YOUR_KEY"
chain_id = 1
bridge_contract = "0x..."
confirmations = 12

[polkadot]
ws_url = "wss://rpc.polkadot.io"
confirmations = 6

[threshold]
scheme = "ecdsa"
threshold = 2
total_validators = 3

[database]
url = "postgresql://user:pass@localhost:5432/bridge_prod"
max_connections = 20

[monitoring]
poll_interval = 5
metrics_port = 9001
log_level = "info"
```

## Health Checks and Monitoring

### Health Check Endpoints

```bash
# Check relayer health
curl http://localhost:3001/health

# Check bridge status
curl http://localhost:3001/status

# Check metrics
curl http://localhost:9001/metrics
```

### Log Monitoring

```bash
# View relayer logs
journalctl -u bridge-relayer -f

# View API logs
journalctl -u bridge-api -f

# Search for errors
journalctl -u bridge-relayer | grep ERROR
```

## Backup and Recovery

### Database Backup

```bash
# Create backup
pg_dump bridge_prod > bridge_backup_$(date +%Y%m%d).sql

# Restore backup
psql bridge_prod < bridge_backup_20240101.sql
```

### Configuration Backup

```bash
# Backup configuration
tar -czf bridge_config_backup.tar.gz /opt/bridge/config/

# Backup keys (secure storage)
gpg --encrypt --recipient admin@bridge.com validator_keys.json
```

## Troubleshooting

### Common Issues

1. **Database Connection Issues**
   ```bash
   # Check database status
   sudo systemctl status postgresql
   
   # Test connection
   psql -h localhost -U bridge_user -d bridge_prod
   ```

2. **Network Connectivity**
   ```bash
   # Test Ethereum connection
   curl -X POST -H "Content-Type: application/json" \
     --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
     $ETHEREUM_RPC_URL
   
   # Test Polkadot connection
   wscat -c $POLKADOT_WS_URL
   ```

3. **Service Failures**
   ```bash
   # Check service status
   sudo systemctl status bridge-relayer
   
   # View recent logs
   journalctl -u bridge-relayer --since "1 hour ago"
   ```

### Emergency Procedures

1. **Bridge Pause**
   ```bash
   # Pause bridge operations
   curl -X POST http://localhost:3001/admin/pause
   ```

2. **Validator Rotation**
   ```bash
   # Update validator set
   curl -X POST http://localhost:3001/admin/validators \
     -H "Content-Type: application/json" \
     -d '{"action": "rotate", "new_validators": [...]}'
   ```

This deployment guide provides comprehensive instructions for deploying the Cross-Chain Bridge in various environments. Always test thoroughly in staging environments before production deployment.
