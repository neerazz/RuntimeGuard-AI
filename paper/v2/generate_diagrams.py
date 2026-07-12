#!/usr/bin/env python3
"""Generate conceptual RuntimeGuard-AI V2 protocol figures."""

from pathlib import Path

import matplotlib.pyplot as plt
from matplotlib.patches import FancyArrowPatch, FancyBboxPatch

OUT = Path(__file__).resolve().parent / "figures"
OUT.mkdir(exist_ok=True)

NAVY = "#17324D"
BLUE = "#3B6EA8"
GREEN = "#3B7A57"
ORANGE = "#C66A1B"
RED = "#A33A3A"
LIGHT = "#F4F6F8"
TEXT = "#1E2933"


def box(ax, x, y, w, h, title, detail, color=BLUE, dashed=False):
    patch = FancyBboxPatch(
        (x, y),
        w,
        h,
        boxstyle="round,pad=0.025,rounding_size=0.035",
        linewidth=1.6,
        edgecolor=color,
        facecolor="white",
        linestyle="--" if dashed else "-",
    )
    ax.add_patch(patch)
    ax.text(x + w / 2, y + h * 0.68, title, ha="center", va="center", fontsize=10.5, weight="bold", color=TEXT)
    ax.text(x + w / 2, y + h * 0.30, detail, ha="center", va="center", fontsize=8.2, color=TEXT, linespacing=1.25)


def arrow(ax, start, end, color=NAVY, label=None, yoff=0.0, dashed=False):
    patch = FancyArrowPatch(
        start,
        end,
        arrowstyle="-|>",
        mutation_scale=13,
        linewidth=1.5,
        color=color,
        linestyle="--" if dashed else "-",
    )
    ax.add_patch(patch)
    if label:
        ax.text((start[0] + end[0]) / 2, (start[1] + end[1]) / 2 + yoff, label, ha="center", va="center", fontsize=8.2, color=color)


def protocol_lifecycle():
    fig, ax = plt.subplots(figsize=(12.8, 4.3))
    ax.set_xlim(0, 13)
    ax.set_ylim(0, 4.3)
    ax.axis("off")

    ax.axvspan(0.15, 7.0, color="#EAF1F8", alpha=0.75, zorder=-2)
    ax.axvspan(7.25, 12.85, color="#EEF5F0", alpha=0.80, zorder=-2)
    ax.text(0.35, 4.0, "SYNCHRONOUS COMMIT PATH", fontsize=10, weight="bold", color=BLUE)
    ax.text(7.45, 4.0, "ASYNCHRONOUS ATTESTATION PATH", fontsize=10, weight="bold", color=GREEN)

    box(ax, 0.35, 2.05, 1.75, 1.25, "1  Evaluate", "compiled policy\nexact-source digest", BLUE)
    box(ax, 2.55, 2.05, 1.85, 1.25, "2  Commit", "sequence + shard\nframed checksum", BLUE)
    box(ax, 4.85, 2.05, 1.75, 1.25, "3  Synchronize", "none / data / full\nexplicit semantics", ORANGE)
    box(ax, 7.55, 2.35, 1.85, 1.15, "5  Recover", "validate frames\ncontiguous order", GREEN)
    box(ax, 9.85, 2.35, 1.75, 1.15, "6  Seal epoch", "Merkle root + count\nchain + Ed25519", GREEN)
    box(ax, 9.85, 0.35, 1.75, 1.15, "7  Verify", "trusted external key\nreceipt → inclusion", GREEN)
    box(ax, 4.85, 0.35, 1.75, 1.15, "4  Return receipt", "signed commitment\ndurable flag", RED)

    arrow(ax, (2.10, 2.67), (2.55, 2.67))
    arrow(ax, (4.40, 2.67), (4.85, 2.67))
    arrow(ax, (5.72, 2.05), (5.72, 1.50), RED, "release", 0.18)
    arrow(ax, (6.60, 2.67), (7.55, 2.92), GREEN, "after commit", 0.20, dashed=True)
    arrow(ax, (9.40, 2.92), (9.85, 2.92), GREEN)
    arrow(ax, (10.72, 2.35), (10.72, 1.50), GREEN)
    arrow(ax, (9.85, 0.92), (6.60, 0.92), RED, "receipt challenge", 0.20, dashed=True)

    ax.text(
        0.35,
        0.28,
        "Invariant: data/full mode returns only after the record's\n"
        "configured host synchronization boundary. Buffered mode\n"
        "returns an explicitly non-durable receipt.",
        fontsize=8.8,
        color=TEXT,
        va="bottom",
    )
    fig.tight_layout(pad=0.4)
    for suffix in ("png", "pdf"):
        fig.savefig(OUT / f"protocol_lifecycle.{suffix}", dpi=220 if suffix == "png" else None, bbox_inches="tight")
    plt.close(fig)


def commit_state_machine():
    fig, ax = plt.subplots(figsize=(9.6, 5.2))
    ax.set_xlim(0, 10)
    ax.set_ylim(0, 6)
    ax.axis("off")

    box(ax, 0.55, 3.95, 2.0, 1.15, "Healthy", "accept new request", BLUE)
    box(ax, 3.35, 3.95, 2.0, 1.15, "Evaluated", "decision not yet\ncommitted", ORANGE)
    box(ax, 6.15, 3.95, 2.0, 1.15, "Committed", "frame complete +\nconfigured sync", GREEN)
    box(ax, 6.15, 1.25, 2.0, 1.15, "Acknowledged", "signed receipt\nreturned", GREEN)
    box(ax, 3.35, 1.25, 2.0, 1.15, "Fail-stopped", "reject further use\nafter append error", RED)
    box(ax, 0.55, 1.25, 2.0, 1.15, "Recovered", "truncate partial tail;\nvalidate prefix", BLUE)

    arrow(ax, (2.55, 4.52), (3.35, 4.52), label="evaluate", yoff=0.18)
    arrow(ax, (5.35, 4.52), (6.15, 4.52), label="append + sync", yoff=0.18)
    arrow(ax, (7.15, 3.95), (7.15, 2.40), color=GREEN, label="sign", yoff=0.0)
    arrow(ax, (4.35, 3.95), (4.35, 2.40), color=RED, label="append error", yoff=0.0)
    restart = FancyArrowPatch(
        (6.15, 1.50),
        (2.55, 1.50),
        arrowstyle="-|>",
        mutation_scale=13,
        linewidth=1.5,
        color=BLUE,
        linestyle="--",
        connectionstyle="arc3,rad=-0.42",
    )
    ax.add_patch(restart)
    ax.text(5.65, 1.02, "restart", ha="center", va="center", fontsize=8.2, color=BLUE)
    arrow(ax, (1.55, 2.40), (1.55, 3.95), color=BLUE, label="validated reopen", yoff=0.0)

    ax.text(
        5.0,
        0.12,
        "No state permits acknowledging an uncommitted data/full-sync record. Exact request replay returns the original commitment;\n"
        "changed-content replay under the same request identifier is rejected.",
        ha="center",
        fontsize=9,
        color=TEXT,
    )
    fig.tight_layout(pad=0.4)
    for suffix in ("png", "pdf"):
        fig.savefig(OUT / f"commit_state_machine.{suffix}", dpi=220 if suffix == "png" else None, bbox_inches="tight")
    plt.close(fig)


if __name__ == "__main__":
    protocol_lifecycle()
    commit_state_machine()
    print(OUT)
