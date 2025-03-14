#[derive(Debug, Clone)]
pub enum StopReason<T = String>
where
    T: Clone,
{
    Saturated,
    IterationLimit,
    TimeLimit,
    NodeLimit,
    Other(T),
}

#[derive(Debug, Clone)]
pub struct Report<T = String>
where
    T: Clone,
{
    pub iterations: usize,
    pub stop_reason: StopReason<T>,
    pub egraph_nodes: usize,
    pub egraph_classes: usize,
    pub total_time: f64,
}
