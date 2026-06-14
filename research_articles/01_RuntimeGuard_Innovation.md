# RuntimeGuard-AI: Bridging the Gap Between EU AI Act Compliance and High-Frequency Inference via Asynchronous Cryptographic Attestation

**Category:** Research and Innovations
**Author:** Neeraj Kumar Singh Beshane
**Date:** December 2025

---

## 1. Abstract

The enforcement of the European Union's AI Act (Regulation 2024/1689), specifically Article 14's mandate for "effective human oversight," presents a formidable engineering challenge for high-frequency AI systems. Existing governance mechanisms necessitate a binary choice: either incur unacceptable latency penalties by blocking inference for synchronous compliance checks, or settle for "evaluation theater" through static, pre-deployment audits that fail to capture dynamic runtime anomalies. This article presents **RuntimeGuard-AI**, a novel architectural paradigm that resolves this tension through the principle of *Latency Separation*. By decoupling lightweight, inline policy enforcement from heavy, asynchronous cryptographic attestation, RuntimeGuard-AI achieves a median latency overhead of just **2.3–4.1%** while providing mathematically rigorous, tamper-evident audit trails via Groth16 Zero-Knowledge Proofs (ZK-SNARKs) and Merkle Trees. We detail the system's formal foundations, its Rust-based reference implementation, and a comparative analysis demonstrating its superiority over traditional observability tools and Trusted Execution Environments (TEEs) for scalable regulatory compliance.

---

## 2. Introduction

### Contextual Background: The Era of Regulated Intelligence
The deployment of Large Language Models (LLMs) and agentic systems in critical sectors—finance, healthcare, employment, and justice—has shifted the focus of AI development from purely performance-driven metrics (accuracy, F1 score) to trust and accountability. This shift is no longer voluntary; it is codified in law. The **EU AI Act**, which entered into force in August 2024, establishes a comprehensive risk-based regulatory framework. For "High-Risk" AI systems, Article 14 mandates that providers design systems to enable *effective human oversight*, ensuring that operators can "correctly interpret the system's output" and "override or reverse" decisions.

### Problem Statement: The Latency-Audit Paradox
A significant technical gap exists between these legal mandates and current infrastructure capabilities. We term this the **Latency-Audit Paradox**:
1.  **The Need for Rigor:** To satisfy regulators, compliance logs must be non-repudiable. A simple text log text file can be edited by any system administrator (a "Rogue Operator" risk). True accountability requires cryptographic binding—digital signatures or Zero-Knowledge Proofs (ZKPs) that prove a specific policy was executed on a specific input.
2.  **The Need for Speed:** Modern AI applications require millisecond-level responsiveness. Users expect chat interfaces to stream tokens instantly; financial algorithms must execute trades in microseconds.
3.  **The Conflict:** Cryptographic proofs are computationally expensive. Generating a succinct ZK-SNARK for a standard policy check can take hundreds of milliseconds to seconds. Blocking the user's request while waiting for this proof destroys the user experience (UX) and renders the system commercially unviable.

### Purpose & Scope
This article explores the research and innovation behind **RuntimeGuard-AI**, the first open-source reference implementation designed explicitly to resolve this paradox. We articulate how a "Two-Path" architecture—separating the *Critical Path* of inference from the *Attestation Path* of compliance—enables high-assurance auditing without compromising performance. We provide a deep dive into the cryptographic primitives used (Groth16, Poseidon Hashes, Merkle Trees) and offer empirical evidence of the system's scalability.

---

## 3. Research Background and Related Work

### 3.1 The Regulatory Landscape
The foundation of this research lies in the specific requirements of the EU AI Act. Unlike the GDPR, which focuses on data privacy, the AI Act focuses on *system safety and fundamental rights*.
-   **Article 14 (Human Oversight):** Requires technical measures to permit human intervention. This implies a need for a reliable signal channel that flags anomalies *as they happen*, not months later during a post-mortem.
-   **Article 15 (Accuracy, Robustness, Cybersecurity):** Mandates that high-risk systems are resilient to errors and attempts to alter their use / performance. This explicitly calls for *Tamper-Evident* logging mechanisms.

### 3.2 Evolution of Cryptographic Logging
Our work builds upon **Certificate Transparency (RFC 6962)**, a standard introduced by Google to fix the broken Certificate Authority (CA) trust model. CT logs use **Merkle Trees**—binary hash trees where every leaf is a certificate and the root is a cryptographic fingerprint of the entire log. CT logs are *append-only* and *publicly verifiable*. RuntimeGuard-AI adapts this model from static SSL certificates to dynamic AI inference events, creating what we term "Inference Transparency."

### 3.3 Zero-Knowledge Machine Learning (ZKML)
The nascent field of **ZKML** has focused primarily on *Computational Integrity*—proving that a specific model architecture (weights $W$) was applied to input $X$ to yield output $Y$ ($Y = M(X, W)$). Projects like **EZKL** and **Modulus Labs** have made strides in optimizing these proofs. However, proving the *entire* inference of a 70B parameter model is still prohibitively slow (minutes to hours). RuntimeGuard-AI innovates by focusing on **Policy Integrity**—proving that a *safety check* was applied—which is a much smaller circuit and thus feasible for near-real-time attestation.

---

## 4. Novel Contribution: The RuntimeGuard Architecture

RuntimeGuard-AI introduces three primary contributions to the field of AI Governance Engineering:

### 4.1 The Principle of Latency Separation (Theorem 1)
We formalize a new system property called **Latency Separation**.
*Definition:* A system satisfies Latency Separation if the response time ($t_{resp}$) observed by the user is statistically independent of the verification time ($t_{verify}$) required by the auditor.

In traditional synchronous designs, $t_{resp} \approx t_{inference} + t_{verify}$. Since $t_{verify}$ (ZK proof generation) is large (~1.4s), the user suffers.
RuntimeGuard-AI achieves independence ($Cov(t_{resp}, t_{verify}) \approx 0$) through a non-blocking, sharded queuing mechanism. The inference path executes a lightweight ($<5$ms) policy check and pushes the result to a bounded memory channel. If the channel is full, it spills to a disk buffer, ensuring no backpressure propagates to the user. The heavy lifting of ZK proof generation happens asynchronously, consuming the log queue at its own pace.

### 4.2 The "Two-Path" Protocol
1.  **Fast Path (The Guard):**
    -   Intercepts the HTTP request.
    -   Executes deterministic rules (e.g., "Block if PII present", "Escalate if Toxicity > 0.8").
    -   Hashes the Input/Output pair ($H_{io} = SHA256(I || O)$).
    -   Appends the decision tuple $(Timestamp, H_{io}, Decision)$ to the local log.
    -   Returns the response immediately.
    
2.  **Slow Path (The Attestor):**
    -   Batches $N$ records (e.g., $N=100$) from the log.
    -   Constructs a Merkle Tree of these records.
    -   Generates a Groth16 ZK-SNARK proving:
        -   **Integrity:** The Merkle Root is correctly computed from the leaves.
        -   **Correctness:** Every leaf allows the policy rules (or was correctly blocked).
        -   **Binding:** The proof is bound to the public Merkle Root.

### 4.3 Tamper-Evidence via Sharded Merkle Forests
To handle high throughput (10k+ RPS), a single Merkle Tree becomes a contention bottleneck. RuntimeGuard-AI implements a **Sharded Merkle Forest**. Traffic is deterministically partitioned (e.g., by User ID) into $K$ shards. Each shard maintains its own independent, append-only Merkle Tree. At the end of an "Epoch" (e.g., 60 seconds), the roots of all $K$ shards are aggregated into a "Super-Root," which is then notarized on a public ledger (e.g., a blockchain or transparency log). This ensures that even if a catastrophic failure occurs, the "Blast Radius" of potential log corruption is limited to a single shard.

---

## 5. Methodology and Implementation

Our research is validated through a production-grade reference implementation in **Rust**.

### 5.1 Technology Stack
-   **Language:** Rust (v1.75+) for memory safety and zero-cost abstractions.
-   **Cryptographic Backend:** `arkworks-rs` for Elliptic Curve arithmetic (`bls12-381` curve) and Groth16 implementation.
-   **Concurrency:** `tokio` async runtime for non-blocking I/O.
-   **Hashing:** `Poseidon` hash function, optimized for ZK constraints (arithmetic complexity) rather than bitwise complexity (like SHA-256), reducing circuit size by 80%.

### 5.2 Circuit Design (The "Compliance Circuit")
The heart of the system is the ZK Circuit. We designed a custom Rank-1 Constraint System (R1CS) that enforces:
1.  **Merkle Path Validity:** For every record, constraints ensure the hash chain from leaf to root is valid.
2.  **Policy Predicates:**
    -   *Threshold Check:* $Score < Threshold$. Implemented via bit-decomposition constraints.
    -   *Membership Check:* $Category \in AllowedList$. Implemented via polynomial equality constraints.

### 5.3 Empirical Evaluation
We benchmarked RuntimeGuard-AI on standard cloud hardware (AMD EPYC, 128GB RAM).

**Key Findings:**
-   **Throughput:** The inline engine sustains **~48,000 requests per second** (RPS) before saturation, limited only by network I/O.
-   **Latency Overhead:** The median overhead introduced by the logging mechanism is **1.2ms** (Chat workload), representing a **2.3%** increase over baseline.
-   **Proof Generation:** Generating a proof for a batch of 50,000 constraints takes **1.38 seconds** on a CPU. While slow for a synchronous transaction, this is perfectly acceptable for an asynchronous batch running every minute.

---

## 6. Comparative Insight

How does this approach compare to existing solutions?

| Feature | **Traditional Observability** (Datadog/Splunk) | **Orchestration Layers** (LangChain/Guardrails) | **Trusted Execution Envs** (Intel SGX/TDX) | **RuntimeGuard-AI** |
| :--- | :--- | :--- | :--- | :--- |
| **Primary Goal** | Debugging / Uptime | Input Validation | Confidentiality | **Compliance / Audit** |
| **Trust Model** | Trust the Admin | Trust the Developer | Trust the Hardware | **Trust the Math** |
| **Integrity** | Mutable (Logs can be deleted) | Ephemeral (Memory only) | High (Encrypted RAM) | **Tamper-Evident** |
| **Overhead** | Low | Low | High (20-40%) | **Low (~2%)** |
| **Verifiability** | None | None | Remote Attestation | **Public ZK Proofs** |

**Analysis:**
-   **vs. Observability:** Tools like Splunk are designed for *availability*. They often sample logs under load (dropping data) and allow admins to delete old logs. This is unacceptable for regulatory evidence. RuntimeGuard-AI prioritizes *finality*—once a root is published, the history is frozen forever.
-   **vs. TEEs:** Running a 70B parameter model inside an SGX enclave is currently infeasible due to memory limits (EPC) and the massive performance penalty of memory encryption. RuntimeGuard-AI provides a "Hybrid" assurance: the *policy logic* is cryptographically proved, even if the model runs on standard GPUs.

---

## 7. Potential Applications

### 7.1 High-Frequency Trading (HFT) Auditing
In HFT, algorithms make buy/sell decisions in microseconds based on AI signals. Regulators (SEC/ESMA) require proof that trading strategies did not violate market abuse rules (e.g., spoofing). Blocking a trade for 1 second to generate a proof is impossible. RuntimeGuard-AI allows the trade to execute instantly while logging the decision to a Merkle Tree, generating a proof at the end of the trading day that *all* trades complied with risk limits.

### 7.2 Medical Diagnostic AI
An AI system triaging patients in an ER must not delay critical care. However, every decision ("Send to ICU" vs "Send to Waiting Room") serves as potential legal evidence in malpractice suits. RuntimeGuard-AI creates an immutable, privacy-preserving record of the triage logic used, ensuring that the hospital can prove exactly why a decision was made without revealing patient PII (thanks to Zero-Knowledge).

### 7.3 Automated Recruitment Systems
Under the EU AI Act (Annex III), recruitment AI is high-risk. Companies must prove they did not discriminate against protected groups. RuntimeGuard-AI can enforce fairness metrics (e.g., "Demographic Parity gap < 0.1") and cryptographically attest that no candidate was rejected solely based on protected attributes, providing a shield against liability.

---

## 8. Broader Implications

### 8.1 The Shift to "Verified Inference"
We argue that the industry is undergoing a phase transition analogous to the move from HTTP to HTTPS. Just as "unencrypted web traffic" is now considered negligent, "unverified AI inference" will soon be viewed as an unacceptable liability. RuntimeGuard-AI paves the way for **Verified Inference** as a standard commodity—where every AI output comes with a cryptographic receipt of its validity.

### 8.2 Environmental Impact
Counter-intuitively, adding cryptographic work may *reduce* the environmental footprint of AI. By providing robust safety guarantees for *smaller, specialized models*, RuntimeGuard-AI reduces the need to rely on massive, general-purpose "frontier" models for every task. A small, efficient model wrapped in a RuntimeGuard safety envelope can be trusted for high-stakes tasks, displacing the energy-hungry giants.

### 8.3 The "Oracle Problem"
A limitation remains: RuntimeGuard-AI proves that a policy was *executed*, not that the policy itself is *ethical*. If a bad policy is deployed ("Reject all women"), the system will faithfully enforce it and prove it was enforced. This highlights that RuntimeGuard is a tool for **Accountability** (doing what you said you would do), not **Ethics** (doing the right thing). The ethical quality of the policy remains a human responsibility.

---

## 9. Conclusion

RuntimeGuard-AI represents a critical maturation point in AI governance technology. It moves the field beyond "Trust Me" (corporate assurances) to "Verify Me" (mathematical proofs). By solving the Latency-Audit Paradox through asynchronous attestation, it provides the first viable path for enterprises to comply with strict regimes like the EU AI Act without crippling the performance that makes AI valuable in the first place. As we look to 2026 and beyond, architectures like RuntimeGuard-AI will likely form the backbone of the "Digital Trust Infrastructure" enabling the safe integration of artificial intelligence into the fabric of society.

---

## 10. References

1.  European Parliament and Council. (2024). *Regulation (EU) 2024/1689 laying down harmonised rules on artificial intelligence (Artificial Intelligence Act)*.
2.  Google. (2013). *Certificate Transparency, RFC 6962*. IETF.
3.  Groth, J. (2016). *On the Size of Pairing-based Non-interactive Arguments*. EUROCRYPT 2016.
4.  Ben-Sasson, E., et al. (2014). *SNARKs for C: Verifying Program Executions Succinctly and in Zero Knowledge*. CRYPTO 2013.
5.  Grassi, L., et al. (2021). *Poseidon: A New Hash Function for Zero-Knowledge Proof Systems*. USENIX Security Symposium.
6.  Sun, L., et al. (2024). *TrustLLM: Trustworthiness in Large Language Models*. ICML 2024.
7.  Kang, D., et al. (2024). *ZKML: An Optimizing System for ML Inference in Zero-Knowledge Proofs*. EuroSys 2024.
