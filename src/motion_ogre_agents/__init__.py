"""Multi-agent orchestration framework for Motion Ogre's autonomous editing logic."""

__version__ = "0.1.0"

from .agent import Agent, AgentState
from .registry import AgentRegistry
from .events import EventLogger, ExecutionHistory

__all__ = ["Agent", "AgentState", "AgentRegistry", "EventLogger", "ExecutionHistory"]
