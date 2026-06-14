#![forbid(unsafe_code)]

mod backend;
mod bucket;
mod classify;
mod error;
#[cfg(feature = "uniffi")]
mod ffi;
mod formatter;
mod locale;
#[cfg(feature = "mf2")]
mod mf2;
mod options;
mod time;

pub use bucket::TimelineDateBucket;
pub use error::{TimelineDateError, TimelineDateResult};
#[cfg(feature = "uniffi")]
pub use ffi::{TimelineDateFfiError, format_feed_label};
pub use formatter::TimelineDateFormatter;
pub use options::{
    FuturePolicy, HourCycle, OldDateTimePolicy, TimelineDateOptions, TimelineDateStyle,
};

#[cfg(feature = "uniffi")]
uniffi::setup_scaffolding!("timeline_date");
