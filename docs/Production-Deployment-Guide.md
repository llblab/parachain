# Production Deployment Guide

This guide provides comprehensive instructions for deploying the Parachain Template to production, covering everything from initial setup to network launch.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Build Process](#build-process)
3. [Chain Specification](#chain-specification)
4. [Security Considerations](#security-considerations)
5. [Network Configuration](#network-configuration)
6. [Deployment Steps](#deployment-steps)
7. [Monitoring & Maintenance](#monitoring--maintenance)
8. [Troubleshooting](#troubleshooting)

## Prerequisites

### System Requirements

- **OS**: Ubuntu 20.04+ or similar Linux distribution
- **CPU**: 4+ cores (8+ recommended for production)
- **RAM**: 8GB minimum (16GB+ recommended)
- **Storage**: 100GB+ SSD storage
- **Network**: Stable internet connection with public IP

### Software Dependencies

```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
rustup default stable
rustup update
rustup target add wasm32-unknown-unknown

# Install additional tools
sudo apt update
sudo apt install -y git clang curl libssl-dev llvm libudev-dev make protobuf-compiler

# Download polkadot-omni-node (latest release)
curl -L -o polkadot-omni-node https://github.com/paritytech/polkadot-sdk/releases/latest/download/polkadot-omni-node
chmod +x polkadot-omni-node
sudo mv polkadot-omni-node /usr/local/bin/

# Verify installation
polkadot-omni-node --version
```

### Required Accounts

- **Relay Chain Account**: Funded account for parachain registration
- **Collator Accounts**: Accounts for block production (2+ recommended)
- **Sudo Account**: Administrative account (disable in production)

## Build Process

### 1. Clone and Build Runtime

```bash
git clone <your-parachain-repo>
cd parachain

# Ensure polkadot-omni-node is available
if ! command -v polkadot-omni-node &> /dev/null; then
    echo "Downloading polkadot-omni-node..."
    curl -L -o polkadot-omni-node https://github.com/paritytech/polkadot-sdk/releases/latest/download/polkadot-omni-node
    chmod +x polkadot-omni-node
    sudo mv polkadot-omni-node /usr/local/bin/
fi

# Build optimized runtime
cargo build --release --manifest-path runtime/Cargo.toml

# Verify WASM output
ls -la target/release/wbuild/parachain-template-runtime/
```

### 2. Generate Chain Specification

```bash
# Create raw chain spec for production
polkadot-omni-node chain-spec-builder create \
  --runtime ./target/release/wbuild/parachain-template-runtime/parachain_template_runtime.wasm \
  --chain-name "Your Parachain Network" \
  --chain-id your-parachain \
  --para-id <YOUR_PARA_ID> \
  --relay-chain polkadot \
  --chain-spec-path production_chain_spec.json \
  named-preset local-testnet

# Convert to raw format for deployment
polkadot-omni-node chain-spec-builder convert-to-raw \
  production_chain_spec.json \
  --output production_chain_spec_raw.json

# Verify the chain spec
polkadot-omni-node chain-spec-builder verify production_chain_spec_raw.json
```

## Chain Specification

### Critical Parameters to Configure

```rust
// In genesis_config_presets.rs - Production values
pub const PARACHAIN_ID: u32 = YOUR_PARA_ID; // Assigned by relay chain

// Update production_genesis function:
fn production_genesis() -> Value {
  testnet_genesis(
    // Production collators (replace with real accounts)
    vec![
      (collator_1_account, collator_1_aura_key),
      (collator_2_account, collator_2_aura_key),
    ],
    // Initial funded accounts
    vec![
      sudo_account,
      treasury_account,
      // Add other initial accounts
    ],
    sudo_account, // Should be None for production
    YOUR_PARA_ID.into(),
  )
}
```

### Asset Configuration Review

Verify production-ready parameters in `assets_config.rs`:

```rust
// Ensure these are production values:
pub const MintMinLiquidity: Balance = 100; // Asset Hub standard
pub const PoolSetupFee: Balance = 10 * EXISTENTIAL_DEPOSIT; // Enable for production
pub const LiquidityWithdrawalFee: sp_runtime::Permill =
  sp_runtime::Permill::from_percent(1); // Consider non-zero fee
```

## Security Considerations

### 1. Key Management

```bash
# Generate production keys securely
polkadot-omni-node key generate --scheme sr25519 --output-type json > collator_keys.json

# Store keys securely (use hardware security modules in production)
# Never commit private keys to version control
```

### 2. Sudo Removal

For production networks, disable sudo access:

```rust
// In runtime/src/lib.rs - Remove or comment out:
// Sudo: pallet_sudo = 70,

// In genesis config:
// sudo: SudoConfig { key: None }, // Disable sudo
```

### 3. Network Security

- Use firewall rules to restrict access
- Enable TLS for RPC endpoints
- Implement proper monitoring and alerting
- Regular security audits

## Network Configuration

### 1. Relay Chain Integration

```bash
# Register parachain on relay chain (requires governance or auction)
# This process varies by relay chain (Polkadot/Kusama/Local)

# For Polkadot/Kusama: Use governance or parachain auction
# For local testing: Use sudo on relay chain
```

### 2. Collator Configuration

```bash
# Production collator startup
polkadot-omni-node \
  --chain production_chain_spec_raw.json \
  --collator \
  --rpc-port 9944 \
  --ws-port 9945 \
  --port 30333 \
  --base-path /var/lib/parachain \
  --name "Collator-01" \
  --rpc-external \
  --rpc-cors all \
  --rpc-methods safe \
  --pruning archive \
  --log info \
  -- \
  --chain /path/to/relay_chain_spec.json \
  --rpc-port 9955 \
  --ws-port 9956 \
  --port 30334
```

### 3. RPC Node Configuration

```bash
# Public RPC node (non-collator)
polkadot-omni-node \
  --chain production_chain_spec_raw.json \
  --rpc-port 9944 \
  --ws-port 9945 \
  --port 30333 \
  --base-path /var/lib/parachain-rpc \
  --name "RPC-Node" \
  --rpc-external \
  --ws-external \
  --rpc-cors all \
  --rpc-methods safe \
  --pruning 1000 \
  --log info \
  -- \
  --chain /path/to/relay_chain_spec.json \
  --rpc-port 9955 \
  --ws-port 9956 \
  --port 30334
```

## Deployment Steps

### Phase 1: Pre-Launch Preparation

1. **Code Audit**: Complete security audit of runtime code
2. **Testing**: Comprehensive testing on testnet environment
3. **Documentation**: Finalize user and developer documentation
4. **Community**: Establish communication channels

### Phase 2: Network Launch

1. **Deploy Infrastructure**:

   ```bash
   # Set up collator nodes
   # Configure monitoring
   # Prepare RPC endpoints
   ```

2. **Register Parachain**:

   ```bash
   # Submit parachain registration
   # Await approval/auction completion
   ```

3. **Start Block Production**:
   ```bash
   # Launch collators
   # Verify block production
   # Monitor network health
   ```

### Phase 3: Post-Launch

1. **Monitor Network Health**
2. **Enable Public Access**
3. **Deploy Frontend/Tools**
4. **Community Onboarding**

## Monitoring & Maintenance

### Essential Metrics

- Block production rate
- Finalization status
- Peer connections
- Memory and CPU usage
- Storage growth
- Transaction throughput

### Monitoring Setup

```bash
# Example Prometheus configuration
# Add to prometheus.yml:
scrape_configs:
  - job_name: 'parachain-nodes'
    static_configs:
      - targets: ['collator-1:9615', 'collator-2:9615']
```

### Log Management

```bash
# Configure log rotation
sudo tee /etc/logrotate.d/parachain << EOF
/var/log/parachain/*.log {
    daily
    rotate 30
    compress
    delaycompress
    missingok
    notifempty
    sharedscripts
}
EOF
```

### Backup Strategy

```bash
# Regular database backups
systemctl stop parachain-collator
tar -czf backup-$(date +%Y%m%d).tar.gz /var/lib/parachain/chains/
systemctl start parachain-collator

# Upload to secure storage
aws s3 cp backup-$(date +%Y%m%d).tar.gz s3://your-backup-bucket/
```

## Troubleshooting

### Common Issues

1. **Node Sync Issues**:

   ```bash
   # Check relay chain connection
   # Verify chain spec matches network
   # Check peer connections
   ```

2. **Block Production Problems**:

   ```bash
   # Verify collator keys
   # Check parachain registration status
   # Monitor relay chain finalization
   ```

3. **Performance Issues**:
   ```bash
   # Monitor resource usage
   # Check database size and pruning
   # Verify network connectivity
   ```

### Emergency Procedures

1. **Runtime Upgrade**: Use governance to deploy fixes
2. **Node Recovery**: Restore from backup and resync
3. **Network Halt**: Coordinate with relay chain governance

### Support Resources

- **Documentation**: Link to technical docs
- **Community**: Discord/Telegram channels
- **Technical Support**: GitHub issues/contact info
- **Monitoring Dashboards**: Grafana/monitoring URLs

## Production Checklist

### Pre-Launch

- [ ] Runtime security audit completed
- [ ] Testnet deployment tested thoroughly
- [ ] Chain specification configured correctly
- [ ] Collator infrastructure deployed
- [ ] Monitoring and alerting configured
- [ ] Backup systems implemented
- [ ] Documentation complete
- [ ] Community channels established

### Launch

- [ ] Parachain registered on relay chain
- [ ] Collators started and producing blocks
- [ ] Network health verified
- [ ] RPC endpoints accessible
- [ ] Basic functionality tested

### Post-Launch

- [ ] Public documentation published
- [ ] Frontend/tools deployed
- [ ] Community onboarding begun
- [ ] Ongoing monitoring verified
- [ ] Incident response procedures tested

## Security Best Practices

1. **Never expose private keys**
2. **Use hardware security modules for key storage**
3. **Implement proper access controls**
4. **Regular security updates**
5. **Monitor for suspicious activity**
6. **Maintain incident response plan**
7. **Regular backups and disaster recovery testing**

## Conclusion

This guide provides the foundation for a secure, robust parachain deployment. Always prioritize security and testing before launching to production. Consider professional security audits and gradual rollout strategies for maximum safety.

For additional support and updates, refer to the project documentation and community resources.

## Additional Notes

- **Binary Management**: The `polkadot-omni-node` binary is not included in the repository and should be downloaded fresh for each deployment
- **Version Compatibility**: Always use the latest stable release of `polkadot-omni-node` that matches your target relay chain version
- **Local Development**: For development, download the binary to your local machine but don't commit it to version control
