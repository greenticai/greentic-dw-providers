use crate::config::StaticPlanningConfig;
use greentic_dw_planning::{PlanDocument, PlanRequest, PlanningProvider};
use greentic_types::{GResult, TenantCtx};
use serde_json::{Map, Value};

/// Deterministic planner that returns a preconfigured plan document.
pub struct StaticPlanningProvider {
    config: StaticPlanningConfig,
}

impl StaticPlanningProvider {
    /// Creates a new static planning provider.
    pub fn new(config: StaticPlanningConfig) -> GResult<Self> {
        config.plan.validate()?;
        Ok(Self { config })
    }

    fn interpolate(&self, request: &PlanRequest) -> PlanDocument {
        if !self.config.interpolate_variables {
            return self.config.plan.clone();
        }

        let vars = interpolation_vars(request);
        let mut plan = self.config.plan.clone();
        plan.plan_id = interpolate_string(&plan.plan_id, &vars);
        plan.metadata = interpolate_value(&plan.metadata, &vars);
        for step in &mut plan.steps {
            step.id = interpolate_string(&step.id, &vars);
            step.kind = interpolate_string(&step.kind, &vars);
            step.depends_on = step
                .depends_on
                .iter()
                .map(|value| interpolate_string(value, &vars))
                .collect();
            step.payload = interpolate_value(&step.payload, &vars);
        }
        plan
    }
}

impl PlanningProvider for StaticPlanningProvider {
    fn plan(&self, _tenant: &TenantCtx, request: PlanRequest) -> GResult<PlanDocument> {
        let plan = self.interpolate(&request);
        plan.validate()?;
        Ok(plan)
    }
}

fn interpolation_vars(request: &PlanRequest) -> Map<String, Value> {
    let mut vars = match &request.context {
        Value::Object(map) => map.clone(),
        _ => Map::new(),
    };
    vars.entry("request_id".to_string())
        .or_insert_with(|| Value::String(request.request_id.clone()));
    vars.entry("goal".to_string())
        .or_insert_with(|| Value::String(request.goal.clone()));
    vars
}

fn interpolate_value(value: &Value, vars: &Map<String, Value>) -> Value {
    match value {
        Value::String(text) => Value::String(interpolate_string(text, vars)),
        Value::Array(values) => Value::Array(
            values
                .iter()
                .map(|item| interpolate_value(item, vars))
                .collect(),
        ),
        Value::Object(map) => Value::Object(
            map.iter()
                .map(|(key, value)| (key.clone(), interpolate_value(value, vars)))
                .collect(),
        ),
        _ => value.clone(),
    }
}

fn interpolate_string(input: &str, vars: &Map<String, Value>) -> String {
    let mut output = input.to_string();
    for (key, value) in vars {
        let replacement = match value {
            Value::String(text) => text.clone(),
            _ => value.to_string(),
        };
        output = output.replace(&format!("{{{{{key}}}}}"), &replacement);
    }
    output
}
