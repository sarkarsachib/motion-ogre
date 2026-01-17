#!/usr/bin/env python3
"""
Bridge script for executing Motion Ogre agents from Node.js backend.

This script is spawned by the Node.js backend and handles agent execution,
progress reporting, and result serialization.
"""

import argparse
import json
import sys
import asyncio
from typing import Dict, Any, Optional

from motion_ogre_agents import AgentRegistry, AgentState
from motion_ogre_agents.agents import (
    DirectorAgent,
    CutAgent,
    MotionAgent,
    ColorAgent,
    AudioAgent,
)


async def execute_agent(
    agent_type: str,
    intent: str,
    execution_id: str,
    context: Optional[Dict[str, Any]] = None,
    timeline_json: Optional[str] = None,
) -> Dict[str, Any]:
    """
    Execute an agent with the given intent and context.

    Args:
        agent_type: Type of agent to execute (e.g., "CutAgent", "MotionAgent")
        intent: Natural language description of the task
        execution_id: Unique identifier for this execution
        context: Additional context for the agent
        timeline_json: Serialized timeline state (optional)

    Returns:
        Agent execution result as a dictionary
    """
    context = context or {}

    # Create agent registry
    registry = AgentRegistry()

    # Register available agents
    registry.register("DirectorAgent", DirectorAgent)
    registry.register("CutAgent", CutAgent)
    registry.register("MotionAgent", MotionAgent)
    registry.register("ColorAgent", ColorAgent)
    registry.register("AudioAgent", AudioAgent)

    # Get agent factory
    agent_factory = registry.get_agent(agent_type)
    if not agent_factory:
        return {
            "success": False,
            "error": f"Unknown agent type: {agent_type}",
            "agent_type": agent_type,
            "execution_id": execution_id,
        }

    # Create agent instance
    try:
        agent = agent_factory(
            agent_id=f"{agent_type}_{execution_id}",
            llm_client=None,  # Will use default environment configuration
        )
    except Exception as e:
        return {
            "success": False,
            "error": f"Failed to create agent: {str(e)}",
            "agent_type": agent_type,
            "execution_id": execution_id,
        }

    # Create task
    from motion_ogre_agents.agent import Task

    task = Task(
        id=execution_id,
        description=intent,
        context=context or {},
    )

    try:
        # Execute the agent
        result = await agent.run(task)

        return {
            "success": True,
            "result": result,
            "agent_type": agent_type,
            "execution_id": execution_id,
            "state": agent.state.value,
        }

    except Exception as e:
        import traceback

        return {
            "success": False,
            "error": f"Agent execution failed: {str(e)}\n{traceback.format_exc()}",
            "agent_type": agent_type,
            "execution_id": execution_id,
            "state": agent.state.value,
        }


async def main():
    """Main entry point for the bridge script."""
    parser = argparse.ArgumentParser(description="Execute Motion Ogre agent")
    parser.add_argument("--agent-type", required=True, help="Type of agent to execute")
    parser.add_argument("--intent", required=True, help="Natural language intent")
    parser.add_argument("--execution-id", required=True, help="Unique execution ID")
    parser.add_argument("--context", help="JSON context dictionary")
    parser.add_argument("--timeline", help="Serialized timeline JSON")

    args = parser.parse_args()

    # Parse context
    context = None
    if args.context:
        try:
            context = json.loads(args.context)
        except json.JSONDecodeError as e:
            print(json.dumps({"error": f"Invalid context JSON: {str(e)}"}), file=sys.stderr)
            sys.exit(1)

    # Parse timeline
    timeline = None
    if args.timeline:
        try:
            timeline = json.loads(args.timeline)
            if context is None:
                context = {}
            context["timeline"] = timeline
        except json.JSONDecodeError as e:
            print(json.dumps({"error": f"Invalid timeline JSON: {str(e)}"}), file=sys.stderr)
            sys.exit(1)

    # Report progress (0%)
    progress_update = {
        "execution_id": args.execution_id,
        "agent_type": args.agent_type,
        "progress": 0,
        "message": "Starting agent execution...",
    }
    print(f"PROGRESS:{json.dumps(progress_update)}", flush=True)

    # Execute agent
    try:
        # Report progress (50% - planning phase)
        progress_update["progress"] = 50
        progress_update["message"] = "Planning execution..."
        print(f"PROGRESS:{json.dumps(progress_update)}", flush=True)

        result = await execute_agent(
            agent_type=args.agent_type,
            intent=args.intent,
            execution_id=args.execution_id,
            context=context,
            timeline_json=args.timeline,
        )

        # Report progress (100% - complete)
        progress_update["progress"] = 100
        progress_update["message"] = "Execution complete"
        print(f"PROGRESS:{json.dumps(progress_update)}", flush=True)

        # Output final result
        print(json.dumps(result), flush=True)
        sys.exit(0)

    except Exception as e:
        import traceback

        error_result = {
            "success": False,
            "error": str(e),
            "traceback": traceback.format_exc(),
            "agent_type": args.agent_type,
            "execution_id": args.execution_id,
        }
        print(json.dumps(error_result), flush=True)
        sys.exit(1)


if __name__ == "__main__":
    asyncio.run(main())
