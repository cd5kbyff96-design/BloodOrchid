"""
ml/python/orchestrator/retry_engine.py
Retry logic with exponential backoff for failed tasks
"""

import asyncio
import logging
import time
from collections import defaultdict
from dataclasses import dataclass
from typing import Callable, Optional, Dict, Any

logger = logging.getLogger(__name__)


@dataclass
class RetryPolicy:
    """Configuration for retry behavior."""
    max_retries: int = 3
    initial_delay_ms: int = 100
    max_delay_ms: int = 30000
    backoff_multiplier: float = 2.0
    jitter: bool = True


class RetryEngine:
    """Manages task retry scheduling with exponential backoff."""
    
    def __init__(self, policy: Optional[RetryPolicy] = None):
        self.policy = policy or RetryPolicy()
        self._retry_queue: Dict[str, float] = {}
        self._task_history: Dict[str, Dict[str, Any]] = defaultdict(dict)
    
    def should_retry(self, task_id: str, error: Exception) -> bool:
        """Determine if a task should be retried."""
        attempts = self._task_history[task_id].get("attempts", 0)
        return attempts < self.policy.max_retries
    
    def calculate_delay(self, attempt: int) -> float:
        """Calculate delay for next retry attempt."""
        delay_ms = self.policy.initial_delay_ms
        for _ in range(attempt):
            delay_ms = min(delay_ms * self.policy.backoff_multiplier, self.policy.max_delay_ms)
        
        if self.policy.jitter:
            import random
            delay_ms *= 0.5 + random.random() * 0.5
        
        return delay_ms / 1000.0
    
    def schedule_retry(self, task_id: str, error: Exception) -> float:
        """Schedule a task for retry. Returns delay in seconds."""
        attempts = self._task_history[task_id].get("attempts", 0) + 1
        self._task_history[task_id]["attempts"] = attempts
        
        delay = self.calculate_delay(attempts)
        self._retry_queue[task_id] = time.time() + delay
        
        logger.warning(f"Scheduled retry for {task_id} in {delay}s (attempt {attempts})")
        return delay
    
    def get_ready_tasks(self) -> list:
        """Get tasks that are ready for retry."""
        now = time.time()
        ready = [
            task_id for task_id, scheduled_time in self._retry_queue.items()
            if scheduled_time <= now
        ]
        for task_id in ready:
            del self._retry_queue[task_id]
        return ready
    
    def reset_attempts(self, task_id: str) -> None:
        """Reset retry attempts for a task."""
        self._task_history[task_id]["attempts"] = 0