# Motion Ogre Backend

Real-time backend bridge for Motion Ogre video editing system.

## Features

- **REST API**: Full CRUD operations for projects and timelines
- **WebSocket Server**: Real-time timeline synchronization and live playhead updates
- **Rust FFI Bridge**: Direct integration with Rust timeline engine
- **Python Agent Bridge**: Spawn and communicate with Python AI agents
- **SQLite Persistence**: Project storage with draft/version management
- **Structured Logging**: Pino-based logging with configurable levels
- **Performance Metrics**: Built-in metrics collection and monitoring

## Architecture

```
┌─────────────┐         ┌─────────────────┐         ┌──────────────┐
│   Frontend  │◄────────┤  Node.js API    │◄────────┤  Rust Engine │
│   (React)   │  HTTP   │  (Express.js)   │  FFI    │  (Timeline)  │
└─────────────┘         └────────┬────────┘         └──────────────┘
                                │ WebSocket
                                │
                         ┌──────▼────────┐
                         │  Python Agent │
                         │  (Director,   │
                         │   Cut, Color, │
                         │   Audio, etc) │
                         └───────────────┘
```

## Installation

### Prerequisites

- Node.js 18+
- Python 3.11+ (for agent execution)
- SQLite 3 (usually included with Node packages)

### Setup

```bash
# Install Node.js dependencies
npm install

# Install Python agent framework (from project root)
cd ../
pip install -e .
cd backend

# Create data directory
mkdir -p data
```

## Running

### Development

```bash
# Start the server with auto-reload
npm run dev
```

The server will start:
- HTTP API on port 3000 (or PORT env var)
- WebSocket server on port 3001 (or WS_PORT env var)

### Production

```bash
# Build TypeScript
npm run build

# Start the server
npm start
```

### Environment Variables

```bash
PORT=3000              # HTTP API port (default: 3000)
WS_PORT=3001           # WebSocket port (default: 3001)
DB_PATH=./data/motion-ogre.db  # SQLite database path
LOG_LEVEL=info         # Log level: fatal|error|warn|info|debug|trace
PYTHON_PATH=python3    # Path to Python executable
```

## API Documentation

### REST API Endpoints

#### Health Check
```
GET /api/health
```

#### Projects
```
GET    /api/projects              # List all projects
POST   /api/projects              # Create new project
GET    /api/projects/:id          # Get specific project
PUT    /api/projects/:id          # Update project
DELETE /api/projects/:id          # Delete project
```

#### Timeline Queries
```
GET /api/projects/:id/timeline/query?numerator=<n>&denominator=<d>
```

Returns active clips at the given timeline time.

#### Drafts
```
GET    /api/projects/:id/drafts   # List project drafts
POST   /api/projects/:id/drafts   # Create draft
DELETE /api/drafts/:id            # Delete draft
```

### WebSocket Protocol

#### Connection
```javascript
const ws = new WebSocket('ws://localhost:3001');

ws.on('message', (data) => {
  const message = JSON.parse(data);
  console.log(message);
});
```

#### Commands

All commands have this structure:
```typescript
{
  id: string;              // Unique command ID
  type: CommandType;       // Command type
  timestamp: number;       // Unix timestamp
  ...additional fields     // Command-specific fields
}
```

##### Create Timeline
```json
{
  "id": "cmd-1",
  "type": "timeline_create",
  "timestamp": 1234567890,
  "frame_rate": { "numerator": 24, "denominator": 1 }
}
```

##### Create Layer
```json
{
  "id": "cmd-2",
  "type": "layer_create",
  "timestamp": 1234567890,
  "project_id": "project-uuid",
  "data": {
    "kind": "Video",
    "z_order": 0,
    "name": "Video Layer 1"
  }
}
```

##### Move Playhead
```json
{
  "id": "cmd-3",
  "type": "playhead_move",
  "timestamp": 1234567890,
  "project_id": "project-uuid",
  "time": { "numerator": 50, "denominator": 1 }
}
```

##### Execute Agent
```json
{
  "id": "cmd-4",
  "type": "agent_execute",
  "timestamp": 1234567890,
  "project_id": "project-uuid",
  "agent_type": "CutAgent",
  "intent": "Trim the first 10 seconds of the video",
  "context": {}
}
```

#### Events

```typescript
{
  type: EventType;         // Event type
  project_id: string;     // Associated project
  timestamp: number;       // Unix timestamp
  ...additional fields     // Event-specific fields
}
```

##### Timeline Updated
```json
{
  "type": "timeline_updated",
  "project_id": "project-uuid",
  "timestamp": 1234567890,
  "timeline": { ... },
  "change_summary": "Created clip: Clip 1"
}
```

##### Playhead Moved
```json
{
  "type": "playhead_moved",
  "project_id": "project-uuid",
  "timestamp": 1234567890,
  "time": { "numerator": 50, "denominator": 1 },
  "is_playing": false
}
```

##### Agent Progress
```json
{
  "type": "agent_progress",
  "project_id": "project-uuid",
  "timestamp": 1234567890,
  "agent_type": "CutAgent",
  "execution_id": "cmd-4",
  "progress": 50,
  "message": "Planning execution..."
}
```

## Testing

```bash
# Run all tests
npm test

# Run integration tests only
npm run test:integration

# Run with coverage
npm test -- --coverage
```

## Project Structure

```
backend/
├── src/
│   ├── api/              # REST API routes
│   ├── bridge/           # FFI bridges (Rust, Python)
│   ├── db/              # Database schema and operations
│   ├── types/           # TypeScript type definitions
│   ├── utils/           # Logging, metrics
│   ├── ws/              # WebSocket server
│   └── server.ts        # Main entry point
├── tests/
│   └── integration/     # Integration tests
├── data/               # SQLite database (created at runtime)
├── package.json
├── tsconfig.json
└── jest.config.js
```

## Development Notes

### Rust FFI Bridge

Currently implemented as a JavaScript mock. To integrate with actual Rust code:

1. Add `napi` and `napi-rs` dependencies
2. Create `crates/motion-ogre-ffi` with napi bindings
3. Run `npm run build:rust` to compile the native module
4. Replace mock implementations with FFI calls

### Python Agent Bridge

The bridge spawns Python processes and communicates via stdout/stderr.

- Agent scripts are in `../src/motion_ogre_agents/`
- Each agent execution is a separate process
- Progress updates are sent via `PROGRESS:` prefixed JSON lines
- Final results are sent as JSON on stdout

### Database Schema

Projects table:
- `id`: UUID primary key
- `name`: Project name
- `created_at`, `updated_at`: Timestamps
- `timeline_json`: Serialized timeline (JSON)
- `version`: Version number for conflict detection

Drafts table:
- `id`: UUID primary key
- `project_id`: Reference to parent project
- `name`: Draft name
- `created_at`: Timestamp
- `timeline_json`: Serialized timeline state

## Performance

The backend is designed to handle:
- 100+ concurrent WebSocket connections
- Sub-100ms API response times
- 10+ active Python agent processes
- Real-time event broadcasting with <50ms latency

## Logging

Logs are structured JSON for easy parsing:

```json
{
  "level": "info",
  "time": "2024-01-17T12:00:00.000Z",
  "name": "MotionOgreBackend",
  "project_id": "uuid",
  "client_id": "uuid",
  "msg": "Client connected"
}
```

## Troubleshooting

### Database Lock Errors

If you see "database is locked" errors:
- Ensure only one backend instance is running
- Check for zombie processes: `lsof data/motion-ogre.db`
- Delete WAL files: `rm data/motion-ogre.db-wal`

### Python Agent Failures

If agents fail to execute:
- Verify Python 3.11+ is installed: `python3 --version`
- Check motion-ogre-agents is installed: `pip show motion-ogre-agents`
- Ensure OPENAI_API_KEY is set (for LLM agents)
- Check logs for stderr output from Python processes

### WebSocket Connection Issues

If clients can't connect:
- Verify WS_PORT is not already in use: `lsof -i :3001`
- Check firewall rules
- Ensure client uses correct WebSocket URL (ws://, not http://)

## License

MIT
