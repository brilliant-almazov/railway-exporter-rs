# 🗺️ Roadmap

## Current Implementation

### API Overview

All requests go to the Railway GraphQL API:

```
POST https://backboard.railway.app/graphql/v2
Authorization: Bearer <RAILWAY_API_TOKEN>
Content-Type: application/json
```

---

### API Endpoint 1: Get Project Information

Fetches project name and list of services.

#### Request

```graphql
query {
  project(id: "PROJECT_ID") {
    name
    services {
      edges {
        node {
          id
          name
        }
      }
    }
  }
}
```

#### Response Example

```json
{
  "data": {
    "project": {
      "name": "my-production-project",
      "services": {
        "edges": [
          {
            "node": {
              "id": "srv_abc123def456",
              "name": "api"
            }
          },
          {
            "node": {
              "id": "srv_xyz789ghi012",
              "name": "postgres"
            }
          },
          {
            "node": {
              "id": "srv_qwe345rty678",
              "name": "redis"
            }
          }
        ]
      }
    }
  }
}
```

#### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `project.name` | String | Project display name |
| `services.edges[].node.id` | String | Unique service ID (used in usage API) |
| `services.edges[].node.name` | String | Service display name |

---

### API Endpoint 2: Get Current Usage

Retrieves **cumulative** resource consumption for the current billing period, grouped by service.

#### Request

```graphql
query {
  usage(
    projectId: "PROJECT_ID"
    measurements: [CPU_USAGE, MEMORY_USAGE_GB, DISK_USAGE_GB, NETWORK_TX_GB, NETWORK_RX_GB]
    groupBy: [SERVICE_ID]
  ) {
    measurement
    value
    tags {
      serviceId
    }
  }
}
```

#### Response Example

```json
{
  "data": {
    "usage": [
      {
        "measurement": "CPU_USAGE",
        "value": 15847.23,
        "tags": {
          "serviceId": "srv_abc123def456"
        }
      },
      {
        "measurement": "MEMORY_USAGE_GB",
        "value": 8234.56,
        "tags": {
          "serviceId": "srv_abc123def456"
        }
      },
      {
        "measurement": "DISK_USAGE_GB",
        "value": 450.12,
        "tags": {
          "serviceId": "srv_abc123def456"
        }
      },
      {
        "measurement": "NETWORK_TX_GB",
        "value": 2.34,
        "tags": {
          "serviceId": "srv_abc123def456"
        }
      },
      {
        "measurement": "NETWORK_RX_GB",
        "value": 5.67,
        "tags": {
          "serviceId": "srv_abc123def456"
        }
      },
      {
        "measurement": "CPU_USAGE",
        "value": 4521.89,
        "tags": {
          "serviceId": "srv_xyz789ghi012"
        }
      }
    ]
  }
}
```

#### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `measurement` | Enum | Type of measurement (see table below) |
| `value` | Float | Cumulative value since billing period start |
| `tags.serviceId` | String | Service ID this measurement belongs to |

#### Available Measurements

| Measurement | Units | What it measures | Pricing applies? |
|-------------|-------|------------------|:----------------:|
| `CPU_USAGE` | vCPU-minutes | Total vCPU time consumed | ✅ Yes |
| `MEMORY_USAGE_GB` | GB-minutes | Memory × time (GB × minutes) | ✅ Yes |
| `DISK_USAGE_GB` | GB-minutes | Disk × time (GB × minutes) | ✅ Yes |
| `NETWORK_TX_GB` | GB | Outbound traffic (egress) | ✅ Yes |
| `NETWORK_RX_GB` | GB | Inbound traffic (ingress) | ❌ Free |

#### Understanding Values

Example: `CPU_USAGE = 15847.23 vCPU-minutes`

This means:
- If you ran 1 vCPU for 15847 minutes, OR
- 2 vCPUs for 7923 minutes, OR
- 0.5 vCPU for 31694 minutes

**Cost calculation:**
```
CPU Cost = 15847.23 × $0.000231 = $3.66 (Pro plan)
```

Example: `MEMORY_USAGE_GB = 8234.56 GB-minutes`

This means:
- If you used 1 GB RAM for 8234 minutes, OR
- 2 GB RAM for 4117 minutes, OR
- 512 MB RAM for 16469 minutes

**Cost calculation:**
```
Memory Cost = 8234.56 × $0.000116 = $0.96 (Pro plan)
```

---

### API Endpoint 3: Get Estimated Monthly Usage

Provides Railway's own projection for the entire billing period (not calendar month!).

#### Request

```graphql
query {
  estimatedUsage(
    projectId: "PROJECT_ID"
    measurements: [CPU_USAGE, MEMORY_USAGE_GB, DISK_USAGE_GB, NETWORK_TX_GB, NETWORK_RX_GB]
  ) {
    measurement
    estimatedValue
  }
}
```

#### Response Example

```json
{
  "data": {
    "estimatedUsage": [
      {
        "measurement": "CPU_USAGE",
        "estimatedValue": 47541.69
      },
      {
        "measurement": "MEMORY_USAGE_GB",
        "estimatedValue": 24703.68
      },
      {
        "measurement": "DISK_USAGE_GB",
        "estimatedValue": 1350.36
      },
      {
        "measurement": "NETWORK_TX_GB",
        "estimatedValue": 7.02
      },
      {
        "measurement": "NETWORK_RX_GB",
        "estimatedValue": 17.01
      }
    ]
  }
}
```

#### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `measurement` | Enum | Type of measurement |
| `estimatedValue` | Float | Railway's projection for the full billing period |

**Note:** This is Railway's own estimate based on current usage patterns. It's **NOT** grouped by service — it returns totals for the entire project.

---

## Potential Future Metrics

These metrics could be added in future versions. Contributions welcome!

### Service-Level Metrics

| Metric | API Field | Priority | Description |
|--------|-----------|:--------:|-------------|
| `railway_service_replicas` | `service.deployments.replicas` | High | Number of running instances |
| `railway_service_status` | `deployment.status` | High | Current state (building/deploying/running/failed/crashed) |
| `railway_service_uptime_seconds` | Calculated from `deployment.createdAt` | Medium | Time since last deploy |
| `railway_service_restart_count` | `deployment.restartCount` | Medium | Number of restarts (health indicator) |
| `railway_service_build_duration_seconds` | `deployment.buildDuration` | Low | Container build time |

### Environment Metrics

| Metric | API Field | Priority | Description |
|--------|-----------|:--------:|-------------|
| `railway_environment_count` | `project.environments` | Medium | Number of environments |
| `railway_environment_usage_usd` | `usage` per environment | Medium | Usage breakdown by environment |

### Volume Metrics

| Metric | API Field | Priority | Description |
|--------|-----------|:--------:|-------------|
| `railway_volume_size_gb` | `volume.size` | Medium | Provisioned volume size |
| `railway_volume_usage_gb` | `volume.usage` | Medium | Actual disk usage |

### Database Metrics

| Metric | API Field | Priority | Description |
|--------|-----------|:--------:|-------------|
| `railway_database_connections` | Plugin metrics | Low | Active connections |
| `railway_database_queries_total` | Plugin metrics | Low | Query count |

### Cron Job Metrics

| Metric | API Field | Priority | Description |
|--------|-----------|:--------:|-------------|
| `railway_cron_last_run_timestamp` | `cronJob.lastRun` | Low | Last execution time |
| `railway_cron_success` | `cronJob.lastStatus` | Low | Success/failure status |

---

## GraphQL Schema Reference

Useful queries for exploring the API:

```graphql
# List all available measurements
{
  __type(name: "UsageMeasurement") {
    enumValues { name }
  }
}

# Get deployment details
{
  project(id: "PROJECT_ID") {
    services {
      edges {
        node {
          id
          name
          deployments(first: 1) {
            edges {
              node {
                id
                status
                createdAt
              }
            }
          }
        }
      }
    }
  }
}

# Get environment list
{
  project(id: "PROJECT_ID") {
    environments {
      edges {
        node {
          id
          name
          isEphemeral
        }
      }
    }
  }
}
```

---

## Important: What Metrics Mean

### Cumulative vs Real-Time

Railway API returns **cumulative values for the billing period**, NOT real-time instant values:

| Metric | Units | Type | What it means |
|--------|-------|------|---------------|
| `CPU_USAGE` | vCPU-minutes | Cumulative | Total minutes consumed since billing start |
| `MEMORY_USAGE_GB` | GB-minutes | Cumulative | Total GB×minutes since billing start |
| `DISK_USAGE_GB` | GB-minutes | Cumulative | Total GB×minutes since billing start |
| `NETWORK_TX_GB` | GB | Cumulative | Total GB transferred since billing start |

**These are NOT:**
- Current CPU load (%)
- Current memory usage (MB)
- Current disk usage (GB)

### Want Real-Time Metrics?

Railway API doesn't expose instant resource usage. Options:

1. **In-container monitoring**: Run `node_exporter` or similar inside your container
2. **Application-level**: Add metrics to your app (e.g., `/metrics` endpoint)
3. **Railway CLI**: Connect to running instances for real-time stats

### Cost Calculation

We calculate costs by multiplying cumulative usage by pricing rates:

```
Cost = CPU_USAGE × $0.000231 (pro) + MEMORY_USAGE_GB × $0.000116 (pro) + ...
```

## Billing Period Notes

- Railway uses **rolling billing** (anniversary-based), not calendar months
- Billing cycle starts from account creation date
- `estimatedUsage` API returns projection for the current billing cycle
- We calculate `daily_average = current_usage / day_of_billing_period`

---

## API Rate Limits

- No official rate limit documentation
- Recommended: poll every 5 minutes (300s) to be safe
- The exporter caches responses internally

---

## Links

- [Railway GraphQL API Reference](https://docs.railway.app/reference/graphql-api)
- [GraphQL Playground](https://railway.app/graphql) (requires authentication)
- [Railway Pricing](https://railway.app/pricing)
- [API Authentication](https://docs.railway.app/reference/graphql-api#authentication)
