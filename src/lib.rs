//! vTPU Runtime: Software-Defined Virtual TPU with Phext-Native Addressing
//!
//! This crate implements the vTPU architecture specified in R23W1.
//! Core innovation: 3-pipe retirement model (D/S/C) achieves 3 ops/cycle
//! on commodity AMD Zen 4 processors via sentron-native instruction scheduling.
//!
//! # Example
//!
//! ```
//! use vtpu_runtime::{SIW, DenseOp, SparseOp, CoordOp, PhextCoord, StreamBuilder};
//!
//! // Build a simple compute kernel
//! let mut builder = StreamBuilder::new();
//!
//! // Load two values from phext coordinates
//! builder.push(SIW::new(
//!     DenseOp::DNOP,
//!     SparseOp::SGATHER { rd: 1, coord_idx: 0, width: 64 },
//!     CoordOp::CNOP,
//!     PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]),
//! ));
//!
//! builder.push(SIW::new(
//!     DenseOp::DNOP,
//!     SparseOp::SGATHER { rd: 2, coord_idx: 1, width: 64 },
//!     CoordOp::CNOP,
//!     PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2]),
//! ));
//!
//! // Add them
//! builder.push(SIW::new(
//!     DenseOp::DADD { rd: 3, rs1: 1, rs2: 2 },
//!     SparseOp::SNOP,
//!     CoordOp::CNOP,
//!     PhextCoord::zero(),
//! ));
//!
//! // Store result
//! builder.push(SIW::new(
//!     DenseOp::DNOP,
//!     SparseOp::SSCATTR { coord_idx: 2, rs: 3, width: 64 },
//!     CoordOp::CNOP,
//!     PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 3]),
//! ));
//!
//! let stream = builder.build();
//! println!("Generated {} SIWs", stream.len());
//! ```

pub mod siw;
pub mod pipes;
pub mod phext_coord;
pub mod telemetry;
pub mod scheduler;
pub mod stream;
pub mod display;
pub mod validation;

pub use siw::SIW;
pub use pipes::{DenseOp, SparseOp, CoordOp, ReductionOp, PrefetchHint, MessageFormat, FenceScope};
pub use phext_coord::PhextCoord;
pub use telemetry::VtpuTelemetry;
pub use scheduler::Scheduler;
pub use stream::StreamBuilder;
pub use validation::{validate_stream, ValidationError};
