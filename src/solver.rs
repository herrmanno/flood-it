//! General solving routine for the 'Flood it' puzzle
//! 
//! # Example
//! ```
//! let instance = Problem::from_stdin();
//! let solution_length = 20; // need to match the problem
//! let optimize = true; // signals to use the internal z3 optimizer
//! let ctx = z3::Context::new(&Default::default());
//! let mut solver_state = init_solver::<T>(ctx, &instance, solution_length, optimize);
//! let Some((result, solution)) = run_solver(solver_state, solution_length);
//! ```

use z3::ast::{Ast, Bool, Int};

use crate::{
    cluster::Cluster,
    problem::{Color, Problem},
    solution::Solution,
};

/// A generic abstraction over z3 solver strategies
pub trait Solver<'c> {
    /// Create a new solver
    fn new(ctx: &'c z3::Context) -> Self;

    /// Assert a condition. See [z3::Solver::assert]
    fn assert(&self, ast: &z3::ast::Bool);

    /// Checks is all assertions hold. See [z3::Solver::check]
    fn check(&self) -> z3::SatResult;

    /// Obtains a model that satisfies the assertions. See [z3::Solver::get_model]
    fn get_model(&self) -> Option<z3::Model>;

    /// Sets an objective to maximize. See [z3::Optimize::maximize]
    fn maximize(&self, objective: &z3::ast::Int);
}

impl<'ctx> Solver<'ctx> for z3::Solver<'ctx> {
    fn new(ctx: &'ctx z3::Context) -> Self {
        z3::Solver::new(ctx)
    }

    fn assert(&self, ast: &z3::ast::Bool) {
       self.assert(ast)
    }

    fn check(&self) -> z3::SatResult {
        self.check()
    }

    fn get_model(&self) -> Option<z3::Model> {
        self.get_model()
    }

    fn maximize(&self, _: &z3::ast::Int) {
        unimplemented!("z3::Solver does not support Solver::maximize")
    }
}

impl<'ctx> Solver<'ctx> for z3::Optimize<'ctx> {
    fn new(ctx: &'ctx z3::Context) -> Self {
        z3::Optimize::new(ctx)
    }

    fn assert(&self, ast: &z3::ast::Bool) {
       self.assert(ast)
    }

    fn check(&self) -> z3::SatResult {
        self.check(&[])
    }

    fn get_model(&self) -> Option<z3::Model> {
        self.get_model()
    }

    fn maximize(&self, objective: &z3::ast::Int) {
        self.maximize(objective)
    }
}

/// The collection of used variables for a solving attempt
struct Model<'a> {
    colors: Vec<z3::ast::Int<'a>>,
    floods: Vec<Vec<z3::ast::Bool<'a>>>,
}

/// Combines a (possably pre-configured) solver w/ the used variables and assertions
pub struct SolverState<'ctx, T> {
    solver: T,
    model: Model<'ctx>,
    asserts: Vec<z3::ast::Bool<'ctx>>
}

impl<'ctx, T> SolverState<'ctx, T> {
    /// Returns all assertions that were given to the solver at this point
    pub fn get_asserts(&self) -> &[z3::ast::Bool<'ctx>] {
        &self.asserts
    }
}

/// Try to solve the given [problem instance][Problem] in `t_max` steps
///
/// # Args
/// - `instance` the problem to solve
/// - `t_max` the length of the solution to search for
/// - `optimize` if z3 should optimize for a minimal solution
///     - if `true`, `t_max` behaves as upper bound
///     - if `false`, `t_max` behaves as exact solution length
pub fn init_solver<'ctx, T: Solver<'ctx>>(
    ctx: &'ctx z3::Context,
    instance: &Problem,
    t_max: usize,
    optimize: bool,
) -> SolverState<'ctx, T> {
    let mut asserts: Vec<z3::ast::Bool<'_>> = Default::default();

    // INIT SOLVER
    let solver = T::new(ctx);

    let mut assert = |ast: &z3::ast::Bool<'ctx>| {
        solver.assert(ast);
        asserts.push(ast.clone());
    };

    // INIT COLOR VARS
    let color_vars: Vec<Int> = (0..t_max)
        .map(|i| Int::new_const(ctx, format!("c_{i}")))
        .collect();

    // ASSERT COLOR VARS
    for var in color_vars.iter() {
        assert(&var.ge(&Int::from_u64(ctx, 0)));
        assert(&var.lt(&Int::from_u64(ctx, instance.num_colors() as u64)));
    }

    for (c1, c2) in color_vars.iter().zip(color_vars.iter().skip(1)) {
        assert(&c1._eq(c2).not());
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
                    ctx,
                    format!("f_{cluster_idx}_{t}"),
                ));
            }
            vars.push(v);
        }
        vars
    };

    // Force improvement in every step when optimizing - FIXME: seems to make the solver *slower*
    #[cfg(not)]
    if optimize {
        let num_clusters = Int::from_u64(&ctx, clusters.len() as u64);
        for t in 0..t_max {
            let vars_t = flooded_vars.iter().map(|vars| &vars[t]);
            let vars_t_plus_1 = flooded_vars.iter().map(|vars| &vars[t + 1]);
            let sum_t = {
                let ints = vars_t
                    .map(|flooded| flooded.ite(&Int::from_u64(&ctx, 1), &Int::from_u64(&ctx, 0)))
                    .collect::<Vec<_>>();
                Int::add(&ctx, ints.iter().collect::<Vec<_>>().as_slice())
            };
            let sum_t_plus_1 = {
                let ints = vars_t_plus_1
                    .map(|flooded| flooded.ite(&Int::from_u64(&ctx, 1), &Int::from_u64(&ctx, 0)))
                    .collect::<Vec<_>>();
                Int::add(&ctx, ints.iter().collect::<Vec<_>>().as_slice())
            };

            assert(&Bool::or(
                &ctx,
                &[&sum_t._eq(&num_clusters), &sum_t_plus_1.gt(&sum_t)],
            ));
        }
    }

    // ASSERT FLOOD VARS (PER CLUSTER)
    for (idx, cluster) in clusters.iter().enumerate() {
        let neighbour_indices =
            cluster.neighbour_clusters(clusters.as_slice(), instance.height(), instance.width());

        let cluster_flooded_vars = &flooded_vars[idx];

        // every cluster must be flooded at last
        assert(cluster_flooded_vars.last().unwrap());

        if idx == start_cluster_idx {
            for a in cluster_flooded_vars.iter() {
                assert(a);
            }
        } else {
            assert(&cluster_flooded_vars.first().unwrap().not());

            for (t, (a, b)) in cluster_flooded_vars
                .iter()
                .zip(cluster_flooded_vars.iter().skip(1))
                .enumerate()
            {
                // if cluster was flooded at t, is must also be flooded at t + 1
                assert(&a.implies(b));

                // cluster's color was choosen at t
                let color_choosen_at_t =
                    color_vars[t]._eq(&Int::from_u64(ctx, cluster.color as u64));

                // any neighbouring cluster was flooded at t
                let any_neighbour_flooded = {
                    let constraints = neighbour_indices.iter().map(|idx| &flooded_vars[*idx][t]);
                    Bool::or(ctx, constraints.collect::<Vec<_>>().as_slice())
                };

                // neighbour was flooded at t + color was choosen at t -> cluster is flooded at t + 1
                assert(
                    &Bool::and(ctx, &[&any_neighbour_flooded, &color_choosen_at_t]).implies(b)
                );

                // cluster was not flooded at t and (no neighbour was flooded at t *or* color was not choosen at t) -> cluster is *not* flooded at t + 1
                assert(
                    &Bool::and(
                        ctx,
                        &[
                            &a.not(),
                            &Bool::or(
                                ctx,
                                &[&any_neighbour_flooded.not(), &color_choosen_at_t.not()],
                            ),
                        ],
                    )
                    .implies(&b.not())
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
                    let all_flooded_t = Bool::and(ctx, flooded_vars_t.as_slice());
                    all_flooded_t.ite(&Int::from_u64(ctx, 1), &Int::from_u64(ctx, 0))
                })
                .collect();

            Int::add(ctx, nums.iter().collect::<Vec<_>>().as_slice())
        };

        solver.maximize(&optimization_goal);
    } else {
        let flooding_at_t_minus_one: Vec<_> =
            flooded_vars.iter().map(|vars| &vars[t_max - 1]).collect();
        let not_all_flooded_at_t_minus_one = Bool::and(ctx, &flooding_at_t_minus_one).not();
        assert(&not_all_flooded_at_t_minus_one);
    }

    let model = Model {
        colors: color_vars,
        floods: flooded_vars,
    };

    SolverState { solver, model, asserts }
}

/// Dispatches a preconfigured solver to z3
pub fn run_solver<'c, T: Solver<'c>>(
    state: SolverState<'c, T>,
    t_max: usize
) -> (z3::SatResult, Option<Solution>) {
    let SolverState {
        solver,
        model: Model { colors: color_vars, floods: flooded_vars },
        ..
    } = state;

    match solver.check() {
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
