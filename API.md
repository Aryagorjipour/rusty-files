# Rusty-Files REST API Documentation

## Overview

The Rusty-Files server provides a high-performance REST API with WebSocket support for real-time file system monitoring and search capabilities.

## Getting Started

### Building

```bash
# Build the server binary
cargo build --release --bin filesearch-server

# Build all binaries
cargo build --release --all
```

### Running

```bash
# Run with default configuration
./target/release/filesearch-server

# Run with custom configuration
FILESEARCH_DATABASE__PATH=./custom.db ./target/release/filesearch-server
```

### Docker

```bash
# Build and run with Docker Compose
docker-compose up -d

# View logs
docker-compose logs -f
```

## Configuration

Configuration can be provided via:
1. TOML file (`config/default.toml` or `config/production.toml`)
2. Environment variables (prefixed with `FILESEARCH_`)

### Environment Variables

```bash
# Server settings
FILESEARCH_SERVER__HOST=0.0.0.0
FILESEARCH_SERVER__PORT=8080
FILESEARCH_SERVER__WORKERS=4

# Database settings
FILESEARCH_DATABASE__PATH=/data/filesearch.db

# Security settings
FILESEARCH_SECURITY__ENABLE_AUTH=false
FILESEARCH_SECURITY__JWT_SECRET=your-secret-here

# Logging
RUST_LOG=info
```

## API Endpoints

Base URL: `http://localhost:8080/api/v1`

### Health Check

**GET** `/health`

Check the health status of the server.

**Response:**
```json
{
  "status": "healthy",
  "version": "0.2.0",
  "uptime_seconds": 3600,
  "checks": [
    {
      "name": "database",
      "status": "healthy",
      "message": null,
      "response_time_ms": 5
    },
    {
      "name": "memory",
      "status": "healthy",
      "message": "128.50 MB",
      "response_time_ms": null
    }
  ]
}
```

### Search Files

**POST** `/search`

Search for files based on a query and filters.

**Request Body:**
```json
{
  "query": "*.rs",
  "mode": "glob",
  "filters": {
    "extensions": ["rs", "toml"],
    "size_min": 1024,
    "size_max": 1048576,
    "modified_after": "2024-01-01T00:00:00Z",
    "scope": "name"
  },
  "limit": 100,
  "offset": 0
}
```

**Query Modes:**
- `exact` - Exact string matching
- `fuzzy` - Fuzzy matching
- `regex` - Regular expression matching
- `glob` - Glob pattern matching (default)

**Search Scopes:**
- `name` - Search in file names only
- `path` - Search in full file paths
- `content` - Search in file contents
- `all` - Search in all fields

**Response:**
```json
{
  "results": [
    {
      "path": "/home/user/project/src/main.rs",
      "name": "main.rs",
      "size": 2048,
      "modified": "2024-01-15T10:30:00Z",
      "file_type": "file",
      "score": 0.95,
      "content_preview": null
    }
  ],
  "total": 42,
  "took_ms": 15,
  "has_more": false
}
```

### Index Directory

**POST** `/index`

Index a directory and its contents.

**Request Body:**
```json
{
  "path": "/home/user/projects",
  "recursive": true,
  "follow_symlinks": false,
  "exclusions": ["node_modules", ".git", "target"]
}
```

**Response:**
```json
{
  "indexed_count": 1523,
  "skipped_count": 42,
  "error_count": 0,
  "took_ms": 2500,
  "status": "completed"
}
```

### Update Index

**POST** `/update`

Incrementally update the index for a specific path.

**Request Body:**
```json
{
  "path": "/home/user/projects"
}
```

**Response:**
```json
{
  "added": 15,
  "updated": 8,
  "removed": 3,
  "took_ms": 150
}
```

### Start Watching

**POST** `/watch`

Start watching a directory for file system changes.

**Request Body:**
```json
{
  "path": "/home/user/projects",
  "recursive": true
}
```

**Response:**
```json
{
  "watch_id": "550e8400-e29b-41d4-a716-446655440000",
  "path": "/home/user/projects",
  "status": "active"
}
```

### Stop Watching

**DELETE** `/watch/{id}`

Stop watching a directory.

**Response:**
```json
{
  "message": "Watch stopped",
  "path": "/home/user/projects"
}
```

### Get Statistics

**GET** `/stats`

Get index and performance statistics.

**Response:**
```json
{
  "total_files": 15234,
  "total_directories": 2341,
  "total_size": 1073741824,
  "index_size_mb": 45.2,
  "last_update": "2024-01-15T10:30:00Z",
  "uptime_seconds": 3600,
  "performance": {
    "total_searches": 1523,
    "avg_search_time_ms": 12.5,
    "cache_hit_rate": 0.85,
    "memory_usage_mb": 128.5
  }
}
```

## WebSocket API

**WebSocket Endpoint:** `ws://localhost:8080/ws`

Connect to the WebSocket endpoint to receive real-time file system change events.

### Event Format

```json
{
  "event_type": "modified",
  "path": "/home/user/projects/src/main.rs",
  "timestamp": "2024-01-15T10:30:00Z"
}
```

**Event Types:**
- `created` - File was created
- `modified` - File was modified
- `deleted` - File was deleted
- `renamed` - File was renamed

### Client Message Format

You can send filter messages to the WebSocket to filter events:

```json
{
  "paths": ["/home/user/projects/src"],
  "event_types": ["modified", "created"]
}
```

## Examples

### Using cURL

```bash
# Health check
curl http://localhost:8080/api/v1/health

# Index a directory
curl -X POST http://localhost:8080/api/v1/index \
  -H "Content-Type: application/json" \
  -d '{
    "path": "/home/user/projects",
    "recursive": true
  }'

# Search for Rust files
curl -X POST http://localhost:8080/api/v1/search \
  -H "Content-Type: application/json" \
  -d '{
    "query": "*.rs",
    "mode": "glob",
    "limit": 10
  }'

# Get statistics
curl http://localhost:8080/api/v1/stats
```

### Using JavaScript

```javascript
// Search for files
async function searchFiles(query) {
  const response = await fetch('http://localhost:8080/api/v1/search', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      query: query,
      mode: 'fuzzy',
      limit: 50
    })
  });

  const data = await response.json();
  return data.results;
}

// Connect to WebSocket for real-time updates
const ws = new WebSocket('ws://localhost:8080/ws');

ws.onopen = () => {
  console.log('WebSocket connected');

  // Send filter
  ws.send(JSON.stringify({
    paths: ['/home/user/projects'],
    event_types: ['modified', 'created']
  }));
};

ws.onmessage = (event) => {
  const change = JSON.parse(event.data);
  console.log('File changed:', change);
};
```

### Using Python

```python
import requests
import json

# Index a directory
def index_directory(path):
    response = requests.post(
        'http://localhost:8080/api/v1/index',
        json={
            'path': path,
            'recursive': True
        }
    )
    return response.json()

# Search files
def search_files(query, mode='fuzzy'):
    response = requests.post(
        'http://localhost:8080/api/v1/search',
        json={
            'query': query,
            'mode': mode,
            'limit': 100
        }
    )
    return response.json()['results']

# Get stats
def get_stats():
    response = requests.get('http://localhost:8080/api/v1/stats')
    return response.json()
```

## Performance

The server is optimized for high performance:

- **Concurrent requests**: Handles thousands of concurrent connections
- **Fast search**: Average search time < 15ms for indexed databases with 100K+ files
- **Memory efficient**: Uses bloom filters and LRU caching to minimize memory usage
- **Incremental indexing**: Only indexes changed files during updates
- **Compression**: Optional gzip compression for responses

## Error Handling

All endpoints return standard HTTP status codes:

- `200 OK` - Success
- `400 Bad Request` - Invalid request parameters
- `404 Not Found` - Resource not found
- `500 Internal Server Error` - Server error

Error responses include details:

```json
{
  "error": "invalid_path",
  "message": "Path does not exist",
  "code": 400,
  "details": null
}
```

## Security

### Authentication (Optional)

Enable authentication in configuration:

```toml
[security]
enable_auth = true
jwt_secret = "your-secret-key"
jwt_expiry = 3600
```

### CORS

CORS is enabled by default for development. Configure in production:

```toml
[server]
enable_cors = true
cors_origins = ["https://yourdomain.com"]
```

### Rate Limiting

Rate limiting is configured per IP address:

```toml
[security]
rate_limit_per_minute = 1000
```

## Monitoring

### Metrics

The `/stats` endpoint provides comprehensive metrics for monitoring:

- Search performance
- Cache hit rates
- Memory usage
- Index statistics

### Logging

Configure logging level via environment:

```bash
RUST_LOG=debug ./filesearch-server
```

Supported levels: `trace`, `debug`, `info`, `warn`, `error`

## Troubleshooting

### Server won't start

Check:
- Port 8080 is not in use
- Database path is writable
- Sufficient memory available

### Slow searches

Optimize:
- Increase cache size in configuration
- Ensure database is on SSD
- Index only necessary directories
- Use more specific search queries

### High memory usage

Reduce:
- Cache size
- Bloom filter capacity
- Number of worker threads

## License

MIT License - See LICENSE file for details
