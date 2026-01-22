"""Director agent for coordinating multi-agent editing workflows."""

from typing import Dict, Any, List, Optional
from ..agent import Agent, AgentConfig, AgentMetadata, Task, ExecutionResult
from ..registry import AgentRegistry
from ..events import EventLogger


class DirectorAgent(Agent):
    """
    Director agent that coordinates multiple specialized editing agents.
    
    The DirectorAgent analyzes editing intents and delegates to specialized agents
    (CutAgent, MotionAgent, ColorAgent, AudioAgent) based on the task requirements.
    """

    def __init__(
        self,
        agent_registry: Optional[AgentRegistry] = None,
        **kwargs
    ):
        """Initialize Director agent."""
        metadata = AgentMetadata(
            name="DirectorAgent",
            description="Coordinates multi-agent editing workflows and delegates to specialized agents",
            capabilities=[
                "analyze_intent",
                "plan_workflow",
                "coordinate_agents",
                "merge_results",
                "ensure_coherence",
            ],
        )
        
        super().__init__(metadata=metadata, **kwargs)
        self.agent_registry = agent_registry or AgentRegistry()
        self.execution_history: List[Dict[str, Any]] = []

    async def plan(self, task: Task) -> Dict[str, Any]:
        """
        Analyze intent and create workflow plan.
        
        Determines which specialized agents are needed and in what order.
        """
        # Analyze the intent to determine required agents
        required_agents = self._analyze_intent(task.description, task.context)
        
        # Create execution plan
        workflow_steps = []
        for agent_type in required_agents:
            workflow_steps.append({
                "agent_type": agent_type,
                "task_description": self._refine_task_for_agent(task.description, agent_type),
                "depends_on": []  # Could add dependencies between steps
            })
        
        if self.config.enable_logging:
            await self.event_logger.log_event(
                "director_plan_created",
                self.agent_id,
                f"Created workflow plan with {len(workflow_steps)} steps",
                data={
                    "task_id": task.id,
                    "workflow_steps": workflow_steps,
                    "required_agents": required_agents,
                },
            )
        
        return {
            "plan": workflow_steps,
            "required_agents": required_agents,
            "agent_id": self.agent_id,
            "task_id": task.id,
        }

    async def execute(self, task: Task, plan: Dict[str, Any]) -> Dict[str, Any]:
        """
        Execute workflow by coordinating multiple agents.
        
        Delegates tasks to specialized agents and collects their results.
        """
        workflow_steps = plan.get("plan", [])
        results = []
        
        for i, step in enumerate(workflow_steps):
            agent_type = step["agent_type"]
            task_description = step["task_description"]
            
            # Get the specialized agent
            agent_factory = self.agent_registry.get_agent(agent_type)
            if not agent_factory:
                error_result = {
                    "agent_type": agent_type,
                    "success": False,
                    "error": f"Agent not found: {agent_type}",
                }
                results.append(error_result)
                continue
            
            # Create agent instance
            try:
                agent = agent_factory(
                    agent_id=f"{agent_type}_{task.id}_step{i}",
                    llm_client=self.llm_client,
                    event_logger=self.event_logger,
                    config=self.config,
                )
            except Exception as e:
                error_result = {
                    "agent_type": agent_type,
                    "success": False,
                    "error": f"Failed to create agent: {str(e)}",
                }
                results.append(error_result)
                continue
            
            # Execute the specialized task
            try:
                subtask = Task(
                    id=f"{task.id}_{agent_type}_{i}",
                    description=task_description,
                    context={
                        **task.context,
                        "director_task": task.description,
                    },
                )
                
                agent_result = await agent.run(subtask)
                
                results.append({
                    "agent_type": agent_type,
                    "success": True,
                    "result": agent_result,
                })
                
            except Exception as e:
                results.append({
                    "agent_type": agent_type,
                    "success": False,
                    "error": str(e),
                })
        
        # Merge results from all agents
        merged_result = self._merge_results(results)
        
        # Store in execution history
        self.execution_history.append({
            "task_id": task.id,
            "workflow_steps": len(workflow_steps),
            "successful_steps": sum(1 for r in results if r.get("success")),
            "results": results,
            "timestamp": task.context.get("timestamp") if task.context else None,
        })
        
        if self.config.enable_logging:
            await self.event_logger.log_event(
                "director_workflow_completed",
                self.agent_id,
                f"Workflow completed: {sum(1 for r in results if r.get('success'))}/{len(results)} steps successful",
                data={
                    "task_id": task.id,
                    "results": results,
                    "merged_result": merged_result,
                },
            )
        
        return {
            "workflow_results": results,
            "merged_result": merged_result,
            "task_id": task.id,
        }

    async def critique(self, task: Task, execution_result: Dict[str, Any]) -> Dict[str, Any]:
        """
        Critique the overall workflow execution.
        
        Evaluates whether the workflow achieved its goals and suggests improvements.
        """
        workflow_results = execution_result.get("workflow_results", [])
        successful = sum(1 for r in workflow_results if r.get("success"))
        total = len(workflow_results)
        
        # Basic quality assessment
        quality_score = int((successful / total) * 10) if total > 0 else 0
        
        # Check for specific issues
        issues = []
        for result in workflow_results:
            if not result.get("success"):
                issues.append(f"{result['agent_type']} failed: {result.get('error', 'Unknown error')}")
        
        # Suggest improvements
        improvements = []
        if quality_score < 8:
            improvements.append("Consider adding more specific context for each agent")
        if any("failed" in issue.lower() for issue in issues):
            improvements.append("Review agent initialization and dependencies")
        if total == 0:
            improvements.append("Ensure intent analysis is matching agent capabilities")
        
        return {
            "success": quality_score >= 7,
            "quality_score": quality_score,
            "retry_needed": quality_score < 5,
            "issues_found": issues,
            "improvements_suggested": improvements,
            "workflow_summary": {
                "total_steps": total,
                "successful_steps": successful,
                "failed_steps": total - successful,
            },
        }

    def _analyze_intent(self, intent: str, context: Dict[str, Any]) -> List[str]:
        """
        Analyze editing intent to determine required agents.
        
        This is a simplified heuristic-based approach. In production,
        this would use the LLM to analyze the intent more intelligently.
        """
        intent_lower = intent.lower()
        required_agents = []
        
        # Keywords for different agent types
        cut_keywords = [
            "cut", "trim", "split", "remove", "delete", "extract",
            "clip", "segment", "duration", "length"
        ]
        
        motion_keywords = [
            "transition", "animate", "move", "motion", "pan", "zoom",
            "rotate", "stabilize", "smooth", "effect", "animation"
        ]
        
        color_keywords = [
            "color", "grade", "correct", "adjust", "filter", "contrast",
            "brightness", "saturation", "lookup", "lut", "look"
        ]
        
        audio_keywords = [
            "audio", "sound", "music", "voice", "mix", "volume",
            "fade", "noise", "sync", "dialogue", "level"
        ]
        
        # Check for keywords
        for keyword in cut_keywords:
            if keyword in intent_lower and "CutAgent" not in required_agents:
                required_agents.append("CutAgent")
                break
        
        for keyword in motion_keywords:
            if keyword in intent_lower and "MotionAgent" not in required_agents:
                required_agents.append("MotionAgent")
                break
        
        for keyword in color_keywords:
            if keyword in intent_lower and "ColorAgent" not in required_agents:
                required_agents.append("ColorAgent")
                break
        
        for keyword in audio_keywords:
            if keyword in intent_lower and "AudioAgent" not in required_agents:
                required_agents.append("AudioAgent")
                break
        
        # Default to CutAgent if no specific keywords found
        if not required_agents:
            required_agents.append("CutAgent")
        
        return required_agents

    def _refine_task_for_agent(self, intent: str, agent_type: str) -> str:
        """
        Refine the overall intent into a specific task for a specialized agent.
        """
        # Add context specific to the agent type
        agent_contexts = {
            "CutAgent": "Focus on cutting, trimming, and temporal operations.",
            "MotionAgent": "Focus on motion effects, transitions, and spatial transformations.",
            "ColorAgent": "Focus on color grading, correction, and visual appearance.",
            "AudioAgent": "Focus on audio editing, mixing, and synchronization.",
        }
        
        context = agent_contexts.get(agent_type, "")
        return f"{intent} {context}"

    def _merge_results(self, results: List[Dict[str, Any]]) -> Dict[str, Any]:
        """
        Merge results from multiple agents into a coherent output.
        """
        merged = {
            "success": all(r.get("success", False) for r in results),
            "agent_results": {},
            "summary": "",
        }
        
        # Collect individual agent results
        for result in results:
            agent_type = result["agent_type"]
            merged["agent_results"][agent_type] = result.get("result", result.get("error"))
        
        # Create summary
        successful = [r["agent_type"] for r in results if r.get("success")]
        failed = [r["agent_type"] for r in results if not r.get("success")]
        
        summary_parts = []
        if successful:
            summary_parts.append(f"Successfully executed: {', '.join(successful)}")
        if failed:
            summary_parts.append(f"Failed: {', '.join(failed)}")
        
        merged["summary"] = "; ".join(summary_parts)
        
        return merged

    def get_execution_history(self) -> List[Dict[str, Any]]:
        """Get the agent's execution history."""
        return self.execution_history.copy()

    def clear_execution_history(self) -> None:
        """Clear the execution history."""
        self.execution_history.clear()
