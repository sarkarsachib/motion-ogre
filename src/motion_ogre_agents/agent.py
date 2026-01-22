"""Base Agent class and functionality."""

from abc import ABC, abstractmethod
from typing import Any, Dict, List, Optional, Type, Union
from pydantic import BaseModel, Field
from datetime import datetime
from tenacity import Retrying, stop_after_attempt, wait_exponential
import asyncio

from .state import AgentState, AgentStateTransition
from .events import EventLogger, ExecutionHistory
from .llm import LLMClient


class AgentMetadata(BaseModel):
    """Metadata about an agent."""
    
    name: str
    description: str
    capabilities: List[str]
    version: Optional[str] = "1.0.0"
    author: Optional[str] = None


class AgentConfig(BaseModel):
    """Configuration for an agent."""
    
    max_retry_attempts: int = Field(default=3, ge=1)
    retry_base_delay: float = Field(default=1.0, gt=0)
    max_timeout: Optional[float] = Field(default=300.0, gt=0)
    enable_cost_tracking: bool = True
    enable_logging: bool = True


class Task(BaseModel):
    """Represents a task to be executed by an agent."""
    
    id: str
    description: str
    context: Dict[str, Any]
    priority: int = Field(default=0, ge=0, le=10)
    timeout: Optional[float] = None
    depends_on: List[str] = Field(default_factory=list)


class ExecutionResult(BaseModel):
    """Result of task execution."""
    
    success: bool
    result: Optional[Dict[str, Any]] = None
    error: Optional[str] = None
    execution_time: float = 0.0
    token_usage: Optional[Dict[str, int]] = None
    timestamp: datetime = Field(default_factory=datetime.now)


class Agent(ABC):
    """Abstract base class for all agents."""
    
    def __init__(
        self,
        metadata: AgentMetadata,
        config: Optional[AgentConfig] = None,
        llm_client: Optional[LLMClient] = None,
        event_logger: Optional[EventLogger] = None,
        agent_id: Optional[str] = None,
    ):
        """Initialize an agent."""
        self.metadata = metadata
        self.config = config or AgentConfig()
        self.llm_client = llm_client or LLMClient()
        self.event_logger = event_logger or EventLogger()
        self.agent_id = agent_id or f"{metadata.name}_{datetime.now().strftime('%Y%m%d_%H%M%S')}"
        
        self._state = AgentState.IDLE
        self._state_history: List[AgentStateTransition] = []
        self._execution_history = ExecutionHistory()
        self._current_task: Optional[Task] = None
        
        # Register transition for initial state
        self._record_state_transition(AgentState.IDLE, "initialized")
    
    @property
    def state(self) -> AgentState:
        """Get current agent state."""
        return self._state
    
    @property
    def execution_history(self) -> ExecutionHistory:
        """Get execution history."""
        return self._execution_history
    
    def _transition_to(self, new_state: AgentState, reason: Optional[str] = None) -> None:
        """Transition agent to a new state."""
        if new_state == self._state:
            return
        
        self._record_state_transition(new_state, reason)
        self._state = new_state
        
        # Log state transition
        if self.config.enable_logging:
            self.event_logger.log_event(
                "agent_state_transition",
                {
                    "agent_id": self.agent_id,
                    "agent_name": self.metadata.name,
                    "from_state": self.state_history[-2].from_state.value if len(self.state_history) >= 2 else None,
                    "to_state": new_state.value,
                    "reason": reason,
                }
            )
    
    def _record_state_transition(self, new_state: AgentState, reason: Optional[str] = None) -> None:
        """Record state transition."""
        from_state = self._state
        transition = AgentStateTransition(
            from_state=from_state,
            to_state=new_state,
            timestamp=datetime.now(),
            reason=reason,
        )
        self._state_history.append(transition)
    
    @property
    def state_history(self) -> List[AgentStateTransition]:
        """Get state transition history."""
        return self._state_history
    
    async def process(self, task: Task) -> ExecutionResult:
        """Main processing loop for an agent."""
        self._current_task = task
        start_time = datetime.now()
        
        try:
            # Transition to planning state
            self._transition_to(AgentState.PLANNING, f"Started processing task: {task.id}")
            
            # Planning phase
            plan_result = await self._execute_with_retry(
                self.plan, task, f"Planning for task {task.id}", max_attempts=2
            )
            
            # Execution phase
            self._transition_to(AgentState.EXECUTING, "Executing task plan")
            execution_result = await self._execute_with_retry(
                self.execute, task, plan_result, f"Executing task {task.id}"
            )
            
            # Critique phase
            self._transition_to(AgentState.CRITIQUING, "Critiquing execution result")
            critique_result = await self._execute_with_retry(
                self.critique, task, execution_result, f"Critiquing task {task.id}"
            )
            
            # Process results
            final_result = self._create_execution_result(
                success=critique_result.get("success", True),
                result=critique_result.get("result", execution_result),
                start_time=start_time,
            )
            
            self._transition_to(AgentState.COMPLETED, "Task processing completed")
            return final_result
            
        except Exception as e:
            error_msg = f"Task processing failed: {str(e)}"
            self._transition_to(AgentState.ERROR, error_msg)
            
            return self._create_execution_result(
                success=False,
                error=error_msg,
                start_time=start_time,
            )
        finally:
            self._transition_to(AgentState.IDLE, "Returning to idle state")
            self._current_task = None
    
    async def _execute_with_retry(
        self,
        method: callable,
        *args,
        operation_name: str = "",
        max_attempts: Optional[int] = None,
    ) -> Any:
        """Execute a method with retry logic."""
        max_attempts = max_attempts or self.config.max_retry_attempts
        
        retry_decorator = Retrying(
            stop=stop_after_attempt(max_attempts),
            wait=wait_exponential(multiplier=self.config.retry_base_delay),
        )
        
        for attempt in range(max_attempts):
            try:
                result = await method(*args)
                
                # Log successful execution
                if self.config.enable_logging:
                    self.event_logger.log_event(
                        "operation_success",
                        {
                            "agent_id": self.agent_id,
                            "operation": operation_name,
                            "attempt": attempt + 1,
                            "max_attempts": max_attempts,
                        }
                    )
                
                return result
                
            except Exception as e:
                error_msg = f"Attempt {attempt + 1}/{max_attempts} failed: {str(e)}"
                
                if self.config.enable_logging:
                    self.event_logger.log_event(
                        "operation_retry",
                        {
                            "agent_id": self.agent_id,
                            "operation": operation_name,
                            "attempt": attempt + 1,
                            "max_attempts": max_attempts,
                            "error": error_msg,
                        }
                    )
                
                if attempt == max_attempts - 1:  # Last attempt
                    raise
                
                # Wait before retry (exponential backoff handled by retry_decorator)
                await asyncio.sleep(
                    self.config.retry_base_delay * (2 ** attempt)
                )
    
    def _create_execution_result(
        self,
        success: bool,
        result: Optional[Dict[str, Any]] = None,
        error: Optional[str] = None,
        start_time: Optional[datetime] = None,
    ) -> ExecutionResult:
        """Create an execution result."""
        end_time = datetime.now()
        execution_time = (end_time - start_time).total_seconds() if start_time else 0.0
        
        exec_result = ExecutionResult(
            success=success,
            result=result,
            error=error,
            execution_time=execution_time,
        )
        
        # Record in execution history
        self._execution_history.add_execution(
            task_id=self._current_task.id if self._current_task else "unknown",
            result=exec_result,
        )
        
        return exec_result
    
    @abstractmethod
    async def plan(self, task: Task) -> Dict[str, Any]:
        """Plan how to execute a task. Must be implemented by subclasses."""
        pass
    
    @abstractmethod
    async def execute(self, task: Task, plan: Dict[str, Any]) -> Dict[str, Any]:
        """Execute a task according to the plan. Must be implemented by subclasses."""
        pass
    
    @abstractmethod
    async def critique(self, task: Task, execution_result: Dict[str, Any]) -> Dict[str, Any]:
        """Critique the execution result and suggest improvements. Must be implemented by subclasses."""
        pass

    async def run(self, task: Task) -> Dict[str, Any]:
        """
        Convenience method that runs the full agent workflow.
        Alias for process() method for backward compatibility.
        """
        result = await self.process(task)
        return {
            "success": result.success,
            "result": result.result,
            "error": result.error,
            "execution_time": result.execution_time,
            "state": self.state.value,
        }
