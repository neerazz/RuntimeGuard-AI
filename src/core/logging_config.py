import logging
from datetime import datetime
from typing import Any, Dict


class JsonFormatter(logging.Formatter):
    """Minimal JSON formatter for structured logs."""

    def format(self, record: logging.LogRecord) -> str:  # type: ignore[override]
        base: Dict[str, Any] = {
            "ts": datetime.utcnow().isoformat() + "Z",
            "level": record.levelname,
            "logger": record.name,
            "message": record.getMessage(),
        }

        for key, value in record.__dict__.items():
            if key.startswith("_") or key in (
                "msg",
                "args",
                "name",
                "levelname",
                "levelno",
                "pathname",
                "filename",
                "module",
                "exc_info",
                "exc_text",
                "stack_info",
                "lineno",
                "funcName",
                "created",
                "msecs",
                "relativeCreated",
                "thread",
                "threadName",
                "processName",
                "process",
            ):
                continue
            base[key] = value

        return self._dict_to_str(base)

    @staticmethod
    def _dict_to_str(data: Dict[str, Any]) -> str:
        parts = []
        for key, value in data.items():
            safe_val = repr(value)
            parts.append(f'"{key}": {safe_val}')
        return "{" + ", ".join(parts) + "}"


def configure_logging(level: str = "INFO") -> None:
    """Configure root logger with JSON formatting."""
    handler = logging.StreamHandler()
    handler.setFormatter(JsonFormatter())
    root = logging.getLogger()
    root.handlers = [handler]
    root.setLevel(level)
    root.propagate = False
    logging.captureWarnings(True)
