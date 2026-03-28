use crate::utm::{MyUtmSpec, MyUtmSpecOptimizationHints};

/// Optimization hints for UTM rule ordering, derived from empirical transition statistics.
pub fn make_my_utm_self_optimization_hints() -> MyUtmSpecOptimizationHints<MyUtmSpec> {
    crate::empirical_transition_stats::make_my_utm_self_optimization_hints()
}
