//! Command line argument parser

use clap::*;

#[derive(clap::Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
   #[command(subcommand)]
   action: Action,
   #[arg(global = true, long = "print-asserts", help = "Print asserts in SMT-LIB format")]
   print_asserts: bool,
   #[arg(global = true, long = "dry-run", help = "Only create asserts but don't solve")]
   dry_run: bool,
}

impl Args {
    pub fn get_action(&self) -> &Action {
        &self.action
    }

    pub fn print_asserts(&self) -> bool {
        self.print_asserts
    }

    pub fn dry_run(&self) -> bool {
        self.dry_run
    }
}

/// Mode of finding an (optimal) solution
#[derive(Debug, Clone, Subcommand)]
pub enum Action {
    #[command(about = "Use z3 optimizer to find minimal solution")]
    Opt {
        #[arg(help = "Upper solution size bound")]
        upper_bound: Option<usize>
    },
    #[command(about = "Find minimal solution by binary search in reasonable bounds")]
    Min,
    #[command(about = "Find minimal solution by binary search in bounds")]
    Search { lower_bound: usize, upper_bound: usize },
    #[command(about = "Find solution with exact size")]
    Exact { size: usize },
    #[command(about = "Find solution with reasonable large size")]
    Solve,
}

impl Action {
    pub fn use_optimizer(&self) -> bool {
        matches!(self, Action::Opt { .. })
    }

    // Get bounds defined by action type with given fallback values `lo` and `hi`
    pub fn get_bounds(&self, lo: usize, hi: usize) -> (usize, usize) {
        match self {
            Action::Opt { upper_bound: Some(upper_bound) } => (*upper_bound, *upper_bound),
            Action::Opt { upper_bound: None } => (hi, hi),
            Action::Min => (lo, hi),
            Action::Search { lower_bound, upper_bound } => (*lower_bound, *upper_bound),
            Action::Exact { size } => (*size, *size),
            Action::Solve => (hi, hi),
        }
    }
}