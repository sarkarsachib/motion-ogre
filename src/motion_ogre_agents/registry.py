"""Agent registry and discovery system."""

from typing import Dict, List, Optional, Type
from .agent import Agent, AgentMetadata


class AgentDiscoveryError(Exception):
    """Exception raised when agent discovery fails."""
    pass


class AgentRegistry:
    """Registry for managing available agents."""
    
    def __init__(self):
        """Initialize agent registry."""
        self._agents: Dict[str, Type[Agent]] = {}
        self._agent_metadata: Dict[str, AgentMetadata] = {}
        self._agent_instances: Dict[str, Agent] = {}
    
    def register_agent(
        self, agent_class: Type[Agent], metadata: Optional[AgentMetadata] = None
    ) -> None:
        """Register an agent class with the registry."""
        if metadata is None:
            # Try to get metadata from class
            if hasattr(agent_class, "metadata"):
                metadata = agent_class.metadata
            else:
                raise AgentDiscoveryError(
                    f"No metadata provided for agent {agent_class.__name__}"
                )
        
        agent_name = metadata.name
        
        if agent_name in self._agents:
            raise AgentDiscoveryError(f"Agent '{agent_name}' is already registered")
        
        self._agents[agent_name] = agent_class
        self._agent_metadata[agent_name] = metadata
    
    def get_agent_class(self, name: str) -> Type[Agent]:
        """Get agent class by name."""
        if name not in self._agents:
            raise AgentDiscoveryError(f"Agent '{name}' not found in registry")
        return self._agents[name]
    
    def get_agent_metadata(self, name: str) -> AgentMetadata:
        """Get agent metadata by name."""
        if name not in self._agent_metadata:
            raise AgentDiscoveryError(f"Metadata for agent '{name}' not found")
        return self._agent_metadata[name]
    
    def list_agents(self) -> List[str]:
        """List all registered agent names."""
        return list(self._agents.keys())
    
    def list_agents_with_capabilities(self) -> Dict[str, List[str]]:
        """List all agents with their capabilities."""
        return {
            name: metadata.capabilities
            for name, metadata in self._agent_metadata.items()
        }
    
    def create_agent(
        self,
        name: str,
        agent_id: Optional[str] = None,
        **kwargs
    ) -> Agent:
        """Create an instance of a registered agent."""
        agent_class = self.get_agent_class(name)
        metadata = self.get_agent_metadata(name)
        
        # Create agent instance
        agent = agent_class(metadata=metadata, agent_id=agent_id, **kwargs)
        
        # Store instance if ID provided
        if agent_id:
            self._agent_instances[agent_id] = agent
        
        return agent
    
    def get_agent_instance(self, agent_id: str) -> Optional[Agent]:
        """Get an agent instance by ID."""
        return self._agent_instances.get(agent_id)
    
    def find_agents_by_capability(self, capability: str) -> List[str]:
        """Find agents that support a specific capability."""
        return [
            name
            for name, metadata in self._agent_metadata.items()
            if capability in metadata.capabilities
        ]
    
    def discover_agents_by_patterns(self, patterns: List[str]) -> List[str]:
        """Discover agents matching naming patterns."""
        matched_agents = []
        
        for pattern in patterns:
            pattern_lower = pattern.lower()
            for agent_name in self._agents.keys():
                if pattern_lower in agent_name.lower():
                    matched_agents.append(agent_name)
        
        return list(set(matched_agents))  # Remove duplicates
    
    def auto_discover_agents(self) -> List[str]:
        """Auto-discover agents from the registry."""
        import importlib
        import pkgutil
        import inspect
        
        discovered = []
        
        # Try to import agents module and scan for agent classes
        try:
            from . import agents
            package_path = agents.__path__
            
            for loader, module_name, is_pkg in pkgutil.iter_modules(package_path):
                try:
                    module = importlib.import_module(f".{module_name}", package="motion_ogre_agents.agents")
                    
                    # Look for agent classes
                    for name, obj in inspect.getmembers(module):
                        if (inspect.isclass(obj) and 
                            issubclass(obj, Agent) and 
                            obj != Agent and
                            hasattr(obj, 'metadata')):
                            
                            try:
                                self.register_agent(obj)
                                discovered.append(name)
                            except AgentDiscoveryError:
                                # Already registered or missing metadata
                                pass
                                
                except ImportError:
                    continue
        except ImportError:
            pass
        
        return discovered
    
    def get_all_metadata(self) -> List[AgentMetadata]:
        """Get metadata for all registered agents."""
        return list(self._agent_metadata.values())
    
    def remove_agent(self, name: str) -> None:
        """Remove an agent from the registry."""
        if name in self._agents:
            del self._agents[name]
        if name in self._agent_metadata:
            del self._agent_metadata[name]
    
    def clear(self) -> None:
        """Clear all registered agents."""
        self._agents.clear()
        self._agent_metadata.clear()
        self._agent_instances.clear()
    
    def __len__(self) -> int:
        """Get number of registered agents."""
        return len(self._agents)
    
    def __contains__(self, name: str) -> bool:
        """Check if agent is registered."""
        return name in self._agents


# Global registry instance
_global_registry: Optional[AgentRegistry] = None


def get_registry() -> AgentRegistry:
    """Get the global agent registry instance."""
    global _global_registry
    if _global_registry is None:
        _global_registry = AgentRegistry()
    return _global_registry


def clear_registry() -> None:
    """Clear the global registry."""
    global _global_registry
    if _global_registry is not None:
        _global_registry.clear()
        _global_registry = None