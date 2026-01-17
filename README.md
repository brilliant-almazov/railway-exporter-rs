# 🚂 Railway Exporter

<p align="center">
  <img src="https://railway.app/brand/logo-light.png" alt="Railway" width="200"/>
</p>

<p align="center">
  <strong>Prometheus exporter for Railway.app billing and usage metrics</strong>
</p>

<p align="center">
  <a href="https://github.com/brilliant-almazov/railway-exporter-rs/actions/workflows/build.yml"><img src="https://github.com/brilliant-almazov/railway-exporter-rs/actions/workflows/build.yml/badge.svg" alt="Build"></a>
  <a href="https://codecov.io/gh/brilliant-almazov/railway-exporter-rs"><img src="https://codecov.io/gh/brilliant-almazov/railway-exporter-rs/branch/master/graph/badge.svg" alt="Coverage"></a>
  <a href="https://opensource.org/licenses/MIT"><img src="https://img.shields.io/badge/License-MIT-yellow.svg" alt="License: MIT"></a>
  <a href="https://ghcr.io/brilliant-almazov/railway-exporter-rs"><img src="https://img.shields.io/badge/ghcr.io-railway--exporter--rs-blue" alt="Docker Image"></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/rust-1.70+-orange.svg" alt="Rust"></a>
</p>


<p align="center">
  <a href="#-quick-start">Quick Start</a> •
  <a href="#-metrics">Metrics</a> •
  <a href="#-configuration">Configuration</a> •
  <a href="#-building-from-source">Build</a> •
  <a href="#-contributing">Contributing</a>
</p>

---

Monitor your [Railway](https://railway.app) spending in real-time with Grafana dashboards. Get alerts before your bill surprises you.

## 🎯 Why?

Railway doesn't provide Prometheus metrics out of the box. This exporter fills the gap:

| Feature | Description |
|---------|-------------|
| 💰 **Cost Tracking** | See exactly how much each service costs in real-time |
| 📈 **Forecasting** | Estimate your monthly bill before it arrives |
| 🚨 **Alerting** | Set up Prometheus alerts when costs exceed thresholds |
| 📊 **Breakdown** | Know which service is eating your budget |
| 🔄 **Self-Monitoring** | The exporter tracks its own resource usage |

## 📊 Metrics

### Per-Service Metrics

| Metric | Description |
|--------|-------------|
| `railway_cpu_usage_vcpu_minutes` | CPU usage in vCPU-minutes |
| `railway_memory_usage_gb_minutes` | Memory usage in GB-minutes |
| `railway_disk_usage_gb_minutes` | Disk usage in GB-minutes |
| `railway_network_tx_gb` | Network egress in GB |
| `railway_network_rx_gb` | Network ingress in GB |
| `railway_service_cost_usd` | Current billing period cost |
| `railway_service_estimated_monthly_usd` | Estimated monthly cost |

### Project-Level Metrics

| Metric | Description |
|--------|-------------|
| `railway_current_usage_usd` | Total current usage |
| `railway_estimated_monthly_usd` | Estimated total monthly cost |
| `railway_daily_average_usd` | Average daily spending |
| `railway_days_in_billing_period` | Days elapsed in billing period |
| `railway_days_remaining_in_month` | Days remaining in month |

### Exporter Metrics

| Metric | Description |
|--------|-------------|
| `railway_exporter_last_scrape_timestamp` | Last successful API scrape |
| `railway_exporter_scrape_duration_seconds` | API scrape duration |

## ⚙️ Configuration

### Environment Variables

| Variable | Required | Default | Description |
|----------|:--------:|---------|-------------|
| `RAILWAY_API_TOKEN` | ✅ | — | [Get token](https://railway.app/account/tokens) |
| `RAILWAY_PROJECT_ID` | ✅ | — | [Find ID](https://docs.railway.app/guides/projects#project-id) |
| `RAILWAY_PLAN` | ❌ | `hobby` | `hobby` or `pro` |
| `SCRAPE_INTERVAL` | ❌ | `300` | API poll interval (60-3600 sec) |
| `PORT` | ❌ | `9333` | HTTP port (1-65535) |
| `RAILWAY_API_URL` | ❌ | See below | GraphQL endpoint |
| `RUST_LOG` | ❌ | `info` | `error`/`warn`/`info`/`debug`/`trace` |

**Default API URL:** `https://backboard.railway.app/graphql/v2`

### Alternative: TOML Configuration

You can use a TOML config file instead of individual environment variables.

#### Option 1: Plain TOML (CONFIG_TOML)

```bash
export CONFIG_TOML='
[railway]
api_token = "railway_xxxxxxxx"
project_id = "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
plan = "pro"

[server]
port = 9333
scrape_interval = 300
'
```

#### Option 2: Base64-encoded TOML (CONFIG_TOML_BASE64)

Useful when your deployment platform doesn't support multi-line environment variables.

**Step 1: Create config.toml file**
```toml
[railway]
api_token = "railway_abc123xyz"
project_id = "550e8400-e29b-41d4-a716-446655440000"
plan = "pro"

[server]
port = 9333
scrape_interval = 300
```

**Step 2: Encode to Base64**
```bash
# Linux/macOS
cat config.toml | base64

# Or one-liner
base64 -i config.toml

# Output example:
# W3JhaWx3YXldCmFwaV90b2tlbiA9ICJyYWlsd2F5X2FiYzEyM3h5eiIKcHJvamVjdF9pZCA9ICI1NTBlODQwMC1lMjliLTQxZDQtYTcxNi00NDY2NTU0NDAwMDAiCnBsYW4gPSAicHJvIgoKW3NlcnZlcl0KcG9ydCA9IDkzMzMKc2NyYXBlX2ludGVydmFsID0gMzAwCg==
```

**Step 3: Set environment variable**
```bash
export CONFIG_TOML_BASE64="W3JhaWx3YXldCmFwaV90b2tlbiA9ICJyYWlsd2F5X2FiYzEyM3h5eiIKcHJvamVjdF9pZCA9ICI1NTBlODQwMC1lMjliLTQxZDQtYTcxNi00NDY2NTU0NDAwMDAiCnBsYW4gPSAicHJvIgoKW3NlcnZlcl0KcG9ydCA9IDkzMzMKc2NyYXBlX2ludGVydmFsID0gMzAwCg=="
```

**Step 4: Use in Docker**
```bash
docker run -d \
  -p 9333:9333 \
  -e CONFIG_TOML_BASE64="W3JhaWx3YXldCmFw...==" \
  ghcr.io/brilliant-almazov/railway-exporter-rs:latest
```

#### Why Base64?

| Scenario | Use |
|----------|-----|
| Docker Compose | `CONFIG_TOML` (multi-line supported) |
| Kubernetes Secrets | `CONFIG_TOML_BASE64` (binary-safe) |
| CI/CD pipelines | `CONFIG_TOML_BASE64` (single-line) |
| Railway deployment | Either works, but env vars are simpler |

**Priority order:** Environment variables > TOML config > Defaults

### Validation Rules

| Variable | Validation |
|----------|------------|
| `RAILWAY_API_TOKEN` | Must not be empty |
| `RAILWAY_PROJECT_ID` | Must not be empty |
| `RAILWAY_PLAN` | Must be exactly `hobby` or `pro` (case-insensitive) |
| `SCRAPE_INTERVAL` | Must be a positive integer |
| `PORT` | Must be a valid port number (1-65535) |

## 🚀 Quick Start

### Docker

```bash
# Use specific commit SHA for stability (recommended)
docker run -d \
  --name railway-exporter \
  -p 9333:9333 \
  -e RAILWAY_API_TOKEN=your-token \
  -e RAILWAY_PROJECT_ID=your-project-id \
  -e RAILWAY_PLAN=pro \
  ghcr.io/brilliant-almazov/railway-exporter-rs:bad9874

# Or use 'latest' for always up-to-date (less stable)
# ghcr.io/brilliant-almazov/railway-exporter-rs:latest
```

### Deploy to Railway

[![Deploy on Railway](https://railway.app/button.svg)](https://railway.app/new/template?template=https://github.com/brilliant-almazov/railway-exporter-rs&envs=RAILWAY_API_TOKEN,RAILWAY_PROJECT_ID,RAILWAY_PLAN&RAILWAY_API_TOKENDesc=Railway+API+Token&RAILWAY_PROJECT_IDDesc=Project+ID+to+monitor&RAILWAY_PLANDesc=Pricing+plan&RAILWAY_PLANDefault=hobby)

Required environment variables:
- `RAILWAY_API_TOKEN` — [Get your API token](https://railway.app/account/tokens)
- `RAILWAY_PROJECT_ID` — Find in [Project Settings](https://docs.railway.app/guides/projects#project-id)
- `RAILWAY_PLAN` — `hobby` (default) or `pro`

### Binary

```bash
export RAILWAY_API_TOKEN=your-token
export RAILWAY_PROJECT_ID=your-project-id
export RAILWAY_PLAN=pro

./railway-exporter
```

## 🔨 Building from Source

### Prerequisites

- [Rust](https://rustup.rs/) 1.70+
- Docker (optional)

### Build

```bash
git clone https://github.com/brilliant-almazov/railway-exporter-rs.git
cd railway-exporter-rs

# Binary
cargo build --release
./target/release/railway-exporter

# Docker (~6.5 MB image)
docker build -t railway-exporter .
```

### Development

```bash
cargo fmt      # Format
cargo clippy   # Lint
cargo test     # Test
```

## 📈 Prometheus & Grafana

### prometheus.yml

```yaml
scrape_configs:
  - job_name: 'railway'
    scrape_interval: 60s
    static_configs:
      - targets: ['railway-exporter:9333']
```

### Grafana Queries

```promql
# Total cost
railway_current_usage_usd{project="my-project"}

# Monthly forecast
railway_estimated_monthly_usd{project="my-project"}

# Top 5 expensive services
topk(5, railway_service_cost_usd)

# Daily average trend
railway_daily_average_usd{project="my-project"}
```

### Alert Rule

```yaml
groups:
  - name: railway
    rules:
      - alert: RailwayHighCost
        expr: railway_estimated_monthly_usd > 50
        for: 1h
        annotations:
          summary: "Railway cost exceeds $50/month"
```

## 🔧 Architecture

```
┌─────────────────┐         ┌─────────────────┐         ┌─────────────────┐
│                 │  5 min  │                 │  scrape │                 │
│  Railway API    │◄───────►│    Exporter     │◄────────│   Prometheus    │
│  (GraphQL)      │         │   (cache)       │         │                 │
│                 │         │                 │         │                 │
└─────────────────┘         └─────────────────┘         └────────┬────────┘
                                                                 │
                                                                 ▼
                                                        ┌─────────────────┐
                                                        │                 │
                                                        │    Grafana      │
                                                        │                 │
                                                        └─────────────────┘
```

The exporter caches Railway API responses. Prometheus scrapes don't hit Railway directly.

## 💰 Cost Calculation

### Pricing Rates

| Resource | Hobby | Pro |
|----------|------:|----:|
| CPU | $0.000463/vCPU-min | $0.000231/vCPU-min |
| Memory | $0.000231/GB-min | $0.000116/GB-min |
| Disk | $0.000021/GB-min | $0.000021/GB-min |
| Egress | $0.10/GB | $0.10/GB |

### How Costs Are Calculated

Railway API returns **cumulative usage** for the billing period:

```
Service Cost = (CPU_USAGE × cpu_rate) + (MEMORY_USAGE_GB × mem_rate) +
               (DISK_USAGE_GB × disk_rate) + (NETWORK_TX_GB × egress_rate)
```

**Example (Pro plan):**
```
CPU:     15847 vCPU-min × $0.000231 = $3.66
Memory:   8234 GB-min   × $0.000116 = $0.96
Disk:      450 GB-min   × $0.000021 = $0.01
Egress:   2.34 GB       × $0.10     = $0.23
─────────────────────────────────────────────
Total:                               = $4.86
```

### Monthly Estimate

```
Estimated Monthly = Current Usage × (30 / days_elapsed)
```

See [ROADMAP.md](ROADMAP.md) for full API documentation.

## 📚 Railway Documentation

- [Railway Docs](https://docs.railway.app/)
- [API Reference](https://docs.railway.app/reference/graphql-api)
- [Pricing Details](https://railway.app/pricing)
- [Getting API Token](https://docs.railway.app/reference/graphql-api#authentication)
- [Project Settings](https://docs.railway.app/guides/projects)

## 🌐 Endpoints

| Endpoint | Description |
|----------|-------------|
| `GET /metrics` | Prometheus metrics |
| `GET /health` | Health check |

## 🤝 Contributing

Contributions welcome! See [CONTRIBUTING.md](CONTRIBUTING.md).

## 📄 License

[MIT](LICENSE) © Anton Brilliantov

---

<p align="center">
  Made with ❤️ and 🦀
</p>
