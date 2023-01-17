//! # Solver for the 'flood it' puzzle
//!
//! The following text is a short overview of the concept of modelling the 'Flood it' puzzle as
//! a SMT problem.
//!
//! ## Idea and modelling
//! A problem consists of an n ⨉ n matrix `M` of integers.  
//! The value M[r, c] denotes the color of the tile at row r and column c.  
//! A single solver run also has a parameter `T` that describes the length of the solution
//! to find.  
//! Furthermore the parameter Cl denotes the number of clusters in M and Co denotes the number of
//! different colors in M.
//!
//! ### Variables
//! The system is built upon the following variables:
//! - [c_0, c_1, ..., c_{T - 1}] := 'color variables' of type integer
//! - [f_0_0, f_0_1, ..., f_{Cl}_T] := 'flood variables' of type bool
//!
//! The meaning of the variables is as follows:
//! - c_i := the i-th color of the solution is j
//! - f_i_t := the cluster i is flooded at time t
//!
//! For a cluster cluster_i to be flooded at time t means that, after applying the n-th move of the
//! solution, there is a tile  
//! (y,x) ∈ cluster_i  
//! such that there exists a  
//! (y,x)-(0,0) path P  
//! where  
//! ∀ (y,x) ∈ P: M_t[y,x] == Color(i)  
//! holds.
//!
//! ### Constraints
//! The system is built upon the following constraints:
//!
//! Color constraints
//! - ∀ i ∈ [0,T): 0 ≤ c_i < Co
//! - ∀ i ∈ [0,T - 1): c_i ≠ c_{i + 1}
//!
//! Static flooding constraints
//! - ∀ i ∈ { i | i ∈ [0,Cl), (0,0) ∈ c_i }: ∀ t ∈ [0,T]: f_i_t
//! - ∀ i ∈ { i | i ∈ [0,Cl), (0,0) ∉ c_i }: ¬ f_i_0
//! - ∀ i ∈ [0,Cl): f_i_T
//!
//! Dynamic flooding constraints
//! - ∀ i ∈ [0, Cl): ∀ t ∈ [0,T): f_i_t -> f_i_{t + 1}
//! - ∀ i ∈ [0, Cl): ∀ t ∈ [0,T): (∃ j ∈ Neighbours(i): f_j_t) ∧ (c_t == Color(i)) -> f_i_{t + 1}
//! - ∀ i ∈ [0, Cl): ∀ t ∈ [0,T): ¬ f_i_t ∧ (¬(∃ j ∈ Neighbours(i): f_j_t) ∨ ¬(c_t == Color(i))) -> ¬ f_i_{t + 1}
//!
//! Where
//! - Neighbours(i) := indices of all clusters adjacent to cluster_i
//! - Color(i) := color of cluster_i
//!

pub mod cli;
pub mod cluster;
mod colorizer;
pub mod printer;
pub mod problem;
pub mod solution;
pub mod solver;
mod util;
