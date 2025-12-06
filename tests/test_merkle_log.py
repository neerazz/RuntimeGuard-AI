from pathlib import Path

from src.core.merkle_log import MerkleAuditLog


def test_merkle_root_changes(tmp_path: Path):
    db = tmp_path / "merkle.db"
    log = MerkleAuditLog(db)
    root_empty = log.get_root()

    idx1, root1 = log.append("first")
    idx2, root2 = log.append("second")

    assert idx1 == 0
    assert idx2 == 1
    assert root1 != root_empty
    assert root2 != root1
    assert log.verify_leaf(0, "first")
    assert log.verify_leaf(1, "second")
