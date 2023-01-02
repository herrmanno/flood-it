use std::fmt::Display;
use std::str::FromStr;

use color_flood_rs::printer;
use color_flood_rs::solution::Solution;
use z3::ast::Ast;
use z3::ast::Bool;
use z3::ast::Int;

use color_flood_rs::cluster::*;
use color_flood_rs::problem::*;

/// External search space limit
enum TimeParam {
    // Use binary search to find minimal solution
    Minimize,
    // Use binary search to find minimal solution in range [lo, hi]
    MinMax(usize, usize),
    // Find solution with fixed size
    Time(usize),
}

impl Display for TimeParam {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimeParam::Minimize => f.write_str("minimal"),
            TimeParam::MinMax(lo, hi) => f.write_fmt(format_args!("minimal between {lo} and {hi}")),
            TimeParam::Time(time) => f.write_fmt(format_args!("{time}")),
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
            Err(())
        }
    }
}

fn main() {
    let instance = Problem::from_stdin();
    let height = instance.height();
    let width = instance.width();
    let num_colors = *instance
        .grid
        .iter()
        .flat_map(|row| row.iter())
        .max()
        .unwrap()
        + 1;
    let t_max: TimeParam = {
        let args = std::env::args().skip(1).collect::<Vec<String>>().join(" ");
        args.parse::<TimeParam>()
            .ok()
            .unwrap_or(TimeParam::Time(height + width))
    };

    println!(
        "Size: {} x {}\nColors: {}\nSteps: {}",
        instance.height(),
        instance.width(),
        num_colors,
        t_max
    );

    println!("{}", instance);

    let (result, solution) = match (t_max, 0, instance.width() * instance.height()) {
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
    };

    println!("{result:?}");
    if let Some(solution) = solution {
        println!("{}", solution);
        printer::print_solution(&instance, &solution);
    }
}

/* TODO: IMPROVEMENTS
    - add 'no two adjacent colors can be equals' constraint
    - calculate color-path length for furthest cluster
        - use as lower bound
    - try z3::optimize to optimize flood_vars
*/
/// Try to solve the given [problem][Problem] in `t_max` steps
fn solve(instance: &Problem, t_max: usize) -> (z3::SatResult, Option<Solution>) {
    // INIT SOLVER
    let ctx = z3::Context::new(&Default::default());
    let solver = z3::Solver::new(&ctx);

    // INIT COLOR VARS
    let color_vars: Vec<Int> = (0..t_max)
        .map(|i| Int::new_const(&ctx, format!("color_{i}")))
        .collect();

    // ASSERT COLOR VARS
    for var in color_vars.iter() {
        solver.assert(&var.ge(&Int::from_u64(&ctx, 0)));
        solver.assert(&var.lt(&Int::from_u64(&ctx, instance.num_colors() as u64)));
    }

    for (c1, c2) in color_vars.iter().zip(color_vars.iter().skip(1)) {
        solver.assert(&c1._eq(c2).not());
    }

    // FIND CLUSTERS
    let clusters = construct_clusters(instance);
    let start_cluster = clusters
        .iter()
        .find(|cluster| cluster.fields.contains(&(0, 0)))
        .unwrap();
    let start_cluster_idx = clusters
        .iter()
        .position(|cluster| std::ptr::eq(cluster, start_cluster))
        .unwrap();

    // INIT FLOODED VARS
    let flooded_vars: Vec<Vec<Bool>> = {
        let mut vars: Vec<Vec<Bool>> = Default::default();
        for cluster_idx in 0..clusters.len() {
            let mut v = vec![];
            for t in 0..=t_max {
                v.push(Bool::new_const(
                    &ctx,
                    format!("Cluster #{cluster_idx} flooded at t = {t}"),
                ));
            }
            vars.push(v);
        }
        vars
    };

    // ASSERT FLOOD VARS (PER CLUSTER)
    for (idx, cluster) in clusters.iter().enumerate() {
        let neighbour_indices =
            cluster.neighbour_clusters(clusters.as_slice(), instance.height(), instance.width());

        let cluster_flooded_vars = &flooded_vars[idx];

        // every cluster must be flooded at last
        solver.assert(cluster_flooded_vars.last().unwrap());

        if idx == start_cluster_idx {
            for a in cluster_flooded_vars.iter() {
                solver.assert(a);
            }
        } else {
            solver.assert(&cluster_flooded_vars.first().unwrap().not());

            for (t, (a, b)) in cluster_flooded_vars
                .iter()
                .zip(cluster_flooded_vars.iter().skip(1))
                .enumerate()
            {
                // if cluster was flooded at t, is must also be flooded at t + 1
                solver.assert(&a.implies(b));

                // cluster's color was choosen at t
                let color_choosen_at_t = Bool::and(
                    &ctx,
                    &[
                        &color_vars[t].le(&Int::from_u64(&ctx, cluster.color as u64)),
                        &color_vars[t].ge(&Int::from_u64(&ctx, cluster.color as u64)),
                    ],
                );

                // any neighbouring cluster was flooded at t
                let any_neighbour_flooded = {
                    let constraints = neighbour_indices.iter().map(|idx| &flooded_vars[*idx][t]);

                    Bool::or(&ctx, constraints.collect::<Vec<_>>().as_slice())
                };

                // neighbour was flooded at t + color was choosen at t -> cluster is flooded at t + 1
                solver.assert(
                    &Bool::and(&ctx, &[&any_neighbour_flooded, &color_choosen_at_t]).implies(b),
                );

                // cluster was not flooded at t and (no neighbour was flooded at t *or* color was not choosen at t) -> cluster is *not* flooded at t + 1
                solver.assert(
                    &Bool::and(
                        &ctx,
                        &[
                            &a.not(),
                            &Bool::or(
                                &ctx,
                                &[&any_neighbour_flooded.not(), &color_choosen_at_t.not()],
                            ),
                        ],
                    )
                    .implies(&b.not()),
                );
            }
        }
    }

    println!("Starting z3 (max steps: {t_max})...");

    match solver.check() {
        z3::SatResult::Unsat => (z3::SatResult::Unsat, None),
        z3::SatResult::Unknown => (z3::SatResult::Unknown, None),
        z3::SatResult::Sat => {
            if let Some(model) = solver.get_model() {
                let color_model = (0..t_max)
                    .into_iter()
                    .map(|idx| {
                        model
                            .eval(&color_vars[idx], false)
                            .and_then(|int| int.as_u64())
                            .map(|color| color as Color)
                    })
                    .collect::<Option<Vec<Color>>>();

                if let Some(colors) = color_model {
                    let solution = Solution::from(colors);
                    (z3::SatResult::Sat, Some(solution))
                } else {
                    (z3::SatResult::Sat, None)
                }
            } else {
                (z3::SatResult::Sat, None)
            }
        }
    }
}
