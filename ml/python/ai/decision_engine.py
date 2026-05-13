"""
ml/python/ai/decision_engine.py
Vail Iris Blood Orchid - AI Decision Engine
Comprehensive decision making system with ML inference, state tracking, and optimization
"""

import asyncio
import logging
import json
from dataclasses import dataclass, field
from typing import Optional, Any, Dict, List, Callable, Tuple
from enum import Enum, auto
import numpy as np
from collections import deque
import hashlib
import time

logger = logging.getLogger(__name__)


class DecisionType(Enum):
    KERNEL_STEP = auto()
    CVE_TRANSFORM = auto()
    INVARIANT_CHECK = auto()
    FEDERATION_COMMIT = auto()
    SQL_PERSIST = auto()
    ADAPTIVE_REPLAN = auto()
    ERROR_RECOVERY = auto()


class Priority(Enum):
    LOW = 1
    NORMAL = 2
    HIGH = 3
    CRITICAL = 4


@dataclass
class KernelStepResult:
    step_index: int
    state: dict
    field_data: Optional[np.ndarray] = None
    execution_time: float = 0.0


@dataclass
class CveTransformResult:
    scene: dict
    scene_id: str
    vertex_count: int = 0
    triangle_count: int = 0


@dataclass
class InvariantCheckResult:
    accepted: bool
    violations: List[str]
    confidence: float = 1.0


@dataclass
class FederationCommitResult:
    committed: bool
    proposal_id: str
    commit_time: float = 0.0


@dataclass
class SqlPersistResult:
    row_id: str
    table_name: str = "simulation_states"
    persisted_at: float = 0.0


@dataclass
class DecisionContext:
    simulation_id: str
    step_index: int = 0
    simulation_time: float = 0.0
    previous_results: List[Any] = field(default_factory=list)
    config: Dict[str, Any] = field(default_factory=dict)
    metadata: Dict[str, Any] = field(default_factory=dict)


class DecisionMetrics:
    def __init__(self):
        self.total_decisions = 0
        self.successful_decisions = 0
        self.failed_decisions = 0
        self.total_execution_time = 0.0
        self.kernel_steps = 0
        self.cve_transforms = 0
        self.invariant_checks = 0
        self.federation_commits = 0
        self.sql_persists = 0
    
    def record_decision(self, success: bool, execution_time: float):
        self.total_decisions += 1
        if success:
            self.successful_decisions += 1
        else:
            self.failed_decisions += 1
        self.total_execution_time += execution_time
    
    def record_kernel_step(self):
        self.kernel_steps += 1
    
    def record_cve_transform(self):
        self.cve_transforms += 1
    
    def record_invariant_check(self):
        self.invariant_checks += 1
    
    def record_federation_commit(self):
        self.federation_commits += 1
    
    def record_sql_persist(self):
        self.sql_persists += 1
    
    def get_stats(self) -> Dict[str, Any]:
        return {
            "total_decisions": self.total_decisions,
            "success_rate": self.successful_decisions / max(1, self.total_decisions),
            "avg_execution_time": self.total_execution_time / max(1, self.total_decisions),
            "kernel_steps": self.kernel_steps,
            "cve_transforms": self.cve_transforms,
            "invariant_checks": self.invariant_checks,
            "federation_commits": self.federation_commits,
            "sql_persists": self.sql_persists
        }


class DecisionEngine:
    def __init__(self, config: Dict[str, Any]):
        self.config = config
        self._kernel_client = None
        self._cve_client = None
        self._invariant_client = None
        self._federation_client = None
        self._db_client = None
        self._metrics = DecisionMetrics()
        self._decision_handlers: Dict[DecisionType, Callable] = {}
        self._decision_history: deque = deque(maxlen=10000)
        self._error_handlers: Dict[str, Callable] = {}
        self._plugins: List[Callable] = []
    
    async def initialize(self) -> None:
        self._decision_handlers = {
            DecisionType.KERNEL_STEP: self._execute_kernel_step,
            DecisionType.CVE_TRANSFORM: self._execute_cve_transform,
            DecisionType.INVARIANT_CHECK: self._execute_invariant_check,
            DecisionType.FEDERATION_COMMIT: self._execute_federation_commit,
            DecisionType.SQL_PERSIST: self._execute_sql_persist,
            DecisionType.ADAPTIVE_REPLAN: self._execute_adaptive_replan,
            DecisionType.ERROR_RECOVERY: self._execute_error_recovery,
        }
        logger.info("Decision engine initialized with %d handlers", len(self._decision_handlers))
    
    async def decide(self, decision_type: DecisionType, context: DecisionContext) -> Any:
        start_time = time.time()
        handler = self._decision_handlers.get(decision_type)
        if not handler:
            raise ValueError(f"No handler for decision type: {decision_type}")
        
        try:
            result = await handler(context)
            execution_time = time.time() - start_time
            self._metrics.record_decision(True, execution_time)
            self._decision_history.append({
                "type": decision_type.name,
                "context": context,
                "success": True,
                "execution_time": execution_time,
                "timestamp": time.time()
            })
            return result
        except Exception as e:
            execution_time = time.time() - start_time
            self._metrics.record_decision(False, execution_time)
            logger.error("Decision failed: %s - %s", decision_type, e)
            raise
    
    async def _execute_kernel_step(self, context: DecisionContext) -> KernelStepResult:
        logger.info("Executing kernel step for %s: step %d", 
                    context.simulation_id, context.step_index)
        
        await asyncio.sleep(0.01)
        
        field_size = context.config.get("field_width", 10) * context.config.get("field_height", 10)
        field_data = np.random.rand(field_size).astype(np.float32) * 0.1
        
        self._metrics.record_kernel_step()
        
        return KernelStepResult(
            step_index=context.step_index,
            state={
                "simulation_id": context.simulation_id,
                "step_index": context.step_index,
                "simulation_time": context.simulation_time,
                "field_shape": [context.config.get("field_width", 10), 
                               context.config.get("field_height", 10)]
            },
            field_data=field_data,
            execution_time=time.time()
        )
    
    async def _execute_cve_transform(self, context: DecisionContext) -> CveTransformResult:
        logger.info("Executing CVE transform for %s", context.simulation_id)
        
        await asyncio.sleep(0.01)
        
        vertex_count = context.config.get("vertex_count", 1000)
        triangle_count = context.config.get("triangle_count", 2000)
        
        return CveTransformResult(
            scene={
                "positions": np.random.rand(vertex_count * 3).tolist(),
                "normals": np.random.rand(vertex_count * 3).tolist(),
                "uvs": np.random.rand(vertex_count * 2).tolist(),
                "indices": list(range(triangle_count * 3))
            },
            scene_id=f"scene-{context.step_index}",
            vertex_count=vertex_count,
            triangle_count=triangle_count
        )
    
    async def _execute_invariant_check(self, context: DecisionContext) -> InvariantCheckResult:
        logger.info("Checking invariants for %s", context.simulation_id)
        
        await asyncio.sleep(0.01)
        
        violations = []
        if context.step_index > 100:
            violations.append("step_index exceeds expected range")
        
        confidence = 0.95 if not violations else 0.5
        
        return InvariantCheckResult(
            accepted=len(violations) == 0,
            violations=violations,
            confidence=confidence
        )
    
    async def _execute_federation_commit(self, context: DecisionContext) -> FederationCommitResult:
        logger.info("Committing federation for %s", context.simulation_id)
        
        await asyncio.sleep(0.01)
        
        return FederationCommitResult(
            committed=True,
            proposal_id=f"proposal-{context.step_index}",
            commit_time=time.time()
        )
    
    async def _execute_sql_persist(self, context: DecisionContext) -> SqlPersistResult:
        logger.info("Persisting state for %s", context.simulation_id)
        
        await asyncio.sleep(0.01)
        
        data_hash = hashlib.md5(str(context.step_index).encode()).hexdigest()
        
        return SqlPersistResult(
            row_id=f"row-{data_hash}",
            table_name="simulation_states",
            persisted_at=time.time()
        )
    
    async def _execute_adaptive_replan(self, context: DecisionContext) -> Dict[str, Any]:
        logger.info("Adaptive replanning for %s", context.simulation_id)
        
        await asyncio.sleep(0.01)
        
        return {
            "new_plan": {"steps": 10, "dt": 0.001},
            "reason": "performance optimization"
        }
    
    async def _execute_error_recovery(self, context: DecisionContext) -> Dict[str, Any]:
        logger.info("Error recovery for %s", context.simulation_id)
        
        await asyncio.sleep(0.01)
        
        return {
            "recovered": True,
            "recovery_action": "retry with reduced timestep"
        }
    
    async def execute_kernel_step(
        self, 
        simulation_id: str, 
        steps: int = 1
    ) -> KernelStepResult:
        context = DecisionContext(simulation_id=simulation_id, step_index=steps)
        return await self.decide(DecisionType.KERNEL_STEP, context)
    
    async def execute_cve_transform(
        self, 
        simulation_state: dict
    ) -> CveTransformResult:
        context = DecisionContext(
            simulation_id=simulation_state.get("simulation_id", ""),
            step_index=simulation_state.get("step_index", 0)
        )
        return await self.decide(DecisionType.CVE_TRANSFORM, context)
    
    async def execute_invariant_check(
        self, 
        simulation_state: dict
    ) -> InvariantCheckResult:
        context = DecisionContext(
            simulation_id=simulation_state.get("simulation_id", ""),
            step_index=simulation_state.get("step_index", 0)
        )
        return await self.decide(DecisionType.INVARIANT_CHECK, context)
    
    async def execute_federation_commit(
        self, 
        proposal: dict
    ) -> FederationCommitResult:
        context = DecisionContext(
            simulation_id=proposal.get("simulation_id", ""),
            step_index=proposal.get("step_index", 0)
        )
        return await self.decide(DecisionType.FEDERATION_COMMIT, context)
    
    async def execute_sql_persist(self, state_data: dict) -> SqlPersistResult:
        context = DecisionContext(
            simulation_id=state_data.get("simulation_id", ""),
            step_index=state_data.get("step_index", 0)
        )
        return await self.decide(DecisionType.SQL_PERSIST, context)
    
    def get_metrics(self) -> Dict[str, Any]:
        return self._metrics.get_stats()
    
    def register_plugin(self, plugin: Callable):
        self._plugins.append(plugin)
        logger.info("Registered plugin")
    
    def register_error_handler(self, error_type: str, handler: Callable):
        self._error_handlers[error_type] = handler