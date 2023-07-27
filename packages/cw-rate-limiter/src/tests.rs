use cosmwasm_std::testing::mock_dependencies;

use super::*;

#[test]
fn test_init() {
    let rate_limiter = RateLimiter::new("rate_limit", "sender");
    let mut deps = mock_dependencies();
    // set Blocks limit
    rate_limiter
        .init(deps.as_mut().storage, &Rate::Blocks(0))
        .unwrap();
    let rate_limit = rate_limiter.query_limit(deps.as_mut().storage).unwrap();
    assert_eq!(rate_limit, Some(Rate::Blocks(0)));
    // set PerBlocks limit
    rate_limiter
        .init(deps.as_mut().storage, &&Rate::PerBlock(1))
        .unwrap();
    let rate_limit = rate_limiter.query_limit(deps.as_mut().storage).unwrap();
    assert_eq!(rate_limit, Some(Rate::PerBlock(1)));
    // set PerBlocks limit
    let error = rate_limiter
        .init(deps.as_mut().storage, &&Rate::PerBlock(0))
        .unwrap_err();
    assert_eq!(error, RateLimitError::ZeroRate);
}

#[test]
fn test_cmp() {
    assert_eq!(Rate::PerBlock(1), Rate::Blocks(1));
    assert_ne!(Rate::PerBlock(0), Rate::Blocks(0));
    assert!(Rate::PerBlock(2) > Rate::Blocks(1));
    assert!(Rate::Blocks(2) < Rate::Blocks(1));
    assert!(Rate::PerBlock(2) > Rate::PerBlock(1));
    assert!(Rate::PerBlock(2) > Rate::Blocks(1));
}

#[test]
fn test_infinity() {
    let infinity = Rate::Blocks(0);
    // bitwise not. largest possible u64.
    assert!(Rate::PerBlock(!0) < infinity);
    assert!(infinity.is_infinite());
    assert!(!Rate::PerBlock(!0).is_infinite());
}

#[test]
fn test_zero() {
    let zero = Rate::PerBlock(0);
    assert!(zero.is_zero());
    assert!(zero < Rate::Blocks(!0));
}
