# Configuration Guidelines

## Environment Variables

The backend service (Pebble) can be configured using the following environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `PEBBLE_HOST` | The host address the backend binds to. | `127.0.0.1` |
| `PEBBLE_PORT` | The port the backend listens on. | `3000` |

### Binding Strategy

- **Local Development**: Keep defaults (`127.0.0.1:3000`) for security and local access via the browser.
- **Docker Deployment**: Set `PEBBLE_HOST=0.0.0.0` to allow the frontend container or a reverse proxy to reach the backend service via the container network.

## Examples

### Docker Compose (.env)
```env
PEBBLE_HOST=0.0.0.0
PEBBLE_PORT=3000
```
