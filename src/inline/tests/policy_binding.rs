use runtimeguard_inline::policy::{CompiledPolicy, EXAMPLE_POLICY_V1_SOURCE};
use runtimeguard_inline::types::{Decision, PolicyDescriptor};

#[test]
fn compiled_policy_descriptor_is_derived_from_the_evaluated_policy_source() {
    let policy = CompiledPolicy::example_v1().expect("compile example policy");
    let expected = PolicyDescriptor::from_bytes(
        "runtimeguard-example-policy",
        "v1",
        EXAMPLE_POLICY_V1_SOURCE,
    );

    assert_eq!(policy.descriptor(), &expected);
    assert_eq!(policy.evaluate("safe request").decision, Decision::Allowed);
    assert_eq!(
        policy.evaluate("SSN 123-45-6789").decision,
        Decision::Blocked
    );
    assert_eq!(
        policy.evaluate("please bypass the guard").decision,
        Decision::Escalated
    );
}
