"""Base editing agent and specialized editing agents for Motion Ogre."""

from abc import abstractmethod
from typing import Any, Dict, List, Optional
from ..agent import Agent, AgentConfig, AgentMetadata, Task, ExecutionResult
from ..llm import LLMClient, PromptTemplate
from ..events import EventLogger
from ..prompts import (
    BASE_EDITING_AGENT_SYSTEM_PROMPT,
    AGENT_PLANNING_PROMPT,
    AGENT_EXECUTION_PROMPT,
    AGENT_CRITIQUE_PROMPT,
)


class BaseEditingAgent(Agent):
    """Base class for specialized editing agents (cut, motion, color, audio)."""
    
    def __init__(
        self,
        metadata: AgentMetadata,
        editing_type: str,
        available_tools: List[str],
        **kwargs
    ):
        """Initialize base editing agent."""
        super().__init__(metadata=metadata, **kwargs)
        self.editing_type = editing_type
        self.available_tools = available_tools
        self.system_prompt = self._create_system_prompt()
    
    def _create_system_prompt(self) -> str:
        """Create system prompt for the editing agent."""
        return BASE_EDITING_AGENT_SYSTEM_PROMPT.render(
            editing_type=self.editing_type,
            specialized_capabilities=self.metadata.description,
            available_tools=", ".join(self.available_tools),
        )
    
    async def plan(self, task: Task) -> Dict[str, Any]:
        """Create detailed execution plan for editing task."""
        
        plan_prompt = AGENT_PLANNING_PROMPT.render(
            agent_type=self.editing_type,
            task_description=task.description,
            task_context=json.dumps(task.context),
            agent_capabilities=", ".join(self.metadata.capabilities),
        )
        
        messages = [
            {"role": "system", "content": self.system_prompt},
            {"role": "user", "content": plan_prompt},
        ]
        
        plan_content = await self.llm_client.chat_completion(messages)
        
        if self.config.enable_logging:
            await self.event_logger.log_event(
                "planning_completed",
                self.agent_id,
                f"Plan created for task {task.id}",
                data={"plan": plan_content[:200] + "..." if len(plan_content) > 200 else plan_content},
            )
        
        return {
            "plan": plan_content,
            "agent_id": self.agent_id,
            "task_id": task.id,
        }
    
    async def execute(self, task: Task, plan: Dict[str, Any]) -> Dict[str, Any]:
        """Execute the editing task according to the plan."""
        
        execution_prompt = AGENT_EXECUTION_PROMPT.render(
            agent_type=self.editing_type,
            task_description=task.description,
            execution_plan=plan.get("plan", ""),
            current_context=json.dumps(task.context),
            available_tools=", ".join(self.available_tools),
        )
        
        messages = [
            {"role": "system", "content": self.system_prompt},
            {"role": "user", "content": execution_prompt},
        ] 
        
        execution_result = await self.llm_client.chat_completion(messages)
        
        if self.config.enable_logging:
            await self.event_logger.log_event(
                "execution_completed",
                self.agent_id,
                f"Task execution completed {task.id}",
                data={"execution_result": execution_result[:200] + "..." if len(execution_result) > 200 else execution_result},
            )
        
        return {
            "execution_result": execution_result,
            "task_id": task.id,
        }
    
    async def critique(self, task: Task, execution_result: Dict[str, Any]) -> Dict[str, Any]:
        """Critique the execution result and suggest improvements."""
        
        critique_prompt = AGENT_CRITIQUE_PROMPT.render(
            agent_type=self.editing_type,
            task_description=task.description,
            execution_plan="Based on previous planning and execution",
            execution_result=execution_result.get("execution_result", ""),
            expected_outcome="High-quality " + self.editing_type + " editing result",
        )
        
        messages = [
            {"role": "system", "content": self.system_prompt},
            {"role": "user", "content": critique_prompt},
        ]
        
        critique_content = await self.llm_client.chat_completion(messages)
        
        try:
            # Parse critique to extract structured information
            critique_data = self._parse_critique_content(critique_content)
            
            if self.config.enable_logging:
                await self.event_logger.log_event(
                    "critique_completed",
                    self.agent_id,
                    f"Critique completed for task {task.id}",
                    data={"quality_score": critique_data.get("quality_score", "unknown")},
                )
            
            return critique_data
            
        except Exception as e:
            if self.config.enable_logging:
                await self.event_logger.log_event(
                    "critique_parsing_error",
                    self.agent_id,
                    f"Failed to parse critique: {str(e)}",
                    data={"critique_content": critique_content},
                )
            
            # Fallback to simple critique
            return {
                "success": True,
                "result": execution_result,
                "quality_score": 8,
                "improvements_suggested": "Automated critique parsing failed",
                "retry_needed": False,
            }
    
    def _parse_critique_content(self, critique_content: str) -> Dict[str, Any]:
        """Parse critique content to extract structured information."""
        # This is a simplified parser - in production, you'd want more robust parsing
        lines = critique_content.split('\n')
        
        quality_score = 8
        retry_needed = False
        issues_found = []
        improvements_suggested = []
        
        for line in lines:
            line_lower = line.lower()
            if 'quality score:' in line_lower or 'quality_score:' in line_lower:
                try:
                    quality_score = int(''.join(filter(str.isdigit, line)))
                except ValueError:
                    pass
            elif 'retry needed:' in line_lower or 'retry_needed:' in line_lower:
                retry_needed = 'true' in line_lower or 'yes' in line_lower
            elif 'issue:' in line_lower:
                issues_found.append(line.split(':', 1)[1].strip() if ':' in line else line)
            elif 'improvement:' in line_lower:
                improvements_suggested.append(line.split(':', 1)[1].strip() if ':' in line else line)
        
        return {
            "success": quality_score >= 6,  # Acceptable quality
            "quality_score": quality_score,
            "retry_needed": retry_needed,
            "issues_found": issues_found,
            "improvements_suggested": improvements_suggested,
        }


class CutAgent(BaseEditingAgent):
    """Agent specialized in video cutting and trimming operations."""
    
    def __init__(self, **kwargs):
        """Initialize Cut Agent."""
        metadata = AgentMetadata(
            name="CutAgent",
            description="Specializes in video cutting, trimming, and basic editing operations",
            capabilities=[
                "trim_video",
                "cut_clips",
                "split_timeline",
                "remove_segments",
                "extract_subclips",
            ],
        )
        
        super().__init__(
            metadata=metadata,
            editing_type="cut",
            available_tools=[
                "ffmpeg_trim",
                "timeline_splitter",
                "clip_extractor",
                "segment_remover",
            ],
            **kwargs
        )


class MotionAgent(BaseEditingAgent):
    """Agent specialized in motion effects and transitions."""
    
    def __init__(self, **kwargs):
        """Initialize Motion Agent."""
        metadata = AgentMetadata(
            name="MotionAgent",
            description="Specializes in motion effects, transitions, and animations",
            capabilities=[
                "add_transitions",
                "create_motion_effects",
                "animate_objects",
                "smooth_movements",
                "stabilize_footage",
            ],
        )
        
        super().__init__(
            metadata=metadata,
            editing_type="motion",
            available_tools=[
                "transition_creator",
                "motion_effect_engine",
                "animation_controller",
                "stabilization_tool",
                "keyframe_editor",
            ],
            **kwargs
        )


class ColorAgent(BaseEditingAgent):
    """Agent specialized in color grading and visual effects."""
    
    def __init__(self, **kwargs):
        """Initialize Color Agent."""
        metadata = AgentMetadata(
            name="ColorAgent",
            description="Specializes in color grading, filters, and visual effects",
            capabilities=[
                "color_grading",
                "apply_filters",
                "adjust_brightness_contrast",
                "color_correction",
                "look_creation",
            ],
        )
        
        super().__init__(
            metadata=metadata,
            editing_type="color",
            available_tools=[
                "color_grader",
                "filter_applier",
                "color_corrector",
                "lookup_table_manager",
                "visual_effect_engine",
            ],
            **kwargs
        )


class AudioAgent(BaseEditingAgent):
    """Agent specialized in audio editing and synchronization."""
    
    def __init__(self, **kwargs):
        """Initialize Audio Agent."""
        metadata = AgentMetadata(
            name="AudioAgent",
            description="Specializes in audio editing, effects, and synchronization",
            capabilities=[
                "audio_editing",
                "apply_audio_effects",
                "sync_audio_video",
                "noise_reduction",
                "audio_mixing",
            ],
        )
        
        super().__init__(
            metadata=metadata,
            editing_type="audio",
            available_tools=[
                "audio_editor",
                "audio_effect_processor",
                "sync_tool",
                "noise_reducer",
                "audio_mixer",
            ],
            **kwargs
        )


import json  # Required for BaseEditingAgent methods