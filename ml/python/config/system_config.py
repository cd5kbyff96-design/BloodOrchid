"""
ml/python/config/system_config.py
System configuration management
"""

import os
import json
import logging
from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional

logger = logging.getLogger(__name__)


@dataclass
class RetryConfig:
    max_retries: int = 3
    initial_delay_ms: int = 100
    max_delay_ms: int = 30000
    backoff_multiplier: float = 2.0


@dataclass
class BridgeConfig:
    rust_endpoint: str = "http://localhost:8080"
    elixir_endpoint: str = "http://localhost:4000"
    c_library_path: str = "/usr/local/lib/libvailiris_kernel.so"


@dataclass
class AiConfig:
    model_path: str = ""
    max_batch_size: int = 10


@dataclass
class DatabaseConfig:
    host: str = "localhost"
    port: int = 5432
    database: str = "vailiris"
    user: str = "vailiris"


@dataclass
class SystemConfig:
    state_dir: str = "/tmp/vailiris_orchestrator"
    log_level: str = "INFO"
    max_concurrent_tasks: int = 10
    retry: RetryConfig = field(default_factory=RetryConfig)
    bridge: BridgeConfig = field(default_factory=BridgeConfig)
    ai: AiConfig = field(default_factory=AiConfig)
    database: DatabaseConfig = field(default_factory=DatabaseConfig)
    
    @classmethod
    def load(cls, config_path: Optional[str] = None) -> "SystemConfig":
        """Load configuration from file or environment."""
        if config_path and Path(config_path).exists():
            data = json.loads(Path(config_path).read_text())
            return cls._from_dict(data)
        
        # Load from environment variables
        return cls(
            state_dir=os.getenv("VAILIRIS_STATE_DIR", "/tmp/vailiris_orchestrator"),
            log_level=os.getenv("VAILIRIS_LOG_LEVEL", "INFO"),
            max_concurrent_tasks=int(os.getenv("VAILIRIS_MAX_CONCURRENT", "10")),
            retry=RetryConfig(
                max_retries=int(os.getenv("VAILIRIS_MAX_RETRIES", "3")),
                initial_delay_ms=int(os.getenv("VAILIRIS_RETRY_DELAY_MS", "100")),
            ),
            bridge=BridgeConfig(
                rust_endpoint=os.getenv("VAILIRIS_RUST_ENDPOINT", "http://localhost:8080"),
                elixir_endpoint=os.getenv("VAILIRIS_ELIXIR_ENDPOINT", "http://localhost:4000"),
            ),
        )
    
    @classmethod
    def _from_dict(cls, data: dict) -> "SystemConfig":
        """Create config from dictionary."""
        return cls(
            state_dir=data.get("state_dir", "/tmp/vailiris_orchestrator"),
            log_level=data.get("log_level", "INFO"),
            max_concurrent_tasks=data.get("max_concurrent_tasks", 10),
            retry=RetryConfig(**data.get("retry", {})),
            bridge=BridgeConfig(**data.get("bridge", {})),
            ai=AiConfig(**data.get("ai", {})),
            database=DatabaseConfig(**data.get("database", {})),
        )