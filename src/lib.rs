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
pub mod bitnet;
pub mod cosmology;
pub mod packer;
pub mod regalloc;
pub mod scheduler;
pub use scheduler::{Scheduler, ScheduleResult, SchedulerRedux, SchedulerFeedback};
pub mod stream;
pub mod display;
pub mod validation;
pub mod ppt;
pub mod memory;
pub mod sentron;
pub mod exec;
pub mod smt;
pub mod hdc;
pub mod hdc_optimized;
pub mod analysis;
pub mod curves;
pub mod c_pipe;
pub mod synchronicity;
pub mod iching;
pub mod harmonic;
pub mod harmonics;
pub mod cognitive;
pub mod wedge;

pub use siw::SIW;
pub use pipes::{DenseOp, SparseOp, CoordOp, ReductionOp, PrefetchHint, MessageFormat, FenceScope, MatchMode, MergeOp};
pub use hdc::{HyperVector, AssociativeMemory, HDC_DEFAULT_WIDTH};
pub use phext_coord::PhextCoord;
pub use telemetry::VtpuTelemetry;
pub use stream::StreamBuilder;
pub use validation::{validate_stream, ValidationError};
pub use ppt::{PhextPageTable, PPTStats, MemoryTier};
pub use memory::Memory;
pub use iching::{Trigram, Element, Hexagram, SentronNode, SemanticCircle};
pub use harmonic::{HarmonicState, HarmonicSentron, coord_to_degree};
pub use sentron::{Sentron, SentronState, RegisterFile};
pub use exec::{run as exec_run, run_standalone as exec_run_standalone, ExecStats};
pub use smt::{SmtPair, TrainStats};
pub use analysis::{PortConflictAnalyzer, CacheThrashDetector, MemoryPatternAnalyzer, LocalityAnalyzer};
pub use curves::{ZOrderCurve, HilbertCurve};
pub use c_pipe::{CPipeExecutor, Message, SentronId, CPipeError};
pub use synchronicity::{WuXing, Bagua, SentronMote, ShellOfNine, TOTAL_MOTES, NODES, MOTES_PER_NODE};
pub use wedge::{Wedge, WedgeExecutor, SMT_THREADS, NODES_PER_WEDGE, TOTAL_NODES};
pub use cognitive::{CognitiveEngine, CognitiveStep, CognitiveResult};
pub mod pool;
pub use pool::SentronPool;
pub mod affinity;
pub use affinity::{CpuSet, pin_thread, get_affinity, num_cpus, num_physical_cores, yield_hint};
pub mod phoenix_scheduler;
pub use phoenix_scheduler::{PhoenixScheduler, NineColorDecision, SentronMetrics, CoreMetrics, SchedulerAction, MigrationReason, NineColorStats};
pub mod perf;
pub mod integration;
pub mod assoc;
pub mod cpu_sched;
pub mod redux;
