"""Message/event system for agent communication with structured logging and execution history tracking."""

import json
import asyncio
import threading
from typing import Any, Dict, List, Optional, Callable, Union
from dataclasses import dataclass, asdict
from datetime import datetime
from enum import Enum
from pathlib import Path
from pydantic import BaseModel, Field
import logging
from uuid import uuid4


class EventType(str, Enum):
    """Types of agent events."""
    
    AGENT_STARTED = "agent_started"
    AGENT_STOPPED = "agent_stopped"
    AGENT_STATE_TRANSITION = "agent_state_transition"
    TASK_ASSIGNED = "task_assigned"
    TASK_COMPLETED = "task_completed"
    TASK_FAILED = "task_failed"
    LLM_CALL_STARTED = "llm_call_started"
    LLM_CALL_COMPLETED = "llm_call_completed"
    LLM_CALL_FAILED = "llm_call_failed"
    PLANNING_STARTED = "planning_started"
    PLANNING_COMPLETED = "planning_completed"
    EXECUTION_STARTED = "execution_started"
    EXECUTION_COMPLETED = "execution_completed"
    CRITIQUE_STARTED = "critique_started"
    CRITIQUE_COMPLETED = "critique_completed"
    AGENT_COMMUNICATION = "agent_communication"
    ERROR_OCCURRED = "error_occurred"
    RETRY_ATTEMPT = "retry_attempt"
    WORKFLOW_STARTED = "workflow_started"
    WORKFLOW_COMPLETED = "workflow_completed"
    WORKFLOW_FAILED = "workflow_failed"


class EventSeverity(str, Enum):
    """Event severity levels."""
    
    DEBUG = "debug"
    INFO = "info"
    WARNING = "warning"
    ERROR = "error"
    CRITICAL = "critical"


@dataclass
class AgentEvent:
    """Represents an agent event with metadata."""
    
    event_id: str
    event_type: EventType
    timestamp: datetime
    source_agent: str
    severity: EventSeverity
    message: str
    data: Optional[Dict[str, Any]] = None
    parent_event_id: Optional[str] = None
    correlation_id: Optional[str] = None
    
    def __post_init__(self):
        if self.data is None:
            self.data = {}
    
    @classmethod
    def create(
        cls,
        event_type: EventType,
        source_agent: str,
        message: str,
        severity: EventSeverity = EventSeverity.INFO,
        data: Optional[Dict[str, Any]] = None,
        parent_event_id: Optional[str] = None,
        correlation_id: Optional[str] = None,
    ) -> "AgentEvent":
        """Create a new agent event."""
        return cls(
            event_id=str(uuid4()),
            event_type=event_type,
            timestamp=datetime.now(),
            source_agent=source_agent,
            severity=severity,
            message=message,
            data=data or {},
            parent_event_id=parent_event_id,
            correlation_id=correlation_id,
        )
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert event to dictionary."""
        return {
            "event_id": self.event_id,
            "event_type": self.event_type.value,
            "timestamp": self.timestamp.isoformat(),
            "source_agent": self.source_agent,
            "severity": self.severity.value,
            "message": self.message,
            "data": self.data,
            "parent_event_id": self.parent_event_id,
            "correlation_id": self.correlation_id,
        }
    
    def to_json(self) -> str:
        """Convert event to JSON string."""
        return json.dumps(self.to_dict(), default=str)


class EventSubscriber:
    """Interface for event subscribers."""
    
    async def on_event(self, event: AgentEvent) -> None:
        """Handle an agent event."""
        pass


class EventLogger:
    """Structured event logging system for agents."""
    
    def __init__(
        self,
        log_file: Optional[Union[str, Path]] = None,
        console_logging: bool = True,
        correlation_id: Optional[str] = None,
    ):
        """Initialize event logger."""
        self.log_file = Path(log_file) if log_file else None
        self.console_logging = console_logging
        self.correlation_id = correlation_id or str(uuid4())
        self.subscribers: List[EventSubscriber] = []
        self._lock = asyncio.Lock()
        
        # Setup file logging if specified
        if self.log_file:
            self.log_file.parent.mkdir(parents=True, exist_ok=True)
        
        # Setup Python logger
        self._logger = logging.getLogger(f"motion_ogre_agents.{self.__class__.__name__}")
    
    def add_subscriber(self, subscriber: EventSubscriber) -> None:
        """Add an event subscriber."""
        self.subscribers.append(subscriber)
    
    def remove_subscriber(self, subscriber: EventSubscriber) -> None:
        """Remove an event subscriber."""
        self.subscribers.remove(subscriber)
    
    async def log_event(
        self,
        event_type: EventType,
        source_agent: str,
        message: str,
        severity: EventSeverity = EventSeverity.INFO,
        data: Optional[Dict[str, Any]] = None,
        parent_event_id: Optional[str] = None,
    ) -> str:
        """Log an agent event."""
        
        event = AgentEvent.create(
            event_type=event_type,
            source_agent=source_agent,
            message=message,
            severity=severity,
            data=data,
            parent_event_id=parent_event_id,
            correlation_id=self.correlation_id,
        )
        
        await self._persist_event(event)
        await self._notify_subscribers(event)
        
        return event.event_id
    
    async def _persist_event(self, event: AgentEvent) -> None:
        """Persist event to storage."""
        async with self._lock:
            try:
                # Log to file if configured
                if self.log_file:
                    with open(self.log_file, "a") as f:
                        f.write(event.to_json() + "\n")
                
                # Log to console if configured
                if self.console_logging:
                    self._log_to_console(event)
                    
            except Exception as e:
                # Don't fail the event system due to logging errors
                self._logger.error(f"Failed to persist event: {e}")
    
    def _log_to_console(self, event: AgentEvent) -> None:
        """Log event to console."""
        log_level = self._get_log_level(event.severity)
        log_data = f" [{json.dumps(event.data)}]" if event.data else ""
        self._logger.log(
            log_level,
            f"[{event.severity.value.upper()}] {event.source_agent} :: {event.message}{log_data}",
        )
    
    def _get_log_level(self, severity: EventSeverity) -> int:
        """Map event severity to Python log level."""
        level_map = {
            EventSeverity.DEBUG: logging.DEBUG,
            EventSeverity.INFO: logging.INFO,
            EventSeverity.WARNING: logging.WARNING,
            EventSeverity.ERROR: logging.ERROR,
            EventSeverity.CRITICAL: logging.CRITICAL,
        }
        return level_map.get(severity, logging.INFO)
    
    async def _notify_subscribers(self, event: AgentEvent) -> None:
        """Notify all subscribers about the event."""
        if not self.subscribers:
            return
        
        # Notify subscribers concurrently
        tasks = [subscriber.on_event(event) for subscriber in self.subscribers]
        await asyncio.gather(*tasks, return_exceptions=True)
    
    def create_child_logger(self, parent_event_id: str) -> "EventLogger":
        """Create a child logger for nested operations."""
        child_logger = EventLogger(
            log_file=self.log_file,
            console_logging=self.console_logging,
            correlation_id=self.correlation_id,
        )
        child_logger.subscribers = self.subscribers.copy()
        return child_logger
    
    # Convenience methods for common event types
    async def log_agent_started(self, agent_name: str, data: Optional[Dict[str, Any]] = None) -> str:
        """Log agent start event."""
        return await self.log_event(
            EventType.AGENT_STARTED,
            agent_name,
            f"Agent {agent_name} started",
            EventSeverity.INFO,
            data,
        )
    
    async def log_agent_stopped(self, agent_name: str, data: Optional[Dict[str, Any]] = None) -> str:
        """Log agent stop event."""
        return await self.log_event(
            EventType.AGENT_STOPPED,
            agent_name,
            f"Agent {agent_name} stopped",
            EventSeverity.INFO,
            data,
        )
    
    async def log_task_assigned(
        self, agent_name: str, task_id: str, data: Optional[Dict[str, Any]] = None
    ) -> str:
        """Log task assignment event."""
        event_data = {"task_id": task_id, "agent_name": agent_name}
        if data:
            event_data.update(data)
        
        return await self.log_event(
            EventType.TASK_ASSIGNED,
            agent_name,
            f"Task {task_id} assigned to {agent_name}",
            EventSeverity.INFO,
            event_data,
        )
    
    async def log_error(
        self, agent_name: str, error: Exception, data: Optional[Dict[str, Any]] = None
    ) -> str:
        """Log error event."""
        event_data = {"error_type": type(error).__name__, "error_message": str(error)}
        if data:
            event_data.update(data)
        
        return await self.log_event(
            EventType.ERROR_OCCURRED,
            agent_name,
            f"Error in {agent_name}: {str(error)}",
            EventSeverity.ERROR,
            event_data,
        )
    
    async def log_llm_call(self, agent_name: str, model: str, prompt_tokens: int) -> str:
        """Log LLM call event."""
        return await self.log_event(
            EventType.LLM_CALL_STARTED,
            agent_name,
            f"LLM call to {model} started",
            EventSeverity.INFO,
            {"model": model, "prompt_tokens": prompt_tokens},
        )


class ExecutionHistory(BaseModel):
    """Tracks execution history for auditability."""
    
    class ExecutionRecord(BaseModel):
        """Single execution record."""
        
        task_id: str
        agent_id: str
        agent_name: str
        start_time: datetime
        end_time: Optional[datetime] = None
        success: bool = False
        result: Optional[Dict[str, Any]] = None
        error: Optional[str] = None
        token_usage: Optional[Dict[str, int]] = None
        execution_time: float = 0.0
    
    executions: List[ExecutionRecord] = Field(default_factory=list)
    
    def add_execution(
        self,
        task_id: str,
        agent_id: str,
        agent_name: str,
        start_time: datetime,
        success: bool,
        result: Optional[Dict[str, Any]] = None,
        error: Optional[str] = None,
        token_usage: Optional[Dict[str, int]] = None,
    ) -> None:
        """Add a new execution record."""
        end_time = datetime.now()
        execution_time = (end_time - start_time).total_seconds()
        
        record = self.ExecutionRecord(
            task_id=task_id,
            agent_id=agent_id,
            agent_name=agent_name,
            start_time=start_time,
            end_time=end_time,
            success=success,
            result=result,
            error=error,
            token_usage=token_usage,
            execution_time=execution_time,
        )
        
        self.executions.append(record)
    
    def get_executions_for_task(self, task_id: str) -> List[ExecutionRecord]:
        """Get all executions for a specific task."""
        return [ex for ex in self.executions if ex.task_id == task_id]
    
    def get_executions_by_agent(self, agent_id: str) -> List[ExecutionRecord]:
        """Get all executions by a specific agent."""
        return [ex for ex in self.executions if ex.agent_id == agent_id]
    
    def get_success_rate(self) -> float:
        """Calculate overall success rate."""
        if not self.executions:
            return 0.0
        successful = sum(1 for ex in self.executions if ex.success)
        return successful / len(self.executions)
    
    def get_average_execution_time(self) -> float:
        """Calculate average execution time."""
        if not self.executions:
            return 0.0
        return sum(ex.execution_time for ex in self.executions) / len(self.executions)
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert execution history to dictionary."""
        return {
            "total_executions": len(self.executions),
            "success_rate": self.get_success_rate(),
            "average_execution_time": self.get_average_execution_time(),
            "executions": [ex.model_dump() for ex in self.executions],
        }


class AgentCommunication:
    """Handles agent-to-agent communication."""
    
    def __init__(self, event_logger: EventLogger):
        """Initialize agent communication system."""
        self.event_logger = event_logger
        self._message_queue: asyncio.Queue[AgentEvent] = asyncio.Queue()
        self._handlers: Dict[EventType, List[Callable]] = {}
    
    async def send_message(
        self,
        sender: str,
        recipient: str,
        message: str,
        data: Optional[Dict[str, Any]] = None,
        message_type: EventType = EventType.AGENT_COMMUNICATION,
    ) -> None:
        """Send a message to another agent."""
        event_data = {
            "sender": sender,
            "recipient": recipient,
            "message": message,
        }
        if data:
            event_data.update(data)
        
        await self.event_logger.log_event(
            message_type,
            sender,
            f"Message to {recipient}: {message}",
            EventSeverity.INFO,
            event_data,
        )
    
    async def broadcast_message(
        self, sender: str, message: str, data: Optional[Dict[str, Any]] = None
    ) -> None:
        """Broadcast a message to all agents."""
        event_data = {"sender": sender, "message": message}
        if data:
            event_data.update(data)
        
        await self.event_logger.log_event(
            EventType.AGENT_COMMUNICATION,
            sender,
            f"Broadcast: {message}",
            EventSeverity.INFO,
            event_data,
        )
    
    def register_handler(self, event_type: EventType, handler: Callable) -> None:
        """Register an event handler."""
        if event_type not in self._handlers:
            self._handlers[event_type] = []
        self._handlers[event_type].append(handler)
    
    def unregister_handler(self, event_type: EventType, handler: Callable) -> None:
        """Unregister an event handler."""
        if event_type in self._handlers:
            self._handlers[event_type].remove(handler)
            if not self._handlers[event_type]:
                del self._handlers[event_type]


class InMemoryEventStore:
    """In-memory event store for testing and development."""
    
    def __init__(self, max_events: int = 10000):
        """Initialize in-memory event store."""
        self.max_events = max_events
        self._events: List[AgentEvent] = []
        self._agent_index: Dict[str, List[AgentEvent]] = {}
        self._type_index: Dict[EventType, List[AgentEvent]] = {}
    
    async def store(self, event: AgentEvent) -> None:
        """Store an event."""
        self._events.append(event)
        
        # Maintain indexes
        if event.source_agent not in self._agent_index:
            self._agent_index[event.source_agent] = []
        self._agent_index[event.source_agent].append(event)
        
        if event.event_type not in self._type_index:
            self._type_index[event.event_type] = []
        self._type_index[event.event_type].append(event)
        
        # Maintain size limit
        if len(self._events) > self.max_events:
            await self._cleanup_old_events()
    
    async def _cleanup_old_events(self) -> None:
        """Clean up old events to maintain size limit."""
        # Remove oldest 20% of events
        remove_count = len(self._events) // 5
        old_events = self._events[:remove_count]
        self._events = self._events[remove_count:]
        
        # Rebuild indexes
        self._agent_index.clear()
        self._type_index.clear()
        
        for event in self._events:
            if event.source_agent not in self._agent_index:
                self._agent_index[event.source_agent] = []
            self._agent_index[event.source_agent].append(event)
            
            if event.event_type not in self._type_index:
                self._type_index[event.event_type] = []
            self._type_index[event.event_type].append(event)
    
    async def get_events_by_agent(
        self, agent_name: str, event_type: Optional[EventType] = None
    ) -> List[AgentEvent]:
        """Get events for a specific agent."""
        events = self._agent_index.get(agent_name, [])
        
        if event_type:
            events = [e for e in events if e.event_type == event_type]
        
        return events
    
    async def get_events_by_type(self, event_type: EventType) -> List[AgentEvent]:
        """Get events of a specific type."""
        return self._type_index.get(event_type, [])
    
    async def get_recent_events(self, limit: int = 100) -> List[AgentEvent]:
        """Get recent events."""
        return self._events[-limit:]