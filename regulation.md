# Motion Ogre Multi-Agent Orchestration Framework

## Framework Structure

### Core Components

1. **Agent Base Architecture** (`src/motion_ogre_agents/agent.py`)
   - Abstract `Agent` class with `plan()`, `execute()`, `critique()` methods
   - Agent state machine with IDLE, PLANNING, EXECUTING, CRITIQUING, WAITING, ERROR, COMPLETED states
   - Task processing with retry logic and error handling
   - Execution tracking and result validation

2. **Agent State Management** (`src/motion_ogre_agents/state.py`)
   - `AgentState` enum for lifecycle management
   - `AgentStateTransition` for tracking state changes with timestamps

3. **LLM Integration** (`src/motion_ogre_agents/llm.py`)
   - `LLMClient` with OpenAI API integration
   - Token counting using tiktoken
   - Cost tracking per model (GPT-4, GPT-4 Turbo, GPT-3.5 Turbo, GPT-4o)
   - Exponential backoff retry logic
   - Prompt templating support

4. **Prompt Templates** (`src/motion_ogre_agents/prompts.py`)
   - DirectorAgent intent parsing and task routing prompts
   - Agent planning, execution, and critique prompts
   - Error recovery and workflow management prompts

5. **Event System** (`src/motion_ogre_agents/events.py`)
   - Structured event logging with `EventLogger`
   - Execution history tracking with `ExecutionHistory`
   - Agent communication system with `AgentCommunication`
   - Event subscribers and correlation ID tracking

6. **Agent Registry** (Note: Individual agent files)
   - `DirectorAgent`: Parses natural language intent and routes to specialists
   - `BaseEditingAgent`: Template for cut, motion, color, audio agents

### Agent Directory Structure

```
src/motion_ogre_agents/
├── __init__.py
├── agent.py              # Base Agent class and core functionality
├── state.py              # State machine definitions
├── llm.py               # OpenAI integration and token management
├── prompts.py           # Prompt templates for agents
├── events.py            # Event system and logging
├── agents/              # Specific agent implementations
│   ├── __init__.py
│   ├── director.py      # DirectorAgent - intent parsing and routing
│   └── base_editing.py  # BaseEditingAgent - template for editing agents
└── registry.py          # Agent registry and discovery (if needed)
```

### Key Features

1. **Agent Lifecycle Management**
   - State-based agent management with automatic transitions
   - Retry logic with exponential backoff
   - Error recovery and exception handling

2. **LLM Cost Management**
   - Token counting for input/output
   - Per-model cost tracking and reporting
   - Budget awareness for API usage

3. **Audit and Observability**
   - Complete execution history tracking
   - Structured event logging with severity levels
   - Agent communication and state transition logs

4. **Extensibility**
   - Base classes for creating new agent types
   - Prompt template system for customizing behavior
   - Event subscription system for monitoring

5. **Testing Support**
   - Configurable retry behavior
   - Mock LLM client support for testing
   - Correlation IDs for request tracing

### Agent Workflow

1. **DirectorAgent** receives natural language editing request
2. **Planning**: Parse intent, identify required operations
3. **Routing**: Assign tasks to appropriate specialized agents
4. **Execution**: Specialized agents execute their tasks
5. **Critique**: Each agent reviews its own results
6. **Coordination**: DirectorAgent orchestrates multi-agent workflows

### Integration Points

- **Rust Core**: Python agents communicate with existing Rust video processing crates
- **Temporal SDK**: Framework ready for Temporal workflow integration
- **OpenAI API**: Direct integration with GPT models for planning and decision making
- **Event System**: Rich logging and monitoring capabilities

### Development Guidelines

1. All agents inherit from base `Agent` class
2. Implement `plan()`, `execute()`, and `critique()` methods
3. Use provided prompt templates for consistency
4. Log all significant events through the event system
5. Track execution metrics for optimization
6. Handle errors gracefully with retry logic

### Testing Strategy

1. Unit tests for each agent component
2. Integration tests for multi-agent workflows
3. Mock LLM responses for deterministic testing
4. Performance tests for token counting and cost tracking