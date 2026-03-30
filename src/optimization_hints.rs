use crate::{
    empirical_transition_stats::make_empirical_transition_stats,
    utm::{MyUtmSpec, MyUtmSpecOptimizationHints},
};

pub fn make_my_utm_self_optimization_hints<'a>(
    spec: &'a MyUtmSpec,
) -> MyUtmSpecOptimizationHints<'a, MyUtmSpec> {
    MyUtmSpecOptimizationHints::from_transition_stats(spec, &make_empirical_transition_stats())
}
