#[derive(Debug, Clone)]
pub enum StopReason {
    Saturated,
    IterationLimit,
    TimeLimit,
    NodeLimit,
    Other(String),
}

#[derive(Debug)]
pub struct Report {
    pub iterations: usize,
    pub stop_reason: StopReason,
    pub egraph_nodes: usize,
    pub egraph_classes: usize,
    pub total_time: f64,
}
