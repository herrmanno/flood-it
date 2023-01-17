//! Pretty-printing solutions

use crate::{colorizer::Colorizer, problem::Problem, solution::Solution};

/// Prints a solution to a problem step by step to stdout
pub fn print_solution(instance: &Problem, solution: &Solution) {
    let colorizer = Colorizer::new();
    let dashes = "".repeat(instance.width());
    let mut instance = instance.clone();

    println!("Step 0");
    println!("{}", dashes);
    println!("{}", instance);

    for (idx, color) in solution.colors.iter().enumerate() {
        instance.apply_color(*color);

        print!("Step {}: ", idx + 1);
        {
            let mut buf = String::new();
            colorizer.write(&mut buf, "  ", *color as usize).unwrap();
            println!("{}", buf);
        }
        println!("{}", dashes);
        println!("{}", instance);
    }
}
