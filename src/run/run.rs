use crate::*;
use std::time::Instant;

// TODO: Turn this into a nicer interface like egg's `Runner`.

pub fn run_eqsat<L: Language, N: Analysis<L>, F>(
    egraph: &mut EGraph<L, N>, 
    rws: Vec<Rewrite<L, N>>, 
    iter_limit: usize, 
    time_limit: usize, 
    mut hook: F
) -> Report where F: FnMut(&mut EGraph<L, N>) -> Result<(), String> + 'static {
    let start_time = Instant::now();
    let mut iterations = 0;
    let stop_reason: StopReason;

    loop {
        let did_change = apply_rewrites(egraph, &rws);
        
        match hook(egraph) {
            Ok(_) => (),
            Err(msg) => { stop_reason = StopReason::Other(msg.to_string()); break }
        }

        if !did_change {
            stop_reason = StopReason::Saturated;
            break
        }

        if iterations >= iter_limit {
            stop_reason = StopReason::IterationLimit;
            break
        }

        if start_time.elapsed().as_secs() >= time_limit.try_into().unwrap() {
            stop_reason = StopReason::TimeLimit;
            break
        }
        
        iterations += 1;
    }

    Report {
        iterations,
        stop_reason,
        egraph_nodes: egraph.total_number_of_nodes(),
        egraph_classes: egraph.classes.len(),
        total_time: start_time.elapsed().as_secs_f64()
    }
}