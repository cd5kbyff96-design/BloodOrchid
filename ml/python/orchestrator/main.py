"""
ml/python/orchestrator/main.py
Vail Iris Python Orchestrator - Main Entry Point
"""

import sys
import signal
import logging
from typing import Optional
from logging.config import dictConfig

from task_router import TaskRouter
from state_manager import StateManager
from retry_engine import RetryEngine
from ai.decision_engine import DecisionEngine
from api.bridge_client import BridgeClient
from config.system_config import SystemConfig

logger = logging.getLogger(__name__)


class Orchestrator:
    """Main Python orchestrator for Vail Iris system."""
    
    def __init__(self, config_path: Optional[str] = None):
        self.config = SystemConfig.load(config_path)
        self.state_manager = StateManager(self.config)
        self.retry_engine = RetryEngine(self.config.retry)
        self.decision_engine = DecisionEngine(self.config.ai)
        self.bridge_client = BridgeClient(self.config.bridge)
        self.task_router = TaskRouter(
            self.state_manager,
            self.retry_engine,
            self.decision_engine
        )
        self.running = False
    
    def start(self) -> int:
        """Start the orchestrator main loop."""
        logger.info("Starting Vail Iris orchestrator")
        
        signal.signal(signal.SIGINT, self._handle_shutdown)
        signal.signal(signal.SIGTERM, self._handle_shutdown)
        
        self.running = True
        
        try:
            while self.running:
                self._run_iteration()
        except Exception as e:
            logger.error(f"Orchestrator crashed: {e}")
            return 1
        
        return 0
    
    def _run_iteration(self) -> None:
        """Run a single orchestration iteration."""
        tasks = self.task_router.get_pending_tasks()
        
        for task in tasks:
            try:
                result = self.task_router.execute_task(task)
                self.state_manager.update_task_result(task.id, result)
            except Exception as e:
                logger.error(f"Task {task.id} failed: {e}")
                self.retry_engine.schedule_retry(task)
    
    def _handle_shutdown(self, signum, frame) -> None:
        """Handle graceful shutdown."""
        logger.info(f"Received signal {signum}, shutting down")
        self.running = False


def configure_logging(log_level: str = "INFO") -> None:
    """Configure structured logging."""
    dictConfig({
        "version": 1,
        "formatters": {
            "standard": {
                "format": "%(asctime)s [%(levelname)s] %(name)s: %(message)s"
            }
        },
        "handlers": {
            "default": {
                "level": log_level,
                "formatter": "standard",
                "class": "logging.StreamHandler",
                "stream": sys.stdout
            }
        },
        "root": {"handlers": ["default"], "level": log_level}
    })


def main() -> int:
    """Main entry point."""
    configure_logging()
    
    config_path = sys.argv[1] if len(sys.argv) > 1 else None
    orchestrator = Orchestrator(config_path)
    
    return orchestrator.start()


if __name__ == "__main__":
    sys.exit(main())