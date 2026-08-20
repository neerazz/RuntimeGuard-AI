use crate::types::{PolicyDescriptor, PolicyResult};
use anyhow::{bail, Context, Result};
use regex::Regex;

pub const EXAMPLE_POLICY_V1_SOURCE: &[u8] = br#"runtimeguard-example-policy/v1
precedence=ssn,card,suspicious,allow
ssn=\b\d{3}-\d{2}-\d{4}\b
card=\b\d{4}[\s-]?\d{4}[\s-]?\d{4}[\s-]?\d{4}\b
suspicious=(?i)\b(hack|exploit|bypass|jailbreak)\b
"#;

/// Header line required at the top of every user-authored policy source file.
pub const USER_POLICY_HEADER: &str = "runtimeguard-policy/v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuleAction {
    Block,
    Escalate,
}

#[derive(Debug, Clone)]
enum RuleMessage {
    /// A fixed reason string recorded when the rule matches.
    Fixed(String),
    /// A template whose `{match}` placeholder is replaced with the
    /// lowercased matched text when the rule matches.
    LowercasedMatch(String),
}

#[derive(Debug, Clone)]
struct CompiledRule {
    pattern: Regex,
    action: RuleAction,
    message: RuleMessage,
}

impl CompiledRule {
    fn render_message(&self, matched: &str) -> String {
        match &self.message {
            RuleMessage::Fixed(text) => text.clone(),
            RuleMessage::LowercasedMatch(template) => {
                template.replace("{match}", &matched.to_lowercase())
            }
        }
    }
}

/// A compiled deterministic policy whose evidence descriptor is derived from
/// the exact source defining the evaluated rules.
///
/// Rules are evaluated in order; the first matching rule decides the outcome
/// and no rule matching yields `Decision::Allowed`.
#[derive(Debug, Clone)]
pub struct CompiledPolicy {
    descriptor: PolicyDescriptor,
    rules: Vec<CompiledRule>,
}

impl CompiledPolicy {
    /// The small deterministic policy used by this research artifact.
    ///
    /// It is an evaluation fixture, not a complete enterprise policy language.
    pub fn example_v1() -> Result<Self> {
        Ok(Self {
            descriptor: PolicyDescriptor::from_bytes(
                "runtimeguard-example-policy",
                "v1",
                EXAMPLE_POLICY_V1_SOURCE,
            ),
            rules: vec![
                CompiledRule {
                    pattern: Regex::new(r"\b\d{3}-\d{2}-\d{4}\b")?,
                    action: RuleAction::Block,
                    message: RuleMessage::Fixed("PII_DETECTED: SSN pattern found".to_owned()),
                },
                CompiledRule {
                    pattern: Regex::new(r"\b\d{4}[\s-]?\d{4}[\s-]?\d{4}[\s-]?\d{4}\b")?,
                    action: RuleAction::Block,
                    message: RuleMessage::Fixed(
                        "PII_DETECTED: payment-card pattern found".to_owned(),
                    ),
                },
                CompiledRule {
                    pattern: Regex::new(r"(?i)\b(hack|exploit|bypass|jailbreak)\b")?,
                    action: RuleAction::Escalate,
                    message: RuleMessage::LowercasedMatch(
                        "SUSPICIOUS_CONTENT: '{match}' detected".to_owned(),
                    ),
                },
            ],
        })
    }

    /// Compiles a user-authored policy from its exact source bytes.
    ///
    /// The policy descriptor digest is derived from the exact bytes passed
    /// here, so any change to the source (including comments or whitespace)
    /// produces a different policy identity. The engine manifest pins that
    /// digest, which is what binds every committed record and receipt to the
    /// exact policy source that produced the decision.
    ///
    /// Source format (line oriented, UTF-8):
    ///
    /// ```text
    /// runtimeguard-policy/v1
    /// id=<policy-id>
    /// version=<policy-version>
    /// # comment lines and blank lines are permitted
    /// block <rule-name> <regex>
    /// escalate <rule-name> <regex>
    /// ```
    ///
    /// Rules are evaluated in file order. `block` returns a blocked decision
    /// with zero confidence; `escalate` returns an escalated decision for
    /// human review. A prompt matching no rule is allowed.
    pub fn from_source(source: &[u8]) -> Result<Self> {
        let text = std::str::from_utf8(source).context("policy source must be valid UTF-8")?;
        let mut lines = text.lines().enumerate();

        let header = loop {
            match lines.next() {
                None => bail!("policy source is empty; expected header '{USER_POLICY_HEADER}'"),
                Some((_, line)) if is_skippable(line) => continue,
                Some((number, line)) => break (number + 1, line.trim()),
            }
        };
        if header.1 != USER_POLICY_HEADER {
            bail!(
                "line {}: expected header '{USER_POLICY_HEADER}', found '{}'",
                header.0,
                header.1
            );
        }

        let mut id: Option<String> = None;
        let mut version: Option<String> = None;
        let mut rules = Vec::new();
        let mut rule_names: Vec<String> = Vec::new();

        for (index, raw_line) in lines {
            let line_number = index + 1;
            let line = raw_line.trim();
            if is_skippable(line) {
                continue;
            }

            if let Some(value) = line.strip_prefix("id=") {
                set_metadata_field(&mut id, "id", value, line_number)?;
                continue;
            }
            if let Some(value) = line.strip_prefix("version=") {
                set_metadata_field(&mut version, "version", value, line_number)?;
                continue;
            }

            let mut parts = line.splitn(3, char::is_whitespace);
            let action_token = parts.next().unwrap_or_default();
            let action = match action_token {
                "block" => RuleAction::Block,
                "escalate" => RuleAction::Escalate,
                other => bail!(
                    "line {line_number}: unknown directive '{other}'; expected 'id=', \
                     'version=', 'block', or 'escalate'"
                ),
            };
            let name = parts
                .next()
                .filter(|name| !name.is_empty())
                .ok_or_else(|| anyhow::anyhow!("line {line_number}: rule is missing a name"))?;
            let pattern_text = parts
                .next()
                .map(str::trim)
                .filter(|pattern| !pattern.is_empty())
                .ok_or_else(|| {
                    anyhow::anyhow!("line {line_number}: rule '{name}' is missing a regex pattern")
                })?;
            if rule_names.iter().any(|existing| existing == name) {
                bail!("line {line_number}: duplicate rule name '{name}'");
            }
            let pattern = Regex::new(pattern_text).with_context(|| {
                format!("line {line_number}: rule '{name}' has an invalid regex")
            })?;

            let message = match action {
                RuleAction::Block => {
                    RuleMessage::Fixed(format!("POLICY_BLOCK: rule '{name}' matched"))
                }
                RuleAction::Escalate => RuleMessage::LowercasedMatch(format!(
                    "POLICY_ESCALATE: rule '{name}' matched '{{match}}'"
                )),
            };
            rule_names.push(name.to_owned());
            rules.push(CompiledRule {
                pattern,
                action,
                message,
            });
        }

        let id = id.ok_or_else(|| anyhow::anyhow!("policy source is missing an 'id=' line"))?;
        let version =
            version.ok_or_else(|| anyhow::anyhow!("policy source is missing a 'version=' line"))?;

        Ok(Self {
            descriptor: PolicyDescriptor::from_bytes(id, version, source),
            rules,
        })
    }

    pub fn descriptor(&self) -> &PolicyDescriptor {
        &self.descriptor
    }

    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }

    pub fn evaluate(&self, prompt: &str) -> PolicyResult {
        for rule in &self.rules {
            if let Some(found) = rule.pattern.find(prompt) {
                let message = rule.render_message(found.as_str());
                return match rule.action {
                    RuleAction::Block => PolicyResult::block(message),
                    RuleAction::Escalate => PolicyResult::escalate(message),
                };
            }
        }
        PolicyResult::allow()
    }
}

fn is_skippable(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.is_empty() || trimmed.starts_with('#')
}

fn set_metadata_field(
    slot: &mut Option<String>,
    field: &str,
    value: &str,
    line_number: usize,
) -> Result<()> {
    let value = value.trim();
    if value.is_empty() {
        bail!("line {line_number}: '{field}=' must not be empty");
    }
    if slot.is_some() {
        bail!("line {line_number}: duplicate '{field}=' line");
    }
    *slot = Some(value.to_owned());
    Ok(())
}
