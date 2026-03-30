use crate::{
    empirical_transition_stats::make_empirical_transition_stats,
    utm::{make_utm_spec, MyUtmSpec, MyUtmSpecOptimizationHints},
};

pub fn make_my_utm_self_optimization_hints() -> MyUtmSpecOptimizationHints<MyUtmSpec> {
    MyUtmSpecOptimizationHints::from_transition_stats(
        &make_utm_spec(),
        &make_empirical_transition_stats(),
    )
}
