pub enum StopReason {
    Saturated,
    IterationLimit,
    TimeLimit,
    Other(String),
}

pub struct Report {
    pub iterations: usize,
    pub stop_reason: StopReason,
    pub egraph_nodes: usize,
    pub egraph_classes: usize,
    pub total_time: f64,
}