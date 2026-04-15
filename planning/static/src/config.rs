use greentic_dw_planning::PlanDocument;

/// Static planning configuration.
#[derive(Clone, Debug)]
pub struct StaticPlanningConfig {
    /// Embedded validated plan template.
    pub plan: PlanDocument,
    /// Whether string placeholders should be interpolated from request context.
    pub interpolate_variables: bool,
}
