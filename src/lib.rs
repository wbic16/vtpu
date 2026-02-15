//! vTPU: Virtual Tensor Processing Unit for Sentron Architectures
//!
//! A software-defined accelerator on commodity AMD R9 hardware.
//! Achieves 3-operation-per-clock retirement through sentron-native
//! instruction scheduling with phext 11D addressing.
//!
//! Hardware just needs good software.

pub mod coord;
pub mod siw;
pub mod pipe;
pub mod sentron;
pub mod exec;
pub mod builder;

pub use coord::PhextCoord;
pub use pipe::{DenseOp, SparseOp, CoordOp};
pub use siw::SIW;
pub use sentron::Sentron;
pub use exec::{run, ExecStats};
