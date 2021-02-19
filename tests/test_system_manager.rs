use entity_system::RefreshPeriod;
use std::time::{Duration, Instant};

#[allow(clippy::eq_op)]
#[test]
fn test_system_manager_01() {
    let now = Instant::now();

    assert!(RefreshPeriod::Stop < RefreshPeriod::EveryTime);
    assert!(RefreshPeriod::Stop < RefreshPeriod::At(now));
    assert!(RefreshPeriod::Stop == RefreshPeriod::Stop);

    assert!(RefreshPeriod::EveryTime > RefreshPeriod::Stop);
    assert!(RefreshPeriod::EveryTime > RefreshPeriod::At(now));
    assert!(RefreshPeriod::EveryTime == RefreshPeriod::EveryTime);

    assert!(RefreshPeriod::At(now) < RefreshPeriod::EveryTime);
    assert!(RefreshPeriod::At(now) > RefreshPeriod::Stop);
    assert!(RefreshPeriod::At(now) == RefreshPeriod::At(now));

    assert!(RefreshPeriod::At(now) < RefreshPeriod::At(now + Duration::from_secs(5)));
    assert!(RefreshPeriod::At(now + Duration::from_secs(5)) > RefreshPeriod::At(now));
}
