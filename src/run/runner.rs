use crate::*;
use std::time::Duration;
use std::time::Instant;

pub struct Iteration<IterData> {
    /// The user provided annotation for this iteration
    pub data: IterData,
    // TODO: add more data things
    pub num_nodes: usize,
    pub finish_time: Option<Instant>,
}
pub trait IterationData<L, N>: Sized
where
    L: Language,
    N: Analysis<L>,
{
    /// Given the current [`Runner`], make the
    /// data to be put in this [`Iteration`].
    fn make(runner: &Runner<L, N, Self>) -> Self;
}

impl<L, N> IterationData<L, N> for ()
where
    L: Language,
    N: Analysis<L>,
{
    fn make(_: &Runner<L, N, Self>) -> Self {}
}

pub struct RunnerLimits {
    iter_limit: usize,
    node_limit: usize,
    start_time: Option<Instant>,
    time_limit: Duration,
}
/// Type alias for the result of a [`Runner`].
pub type RunnerResult<T> = Result<T, StopReason>;

impl RunnerLimits {
    fn check_limits<L, N>(&self, iteration: usize, eg: &EGraph<L, N>) -> RunnerResult<()>
    where
        L: Language,
        N: Analysis<L>,
    {
        let elapsed = self.start_time.unwrap().elapsed();
        if iteration > self.iter_limit {
            Err(StopReason::IterationLimit)
        } else if eg.total_number_of_nodes() > self.node_limit {
            Err(StopReason::NodeLimit)
        } else if elapsed > self.time_limit {
            Err(StopReason::TimeLimit)
        } else {
            Ok(())
        }
    }
}

pub struct Runner<L: Language, N: Analysis<L>, IterData>
where
    IterData: IterationData<L, N>,
{
    /// The [`EGraph`] used.
    pub egraph: EGraph<L, N>,
    /// Data accumulated over each [`Iteration`].
    pub iterations: Vec<Iteration<IterData>>,
    /// The roots of expressions added by the
    /// [`with_expr`](Runner::with_expr()) method, in insertion order.
    pub roots: Vec<AppliedId>,
    /// Why the `Runner` stopped. This will be `None` if it hasn't
    /// stopped yet.
    pub stop_reason: Option<StopReason>,

    // Initial expressions in the EGraph
    pub limits: RunnerLimits,
    /// hooks
    pub hooks: Vec<Box<dyn FnMut(&mut Self) -> Result<(), String> + 'static>>,
}

impl<L, N, IterData> Runner<L, N, IterData>
where
    L: Language,
    N: Analysis<L>,
    IterData: IterationData<L, N>,
{
    pub fn new() -> Self {
        Self {
            egraph: EGraph::new(),
            iterations: vec![],
            stop_reason: None,
            limits: RunnerLimits {
                iter_limit: 30,
                node_limit: 10_000,
                time_limit: Duration::from_secs(60),
                // The start_time is initialized when the Runner is ran
                start_time: None,
            },
            hooks: vec![],
            roots: vec![],
        }
    }
    pub fn with_expr(mut self, expr: &RecExpr<L>) -> Self {
        let id = self.egraph.add_expr(expr.clone());
        self.roots.push(id);
        self
    }
    pub fn with_hook<F>(mut self, hook: F) -> Self
    where
        F: FnMut(&mut Self) -> Result<(), String> + 'static,
    {
        self.hooks.push(Box::new(hook));
        self
    }
    pub fn with_egraph(mut self, egraph: EGraph<L, N>) -> Self {
        // You should probably not use this if you use `with_expr` as well
        self.egraph = egraph;
        self
    }
    pub fn with_node_limit(mut self, node_limit: usize) -> Self {
        self.limits.node_limit = node_limit;
        self
    }
    pub fn with_iter_limit(mut self, iter_limit: usize) -> Self {
        self.limits.iter_limit = iter_limit;
        self
    }
    pub fn with_time_limit(mut self, time_limit: Duration) -> Self {
        self.limits.time_limit = time_limit;
        self
    }

    fn check_limits(&mut self) -> RunnerResult<()> {
        self.limits
            .check_limits(self.iterations.len(), &self.egraph)
    }
    pub fn run(&mut self, rewrites: &[Rewrite<L, N>]) -> Report {
        loop {
            if let Some(_) = self.stop_reason {
                break;
            }
            let iter = self.run_one(rewrites);
            self.iterations.push(iter);
        }
        Report {
            iterations: self.iterations.len(),
            stop_reason: self.stop_reason.clone().unwrap(),
            egraph_nodes: self.egraph.total_number_of_nodes(),
            egraph_classes: self.egraph.classes.len(),
            total_time: self
                .iterations
                .last()
                .unwrap()
                .finish_time
                .unwrap()
                .duration_since(self.limits.start_time.unwrap())
                .as_secs_f64(),
        }
    }
    fn run_one(&mut self, rewrites: &[Rewrite<L, N>]) -> Iteration<IterData> {
        assert!(self.stop_reason.is_none());

        // if the runner has not started, start the timer
        self.limits.start_time.get_or_insert_with(Instant::now);

        let mut result = self.check_limits();
        let mut hooks = std::mem::take(&mut self.hooks);

        result = result.and_then(|_| {
            hooks.iter_mut().try_for_each(|hook| {
                hook(self).map_err(|err_msg| StopReason::Other(String::from(err_msg)))
            })
        });

        if let Err(stop_reason) = result {
            self.stop_reason = Some(stop_reason);
            return Iteration {
                data: IterData::make(self),
                num_nodes: self.egraph.total_number_of_nodes(),
                finish_time: Some(Instant::now()),
            };
        }

        let progress = apply_rewrites(&mut self.egraph, rewrites);
        if !progress {
            self.stop_reason = self.stop_reason.clone().or(Some(StopReason::Saturated));
        }
        Iteration {
            data: IterData::make(self),
            num_nodes: self.egraph.total_number_of_nodes(),
            finish_time: Some(Instant::now()),
        }
    }
}

impl<L, N> Default for Runner<L, N, ()>
where
    L: Language,
    N: Analysis<L> + Default,
{
    fn default() -> Self {
        Runner::new()
    }
}
