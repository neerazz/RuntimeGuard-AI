import logging
import sqlite3
from pathlib import Path
from typing import Iterable, Union


LOGGER = logging.getLogger(__name__)
DEFAULT_DB_PATH = Path("data/audit_log.db")
SCHEMA_PATH = Path(__file__).resolve().parent / "schema.sql"


def get_connection(db_path: Union[Path, str] = DEFAULT_DB_PATH) -> sqlite3.Connection:
    """Return a SQLite connection with row factory and FK enforcement."""
    conn = sqlite3.connect(str(db_path), check_same_thread=False)
    conn.row_factory = sqlite3.Row
    conn.execute("PRAGMA foreign_keys = ON;")
    return conn


def init_db(db_path: Union[Path, str] = DEFAULT_DB_PATH) -> None:
    """Create the database schema if it does not exist."""
    db_path = Path(db_path)
    db_path.parent.mkdir(parents=True, exist_ok=True)
    with open(SCHEMA_PATH, "r", encoding="utf-8") as f:
        schema_sql = f.read()

    conn = get_connection(db_path)
    try:
        conn.executescript(schema_sql)
        conn.commit()
        LOGGER.info("database_initialized path=%s", db_path)
    finally:
        conn.close()


def execute_many(conn: sqlite3.Connection, statements: Iterable[str]) -> None:
    """Execute multiple SQL statements in a transaction."""
    cur = conn.cursor()
    for statement in statements:
        cur.execute(statement)
    conn.commit()
