import hashlib
import logging
import sqlite3
from typing import List, Optional, Tuple

from .db import DEFAULT_DB_PATH, get_connection, init_db

LOGGER = logging.getLogger(__name__)


def _hash_pair(left: str, right: str) -> str:
    return hashlib.sha256((left + right).encode()).hexdigest()


class MerkleAuditLog:
    """Append-only Merkle log backed by SQLite."""

    def __init__(self, db_path=DEFAULT_DB_PATH) -> None:
        self.db_path = db_path
        init_db(db_path)

    def append(self, data_hash: str) -> Tuple[int, str]:
        """Append a new leaf and return (index, new_root)."""
        conn = get_connection(self.db_path)
        try:
            leaf_hash = hashlib.sha256(data_hash.encode()).hexdigest()
            index = self._next_index(conn)
            conn.execute(
                """
                INSERT INTO merkle_nodes (index_val, level, hash, data_hash)
                VALUES (?, 0, ?, ?)
                """,
                (index, leaf_hash, data_hash),
            )
            conn.commit()
            leaves = self._get_leaf_hashes(conn)
            root = self._compute_root(leaves)
            LOGGER.info("merkle_append index=%s root=%s", index, root)
            return index, root
        finally:
            conn.close()

    def get_root(self) -> str:
        conn = get_connection(self.db_path)
        try:
            leaves = self._get_leaf_hashes(conn)
            return self._compute_root(leaves)
        finally:
            conn.close()

    def verify_leaf(self, index: int, data_hash: str) -> bool:
        conn = get_connection(self.db_path)
        try:
            cur = conn.execute(
                "SELECT hash FROM merkle_nodes WHERE index_val=? AND level=0", (index,)
            )
            row = cur.fetchone()
            if not row:
                return False
            stored = row["hash"]
            expected = hashlib.sha256(data_hash.encode()).hexdigest()
            if stored != expected:
                return False
            recomputed_root = self._compute_root(self._get_leaf_hashes(conn))
            LOGGER.info("merkle_verify index=%s ok=%s root=%s", index, True, recomputed_root)
            return True
        finally:
            conn.close()

    def _next_index(self, conn: sqlite3.Connection) -> int:
        cur = conn.execute("SELECT COUNT(*) as c FROM merkle_nodes WHERE level=0")
        return int(cur.fetchone()["c"])

    def _get_leaf_hashes(self, conn: sqlite3.Connection) -> List[str]:
        cur = conn.execute(
            "SELECT hash FROM merkle_nodes WHERE level=0 ORDER BY index_val ASC"
        )
        return [row["hash"] for row in cur.fetchall()]

    def _compute_root(self, leaves: List[str]) -> str:
        if not leaves:
            return hashlib.sha256("empty".encode()).hexdigest()
        level = leaves[:]
        while len(level) > 1:
            next_level: List[str] = []
            for i in range(0, len(level), 2):
                left = level[i]
                right = level[i + 1] if i + 1 < len(level) else level[i]
                next_level.append(_hash_pair(left, right))
            level = next_level
        return level[0]
