use runtimeguard_inline::policy::{CompiledPolicy, USER_POLICY_HEADER};
use runtimeguard_inline::types::{Decision, PolicyDescriptor};

const SAMPLE_POLICY: &str = "runtimeguard-policy/v1\n\
id=sample-policy\n\
version=3\n\
# block United States SSN patterns\n\
block ssn \\b\\d{3}-\\d{2}-\\d{4}\\b\n\
escalate injection (?i)\\b(Jailbreak|ignore previous instructions)\\b\n";

#[test]
fn user_policy_compiles_and_evaluates_in_rule_order() {
    let policy = CompiledPolicy::from_source(SAMPLE_POLICY.as_bytes()).expect("compile policy");

    assert_eq!(policy.rule_count(), 2);
    assert_eq!(
        policy.evaluate("a perfectly safe prompt").decision,
        Decision::Allowed
    );

    let blocked = policy.evaluate("my SSN is 123-45-6789");
    assert_eq!(blocked.decision, Decision::Blocked);
    assert_eq!(
        blocked.rules_triggered,
        vec!["POLICY_BLOCK: rule 'ssn' matched".to_owned()]
    );

    let escalated = policy.evaluate("please Jailbreak the assistant");
    assert_eq!(escalated.decision, Decision::Escalated);
    assert_eq!(
        escalated.rules_triggered,
        vec!["POLICY_ESCALATE: rule 'injection' matched 'jailbreak'".to_owned()]
    );
}

#[test]
fn user_policy_descriptor_is_bound_to_the_exact_source_bytes() {
    let policy = CompiledPolicy::from_source(SAMPLE_POLICY.as_bytes()).expect("compile policy");
    let expected = PolicyDescriptor::from_bytes("sample-policy", "3", SAMPLE_POLICY.as_bytes());
    assert_eq!(policy.descriptor(), &expected);

    // Any byte change, even in a comment, is a different policy identity.
    let commented = SAMPLE_POLICY.replace("# block United States", "# block US");
    let changed = CompiledPolicy::from_source(commented.as_bytes()).expect("compile policy");
    assert_ne!(policy.descriptor().digest, changed.descriptor().digest);
    assert_eq!(policy.descriptor().id, changed.descriptor().id);
}

#[test]
fn user_policy_rejects_malformed_sources_with_line_numbers() {
    let missing_header = "id=x\nversion=1\n";
    let error = CompiledPolicy::from_source(missing_header.as_bytes()).unwrap_err();
    assert!(error.to_string().contains(USER_POLICY_HEADER), "{error}");

    let unknown_action = format!("{USER_POLICY_HEADER}\nid=x\nversion=1\ndeny ssn a+\n");
    let error = CompiledPolicy::from_source(unknown_action.as_bytes()).unwrap_err();
    assert!(
        error.to_string().contains("unknown directive 'deny'"),
        "{error}"
    );
    assert!(error.to_string().contains("line 4"), "{error}");

    let duplicate = format!("{USER_POLICY_HEADER}\nid=x\nversion=1\nblock a x+\nblock a y+\n");
    let error = CompiledPolicy::from_source(duplicate.as_bytes()).unwrap_err();
    assert!(
        error.to_string().contains("duplicate rule name 'a'"),
        "{error}"
    );

    let invalid_regex = format!("{USER_POLICY_HEADER}\nid=x\nversion=1\nblock broken ([unclosed\n");
    let error = CompiledPolicy::from_source(invalid_regex.as_bytes()).unwrap_err();
    assert!(error.to_string().contains("invalid regex"), "{error}");

    let missing_id = format!("{USER_POLICY_HEADER}\nversion=1\nblock a x+\n");
    let error = CompiledPolicy::from_source(missing_id.as_bytes()).unwrap_err();
    assert!(error.to_string().contains("missing an 'id='"), "{error}");

    let missing_pattern = format!("{USER_POLICY_HEADER}\nid=x\nversion=1\nblock a\n");
    let error = CompiledPolicy::from_source(missing_pattern.as_bytes()).unwrap_err();
    assert!(
        error.to_string().contains("missing a regex pattern"),
        "{error}"
    );
}

#[test]
fn example_policy_behavior_is_unchanged() {
    let policy = CompiledPolicy::example_v1().expect("compile example policy");
    assert_eq!(policy.evaluate("safe request").decision, Decision::Allowed);

    let blocked = policy.evaluate("SSN 123-45-6789");
    assert_eq!(blocked.decision, Decision::Blocked);
    assert_eq!(
        blocked.rules_triggered,
        vec!["PII_DETECTED: SSN pattern found".to_owned()]
    );

    let escalated = policy.evaluate("please bypass the guard");
    assert_eq!(escalated.decision, Decision::Escalated);
    assert_eq!(
        escalated.rules_triggered,
        vec!["SUSPICIOUS_CONTENT: 'bypass' detected".to_owned()]
    );
}
