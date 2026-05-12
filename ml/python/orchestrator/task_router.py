"""
ml/python/orchestrator/task_router.py
Task routing and execution for Vail Iris orchestrator
"""

import asyncio
import logging
from dataclasses import dataclass
from enum import Enum
from typing import Optional, List, Any, Callable

logger = logging.getLogger(__name__)


class TaskType(Enum):
    KERNEL_STEP = "kernel_step"
    CVE_TRANSFORM = "cve_transform"
    INVARIANT_CHECK = "invariant_check"
    FEDERATION_COMMIT = "federation_commit"
    SQL_PERSIST = "sql_persist"


@dataclass
class Task:
    """Represents an executable task."""
    id: str
    task_type: TaskType
    payload: dict
    priority: int = 0
    max_retries: int = 3
    retry_count: int = 0
    metadata: dict = None
    
    def __post_init__(self):
        if self.metadata is None:
            self.metadata = {}


@dataclass
class TaskResult:
    """Result of task execution."""
    task_id: str
    success: bool
    output: Any
    error: Optional[str] = None
    execution_time_ms: float = 0.0


class TaskRouter:
    """Routes and executes tasks across the Vail Iris system."""
    
    def __init__(
        self,
        state_manager,
        retry_engine,
        decision_engine,
        max_concurrent: int = 10
    ):
        self.state_manager = state_manager
        self.retry_engine = retry_engine
        self.decision_engine = decision_engine
        self.max_concurrent = max_concurrent
        self._handlers = {}
        self._semaphore = asyncio.Semaphore(max_concurrent)
        
        self._register_handlers()
    
    def _register_handlers(self) -> None:
        """Register task type handlers."""
        self._handlers[TaskType.KERNEL_STEP] = self._handle_kernel_step
        self._handlers[TaskType.CVE_TRANSFORM] = self._handle_cve_transform
        self._handlers[TaskType.INVARIANT_CHECK] = self._handle_invariant_check
        self._handlers[TaskType.FEDERATION_COMMIT] = self._handle_federation_commit
        self._handlers[TaskType.SQL_PERSIST] = self._handle_sql_persist
    
    def get_pending_tasks(self) -> List[Task]:
        """Get tasks pending execution from the state manager."""
        return self.state_manager.get_pending_tasks()
    
    async def execute_task(self, task: Task) -> TaskResult:
        """Execute a single task."""
        import time
        start_time = time.time()
        
        async with self._semaphore:
            handler = self._handlers.get(task.task_type)
            if not handler:
                return TaskResult(
                    task_id=task.id,
                    success=False,
                    output=None,
                    error=f"No handler for task type {task.task_type}"
                )
            
            try:
                output = await handler(task.payload)
                return TaskResult(
                    task_id=task.id,
                    success=True,
                    output=output,
                    execution_time_ms=(time.time() - start_time) * 1000
                )
            except Exception as e:
                logger.error(f"Task {task.id} failed: {e}")
                return TaskResult(
                    task_id=task.id,
                    success=False,
                    output=None,
                    error=str(e),
                    execution_time_ms=(time.time() - start_time) * 1000
                )
    
    async def _handle_kernel_step(self, payload: dict) -> dict:
        """Execute kernel step task."""
        steps = payload.get("steps", 1)
        simulation_id = payload.get("simulation_id")
        
        result = await self.decision_engine.execute_kernel_step(
            simulation_id, steps
        )
        
        return {"step_index": result.step_index, "state": result.state}
    
    async def _handle_cve_transform(self, payload: dict) -> dict:
        """Execute CVE transformation task."""
        simulation_state = payload.get("simulation_state")
        
        result = await self.decision_engine.execute_cve_transform(
            simulation_state
        )
        
        return {"scene": result.scene, "scene_id": result.scene_id}
    
    async def _handle_invariant_check(self, payload: dict) -> dict:
        """Execute invariant check task."""
        simulation_state = payload.get("simulation_state")
        
        result = await self.decision_engine.execute_invariant_check(
            simulation_state
        )
        
        return {"accepted": result.accepted, "violations": result.violations}
    
    async def _handle_federation_commit(self, payload: dict) -> dict:
        """Execute federation commit task."""
        proposal = payload.get("proposal")
        
        result = await self.decision_engine.execute_federation_commit(
            proposal
        )
        
        return {"committed": result.committed, "proposal_id": result.proposal_id}
    
    async def _handle_sql_persist(self, payload: dict) -> dict:
        """Execute SQL persistence task."""
        state_data = payload.get("state_data")
        
        result = await self.decision_engine.execute_sql_persist(state_data)
        
        return {"persisted": True, "row_id": result.row_id}