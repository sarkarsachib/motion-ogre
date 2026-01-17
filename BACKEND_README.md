# Motion Ogre

A modern, real-time video editing system built with Rust, Python, and Node.js.

## Architecture

Motion Ogre is composed of three main components:

1. **Rust Timeline Engine**: Core editing logic with frame-accurate time handling
2. **Python Agent Framework**: AI-powered multi-agent orchestration for autonomous editing
3. **Node.js Backend**: Real-time API and WebSocket server bridging frontend to engine and agents

```
┌─────────────┐         ┌─────────────────┐         ┌──────────────┐
│   Frontend  │◄────────┤  Node.js API    │◄────────┤  Rust Engine │
│   (React)   │  HTTP   │  (Express.js)   │  FFI    │  (Timeline)  │
└─────────────┘         └────────┬────────┘         └──────────────┘
                                │ WebSocket
                         ┌──────▼────────┐
                         │  Python Agent │
                         │  (Director,   │
                         │   Cut, Color, │
                         │   Audio, etc) │
                         └───────────────┘
```

## Components

### Rust Engine (`crates/`)

- **motion-ogre-time**: Frame-accurate rational time primitives
- **motion-ogre-timeline**: Timeline data structures (layers, tracks, clips)
- **motion-ogre-animation**: Animation keyframes and interpolation
- **motion-ogre-playback**: Timeline playback and rendering
- **motion-ogre-editing**: High-level editing operations

### Python Agents (`src/motion_ogre_agents/`)

- **DirectorAgent**: Coordinates multi-agent workflows
- **CutAgent**: Video cutting and trimming operations
- **MotionAgent**: Motion effects and transitions
- **ColorAgent**: Color grading and visual effects
- **AudioAgent**: Audio editing and synchronization

### Node.js Backend (`backend/`)

- **REST API**: Project CRUD, timeline queries, rendering jobs
- **WebSocket Server**: Real-time sync, playhead updates, agent broadcasts
- **Rust Bridge**: FFI bindings to timeline engine
- **Python Bridge**: Process spawning and agent communication
- **Database**: SQLite persistence with draft/version management

## Getting Started

### Prerequisites

- Rust 1.70+ with stable toolchain
- Python 3.11+
- Node.js 18+

### Installation

```bash
# Clone repository
git clone https://gitlab.com/nr-labs/motion-ogre.git
cd motion-ogre

# Install Rust dependencies
cargo build --release

# Install Python agents
pip install -e .

# Install Node.js backend
cd backend
npm install
cd ..
```

### Running the Backend

```bash
cd backend

# Development mode (with auto-reload)
npm run dev

# Production mode
npm run build
npm start
```

The backend will start:
- HTTP API: `http://localhost:3000`
- WebSocket: `ws://localhost:3001`

## API Documentation

### REST API

See [backend/README.md](backend/README.md) for complete API documentation.

### WebSocket Protocol

See [backend/README.md](backend/README.md) for WebSocket protocol details.

## Development

### Rust Engine

```bash
# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run

# Build for release
cargo build --release
```

### Python Agents

```bash
# Run tests
pytest tests/

# With coverage
pytest --cov=src/motion_ogre_agents

# Format code
black src/ tests/
isort src/ tests/

# Type check
mypy src/
```

### Node.js Backend

```bash
# Run tests
npm test

# Integration tests only
npm run test:integration

# Build
npm run build

# Lint
npm run lint
```

## Project Structure

```
motion-ogre/
├── crates/                  # Rust crates
│   ├── motion-ogre-time/    # Time primitives
│   ├── motion-ogre-timeline/ # Timeline data structures
│   ├── motion-ogre-animation/# Animation logic
│   ├── motion-ogre-playback/ # Playback engine
│   └── motion-ogre-editing/ # Editing operations
├── src/motion_ogre_agents/  # Python agents
│   ├── agents/              # Agent implementations
│   ├── agent.py            # Base agent class
│   ├── llm.py             # LLM client
│   ├── registry.py         # Agent registry
│   ├── events.py           # Event logging
│   ├── prompts.py          # Prompt templates
│   └── bridge.py          # Node.js bridge script
├── backend/               # Node.js backend
│   ├── src/
│   │   ├── api/          # REST routes
│   │   ├── bridge/       # FFI bridges
│   │   ├── db/          # Database schema
│   │   ├── types/        # TypeScript types
│   │   ├── utils/        # Logging, metrics
│   │   ├── ws/          # WebSocket server
│   │   └── server.ts     # Main entry
│   └── tests/           # Integration tests
└── tests/                # Cross-component tests
```

## Configuration

### Environment Variables

Backend configuration (copy `backend/.env.example` to `backend/.env`):

```bash
PORT=3000              # HTTP API port
WS_PORT=3001           # WebSocket port
DB_PATH=./data/motion-ogre.db
LOG_LEVEL=info
PYTHON_PATH=python3
OPENAI_API_KEY=sk-...   # Required for AI agents
```

### Rust Configuration

Set `RUST_LOG` environment variable:

```bash
RUST_LOG=info      # Standard logging
RUST_LOG=debug     # Verbose logging
RUST_LOG=trace     # Extra verbose
```

## License

MIT OR Apache-2.0

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## Support

- Issues: https://gitlab.com/nr-labs/motion-ogre/-/issues
- Documentation: https://docs.motion-ogre.com (coming soon)
