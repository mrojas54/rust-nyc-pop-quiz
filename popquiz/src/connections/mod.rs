//! The audience's side: sessions and the live connection.
//!
//! - [`sessions`] — participant sessions and the answer store (T-04b).
//! - [`ws`] — the transport: one broadcast per room to the wall, the buzzers
//!   and the host, and reconnect with the same session token (T-04c).
//! - [`requestlog`] — AC-55's evidence: a line per failed participant request
//!   and a per-room summary, on standard error for `fly logs` (T-21).

pub mod requestlog;
pub mod sessions;
pub mod ws;
