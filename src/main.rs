use clap::Parser;

use color_flood_rs::cli::Args;
use color_flood_rs::cluster::*;
use color_flood_rs::printer;
use color_flood_rs::problem::*;
use color_flood_rs::solution::Solution;
use color_flood_rs::solver::{init_solver, run_solver, Solver};

/// Calls [solve] with correct Solver Type
macro_rules! solve {
    ($ctx: expr, $instance: expr, $args: expr) => {
        if $args.get_action().use_optimizer() {
            solve::<z3::Optimize>(&$ctx, &$instance, &$args)
        } else {
            solve::<z3::Solver>(&$ctx, &$instance, &$args)
        }
    };
}

fn main() {
    let args = Args::parse();

    // only load problem instance if stdin isn't a tty
    let instance = if atty::isnt(atty::Stream::Stdin) {
        Problem::from_stdin()
    } else {
        eprintln!("No problem supplied on stdin");
        return;
    };

    /* TODO: IMPROVEMENTS
        - calculate color-path length for furthest cluster
            - use as lower bound
    */

    println!("{}", instance);

    let ctx = z3::Context::new(&Default::default());
    if let Some((result, solution)) = solve!(ctx, instance, args) {
        println!("{result:?}");

        if result == z3::SatResult::Sat {
            if let Some(solution) = solution {
                println!("{}", solution);
                printer::print_solution(&instance, &solution);
            } else {
                println!("Could not extract solution");
            }
        }
    }
}

/// Solves an instance via optimization or by performing binary search over the solution length
fn solve<'c, T>(
    ctx: &'c z3::Context,
    instance: &Problem,
    args: &Args,
) -> Option<(z3::SatResult, Option<Solution>)>
where
    T: Solver<'c>,
{
    let action = args.get_action();
    let optimize = action.use_optimizer();

    // Upper bound for solution length
    //
    // min { 2n + (√2c)n + c, c * (n − 1) } ⋃ { #clusters }
    // See https://arxiv.org/pdf/1001.4420.pdf #Section_6
    let max_moves = {
        let num_clusters = Cluster::from_problem(&instance).len();
        let n = instance.height();
        let c = instance.num_colors();
        [
            num_clusters,
            // upper bound
            c * (n - 1),
            // asymptotic upper bound
            2 * n + c + ((2 * c) as f32).sqrt().ceil() as usize * n,
        ]
        .into_iter()
        .min()
        .unwrap()
    };

    // Moving bounds for binary search
    let (mut lo, mut hi) = action.get_bounds(0, max_moves);

    println!(
        "Size: {} x {}\nColors: {}\nStrategy: {:?}\nSolution bounds: [{},{}]\n",
        instance.height(),
        instance.width(),
        instance.num_colors(),
        action,
        lo,
        hi,
    );

    // t := solution size (= (max) number of colors in solution's color sequence)
    let mut t = (hi + lo) / 2;
    // let context = z3::Context::new(&Default::default());
    let mut solver_state = init_solver::<T>(ctx, &instance, t, optimize);

    if args.print_asserts() {
        println!("Got {} asserts:", solver_state.get_asserts().len());
        for assert in solver_state.get_asserts() {
            println!("{}", assert);
        }
    }

    if args.dry_run() {
        return None;
    }

    // do binary search to find best solution. Note: if lo = hi only one search run is performed
    let (result, solution) = {
        let mut ret = (z3::SatResult::Unknown, None);
        loop {
            println!("Starting z3 with size {t}...");

            let tmp = run_solver(solver_state, t);
            match tmp.0 {
                z3::SatResult::Unsat => {
                    lo = t + 1;
                }
                z3::SatResult::Unknown => {
                    lo = t + 1;
                }
                z3::SatResult::Sat => {
                    hi = t - 1;
                    ret = tmp.clone();
                }
            }

            t = (hi + lo) / 2;

            if lo > hi {
                if ret.0 == z3::SatResult::Sat {
                    break ret;
                } else {
                    break tmp;
                }
            }

            solver_state = init_solver(ctx, &instance, t, optimize)
        }
    };

    Some((result, solution))
}
