"""Agent state machine definitions."""

from enum import Enum
from typing import Optional
from pydantic import BaseModel
from datetime import datetime


class AgentState(Enum):
    """Agent lifecycle states."""
    
    IDLE = "idle"
    PLANNING = "planning"
    EXECUTING = "executing"
    CRITIQUING = "critiquing"
    WAITING = "waiting"
    ERROR = "error"
    COMPLETED = "completed"


class AgentStateTransition(BaseModel):
    """Record of agent state transition."""
    
    from_state: AgentState
    to_state: AgentState
    timestamp: datetime
    reason: Optional[str] = None
    metadata: Optional[dict] = None
