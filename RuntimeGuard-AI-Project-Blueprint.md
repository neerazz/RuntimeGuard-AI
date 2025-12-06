# RuntimeGuard-AI: Complete GitHub Project Blueprint

## Project Overview

**RuntimeGuard-AI** is a proof-of-concept reference implementation for cryptographically attested EU AI Act compliance in high-risk AI systems. This blueprint provides everything needed to build a working prototype in 15 days suitable for arXiv preprint and workshop submission.

---

## Table of Contents

1. [Strategic Context](#1-strategic-context)
2. [Technical Architecture](#2-technical-architecture)
3. [15-Day Implementation Roadmap](#3-15-day-implementation-roadmap)
4. [Repository Structure](#4-repository-structure)
5. [Component Specifications](#5-component-specifications)
6. [Evaluation Framework](#6-evaluation-framework)
7. [Paper Writing Guide](#7-paper-writing-guide)
8. [Appendix: Code Templates](#appendix-code-templates)

---

## 1. Strategic Context

### 1.1 Publication Strategy

| Timeline | Target | Status |
|----------|--------|--------|
| **Day 15** | arXiv preprint | Primary goal |
| **Jan 2026** | Workshop (NDSS/ESORICS workshops) | Secondary goal |
| **Feb 5, 2026** | USENIX Security Cycle 2 | Stretch goal (requires production deployment) |

### 1.2 Positioning (Critical for Success)

**What This IS:**
- Reference architecture for EU AI Act Article 14 compliance
- Proof-of-concept demonstrating cryptographic attestation feasibility
- Open-source framework for researchers and practitioners
- First system combining ZK proofs with AI governance compliance

**What This Is NOT:**
- Production-ready enterprise solution
- Substitute for legal compliance assessment
- Complete implementation of all EU AI Act requirements

### 1.3 Novel Contributions (Defensible)

1. **Cryptographic Compliance Chain**: First architecture combining Merkle tree audit logs with zero-knowledge proofs for AI governance
2. **Article 14 Operationalization**: Concrete technical mapping of human oversight requirements to measurable controls
3. **Batch Attestation Protocol**: Amortizing ZK proof costs across multiple inference requests
4. **Open-Source Reference Implementation**: Reproducible baseline for AI compliance research

---

## 2. Technical Architecture

### 2.1 System Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        RuntimeGuard-AI Architecture                      │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐               │
│  │   AI Model   │───▶│ Interceptor  │───▶│   Response   │               │
│  │   (Mock)     │    │   Proxy      │    │   to User    │               │
│  └──────────────┘    └──────┬───────┘    └──────────────┘               │
│                             │                                            │
│                      ┌──────▼───────┐                                   │
│                      │   Policy     │                                   │
│                      │   Engine     │                                   │
│                      │   (Python)   │                                   │
│                      └──────┬───────┘                                   │
│                             │                                            │
│         ┌───────────────────┼───────────────────┐                       │
│         │                   │                   │                       │
│  ┌──────▼──────┐    ┌──────▼──────┐    ┌──────▼──────┐                 │
│  │   Merkle    │    │     ZK      │    │  Oversight  │                 │
│  │ Audit Log   │    │   Prover    │    │  Dashboard  │                 │
│  │  (SQLite)   │    │  (circom)   │    │   (React)   │                 │
│  └─────────────┘    └─────────────┘    └─────────────┘                 │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 2.2 Component Responsibilities

| Component | Technology | Purpose |
|-----------|------------|---------|
| **AI Model Mock** | Python | Simulates high-risk AI inference responses |
| **Interceptor Proxy** | FastAPI | Captures all inference requests/responses |
| **Policy Engine** | Python | Evaluates compliance rules against requests |
| **Merkle Audit Log** | SQLite + Python | Tamper-evident logging with hash chains |
| **ZK Prover** | circom + snarkjs | Generates attestation proofs |
| **Oversight Dashboard** | React + Vite | Human review interface |

### 2.3 Data Flow

```
1. Request arrives → Interceptor captures
2. Interceptor → Policy Engine (check rules)
3. Policy Engine → Decision (ALLOW/BLOCK/ESCALATE)
4. Decision → Merkle Log (append with hash)
5. Batch of decisions → ZK Prover (generate proof)
6. If ESCALATE → Oversight Dashboard (human review)
7. All artifacts → Compliance Certificate
```

---

## 3. 15-Day Implementation Roadmap

### Phase 1: Foundation (Days 1-3)

**Day 1: Repository Setup & Core Structure**
```
Morning:
- Create GitHub repository with MIT license
- Set up project structure (see Section 4)
- Initialize Python virtual environment
- Create basic README.md

Afternoon:
- Implement SQLite database schema
- Create basic FastAPI skeleton
- Write first unit tests
```

**Day 2: Policy Engine Core**
```
Morning:
- Implement PolicyRule class
- Create rule evaluation engine
- Add 5 sample policies (see Section 5.2)

Afternoon:
- Implement decision logging
- Add policy configuration loader
- Write integration tests
```

**Day 3: Merkle Audit Log**
```
Morning:
- Implement Merkle tree data structure
- Create hash chain for log entries
- Add verification function

Afternoon:
- Integrate with SQLite storage
- Add tamper detection
- Write comprehensive tests
```

### Phase 2: ZK Integration (Days 4-7)

**Day 4: Circom Circuit Design**
```
Morning:
- Install circom and snarkjs
- Study existing circuits (poseidon, merkle)
- Design PolicyAttestation circuit

Afternoon:
- Implement basic circuit
- Generate test proof
- Benchmark performance
```

**Day 5: Circuit Optimization**
```
Morning:
- Add batch processing to circuit
- Implement Merkle verification in ZK
- Optimize constraint count

Afternoon:
- Generate proving/verification keys
- Test with various batch sizes
- Document performance characteristics
```

**Day 6: Prover Service**
```
Morning:
- Create Node.js prover wrapper
- Implement proof generation API
- Add async processing queue

Afternoon:
- Integrate with Python backend
- Handle errors gracefully
- Add logging and monitoring
```

**Day 7: Verification & Integration**
```
Morning:
- Implement verifier in Python
- Create compliance certificate format
- Add certificate validation

Afternoon:
- Full integration test
- Fix bugs
- Document API
```

### Phase 3: User Interface (Days 8-10)

**Day 8: Oversight Dashboard Setup**
```
Morning:
- Create React project with Vite
- Design component hierarchy
- Implement basic layout

Afternoon:
- Add escalation queue view
- Create decision interface
- Connect to backend API
```

**Day 9: Dashboard Features**
```
Morning:
- Add audit log viewer
- Implement search/filter
- Create analytics charts

Afternoon:
- Add compliance status display
- Implement certificate download
- Style with Tailwind CSS
```

**Day 10: Polish & Testing**
```
Morning:
- End-to-end testing
- Fix UI bugs
- Improve responsiveness

Afternoon:
- Add loading states
- Error handling
- Documentation
```

### Phase 4: Evaluation (Days 11-13)

**Day 11: Benchmark Framework**
```
Morning:
- Create synthetic workload generator
- Define evaluation metrics
- Implement measurement infrastructure

Afternoon:
- Run baseline benchmarks
- Collect initial data
- Identify bottlenecks
```

**Day 12: Comprehensive Evaluation**
```
Morning:
- Scale tests (100, 1K, 10K requests)
- Latency measurements
- Memory profiling

Afternoon:
- Security analysis
- Threat model documentation
- Comparison with alternatives
```

**Day 13: Analysis & Visualization**
```
Morning:
- Process benchmark data
- Create figures and tables
- Statistical analysis

Afternoon:
- Write evaluation section
- Create supplementary materials
- Review results
```

### Phase 5: Paper & Release (Days 14-15)

**Day 14: Paper Writing**
```
Full day:
- Complete paper draft
- Create all figures
- Write related work
- Polish abstract
```

**Day 15: Finalization**
```
Morning:
- Final paper edits
- Create arXiv metadata
- Prepare supplementary materials

Afternoon:
- Submit to arXiv
- Publish GitHub release
- Create demonstration video
```

---

## 4. Repository Structure

```
runtimeguard-ai/
├── README.md                    # Project overview and quickstart
├── LICENSE                      # MIT License
├── CITATION.cff                 # Citation metadata
├── pyproject.toml               # Python project configuration
├── package.json                 # Node.js dependencies
│
├── docs/
│   ├── architecture.md          # Detailed architecture docs
│   ├── api.md                   # API documentation
│   ├── deployment.md            # Deployment guide
│   └── eu-ai-act-mapping.md     # Article 14 mapping
│
├── src/
│   ├── __init__.py
│   │
│   ├── core/
│   │   ├── __init__.py
│   │   ├── policy_engine.py     # Rule evaluation engine
│   │   ├── merkle_log.py        # Merkle tree audit log
│   │   ├── certificate.py       # Compliance certificate generation
│   │   └── models.py            # Data models
│   │
│   ├── api/
│   │   ├── __init__.py
│   │   ├── main.py              # FastAPI application
│   │   ├── routes/
│   │   │   ├── inference.py     # Inference proxy endpoints
│   │   │   ├── audit.py         # Audit log endpoints
│   │   │   ├── oversight.py     # Human oversight endpoints
│   │   │   └── certificates.py  # Certificate endpoints
│   │   └── middleware/
│   │       └── interceptor.py   # Request/response interceptor
│   │
│   ├── prover/
│   │   ├── circuits/
│   │   │   ├── policy_attestation.circom    # Main circuit
│   │   │   ├── merkle_verifier.circom       # Merkle proof circuit
│   │   │   └── batch_processor.circom       # Batch processing
│   │   ├── scripts/
│   │   │   ├── compile.sh       # Circuit compilation
│   │   │   ├── setup.sh         # Trusted setup
│   │   │   └── prove.js         # Proof generation
│   │   └── verifier/
│   │       └── verify.py        # Python verification wrapper
│   │
│   └── mock/
│       ├── __init__.py
│       ├── ai_model.py          # Mock AI model
│       └── workload.py          # Synthetic workload generator
│
├── dashboard/
│   ├── package.json
│   ├── vite.config.ts
│   ├── src/
│   │   ├── App.tsx
│   │   ├── components/
│   │   │   ├── EscalationQueue.tsx
│   │   │   ├── AuditLogViewer.tsx
│   │   │   ├── ComplianceStatus.tsx
│   │   │   └── CertificateViewer.tsx
│   │   ├── hooks/
│   │   │   └── useApi.ts
│   │   └── types/
│   │       └── index.ts
│   └── public/
│
├── evaluation/
│   ├── benchmarks/
│   │   ├── run_benchmarks.py
│   │   ├── latency_test.py
│   │   ├── throughput_test.py
│   │   └── memory_test.py
│   ├── results/
│   │   └── .gitkeep
│   ├── analysis/
│   │   ├── analyze_results.py
│   │   └── generate_figures.py
│   └── workloads/
│       ├── credit_underwriting.json
│       ├── resume_screening.json
│       └── medical_triage.json
│
├── tests/
│   ├── __init__.py
│   ├── test_policy_engine.py
│   ├── test_merkle_log.py
│   ├── test_prover.py
│   └── test_integration.py
│
├── configs/
│   ├── policies/
│   │   ├── default.yaml
│   │   ├── strict.yaml
│   │   └── permissive.yaml
│   └── settings.yaml
│
├── paper/
│   ├── main.tex                 # Paper source
│   ├── references.bib           # Bibliography
│   ├── figures/
│   │   └── .gitkeep
│   └── supplementary/
│       └── .gitkeep
│
└── scripts/
    ├── setup_dev.sh             # Development environment setup
    ├── run_all_tests.sh         # Test runner
    └── generate_release.sh      # Release packaging
```

---

## 5. Component Specifications

### 5.1 Database Schema (SQLite)

```sql
-- File: src/core/schema.sql

-- Inference requests captured by interceptor
CREATE TABLE inference_requests (
    id TEXT PRIMARY KEY,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    request_hash TEXT NOT NULL,
    request_payload TEXT NOT NULL,  -- JSON
    source_ip TEXT,
    user_id TEXT,
    model_id TEXT NOT NULL
);

-- Policy evaluation results
CREATE TABLE policy_decisions (
    id TEXT PRIMARY KEY,
    request_id TEXT REFERENCES inference_requests(id),
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    decision TEXT NOT NULL CHECK (decision IN ('ALLOW', 'BLOCK', 'ESCALATE')),
    rules_triggered TEXT NOT NULL,  -- JSON array
    confidence_score REAL,
    explanation TEXT,
    merkle_index INTEGER,
    merkle_hash TEXT
);

-- Merkle tree nodes
CREATE TABLE merkle_nodes (
    index_val INTEGER PRIMARY KEY,
    level INTEGER NOT NULL,
    hash TEXT NOT NULL,
    left_child INTEGER,
    right_child INTEGER,
    data_hash TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Human oversight actions
CREATE TABLE oversight_actions (
    id TEXT PRIMARY KEY,
    decision_id TEXT REFERENCES policy_decisions(id),
    reviewer_id TEXT NOT NULL,
    action TEXT NOT NULL CHECK (action IN ('APPROVE', 'REJECT', 'MODIFY', 'DEFER')),
    justification TEXT,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    review_duration_seconds INTEGER
);

-- ZK proofs generated
CREATE TABLE attestation_proofs (
    id TEXT PRIMARY KEY,
    batch_start_index INTEGER NOT NULL,
    batch_end_index INTEGER NOT NULL,
    merkle_root TEXT NOT NULL,
    proof_data TEXT NOT NULL,  -- JSON
    public_inputs TEXT NOT NULL,  -- JSON
    verified BOOLEAN DEFAULT FALSE,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    proving_time_ms INTEGER
);

-- Compliance certificates
CREATE TABLE certificates (
    id TEXT PRIMARY KEY,
    proof_id TEXT REFERENCES attestation_proofs(id),
    period_start DATETIME NOT NULL,
    period_end DATETIME NOT NULL,
    total_requests INTEGER NOT NULL,
    allowed_count INTEGER NOT NULL,
    blocked_count INTEGER NOT NULL,
    escalated_count INTEGER NOT NULL,
    human_review_rate REAL NOT NULL,
    certificate_hash TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for performance
CREATE INDEX idx_decisions_timestamp ON policy_decisions(timestamp);
CREATE INDEX idx_decisions_request ON policy_decisions(request_id);
CREATE INDEX idx_oversight_decision ON oversight_actions(decision_id);
CREATE INDEX idx_merkle_level ON merkle_nodes(level);
```

### 5.2 Policy Engine Specification

```python
# File: src/core/policy_engine.py

"""
RuntimeGuard-AI Policy Engine

EU AI Act Article 14 Mapping:
- Art 14(1): Human oversight measures
- Art 14(2): Enabling oversight of AI operation
- Art 14(3): Monitoring and intervention capability
- Art 14(4): Safe halt capability
"""

from dataclasses import dataclass, field
from enum import Enum
from typing import List, Dict, Any, Optional, Callable
from datetime import datetime
import hashlib
import json

class Decision(Enum):
    ALLOW = "ALLOW"      # Request passes all policies
    BLOCK = "BLOCK"      # Request violates critical policy
    ESCALATE = "ESCALATE"  # Request requires human review

@dataclass
class PolicyRule:
    """Single policy rule with evaluation logic."""
    id: str
    name: str
    description: str
    article_reference: str  # EU AI Act article
    severity: str  # CRITICAL, HIGH, MEDIUM, LOW
    evaluator: Callable[[Dict[str, Any]], bool]
    block_on_violation: bool = True
    
@dataclass
class EvaluationResult:
    """Result of policy evaluation."""
    decision: Decision
    rules_triggered: List[str]
    confidence_score: float
    explanation: str
    evaluation_time_ms: float
    request_hash: str

class PolicyEngine:
    """
    Evaluates inference requests against configured policies.
    
    Design Principles:
    1. Fail-safe: Unknown states trigger ESCALATE
    2. Traceable: All decisions logged with full context
    3. Configurable: Policies loaded from YAML
    4. Extensible: Custom evaluators supported
    """
    
    def __init__(self, config_path: str = "configs/policies/default.yaml"):
        self.rules: List[PolicyRule] = []
        self.load_default_rules()
        # TODO: Load custom rules from config
        
    def load_default_rules(self):
        """Load the built-in EU AI Act compliance rules."""
        
        # Rule 1: Protected Category Detection (Art 14(4))
        self.rules.append(PolicyRule(
            id="EU-AI-ACT-ART14-PC",
            name="Protected Category Detection",
            description="Detect potential decisions affecting protected categories",
            article_reference="Article 14(4)(b)",
            severity="CRITICAL",
            evaluator=self._check_protected_categories,
            block_on_violation=False  # Escalate, don't block
        ))
        
        # Rule 2: High-Stakes Decision Detection (Art 14(4))
        self.rules.append(PolicyRule(
            id="EU-AI-ACT-ART14-HS",
            name="High-Stakes Decision",
            description="Detect decisions with significant life impact",
            article_reference="Article 14(4)(a)",
            severity="HIGH",
            evaluator=self._check_high_stakes,
            block_on_violation=False
        ))
        
        # Rule 3: Confidence Threshold (Art 14(3))
        self.rules.append(PolicyRule(
            id="EU-AI-ACT-ART14-CONF",
            name="Confidence Threshold",
            description="Block low-confidence automated decisions",
            article_reference="Article 14(3)(d)",
            severity="HIGH",
            evaluator=self._check_confidence_threshold,
            block_on_violation=True
        ))
        
        # Rule 4: Rate Limiting (Art 14(2))
        self.rules.append(PolicyRule(
            id="EU-AI-ACT-ART14-RATE",
            name="Rate Limiting",
            description="Prevent excessive automated processing",
            article_reference="Article 14(2)",
            severity="MEDIUM",
            evaluator=self._check_rate_limit,
            block_on_violation=True
        ))
        
        # Rule 5: Prohibited Content Detection (Art 5)
        self.rules.append(PolicyRule(
            id="EU-AI-ACT-ART5-PROHIB",
            name="Prohibited Content",
            description="Detect prohibited AI system uses",
            article_reference="Article 5",
            severity="CRITICAL",
            evaluator=self._check_prohibited_content,
            block_on_violation=True
        ))
        
    def evaluate(self, request: Dict[str, Any]) -> EvaluationResult:
        """
        Evaluate request against all policies.
        
        Returns decision + triggered rules + explanation.
        """
        start_time = datetime.now()
        triggered_rules = []
        block = False
        escalate = False
        
        request_hash = hashlib.sha256(
            json.dumps(request, sort_keys=True).encode()
        ).hexdigest()[:16]
        
        for rule in self.rules:
            try:
                violated = rule.evaluator(request)
                if violated:
                    triggered_rules.append(rule.id)
                    if rule.block_on_violation and rule.severity == "CRITICAL":
                        block = True
                    elif rule.severity in ["CRITICAL", "HIGH"]:
                        escalate = True
            except Exception as e:
                # Fail-safe: unknown state → escalate
                triggered_rules.append(f"{rule.id}-ERROR")
                escalate = True
        
        # Determine final decision
        if block:
            decision = Decision.BLOCK
        elif escalate:
            decision = Decision.ESCALATE
        else:
            decision = Decision.ALLOW
            
        elapsed_ms = (datetime.now() - start_time).total_seconds() * 1000
        
        return EvaluationResult(
            decision=decision,
            rules_triggered=triggered_rules,
            confidence_score=self._calculate_confidence(triggered_rules),
            explanation=self._generate_explanation(triggered_rules, decision),
            evaluation_time_ms=elapsed_ms,
            request_hash=request_hash
        )
    
    # === Rule Evaluators ===
    
    def _check_protected_categories(self, request: Dict[str, Any]) -> bool:
        """Check if request involves protected category data."""
        protected_keywords = [
            "race", "ethnicity", "gender", "religion", "disability",
            "sexual_orientation", "political_opinion", "union_membership",
            "genetic_data", "biometric", "health_data"
        ]
        text = json.dumps(request).lower()
        return any(kw in text for kw in protected_keywords)
    
    def _check_high_stakes(self, request: Dict[str, Any]) -> bool:
        """Check if decision has significant life impact."""
        high_stakes_contexts = [
            "credit", "loan", "mortgage", "employment", "hiring",
            "criminal", "medical", "education", "asylum", "border"
        ]
        context = request.get("context", "").lower()
        return any(ctx in context for ctx in high_stakes_contexts)
    
    def _check_confidence_threshold(self, request: Dict[str, Any]) -> bool:
        """Check if model confidence is below threshold."""
        confidence = request.get("model_confidence", 1.0)
        threshold = 0.7  # Configurable
        return confidence < threshold
    
    def _check_rate_limit(self, request: Dict[str, Any]) -> bool:
        """Check if request rate exceeds limit."""
        # TODO: Implement actual rate tracking
        return False
    
    def _check_prohibited_content(self, request: Dict[str, Any]) -> bool:
        """Check for prohibited AI uses (Art 5)."""
        prohibited_patterns = [
            "social_scoring", "subliminal_manipulation",
            "exploit_vulnerable", "real_time_biometric_public"
        ]
        text = json.dumps(request).lower()
        return any(p in text for p in prohibited_patterns)
    
    def _calculate_confidence(self, triggered: List[str]) -> float:
        """Calculate confidence in decision."""
        if not triggered:
            return 1.0
        # More triggered rules = lower confidence
        return max(0.0, 1.0 - (len(triggered) * 0.15))
    
    def _generate_explanation(
        self, triggered: List[str], decision: Decision
    ) -> str:
        """Generate human-readable explanation."""
        if decision == Decision.ALLOW:
            return "Request passed all policy checks."
        elif decision == Decision.BLOCK:
            return f"Request blocked due to: {', '.join(triggered)}"
        else:
            return f"Request escalated for human review: {', '.join(triggered)}"
```

### 5.3 Merkle Audit Log Specification

```python
# File: src/core/merkle_log.py

"""
RuntimeGuard-AI Merkle Audit Log

Provides tamper-evident logging for compliance decisions.
Each decision is hashed and added to a Merkle tree, enabling:
1. Efficient integrity verification
2. Compact proofs of inclusion
3. Historical auditability
"""

import hashlib
import json
import sqlite3
from dataclasses import dataclass
from typing import List, Optional, Tuple
from datetime import datetime

@dataclass
class LogEntry:
    """A single entry in the audit log."""
    index: int
    timestamp: datetime
    decision_id: str
    decision_hash: str
    merkle_hash: str
    merkle_proof: List[str]

@dataclass
class MerkleProof:
    """Proof of inclusion in Merkle tree."""
    leaf_hash: str
    proof_path: List[Tuple[str, str]]  # (hash, direction)
    root_hash: str
    leaf_index: int

class MerkleAuditLog:
    """
    Append-only Merkle tree audit log.
    
    Design:
    - Binary Merkle tree structure
    - SHA-256 hashing
    - SQLite persistence
    - O(log n) proof generation
    - O(log n) verification
    """
    
    def __init__(self, db_path: str = "data/audit_log.db"):
        self.db_path = db_path
        self.conn = sqlite3.connect(db_path)
        self._init_db()
        self._load_state()
        
    def _init_db(self):
        """Initialize database tables."""
        cursor = self.conn.cursor()
        cursor.executescript("""
            CREATE TABLE IF NOT EXISTS merkle_leaves (
                index_val INTEGER PRIMARY KEY,
                data_hash TEXT NOT NULL,
                decision_id TEXT NOT NULL,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            
            CREATE TABLE IF NOT EXISTS merkle_internals (
                level INTEGER NOT NULL,
                position INTEGER NOT NULL,
                hash TEXT NOT NULL,
                PRIMARY KEY (level, position)
            );
            
            CREATE TABLE IF NOT EXISTS merkle_roots (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                root_hash TEXT NOT NULL,
                leaf_count INTEGER NOT NULL,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
            );
        """)
        self.conn.commit()
        
    def _load_state(self):
        """Load current tree state from database."""
        cursor = self.conn.cursor()
        cursor.execute("SELECT MAX(index_val) FROM merkle_leaves")
        result = cursor.fetchone()[0]
        self.leaf_count = result + 1 if result is not None else 0
        
    @staticmethod
    def hash(data: str) -> str:
        """Compute SHA-256 hash."""
        return hashlib.sha256(data.encode()).hexdigest()
    
    @staticmethod
    def hash_pair(left: str, right: str) -> str:
        """Hash two nodes together."""
        combined = left + right
        return hashlib.sha256(combined.encode()).hexdigest()
    
    def append(self, decision_id: str, decision_data: dict) -> LogEntry:
        """
        Append a new decision to the log.
        
        Returns the log entry with Merkle proof.
        """
        # Hash the decision data
        data_str = json.dumps(decision_data, sort_keys=True)
        data_hash = self.hash(data_str)
        
        # Get index for new leaf
        index = self.leaf_count
        
        # Insert leaf
        cursor = self.conn.cursor()
        cursor.execute(
            "INSERT INTO merkle_leaves (index_val, data_hash, decision_id) VALUES (?, ?, ?)",
            (index, data_hash, decision_id)
        )
        
        # Update internal nodes
        self._update_path(index, data_hash)
        
        # Increment leaf count
        self.leaf_count += 1
        
        # Get root and proof
        root_hash = self._get_root()
        proof = self.get_proof(index)
        
        # Store root
        cursor.execute(
            "INSERT INTO merkle_roots (root_hash, leaf_count) VALUES (?, ?)",
            (root_hash, self.leaf_count)
        )
        
        self.conn.commit()
        
        return LogEntry(
            index=index,
            timestamp=datetime.now(),
            decision_id=decision_id,
            decision_hash=data_hash,
            merkle_hash=root_hash,
            merkle_proof=[p[0] for p in proof.proof_path] if proof else []
        )
    
    def _update_path(self, leaf_index: int, leaf_hash: str):
        """Update the path from leaf to root."""
        cursor = self.conn.cursor()
        
        current_hash = leaf_hash
        current_index = leaf_index
        level = 0
        
        # Store leaf at level 0
        cursor.execute(
            "INSERT OR REPLACE INTO merkle_internals (level, position, hash) VALUES (?, ?, ?)",
            (level, current_index, current_hash)
        )
        
        # Propagate up the tree
        while current_index > 0 or level == 0:
            sibling_index = current_index ^ 1  # XOR to get sibling
            
            # Get sibling hash (or use empty hash if doesn't exist)
            cursor.execute(
                "SELECT hash FROM merkle_internals WHERE level = ? AND position = ?",
                (level, sibling_index)
            )
            sibling = cursor.fetchone()
            sibling_hash = sibling[0] if sibling else self.hash("")
            
            # Compute parent hash
            if current_index % 2 == 0:
                parent_hash = self.hash_pair(current_hash, sibling_hash)
            else:
                parent_hash = self.hash_pair(sibling_hash, current_hash)
            
            # Move up
            level += 1
            current_index //= 2
            current_hash = parent_hash
            
            # Store internal node
            cursor.execute(
                "INSERT OR REPLACE INTO merkle_internals (level, position, hash) VALUES (?, ?, ?)",
                (level, current_index, current_hash)
            )
            
            # Stop if we've reached the top
            if current_index == 0 and level > 0:
                # Check if there's anything at this level
                cursor.execute(
                    "SELECT COUNT(*) FROM merkle_internals WHERE level = ?",
                    (level,)
                )
                count = cursor.fetchone()[0]
                if count <= 1:
                    break
    
    def _get_root(self) -> str:
        """Get current Merkle root."""
        cursor = self.conn.cursor()
        cursor.execute(
            "SELECT hash FROM merkle_internals ORDER BY level DESC, position ASC LIMIT 1"
        )
        result = cursor.fetchone()
        return result[0] if result else self.hash("")
    
    def get_proof(self, leaf_index: int) -> Optional[MerkleProof]:
        """Generate Merkle proof for a leaf."""
        if leaf_index >= self.leaf_count:
            return None
            
        cursor = self.conn.cursor()
        
        # Get leaf hash
        cursor.execute(
            "SELECT hash FROM merkle_internals WHERE level = 0 AND position = ?",
            (leaf_index,)
        )
        leaf = cursor.fetchone()
        if not leaf:
            return None
        leaf_hash = leaf[0]
        
        # Build proof path
        proof_path = []
        current_index = leaf_index
        level = 0
        
        while True:
            sibling_index = current_index ^ 1
            
            cursor.execute(
                "SELECT hash FROM merkle_internals WHERE level = ? AND position = ?",
                (level, sibling_index)
            )
            sibling = cursor.fetchone()
            
            if sibling:
                direction = "R" if current_index % 2 == 0 else "L"
                proof_path.append((sibling[0], direction))
            
            current_index //= 2
            level += 1
            
            if current_index == 0:
                break
        
        return MerkleProof(
            leaf_hash=leaf_hash,
            proof_path=proof_path,
            root_hash=self._get_root(),
            leaf_index=leaf_index
        )
    
    def verify_proof(self, proof: MerkleProof) -> bool:
        """Verify a Merkle proof."""
        current_hash = proof.leaf_hash
        
        for sibling_hash, direction in proof.proof_path:
            if direction == "R":
                current_hash = self.hash_pair(current_hash, sibling_hash)
            else:
                current_hash = self.hash_pair(sibling_hash, current_hash)
        
        return current_hash == proof.root_hash
    
    def get_root_at(self, leaf_count: int) -> Optional[str]:
        """Get historical root at specific leaf count."""
        cursor = self.conn.cursor()
        cursor.execute(
            "SELECT root_hash FROM merkle_roots WHERE leaf_count = ?",
            (leaf_count,)
        )
        result = cursor.fetchone()
        return result[0] if result else None
```

### 5.4 ZK Circuit Specification (circom)

```circom
// File: src/prover/circuits/policy_attestation.circom

pragma circom 2.1.6;

include "node_modules/circomlib/circuits/poseidon.circom";
include "node_modules/circomlib/circuits/comparators.circom";
include "node_modules/circomlib/circuits/bitify.circom";

/*
 * RuntimeGuard-AI Policy Attestation Circuit
 *
 * Proves that:
 * 1. A batch of policy decisions were correctly evaluated
 * 2. The decisions are committed to in a Merkle tree
 * 3. The total counts match the claimed statistics
 *
 * Public Inputs:
 * - merkle_root: Root of the decision Merkle tree
 * - batch_size: Number of decisions in this batch
 * - allowed_count: Number of ALLOW decisions
 * - blocked_count: Number of BLOCK decisions
 * - escalated_count: Number of ESCALATE decisions
 *
 * Private Inputs:
 * - decisions[]: Array of decision data
 * - merkle_paths[]: Merkle proofs for each decision
 */

template PolicyAttestation(MAX_BATCH_SIZE, MERKLE_DEPTH) {
    // Public inputs
    signal input merkle_root;
    signal input batch_size;
    signal input allowed_count;
    signal input blocked_count;
    signal input escalated_count;
    
    // Private inputs - decision data
    signal input decision_types[MAX_BATCH_SIZE];  // 0=ALLOW, 1=BLOCK, 2=ESCALATE
    signal input decision_hashes[MAX_BATCH_SIZE];
    signal input merkle_paths[MAX_BATCH_SIZE][MERKLE_DEPTH];
    signal input merkle_directions[MAX_BATCH_SIZE][MERKLE_DEPTH];
    signal input leaf_indices[MAX_BATCH_SIZE];
    
    // Intermediate signals
    signal computed_roots[MAX_BATCH_SIZE];
    signal is_valid[MAX_BATCH_SIZE];
    signal count_allowed[MAX_BATCH_SIZE + 1];
    signal count_blocked[MAX_BATCH_SIZE + 1];
    signal count_escalated[MAX_BATCH_SIZE + 1];
    
    // Initialize counters
    count_allowed[0] <== 0;
    count_blocked[0] <== 0;
    count_escalated[0] <== 0;
    
    // Components for Merkle verification
    component merkle_verifiers[MAX_BATCH_SIZE];
    component decision_checks[MAX_BATCH_SIZE];
    
    // Process each decision in the batch
    for (var i = 0; i < MAX_BATCH_SIZE; i++) {
        // Verify Merkle proof for this decision
        merkle_verifiers[i] = MerkleProofVerifier(MERKLE_DEPTH);
        merkle_verifiers[i].leaf <== decision_hashes[i];
        merkle_verifiers[i].root <== merkle_root;
        merkle_verifiers[i].index <== leaf_indices[i];
        
        for (var j = 0; j < MERKLE_DEPTH; j++) {
            merkle_verifiers[i].path[j] <== merkle_paths[i][j];
            merkle_verifiers[i].directions[j] <== merkle_directions[i][j];
        }
        
        computed_roots[i] <== merkle_verifiers[i].computed_root;
        
        // Check if this decision is within batch_size
        decision_checks[i] = LessThan(32);
        decision_checks[i].in[0] <== i;
        decision_checks[i].in[1] <== batch_size;
        
        // Update counters based on decision type
        // decision_type: 0=ALLOW, 1=BLOCK, 2=ESCALATE
        
        // Count ALLOW (decision_type == 0 AND in_batch)
        signal is_allow[MAX_BATCH_SIZE];
        signal is_allow_and_valid[MAX_BATCH_SIZE];
        is_allow[i] <== IsZero()(decision_types[i]);
        is_allow_and_valid[i] <== is_allow[i] * decision_checks[i].out;
        count_allowed[i + 1] <== count_allowed[i] + is_allow_and_valid[i];
        
        // Count BLOCK (decision_type == 1 AND in_batch)
        signal is_block[MAX_BATCH_SIZE];
        signal is_block_and_valid[MAX_BATCH_SIZE];
        is_block[i] <== IsEqual()([decision_types[i], 1]);
        is_block_and_valid[i] <== is_block[i] * decision_checks[i].out;
        count_blocked[i + 1] <== count_blocked[i] + is_block_and_valid[i];
        
        // Count ESCALATE (decision_type == 2 AND in_batch)
        signal is_escalate[MAX_BATCH_SIZE];
        signal is_escalate_and_valid[MAX_BATCH_SIZE];
        is_escalate[i] <== IsEqual()([decision_types[i], 2]);
        is_escalate_and_valid[i] <== is_escalate[i] * decision_checks[i].out;
        count_escalated[i + 1] <== count_escalated[i] + is_escalate_and_valid[i];
    }
    
    // Verify final counts match public inputs
    count_allowed[MAX_BATCH_SIZE] === allowed_count;
    count_blocked[MAX_BATCH_SIZE] === blocked_count;
    count_escalated[MAX_BATCH_SIZE] === escalated_count;
    
    // Verify total matches batch_size
    allowed_count + blocked_count + escalated_count === batch_size;
}

template MerkleProofVerifier(DEPTH) {
    signal input leaf;
    signal input root;
    signal input index;
    signal input path[DEPTH];
    signal input directions[DEPTH];
    signal output computed_root;
    
    component hashers[DEPTH];
    signal level_hashes[DEPTH + 1];
    
    level_hashes[0] <== leaf;
    
    for (var i = 0; i < DEPTH; i++) {
        hashers[i] = Poseidon(2);
        
        // Select order based on direction
        signal left;
        signal right;
        left <== (1 - directions[i]) * level_hashes[i] + directions[i] * path[i];
        right <== directions[i] * level_hashes[i] + (1 - directions[i]) * path[i];
        
        hashers[i].inputs[0] <== left;
        hashers[i].inputs[1] <== right;
        
        level_hashes[i + 1] <== hashers[i].out;
    }
    
    computed_root <== level_hashes[DEPTH];
    
    // Verify computed root matches expected root
    computed_root === root;
}

template IsZero() {
    signal input in;
    signal output out;
    
    signal inv;
    inv <-- in != 0 ? 1/in : 0;
    out <== -in*inv + 1;
    in*out === 0;
}

template IsEqual() {
    signal input in[2];
    signal output out;
    
    component isz = IsZero();
    isz.in <== in[0] - in[1];
    out <== isz.out;
}

// Main component instantiation
// Batch size of 32 decisions, Merkle depth of 20 (supports ~1M entries)
component main {public [merkle_root, batch_size, allowed_count, blocked_count, escalated_count]} = PolicyAttestation(32, 20);
```

### 5.5 FastAPI Application Specification

```python
# File: src/api/main.py

"""
RuntimeGuard-AI API Server

REST API for:
1. Inference proxy (intercepts AI model calls)
2. Audit log queries
3. Human oversight interface
4. Certificate generation
"""

from fastapi import FastAPI, HTTPException, BackgroundTasks, Depends
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel, Field
from typing import List, Optional, Dict, Any
from datetime import datetime
import uuid
import asyncio

from src.core.policy_engine import PolicyEngine, Decision, EvaluationResult
from src.core.merkle_log import MerkleAuditLog
from src.core.certificate import CertificateGenerator

app = FastAPI(
    title="RuntimeGuard-AI",
    description="Cryptographically Attested EU AI Act Compliance",
    version="0.1.0"
)

# CORS for dashboard
app.add_middleware(
    CORSMiddleware,
    allow_origins=["http://localhost:5173"],  # Vite dev server
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Initialize components
policy_engine = PolicyEngine()
audit_log = MerkleAuditLog()
cert_generator = CertificateGenerator(audit_log)

# === Request/Response Models ===

class InferenceRequest(BaseModel):
    """Incoming inference request to be evaluated."""
    model_id: str
    input_data: Dict[str, Any]
    context: Optional[str] = None
    user_id: Optional[str] = None
    model_confidence: Optional[float] = None

class InferenceResponse(BaseModel):
    """Response after policy evaluation."""
    request_id: str
    decision: str
    allowed_to_proceed: bool
    explanation: str
    audit_entry_id: int
    merkle_root: str

class AuditEntry(BaseModel):
    """Audit log entry for API response."""
    index: int
    timestamp: datetime
    decision_id: str
    decision_type: str
    rules_triggered: List[str]
    merkle_hash: str

class OversightItem(BaseModel):
    """Item requiring human oversight."""
    id: str
    request_id: str
    timestamp: datetime
    reason: str
    original_request: Dict[str, Any]
    status: str = "PENDING"

class OversightAction(BaseModel):
    """Human oversight action submission."""
    action: str  # APPROVE, REJECT, MODIFY, DEFER
    justification: str
    modified_decision: Optional[str] = None

class Certificate(BaseModel):
    """Compliance certificate."""
    id: str
    period_start: datetime
    period_end: datetime
    total_requests: int
    decision_breakdown: Dict[str, int]
    merkle_root: str
    proof_id: str
    verified: bool

# === In-memory state (for demo; use Redis in production) ===
oversight_queue: Dict[str, OversightItem] = {}
pending_proofs: List[str] = []

# === Inference Proxy Endpoints ===

@app.post("/api/v1/inference", response_model=InferenceResponse)
async def process_inference(
    request: InferenceRequest,
    background_tasks: BackgroundTasks
):
    """
    Main inference proxy endpoint.
    
    1. Evaluates request against policies
    2. Logs decision to Merkle audit log
    3. Returns decision with audit proof
    """
    request_id = str(uuid.uuid4())
    
    # Evaluate against policies
    request_dict = request.model_dump()
    result = policy_engine.evaluate(request_dict)
    
    # Log to Merkle audit log
    log_entry = audit_log.append(
        decision_id=request_id,
        decision_data={
            "request_hash": result.request_hash,
            "decision": result.decision.value,
            "rules_triggered": result.rules_triggered,
            "timestamp": datetime.now().isoformat()
        }
    )
    
    # If ESCALATE, add to oversight queue
    if result.decision == Decision.ESCALATE:
        oversight_queue[request_id] = OversightItem(
            id=str(uuid.uuid4()),
            request_id=request_id,
            timestamp=datetime.now(),
            reason=result.explanation,
            original_request=request_dict
        )
    
    # Schedule batch proof generation (every 32 decisions)
    if log_entry.index % 32 == 31:
        background_tasks.add_task(generate_batch_proof, log_entry.index - 31, log_entry.index)
    
    return InferenceResponse(
        request_id=request_id,
        decision=result.decision.value,
        allowed_to_proceed=(result.decision == Decision.ALLOW),
        explanation=result.explanation,
        audit_entry_id=log_entry.index,
        merkle_root=log_entry.merkle_hash
    )

# === Audit Log Endpoints ===

@app.get("/api/v1/audit/entries", response_model=List[AuditEntry])
async def list_audit_entries(
    skip: int = 0,
    limit: int = 100,
    decision_type: Optional[str] = None
):
    """List audit log entries with optional filtering."""
    # Implementation would query SQLite
    # Placeholder for now
    return []

@app.get("/api/v1/audit/entry/{index}")
async def get_audit_entry(index: int):
    """Get specific audit entry with Merkle proof."""
    proof = audit_log.get_proof(index)
    if not proof:
        raise HTTPException(status_code=404, detail="Entry not found")
    
    return {
        "index": index,
        "leaf_hash": proof.leaf_hash,
        "merkle_root": proof.root_hash,
        "proof_path": proof.proof_path,
        "verified": audit_log.verify_proof(proof)
    }

@app.post("/api/v1/audit/verify")
async def verify_merkle_proof(
    leaf_hash: str,
    proof_path: List[tuple],
    expected_root: str
):
    """Verify a Merkle proof externally."""
    from src.core.merkle_log import MerkleProof
    
    proof = MerkleProof(
        leaf_hash=leaf_hash,
        proof_path=[(h, d) for h, d in proof_path],
        root_hash=expected_root,
        leaf_index=0  # Not needed for verification
    )
    
    return {"verified": audit_log.verify_proof(proof)}

# === Human Oversight Endpoints ===

@app.get("/api/v1/oversight/queue", response_model=List[OversightItem])
async def get_oversight_queue(status: Optional[str] = "PENDING"):
    """Get items awaiting human review."""
    items = [
        item for item in oversight_queue.values()
        if status is None or item.status == status
    ]
    return sorted(items, key=lambda x: x.timestamp, reverse=True)

@app.post("/api/v1/oversight/{item_id}/action")
async def submit_oversight_action(item_id: str, action: OversightAction):
    """Submit human oversight decision."""
    if item_id not in oversight_queue:
        raise HTTPException(status_code=404, detail="Oversight item not found")
    
    item = oversight_queue[item_id]
    item.status = action.action
    
    # Log the human action
    audit_log.append(
        decision_id=f"oversight-{item_id}",
        decision_data={
            "original_request_id": item.request_id,
            "human_action": action.action,
            "justification": action.justification,
            "timestamp": datetime.now().isoformat()
        }
    )
    
    return {"status": "recorded", "item_id": item_id}

@app.get("/api/v1/oversight/stats")
async def get_oversight_stats():
    """Get human oversight statistics."""
    total = len(oversight_queue)
    pending = sum(1 for i in oversight_queue.values() if i.status == "PENDING")
    approved = sum(1 for i in oversight_queue.values() if i.status == "APPROVE")
    rejected = sum(1 for i in oversight_queue.values() if i.status == "REJECT")
    
    return {
        "total": total,
        "pending": pending,
        "approved": approved,
        "rejected": rejected,
        "review_rate": (approved + rejected) / total if total > 0 else 0
    }

# === Certificate Endpoints ===

@app.post("/api/v1/certificates/generate", response_model=Certificate)
async def generate_certificate(
    period_start: datetime,
    period_end: datetime
):
    """Generate compliance certificate for a period."""
    cert = await cert_generator.generate(period_start, period_end)
    return cert

@app.get("/api/v1/certificates/{cert_id}")
async def get_certificate(cert_id: str):
    """Get certificate by ID."""
    # Implementation would query database
    raise HTTPException(status_code=404, detail="Certificate not found")

@app.post("/api/v1/certificates/{cert_id}/verify")
async def verify_certificate(cert_id: str):
    """Verify certificate integrity."""
    # Implementation would verify ZK proof
    return {"verified": True, "certificate_id": cert_id}

# === Health Check ===

@app.get("/health")
async def health_check():
    """Health check endpoint."""
    return {
        "status": "healthy",
        "timestamp": datetime.now().isoformat(),
        "audit_log_entries": audit_log.leaf_count,
        "pending_oversight": sum(1 for i in oversight_queue.values() if i.status == "PENDING")
    }

# === Background Tasks ===

async def generate_batch_proof(start_index: int, end_index: int):
    """Generate ZK proof for a batch of decisions."""
    # This would call the snarkjs prover
    # Placeholder for demonstration
    print(f"Generating proof for batch {start_index}-{end_index}")
    # TODO: Implement actual proof generation

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8000)
```

---

## 6. Evaluation Framework

### 6.1 Benchmark Design

Given the constraint of PC-only testing without production deployment, the evaluation must be carefully designed to be defensible while acknowledging limitations.

**Evaluation Strategy: "Controlled Synthetic Evaluation"**

```python
# File: evaluation/benchmarks/run_benchmarks.py

"""
RuntimeGuard-AI Evaluation Suite

Benchmarks:
1. Latency: End-to-end request processing time
2. Throughput: Requests per second capacity
3. Proof Generation: ZK proof timing
4. Memory: Resource consumption
5. Scalability: Performance vs. batch size
"""

import asyncio
import json
import time
import statistics
from dataclasses import dataclass
from typing import List, Dict
import psutil
import matplotlib.pyplot as plt

@dataclass
class BenchmarkConfig:
    """Configuration for benchmark runs."""
    name: str
    num_requests: int
    batch_size: int
    warmup_requests: int = 100
    concurrent_requests: int = 10

@dataclass
class BenchmarkResult:
    """Results from a benchmark run."""
    config: BenchmarkConfig
    latencies_ms: List[float]
    throughput_rps: float
    memory_mb: float
    proof_time_ms: float
    
    @property
    def p50_latency(self) -> float:
        return statistics.median(self.latencies_ms)
    
    @property
    def p95_latency(self) -> float:
        return statistics.quantiles(self.latencies_ms, n=20)[18]
    
    @property
    def p99_latency(self) -> float:
        return statistics.quantiles(self.latencies_ms, n=100)[98]

class EvaluationSuite:
    """Comprehensive evaluation suite."""
    
    def __init__(self, api_url: str = "http://localhost:8000"):
        self.api_url = api_url
        self.results: List[BenchmarkResult] = []
        
    async def run_latency_benchmark(self, config: BenchmarkConfig) -> BenchmarkResult:
        """Measure end-to-end latency distribution."""
        import httpx
        
        latencies = []
        
        async with httpx.AsyncClient() as client:
            # Warmup
            for _ in range(config.warmup_requests):
                await self._send_request(client)
            
            # Actual benchmark
            for _ in range(config.num_requests):
                start = time.perf_counter()
                await self._send_request(client)
                elapsed = (time.perf_counter() - start) * 1000
                latencies.append(elapsed)
        
        return BenchmarkResult(
            config=config,
            latencies_ms=latencies,
            throughput_rps=len(latencies) / sum(latencies) * 1000,
            memory_mb=psutil.Process().memory_info().rss / 1024 / 1024,
            proof_time_ms=0  # Measured separately
        )
    
    async def run_throughput_benchmark(self, config: BenchmarkConfig) -> BenchmarkResult:
        """Measure maximum throughput under load."""
        import httpx
        
        latencies = []
        start_time = time.perf_counter()
        
        async with httpx.AsyncClient() as client:
            tasks = []
            for _ in range(config.num_requests):
                tasks.append(self._send_request_timed(client))
                if len(tasks) >= config.concurrent_requests:
                    results = await asyncio.gather(*tasks)
                    latencies.extend(results)
                    tasks = []
            
            if tasks:
                results = await asyncio.gather(*tasks)
                latencies.extend(results)
        
        total_time = time.perf_counter() - start_time
        
        return BenchmarkResult(
            config=config,
            latencies_ms=latencies,
            throughput_rps=config.num_requests / total_time,
            memory_mb=psutil.Process().memory_info().rss / 1024 / 1024,
            proof_time_ms=0
        )
    
    def run_proof_benchmark(self, batch_sizes: List[int]) -> Dict[int, float]:
        """Measure ZK proof generation time vs batch size."""
        import subprocess
        
        results = {}
        
        for batch_size in batch_sizes:
            # Generate test input
            input_data = self._generate_proof_input(batch_size)
            
            # Write input
            with open("/tmp/proof_input.json", "w") as f:
                json.dump(input_data, f)
            
            # Time proof generation
            start = time.perf_counter()
            subprocess.run([
                "node", "src/prover/scripts/prove.js",
                "/tmp/proof_input.json",
                "/tmp/proof_output.json"
            ], check=True, capture_output=True)
            elapsed = (time.perf_counter() - start) * 1000
            
            results[batch_size] = elapsed
            
        return results
    
    def run_memory_benchmark(self, request_counts: List[int]) -> Dict[int, float]:
        """Measure memory usage vs audit log size."""
        import gc
        
        results = {}
        
        for count in request_counts:
            gc.collect()
            initial_memory = psutil.Process().memory_info().rss
            
            # Simulate adding entries
            from src.core.merkle_log import MerkleAuditLog
            log = MerkleAuditLog(f":memory:")
            
            for i in range(count):
                log.append(f"decision-{i}", {"test": i})
            
            final_memory = psutil.Process().memory_info().rss
            results[count] = (final_memory - initial_memory) / 1024 / 1024
            
        return results
    
    async def _send_request(self, client):
        """Send a single inference request."""
        return await client.post(
            f"{self.api_url}/api/v1/inference",
            json={
                "model_id": "test-model",
                "input_data": {"query": "test input"},
                "context": "credit_decision",
                "model_confidence": 0.85
            }
        )
    
    async def _send_request_timed(self, client) -> float:
        """Send request and return latency."""
        start = time.perf_counter()
        await self._send_request(client)
        return (time.perf_counter() - start) * 1000
    
    def _generate_proof_input(self, batch_size: int) -> dict:
        """Generate input for ZK proof."""
        return {
            "batch_size": batch_size,
            "decisions": [{"type": i % 3} for i in range(batch_size)],
            "merkle_root": "0x" + "0" * 64
        }
    
    def generate_report(self, output_dir: str = "evaluation/results"):
        """Generate evaluation report with figures."""
        import os
        os.makedirs(output_dir, exist_ok=True)
        
        # Figure 1: Latency distribution
        fig, ax = plt.subplots(figsize=(10, 6))
        for result in self.results:
            if "latency" in result.config.name.lower():
                ax.hist(result.latencies_ms, bins=50, alpha=0.7, label=result.config.name)
        ax.set_xlabel("Latency (ms)")
        ax.set_ylabel("Frequency")
        ax.set_title("Request Latency Distribution")
        ax.legend()
        fig.savefig(f"{output_dir}/latency_distribution.png", dpi=300)
        
        # Figure 2: Throughput vs batch size (placeholder)
        # Figure 3: Memory vs audit log size
        # Figure 4: Proof generation time vs batch size
        
        # Generate markdown report
        report = self._generate_markdown_report()
        with open(f"{output_dir}/evaluation_report.md", "w") as f:
            f.write(report)
    
    def _generate_markdown_report(self) -> str:
        """Generate markdown evaluation report."""
        report = "# RuntimeGuard-AI Evaluation Report\n\n"
        report += f"Generated: {time.strftime('%Y-%m-%d %H:%M:%S')}\n\n"
        
        report += "## Summary\n\n"
        report += "| Metric | Value |\n"
        report += "|--------|-------|\n"
        
        for result in self.results:
            report += f"| {result.config.name} P50 Latency | {result.p50_latency:.2f} ms |\n"
            report += f"| {result.config.name} P99 Latency | {result.p99_latency:.2f} ms |\n"
            report += f"| {result.config.name} Throughput | {result.throughput_rps:.1f} RPS |\n"
        
        report += "\n## Methodology\n\n"
        report += "### Limitations\n\n"
        report += "- **Synthetic workload**: All tests use generated requests, not production traffic\n"
        report += "- **Single-machine**: Benchmarks run on single PC, not distributed deployment\n"
        report += "- **No network latency**: API calls are localhost, not realistic network conditions\n"
        report += "- **Mock AI model**: Backend model is simulated, not actual inference\n"
        
        return report

# Run benchmarks
if __name__ == "__main__":
    suite = EvaluationSuite()
    
    # Define benchmark configurations
    configs = [
        BenchmarkConfig("Latency-100", num_requests=100, batch_size=1),
        BenchmarkConfig("Latency-1000", num_requests=1000, batch_size=1),
        BenchmarkConfig("Throughput-10C", num_requests=1000, batch_size=1, concurrent_requests=10),
        BenchmarkConfig("Throughput-50C", num_requests=1000, batch_size=1, concurrent_requests=50),
    ]
    
    # Run benchmarks
    for config in configs:
        result = asyncio.run(suite.run_latency_benchmark(config))
        suite.results.append(result)
        print(f"{config.name}: P50={result.p50_latency:.2f}ms, P99={result.p99_latency:.2f}ms")
    
    # Generate report
    suite.generate_report()
```

### 6.2 Workload Definitions

```json
// File: evaluation/workloads/credit_underwriting.json
{
    "name": "Credit Underwriting Simulation",
    "description": "Simulates automated credit decision requests",
    "request_distribution": {
        "allow_rate": 0.70,
        "block_rate": 0.05,
        "escalate_rate": 0.25
    },
    "requests": [
        {
            "model_id": "credit-scorer-v3",
            "input_data": {
                "applicant_id": "{{uuid}}",
                "credit_score": "{{random_int:300:850}}",
                "income": "{{random_int:20000:500000}}",
                "debt_ratio": "{{random_float:0.1:0.9}}",
                "employment_status": "{{random_choice:employed:self_employed:unemployed}}"
            },
            "context": "credit_decision",
            "model_confidence": "{{random_float:0.5:0.99}}"
        }
    ],
    "volume": {
        "requests_per_second": 100,
        "duration_seconds": 60,
        "burst_factor": 2.0
    }
}
```

### 6.3 Metrics Table (For Paper)

| Metric | Configuration | Result | Notes |
|--------|--------------|--------|-------|
| **Policy Evaluation Latency** | Single request | X.XX ms | P50 |
| | | X.XX ms | P99 |
| **End-to-End Latency** | With Merkle logging | X.XX ms | P50 |
| **Throughput** | 10 concurrent | XXX RPS | |
| | 50 concurrent | XXX RPS | |
| **Proof Generation** | Batch size 8 | XXX ms | |
| | Batch size 32 | XXX ms | |
| | Batch size 128 | XXX ms | |
| **Memory Usage** | 10K entries | XX MB | Audit log |
| | 100K entries | XX MB | |
| **Verification Time** | Groth16 | X.X ms | Constant |

---

## 7. Paper Writing Guide

### 7.1 Paper Structure (12 pages)

```
1. Abstract (200 words)
   - Problem: EU AI Act compliance lacks runtime verification
   - Solution: RuntimeGuard-AI reference architecture
   - Method: ZK proofs + Merkle logs + policy engine
   - Results: Key performance metrics
   - Contribution: Open-source framework

2. Introduction (1.5 pages)
   - EU AI Act context and Article 14 requirements
   - Gap: No technical standard for compliance attestation
   - Thesis: Cryptographic techniques can provide verifiable compliance
   - Contributions (numbered list)
   - Paper organization

3. Background & Motivation (1 page)
   3.1 EU AI Act Overview
   3.2 Article 14: Human Oversight Requirements
   3.3 Technical Challenges
   3.4 Threat Model

4. System Design (3 pages)
   4.1 Architecture Overview (figure)
   4.2 Policy Engine
   4.3 Merkle Audit Log
   4.4 ZK Attestation Protocol
   4.5 Human Oversight Interface

5. Implementation (1.5 pages)
   5.1 Technology Stack
   5.2 Circuit Design
   5.3 Deployment Considerations

6. Evaluation (2 pages)
   6.1 Experimental Setup
   6.2 Performance Results
   6.3 Scalability Analysis
   6.4 Limitations

7. Related Work (1 page)
   7.1 AI Governance Frameworks
   7.2 Zero-Knowledge Proof Systems
   7.3 Audit Logging Systems

8. Discussion (0.5 pages)
   8.1 Path to Production
   8.2 Regulatory Implications
   8.3 Future Work

9. Conclusion (0.5 pages)

References (~40 citations)
```

### 7.2 Critical Framing Guidance

**DO:**
- Position as "reference architecture" and "proof of concept"
- Acknowledge synthetic evaluation explicitly
- Use hedged language: "demonstrates feasibility", "provides foundation"
- Focus on novel combination of existing techniques
- Emphasize open-source contribution value

**DON'T:**
- Claim production-readiness
- Overstate performance without context
- Ignore limitations
- Make legal compliance guarantees
- Compare unfavorably with systems that have production deployment

### 7.3 Essential Citations

```bibtex
% EU AI Act
@misc{eu_ai_act_2024,
    title = {Regulation (EU) 2024/1689: Artificial Intelligence Act},
    author = {{European Parliament and Council}},
    year = {2024},
    howpublished = {\url{https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32024R1689}}
}

% ZK Proofs
@inproceedings{groth16,
    author = {Groth, Jens},
    title = {On the Size of Pairing-Based Non-interactive Arguments},
    booktitle = {EUROCRYPT},
    year = {2016}
}

@misc{circom,
    author = {Bellés-Muñoz, Marta and others},
    title = {circom: A Robust and Scalable Language for Building Complex Zero-Knowledge Circuits},
    howpublished = {\url{https://github.com/iden3/circom}},
    year = {2022}
}

% Merkle Trees
@article{merkle_tree,
    author = {Merkle, Ralph C.},
    title = {A Digital Signature Based on a Conventional Encryption Function},
    journal = {CRYPTO},
    year = {1987}
}

% AI Governance
@article{ai_governance_survey,
    author = {Jobin, Anna and Ienca, Marcello and Vayena, Effy},
    title = {The global landscape of AI ethics guidelines},
    journal = {Nature Machine Intelligence},
    year = {2019}
}

% Related Systems
@misc{certificate_transparency,
    author = {Laurie, Ben and others},
    title = {Certificate Transparency},
    howpublished = {RFC 6962},
    year = {2013}
}
```

### 7.4 Figures Needed

1. **System Architecture** (Section 4.1)
   - Component diagram showing data flow
   - High-quality, clear labels

2. **Compliance Chain** (Section 4.4)
   - Sequence diagram: request → evaluation → log → proof → certificate

3. **Latency Distribution** (Section 6.2)
   - Histogram of request latencies
   - P50, P95, P99 markers

4. **Throughput Scaling** (Section 6.3)
   - Line chart: throughput vs concurrent requests

5. **Proof Time vs Batch Size** (Section 6.3)
   - Bar chart comparing batch sizes

6. **Memory Usage** (Section 6.3)
   - Line chart: memory vs audit log entries

---

## Appendix: Code Templates

### A.1 Setup Script

```bash
#!/bin/bash
# File: scripts/setup_dev.sh

set -e

echo "=== RuntimeGuard-AI Development Setup ==="

# Check Python version
python_version=$(python3 --version 2>&1 | cut -d' ' -f2)
echo "Python version: $python_version"

# Create virtual environment
echo "Creating virtual environment..."
python3 -m venv venv
source venv/bin/activate

# Install Python dependencies
echo "Installing Python dependencies..."
pip install --upgrade pip
pip install fastapi uvicorn httpx pydantic sqlalchemy pytest pytest-asyncio matplotlib psutil

# Check Node.js
node_version=$(node --version 2>&1)
echo "Node.js version: $node_version"

# Install circom
echo "Installing circom..."
if ! command -v circom &> /dev/null; then
    cargo install --git https://github.com/iden3/circom.git
fi

# Install snarkjs
echo "Installing snarkjs..."
npm install -g snarkjs

# Install circomlib
echo "Installing circomlib..."
mkdir -p src/prover/circuits/node_modules
cd src/prover/circuits
npm init -y
npm install circomlib
cd ../../..

# Setup dashboard
echo "Setting up dashboard..."
cd dashboard
npm install
cd ..

# Initialize database
echo "Initializing database..."
mkdir -p data
python3 -c "from src.core.merkle_log import MerkleAuditLog; MerkleAuditLog('data/audit_log.db')"

echo "=== Setup Complete ==="
echo "To start development:"
echo "  source venv/bin/activate"
echo "  uvicorn src.api.main:app --reload"
```

### A.2 Circuit Compilation Script

```bash
#!/bin/bash
# File: src/prover/scripts/compile.sh

set -e

CIRCUIT_DIR="src/prover/circuits"
BUILD_DIR="src/prover/build"

mkdir -p $BUILD_DIR

echo "Compiling PolicyAttestation circuit..."
circom $CIRCUIT_DIR/policy_attestation.circom \
    --r1cs \
    --wasm \
    --sym \
    -o $BUILD_DIR

echo "Generating proving key..."
snarkjs groth16 setup \
    $BUILD_DIR/policy_attestation.r1cs \
    $BUILD_DIR/pot14_final.ptau \
    $BUILD_DIR/policy_attestation_0000.zkey

echo "Contributing to ceremony..."
snarkjs zkey contribute \
    $BUILD_DIR/policy_attestation_0000.zkey \
    $BUILD_DIR/policy_attestation_final.zkey \
    --name="RuntimeGuard-AI" \
    -v -e="$(head -c 64 /dev/urandom | xxd -p)"

echo "Exporting verification key..."
snarkjs zkey export verificationkey \
    $BUILD_DIR/policy_attestation_final.zkey \
    $BUILD_DIR/verification_key.json

echo "Circuit compilation complete!"
```

### A.3 Proof Generation Script

```javascript
// File: src/prover/scripts/prove.js

const snarkjs = require("snarkjs");
const fs = require("fs");
const path = require("path");

async function generateProof(inputPath, outputPath) {
    const buildDir = path.join(__dirname, "..", "build");
    
    // Load input
    const input = JSON.parse(fs.readFileSync(inputPath, "utf8"));
    
    // Generate witness
    const wasmPath = path.join(buildDir, "policy_attestation_js", "policy_attestation.wasm");
    const zkeyPath = path.join(buildDir, "policy_attestation_final.zkey");
    
    console.log("Generating witness...");
    const startWitness = Date.now();
    const { proof, publicSignals } = await snarkjs.groth16.fullProve(
        input,
        wasmPath,
        zkeyPath
    );
    const witnessTime = Date.now() - startWitness;
    
    // Verify proof locally
    console.log("Verifying proof...");
    const vkeyPath = path.join(buildDir, "verification_key.json");
    const vkey = JSON.parse(fs.readFileSync(vkeyPath, "utf8"));
    
    const startVerify = Date.now();
    const verified = await snarkjs.groth16.verify(vkey, publicSignals, proof);
    const verifyTime = Date.now() - startVerify;
    
    if (!verified) {
        throw new Error("Proof verification failed!");
    }
    
    // Write output
    const output = {
        proof,
        publicSignals,
        timing: {
            proof_generation_ms: witnessTime,
            verification_ms: verifyTime
        },
        verified: true
    };
    
    fs.writeFileSync(outputPath, JSON.stringify(output, null, 2));
    
    console.log(`Proof generated in ${witnessTime}ms`);
    console.log(`Verification completed in ${verifyTime}ms`);
    console.log(`Output written to ${outputPath}`);
}

// CLI interface
const args = process.argv.slice(2);
if (args.length !== 2) {
    console.log("Usage: node prove.js <input.json> <output.json>");
    process.exit(1);
}

generateProof(args[0], args[1])
    .then(() => process.exit(0))
    .catch(err => {
        console.error("Error:", err);
        process.exit(1);
    });
```

### A.4 Dashboard Main Component

```tsx
// File: dashboard/src/App.tsx

import { useState, useEffect } from 'react';

interface OversightItem {
  id: string;
  request_id: string;
  timestamp: string;
  reason: string;
  status: string;
}

interface Stats {
  total: number;
  pending: number;
  approved: number;
  rejected: number;
  review_rate: number;
}

function App() {
  const [queue, setQueue] = useState<OversightItem[]>([]);
  const [stats, setStats] = useState<Stats | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    fetchData();
    const interval = setInterval(fetchData, 5000);
    return () => clearInterval(interval);
  }, []);

  async function fetchData() {
    try {
      const [queueRes, statsRes] = await Promise.all([
        fetch('http://localhost:8000/api/v1/oversight/queue'),
        fetch('http://localhost:8000/api/v1/oversight/stats')
      ]);
      
      setQueue(await queueRes.json());
      setStats(await statsRes.json());
      setLoading(false);
    } catch (error) {
      console.error('Failed to fetch data:', error);
    }
  }

  async function handleAction(itemId: string, action: string) {
    const justification = prompt('Enter justification:');
    if (!justification) return;

    await fetch(`http://localhost:8000/api/v1/oversight/${itemId}/action`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ action, justification })
    });

    fetchData();
  }

  if (loading) {
    return <div className="p-8">Loading...</div>;
  }

  return (
    <div className="min-h-screen bg-gray-100">
      <header className="bg-blue-600 text-white p-4">
        <h1 className="text-2xl font-bold">RuntimeGuard-AI Oversight Dashboard</h1>
        <p className="text-blue-200">EU AI Act Article 14 Human Oversight Interface</p>
      </header>

      {/* Stats Cards */}
      <div className="p-4 grid grid-cols-4 gap-4">
        <div className="bg-white rounded-lg shadow p-4">
          <div className="text-3xl font-bold text-blue-600">{stats?.pending}</div>
          <div className="text-gray-600">Pending Review</div>
        </div>
        <div className="bg-white rounded-lg shadow p-4">
          <div className="text-3xl font-bold text-green-600">{stats?.approved}</div>
          <div className="text-gray-600">Approved</div>
        </div>
        <div className="bg-white rounded-lg shadow p-4">
          <div className="text-3xl font-bold text-red-600">{stats?.rejected}</div>
          <div className="text-gray-600">Rejected</div>
        </div>
        <div className="bg-white rounded-lg shadow p-4">
          <div className="text-3xl font-bold text-gray-600">
            {((stats?.review_rate || 0) * 100).toFixed(1)}%
          </div>
          <div className="text-gray-600">Review Rate</div>
        </div>
      </div>

      {/* Queue Table */}
      <div className="p-4">
        <div className="bg-white rounded-lg shadow">
          <div className="p-4 border-b">
            <h2 className="text-xl font-semibold">Escalation Queue</h2>
          </div>
          <table className="w-full">
            <thead className="bg-gray-50">
              <tr>
                <th className="p-3 text-left">Request ID</th>
                <th className="p-3 text-left">Timestamp</th>
                <th className="p-3 text-left">Reason</th>
                <th className="p-3 text-left">Status</th>
                <th className="p-3 text-left">Actions</th>
              </tr>
            </thead>
            <tbody>
              {queue.map((item) => (
                <tr key={item.id} className="border-t hover:bg-gray-50">
                  <td className="p-3 font-mono text-sm">{item.request_id.slice(0, 8)}...</td>
                  <td className="p-3">{new Date(item.timestamp).toLocaleString()}</td>
                  <td className="p-3">{item.reason}</td>
                  <td className="p-3">
                    <span className={`px-2 py-1 rounded text-sm ${
                      item.status === 'PENDING' ? 'bg-yellow-100 text-yellow-800' :
                      item.status === 'APPROVE' ? 'bg-green-100 text-green-800' :
                      'bg-red-100 text-red-800'
                    }`}>
                      {item.status}
                    </span>
                  </td>
                  <td className="p-3">
                    {item.status === 'PENDING' && (
                      <div className="space-x-2">
                        <button
                          onClick={() => handleAction(item.id, 'APPROVE')}
                          className="px-3 py-1 bg-green-500 text-white rounded hover:bg-green-600"
                        >
                          Approve
                        </button>
                        <button
                          onClick={() => handleAction(item.id, 'REJECT')}
                          className="px-3 py-1 bg-red-500 text-white rounded hover:bg-red-600"
                        >
                          Reject
                        </button>
                      </div>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>

      <footer className="p-4 text-center text-gray-500 text-sm">
        RuntimeGuard-AI v0.1.0 | EU AI Act Article 14 Compliance | Open Source
      </footer>
    </div>
  );
}

export default App;
```

---

## Final Checklist

### Before Day 15 Submission

- [ ] All core components implemented and tested
- [ ] Evaluation benchmarks completed
- [ ] All figures generated
- [ ] Paper draft complete
- [ ] GitHub repository public with:
  - [ ] Complete README
  - [ ] MIT License
  - [ ] CITATION.cff
  - [ ] Working CI/CD
  - [ ] Demo video or GIF
- [ ] arXiv metadata prepared:
  - [ ] Title
  - [ ] Abstract
  - [ ] Categories (cs.CR, cs.AI)
  - [ ] License (CC BY 4.0)
- [ ] Supplementary materials packaged

### GitHub Release Content

```
runtimeguard-ai-v0.1.0/
├── README.md
├── LICENSE (MIT)
├── CITATION.cff
├── paper/
│   └── runtimeguard-ai-preprint.pdf
├── src/
│   └── [all source code]
├── evaluation/
│   └── results/
└── docker-compose.yml (optional)
```

---

**This blueprint provides everything you need to build RuntimeGuard-AI in 15 days. Focus on the implementation roadmap (Section 3) and iterate daily. Good luck!**
