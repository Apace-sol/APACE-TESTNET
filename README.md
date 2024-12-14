# APACE TESTNET (Decentralized Autonomous DataBase System)

APACE TESTNET is the test network for our revolutionary decentralized database system. It provides a safe environment for testing and development of decentralized database applications.

## Core Features

- Decentralized Architecture
- Smart Contract Integration
- High Performance
- Scalable Design
- Security First
- Cross-Chain Compatibility

## Node Requirements (Basic Node)

### Hardware Requirements
- CPU: 2+ cores
- RAM: 4GB minimum
- Storage: 50GB SSD minimum
- Network: Stable internet connection with minimum 5Mbps up/down

### Software Requirements
- Operating System: Ubuntu 20.04+ / macOS 12+ / Windows 10/11
- Rust 1.70.0 or higher
- Git

## Optional Features

### Lightweight LLM Support (Optional)

APACE TESTNET optionally supports running lightweight LLM capabilities. This is an advanced feature that requires additional hardware resources.

> **Note**: LLM support is completely optional. You can run a basic APACE node without enabling LLM features.

#### Additional Hardware Requirements for LLM
- CPU: 6+ cores recommended
- RAM: 16GB minimum
- Storage: Additional 10GB SSD
- GPU: NVIDIA GPU with 6GB+ VRAM (optional, for better performance)
- Additional Software: CUDA Toolkit 11.8+ (if using GPU)

#### LLM Specifications
- Model: LLaMA-2 7B v2.0
- Version: 2.0.1 (December 2023)
- Quantization: 4-bit GPTQ quantization
- Memory Usage: ~8GB RAM when active
- Disk Space: ~5GB for model files
- Response Time: 2-3 seconds for short responses
- Context Window: 4096 tokens
- Language Support: Multilingual (40+ languages)

## Node Setup Guide

### 1. Basic Node Setup

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone the repository
git clone https://github.com/Apace-sol/APACE-TESTNET
cd apace-node
```

### 2. Configure Your Node

Create a configuration file at `config/node.toml`:

```toml
# Basic node configuration
node_id = "auto"  # Will be automatically generated
host = "0.0.0.0"  # Listen on all interfaces
port = 8000
storage_path = "./data"
max_connections = 50
consensus_timeout = 5000  # Milliseconds
bootstrap_nodes = [
    "testnet.apace.io:8000",
    "testnet2.apace.io:8000"
]

# Optional LLM configuration (disabled by default)
[llm]
enabled = false  # Set to true to enable LLM features
model_path = "./models/llama-2-7b-q4.safetensors"
tokenizer_path = "./models/tokenizer.json"
max_batch_size = 4
use_gpu = false  # Set to true if using GPU
```

### 3. Start Your Node

For basic node (without LLM):
```bash
# Build and start basic node
cargo build --release
./target/release/apace-node --config config/node.toml
```

For node with LLM support (optional):
```bash
# First, download LLM model files (about 5GB)
./scripts/download_models.sh

# Build with GPU support
cargo build --release --features llm,cuda

# Or build with CPU-only LLM support
cargo build --release --features llm

# Start node with LLM enabled (modify config.toml first)
./target/release/apace-node --config config/node.toml
```

## Performance Optimization

### Basic Node Optimization
- Monitor disk space usage
- Adjust network bandwidth usage
- Configure max_connections based on available resources

### LLM Optimization (if enabled)
- Start with CPU-only mode first
- Enable GPU support if available
- Adjust batch size based on available memory
- Consider running during off-peak hours

## Troubleshooting

1. **Basic Node Issues**
   - Check network connectivity
   - Verify bootstrap nodes are accessible
   - Ensure port 8000 is open

2. **LLM Issues (if enabled)**
   - Insufficient memory: Disable LLM feature or upgrade RAM
   - Slow response: Consider enabling GPU support
   - High CPU usage: Reduce batch size or disable LLM

## Support

For technical support:
- Check the [Documentation](https://docs.apace.cash)
- Open an issue on GitHub

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
