use std::fmt::Display;
use std::str::FromStr;

use color_flood_rs::cluster::*;
use color_flood_rs::printer;
use color_flood_rs::problem::*;
use color_flood_rs::solver::solve;

/// External search space limit
enum TimeParam {
    /// Use z3 optimization goal to find minimal solution
    Optimize,
    /// Use z3 optimization goal to find minimal solution w/ upper bound
    OptimizeMax(usize),
    /// Use binary search to find minimal solution
    Minimize,
    /// Use binary search to find minimal solution in range [lo, hi]
    MinMax(usize, usize),
    /// Find solution with fixed size
    Time(usize),
    /// Find solution for reasonable upper bound
    Default,
}

impl Display for TimeParam {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimeParam::Optimize => f.write_str("Optimize through z3"),
            TimeParam::OptimizeMax(hi) => {
                f.write_fmt(format_args!("Optimize through z3 w/ upper bound {hi}"))
            }
            TimeParam::Minimize => f.write_str("minimal"),
            TimeParam::MinMax(lo, hi) => f.write_fmt(format_args!("minimal between {lo} and {hi}")),
            TimeParam::Time(time) => f.write_fmt(format_args!("{time}")),
            TimeParam::Default => f.write_str("default (# of clusters)"),
        }
    }
}

impl FromStr for TimeParam {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (a, b) = s.split_once(' ').unwrap_or((s, ""));

        match (a, b) {
            ("", "") => Ok(TimeParam::Default),
            ("opt", "") => Ok(TimeParam::Optimize),
            ("opt", b) => {
                if let Ok(hi) = b.parse::<usize>() {
                    Ok(TimeParam::OptimizeMax(hi))
                } else {
                    Err(format!("Expected number. Got {b}"))
                }
            }
            ("min", _) => Ok(TimeParam::Minimize),
            (a, "") => {
                if let Ok(exact) = a.parse::<usize>() {
                    Ok(TimeParam::Time(exact))
                } else {
                    Err(format!("Expected number. Got {a}"))
                }
            }
            (a, b) => match (a.parse(), b.parse()) {
                (Ok(lo), Ok(hi)) => Ok(TimeParam::MinMax(lo, hi)),
                (Err(a), _) => Err(format!("Expected number. Got {a}")),
                (_, Err(b)) => Err(format!("Expected number. Got {b}")),
            },
        }
    }
}

fn main() {
    let instance = Problem::from_stdin();
    let t_max: TimeParam = {
        let args = std::env::args().skip(1).collect::<Vec<String>>().join(" ");
        args.parse::<TimeParam>().unwrap()
    };

    println!(
        "Size: {} x {}\nColors: {}\nSteps: {}",
        instance.height(),
        instance.width(),
        instance.num_colors(),
        t_max
    );

    println!("{}", instance);

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
    let (result, solution) = match (t_max, 0, max_moves) {
        (TimeParam::Optimize, _, _) => solve(&instance, max_moves, true),
        (TimeParam::OptimizeMax(hi), _, _) => solve(&instance, hi, true),
        // do binary search to find best solution
        (TimeParam::Minimize, lo, hi) | (TimeParam::MinMax(lo, hi), _, _) => {
            let mut lo = lo;
            let mut hi = hi;
            let mut ret = (z3::SatResult::Unknown, None);
            loop {
                let t = (hi + lo) / 2;
                let tmp = solve(&instance, t, false);
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

                if lo > hi {
                    if ret.0 == z3::SatResult::Sat {
                        break ret;
                    } else {
                        break tmp;
                    }
                }
            }
        }
        (TimeParam::Time(time), _, _) => solve(&instance, time, false),
        (TimeParam::Default, _, _) => solve(&instance, max_moves, false),
    };

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
