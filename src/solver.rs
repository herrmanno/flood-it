//! General solving routine for the 'Flood it' puzzle

use z3::ast::{Ast, Bool, Int};

use crate::{
    cluster::Cluster,
    problem::{Color, Problem},
    solution::Solution,
};

/// Try to solve the given [problem][Problem] in `t_max` steps
///
/// # Args
/// - `instance` the problem to solve
/// - `t_max` the length of the solution to search for
/// - `optimize` if z3 should optimize for a minimal solution
///     - if `true`, `t_max` behaves as upper bound
///     - if `false`, `t_max` behaves as exact length constraint
pub fn solve(
    instance: &Problem,
    t_max: usize,
    optimize: bool,
) -> (z3::SatResult, Option<Solution>) {
    /* TODO: IMPROVEMENTS
        - calculate color-path length for furthest cluster
            - use as lower bound
        - try z3::optimize to optimize flood_vars
    */

    // INIT SOLVER
    let ctx = z3::Context::new(&Default::default());
    let solver = z3::Optimize::new(&ctx);

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
    let clusters = Cluster::from_problem(instance);
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
                let color_choosen_at_t =
                    color_vars[t]._eq(&Int::from_u64(&ctx, cluster.color as u64));

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

    if optimize {
        let optimization_goal = {
            let nums: Vec<_> = (0..=t_max)
                .into_iter()
                .map(|t| {
                    let flooded_vars_t: Vec<_> = flooded_vars.iter().map(|vars| &vars[t]).collect();
                    let all_flooded_t = Bool::and(&ctx, flooded_vars_t.as_slice());
                    all_flooded_t.ite(&Int::from_u64(&ctx, 1), &Int::from_u64(&ctx, 0))
                })
                .collect();

            Int::add(&ctx, nums.iter().collect::<Vec<_>>().as_slice())
        };

        solver.maximize(&optimization_goal);
    } else {
        let flooding_at_t_minus_one: Vec<_> =
            flooded_vars.iter().map(|vars| &vars[t_max - 1]).collect();
        let not_all_flooded_at_t_minus_one = Bool::and(&ctx, &flooding_at_t_minus_one).not();
        solver.assert(&not_all_flooded_at_t_minus_one);
    }

    println!("Starting z3 (max steps: {t_max})...");

    match solver.check(&[]) {
        z3::SatResult::Unsat => (z3::SatResult::Unsat, None),
        z3::SatResult::Unknown => (z3::SatResult::Unknown, None),
        z3::SatResult::Sat => {
            if let Some(model) = solver.get_model() {
                let flood_model: Vec<Vec<_>> = flooded_vars
                    .iter()
                    .map(|vars| {
                        vars.iter()
                            .map(|var| {
                                model
                                    .eval(var, false)
                                    .and_then(|b| b.as_bool())
                                    .expect("Could not read flood var value from model")
                            })
                            .collect()
                    })
                    .collect();

                let solution_length = (0..flood_model.len())
                    .into_iter()
                    .position(|i| {
                        flood_model
                            .iter()
                            .map(|vars| vars[i])
                            .all(|flooded| flooded)
                    })
                    .unwrap_or(t_max);

                println!("Solution length: {}", solution_length);

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
                    let solution = Solution::from(&colors[0..solution_length]);
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
