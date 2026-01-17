"""LLM integration with OpenAI API client, token counting, and cost tracking."""

import asyncio
from typing import Any, Dict, List, Optional, Union
from enum import Enum
import tiktoken
from tenacity import (
    Retrying,
    retry_if_exception_type,
    stop_after_attempt,
    wait_exponential,
)
import openai
from openai import AsyncOpenAI, OpenAIError
import logging
from pydantic import BaseModel, Field
from datetime import datetime


class ModelType(str, Enum):
    """Supported OpenAI model types."""
    
    GPT_4 = "gpt-4"
    GPT_4_TURBO = "gpt-4-turbo-preview"
    GPT_3_5_TURBO = "gpt-3.5-turbo"
    GPT_4o = "gpt-4o"


class TokenUsage(BaseModel):
    """Token usage statistics for an API call."""
    
    prompt_tokens: int = Field(ge=0)
    completion_tokens: int = Field(ge=0)
    total_tokens: int = Field(ge=0)


class CostTracking(BaseModel):
    """Cost tracking information."""
    
    model: str
    prompt_tokens: int = Field(ge=0)
    completion_tokens: int = Field(ge=0)
    estimated_cost: float = Field(ge=0.0)
    timestamp: datetime = Field(default_factory=datetime.now)


# Model pricing per token (in USD)
# Prices as of 2024 - update as needed
MODEL_PRICING = {
    ModelType.GPT_4: {
        "prompt": 0.03 / 1000,  # $0.03 per 1K tokens
        "completion": 0.06 / 1000,  # $0.06 per 1K tokens
    },
    ModelType.GPT_4_TURBO: {
        "prompt": 0.01 / 1000,  # $0.01 per 1K tokens
        "completion": 0.03 / 1000,  # $0.03 per 1K tokens
    },
    ModelType.GPT_3_5_TURBO: {
        "prompt": 0.0005 / 1000,  # $0.0005 per 1K tokens
        "completion": 0.0015 / 1000,  # $0.0015 per 1K tokens
    },
    ModelType.GPT_4o: {
        "prompt": 0.005 / 1000,  # $0.005 per 1K tokens
        "completion": 0.015 / 1000,  # $0.015 per 1K tokens
    },
}


class PromptTemplate:
    """Template for LLM prompts with variable substitution."""
    
    def __init__(self, template: str, variables: Optional[Dict[str, str]] = None):
        """Initialize a prompt template."""
        self.template = template
        self.variables = variables or {}
    
    def with_variables(self, **variables) -> "PromptTemplate":
        """Create a new template with additional variables."""
        merged_vars = {**self.variables, **variables}
        return PromptTemplate(self.template, merged_vars)
    
    def render(self, **kwargs) -> str:
        """Render the template with all variables."""
        all_vars = {**self.variables, **kwargs}
        try:
            return self.template.format(**all_vars)
        except KeyError as e:
            raise ValueError(f"Missing required variable: {e}")


class LLMClient:
    """OpenAI API client with token counting, cost tracking, and retry logic."""
    
    def __init__(
        self,
        api_key: Optional[str] = None,
        model: ModelType = ModelType.GPT_4,
        temperature: float = 0.7,
        max_tokens: Optional[int] = None,
        enable_cost_tracking: bool = True,
        max_retry_attempts: int = 3,
        retry_base_delay: float = 1.0,
    ):
        """Initialize LLM client."""
        self.client = AsyncOpenAI(api_key=api_key)
        self.model = model
        self.temperature = temperature
        self.max_tokens = max_tokens
        self.enable_cost_tracking = enable_cost_tracking
        self._cost_history: List[CostTracking] = []
        
        # Initialize tiktoken encoder
        try:
            self._encoder = tiktoken.encoding_for_model(model.value)
        except KeyError:
            # Fallback to cl100k_base for newer models
            self._encoder = tiktoken.get_encoding("cl100k_base")
        
        # Setup retry configuration
        self.retry_decorator = Retrying(
            stop=stop_after_attempt(max_retry_attempts),
            wait=wait_exponential(multiplier=retry_base_delay),
            retry=retry_if_exception_type(OpenAIError),
        )
        
        self.logger = logging.getLogger(__name__)
    
    def count_tokens(self, text: str) -> int:
        """Count tokens in a text string."""
        return len(self._encoder.encode(text))
    
    def count_message_tokens(self, messages: List[Dict[str, str]]) -> int:
        """Count tokens in a list of messages."""
        total_tokens = 0
        for message in messages:
            # Add tokens for role and content
            content = message.get("content", "")
            total_tokens += self.count_tokens(content)
            # Add overhead for message structure (approximately 4 tokens per message)
            total_tokens += 4
        return total_tokens
    
    def calculate_cost(
        self,
        model: str,
        prompt_tokens: int,
        completion_tokens: int,
    ) -> float:
        """Calculate cost for token usage."""
        pricing = MODEL_PRICING.get(model, MODEL_PRICING[ModelType.GPT_4])
        
        prompt_cost = prompt_tokens * pricing["prompt"]
        completion_cost = completion_tokens * pricing["completion"]
        
        return prompt_cost + completion_cost
    
    async def chat_completion(
        self,
        messages: List[Dict[str, str]],
        model: Optional[str] = None,
        temperature: Optional[float] = None,
        max_tokens: Optional[int] = None,
        stream: bool = False,
        **kwargs
    ) -> Union[str, Dict[str, Any]]:
        """Generate chat completion with retry logic and cost tracking."""
        
        model = model or self.model.value
        temperature = temperature if temperature is not None else self.temperature
        max_tokens = max_tokens or self.max_tokens
        
        # Count input tokens
        prompt_tokens = self.count_message_tokens(messages)
        
        # Log request
        self.logger.debug(f"API request - Model: {model}, Prompt tokens: {prompt_tokens}")
        
        try:
            response = await self.client.chat.completions.create(
                model=model,
                messages=messages,
                temperature=temperature,
                max_tokens=max_tokens,
                stream=stream,
                **kwargs
            )
            
            if stream:
                # Handle streaming response
                return await self._handle_streaming_response(response, model, prompt_tokens)
            else:
                # Handle regular response
                return await self._handle_regular_response(response, model, prompt_tokens)
                
        except OpenAIError as e:
            self.logger.error(f"OpenAI API error: {e}")
            raise
        except Exception as e:
            self.logger.error(f"Unexpected error during API call: {e}")
            raise
    
    async def _handle_regular_response(self, response, model: str, prompt_tokens: int) -> str:
        """Handle non-streaming API response."""
        
        if not response.choices or not response.choices[0].message:
            raise ValueError("Invalid response from API: no completion")
        
        completion_text = response.choices[0].message.content
        completion_tokens = response.usage.completion_tokens if response.usage else 0
        total_tokens = response.usage.total_tokens if response.usage else 0
        
        # Track costs
        if self.enable_cost_tracking:
            cost_tracking = CostTracking(
                model=model,
                prompt_tokens=prompt_tokens,
                completion_tokens=completion_tokens,
                estimated_cost=self.calculate_cost(model, prompt_tokens, completion_tokens),
            )
            self._cost_history.append(cost_tracking)
        
        # Log response details
        self.logger.debug(
            f"API response - Completion tokens: {completion_tokens}, "
            f"Total tokens: {total_tokens}"
        )
        
        return completion_text
    
    async def _handle_streaming_response(self, response, model: str, prompt_tokens: int) -> Dict[str, Any]:
        """Handle streaming API response."""
        
        chunks = []
        completion_tokens = 0
        
        async for chunk in response:
            if chunk.choices and chunk.choices[0].delta.content:
                content = chunk.choices[0].delta.content
                chunks.append(content)
                completion_tokens += self.count_tokens(content)
        
        completion_text = "".join(chunks)
        total_tokens = prompt_tokens + completion_tokens
        
        # Track costs
        if self.enable_cost_tracking:
            cost_tracking = CostTracking(
                model=model,
                prompt_tokens=prompt_tokens,
                completion_tokens=completion_tokens,
                estimated_cost=self.calculate_cost(model, prompt_tokens, completion_tokens),
            )
            self._cost_history.append(cost_tracking)
        
        # Log response details
        self.logger.debug(
            f"API streaming response - Completion tokens: {completion_tokens}, "
            f"Total tokens: {total_tokens}"
        )
        
        return {
            "content": completion_text,
            "prompt_tokens": prompt_tokens,
            "completion_tokens": completion_tokens,
            "total_tokens": total_tokens,
        }
    
    async def generate_plan(
        self,
        prompt_template: PromptTemplate,
        **variables
    ) -> str:
        """Generate a plan using a prompt template."""
        
        rendered_prompt = prompt_template.render(**variables)
        
        messages = [
            {
                "role": "system",
                "content": "You are an AI assistant that creates detailed execution plans.",
            },
            {
                "role": "user",
                "content": rendered_prompt,
            },
        ]
        
        return await self.chat_completion(messages)
    
    def get_cost_summary(self) -> Dict[str, Any]:
        """Get summary of costs incurred."""
        
        if not self._cost_history:
            return {
                "total_cost": 0.0,
                "total_prompt_tokens": 0,
                "total_completion_tokens": 0,
                "call_count": 0,
            }
        
        total_cost = sum(ct.estimated_cost for ct in self._cost_history)
        total_prompt_tokens = sum(ct.prompt_tokens for ct in self._cost_history)
        total_completion_tokens = sum(ct.completion_tokens for ct in self._cost_history)
        
        return {
            "total_cost": round(total_cost, 6),
            "total_prompt_tokens": total_prompt_tokens,
            "total_completion_tokens": total_completion_tokens,
            "total_tokens": total_prompt_tokens + total_completion_tokens,
            "call_count": len(self._cost_history),
            "by_model": self._get_costs_by_model(),
        }
    
    def _get_costs_by_model(self) -> Dict[str, Dict[str, Any]]:
        """Get cost breakdown by model."""
        
        by_model = {}
        
        for cost_tracking in self._cost_history:
            model = cost_tracking.model
            if model not in by_model:
                by_model[model] = {
                    "total_cost": 0.0,
                    "total_prompt_tokens": 0,
                    "total_completion_tokens": 0,
                    "call_count": 0,
                }
            
            by_model[model]["total_cost"] += cost_tracking.estimated_cost
            by_model[model]["total_prompt_tokens"] += cost_tracking.prompt_tokens
            by_model[model]["total_completion_tokens"] += cost_tracking.completion_tokens
            by_model[model]["call_count"] += 1
        
        return by_model
    
    def reset_cost_tracking(self) -> None:
        """Reset cost tracking history."""
        self._cost_history.clear()