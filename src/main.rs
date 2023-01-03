use std::fmt::Display;
use std::str::FromStr;

use color_flood_rs::cluster::*;
use color_flood_rs::printer;
use color_flood_rs::problem::*;
use color_flood_rs::solver::solve;

/// External search space limit
enum TimeParam {
    // Use binary search to find minimal solution
    Minimize,
    // Use binary search to find minimal solution in range [lo, hi]
    MinMax(usize, usize),
    // Find solution with fixed size
    Time(usize),
    Default,
}

impl Display for TimeParam {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimeParam::Minimize => f.write_str("minimal"),
            TimeParam::MinMax(lo, hi) => f.write_fmt(format_args!("minimal between {lo} and {hi}")),
            TimeParam::Time(time) => f.write_fmt(format_args!("{time}")),
            TimeParam::Default => f.write_str("default (# of clusters)"),
        }
    }
}

impl FromStr for TimeParam {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "min" {
            Ok(TimeParam::Minimize)
        } else if let Some((lo, hi)) = s
            .split_once(' ')
            .and_then(|(a, b)| Some((a.parse::<usize>().ok()?, b.parse::<usize>().ok()?)))
        {
            Ok(TimeParam::MinMax(lo, hi))
        } else if let Ok(time) = s.parse::<usize>() {
            Ok(TimeParam::Time(time))
        } else {
            Ok(TimeParam::Default)
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

    let num_clusters = Cluster::from_problem(&instance).len();
    let (result, solution) = match (t_max, 0, num_clusters) {
        // do binary search to find best solution
        (TimeParam::Minimize, lo, hi) | (TimeParam::MinMax(lo, hi), _, _) => {
            let mut lo = lo;
            let mut hi = hi;
            let mut ret = (z3::SatResult::Unknown, None);
            loop {
                let t = (hi + lo) / 2;
                let tmp = solve(&instance, t);
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
        (TimeParam::Time(time), _, _) => solve(&instance, time),
        (TimeParam::Default, _, _) => solve(&instance, num_clusters),
    };

    println!("{result:?}");

    if let Some(solution) = solution {
        println!("{}", solution);
        printer::print_solution(&instance, &solution);
    }
}
