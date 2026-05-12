"""
ml/python/orchestrator/state_manager.py
State management for the orchestrator
"""

import json
import logging
from dataclasses import dataclass, field
from typing import Dict, List, Optional, Any
from pathlib import Path

logger = logging.getLogger(__name__)


@dataclass
class TaskRecord:
    """Record of a task's execution state."""
    task_id: str
    task_type: str
    status: str  # pending, running, completed, failed, retried
    payload: dict
    result: Optional[dict] = None
    error: Optional[str] = None
    retry_count: int = 0
    created_at: Optional[float] = None
    updated_at: Optional[float] = None


class StateManager:
    """Manages orchestrator state and task persistence."""
    
    def __init__(self, config):
        self.config = config
        self._tasks: Dict[str, TaskRecord] = {}
        self._state_file = Path(config.state_dir) / "orchestrator_state.json"
        self._load_state()
    
    def _load_state(self) -> None:
        """Load state from disk."""
        if self._state_file.exists():
            try:
                data = json.loads(self._state_file.read_text())
                for task_data in data.get("tasks", []):
                    task = TaskRecord(**task_data)
                    self._tasks[task.task_id] = task
            except Exception as e:
                logger.error(f"Failed to load state: {e}")
    
    def _save_state(self) -> None:
        """Persist state to disk."""
        data = {
            "tasks": [
                {
                    "task_id": t.task_id,
                    "task_type": t.task_type,
                    "status": t.status,
                    "payload": t.payload,
                    "result": t.result,
                    "error": t.error,
                    "retry_count": t.retry_count,
                    "created_at": t.created_at,
                    "updated_at": t.updated_at
                }
                for t in self._tasks.values()
            ]
        }
        self._state_file.parent.mkdir(parents=True, exist_ok=True)
        self._state_file.write_text(json.dumps(data))
    
    def register_task(self, task_record: TaskRecord) -> None:
        """Register a new task."""
        import time
        task_record.created_at = time.time()
        task_record.updated_at = time.time()
        self._tasks[task_record.task_id] = task_record
        self._save_state()
    
    def update_task(self, task_id: str, updates: dict) -> None:
        """Update task state."""
        import time
        if task_id in self._tasks:
            task = self._tasks[task_id]
            for key, value in updates.items():
                if hasattr(task, key):
                    setattr(task, key, value)
            task.updated_at = time.time()
            self._save_state()
    
    def update_task_result(self, task_id: str, result: Any) -> None:
        """Update task with execution result."""
        self.update_task(task_id, {"status": "completed", "result": result})
    
    def get_pending_tasks(self) -> List[TaskRecord]:
        """Get all tasks with pending status."""
        return [
            t for t in self._tasks.values()
            if t.status in ("pending", "retried")
        ]
    
    def get_task(self, task_id: str) -> Optional[TaskRecord]:
        """Get a specific task by ID."""
        return self._tasks.get(task_id)
    
    def get_simulation_state(self, simulation_id: str) -> Optional[dict]:
        """Get current state for a simulation."""
        for task in self._tasks.values():
            if (task.task_type == "kernel_step" and 
                task.payload.get("simulation_id") == simulation_id and
                task.status == "completed"):
                return task.result
        return None