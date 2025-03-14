// Copyright 2020 TiKV Project Authors. Licensed under Apache-2.0.

// Suppress a false-positive lint from clippy
#![allow(clippy::needless_doctest_main)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(not(feature = "enable"), allow(dead_code))]
#![cfg_attr(not(feature = "enable"), allow(unused_mut))]
#![cfg_attr(not(feature = "enable"), allow(unused_imports))]
#![cfg_attr(not(feature = "enable"), allow(unused_variables))]
#![cfg_attr(target_family = "wasm", allow(dead_code))]

pub mod collector;
mod event;
pub mod future;
pub mod local;
mod macros;
mod span;
#[doc(hidden)]
pub mod util;

pub use crate::collector::global_collector::flush;
pub use crate::collector::global_collector::set_reporter;
pub use crate::event::Event;
pub use crate::span::Span;
pub mod fastant;

pub mod prelude {
    //! A "prelude" for crates using `fastrace`.
    #[doc(no_inline)]
    pub use crate::collector::SpanContext;
    #[doc(no_inline)]
    pub use crate::collector::SpanId;
    #[doc(no_inline)]
    pub use crate::collector::SpanRecord;
    #[doc(no_inline)]
    pub use crate::collector::TraceId;
    #[doc(no_inline)]
    pub use crate::event::Event;
    #[doc(no_inline)]
    pub use crate::file_location;
    #[allow(deprecated)]
    #[doc(no_inline)]
    pub use crate::full_name;
    #[doc(no_inline)]
    pub use crate::func_name;
    #[doc(no_inline)]
    pub use crate::func_path;
    #[doc(no_inline)]
    pub use crate::future::FutureExt as _;
    #[doc(no_inline)]
    pub use crate::local::LocalSpan;
    #[doc(no_inline)]
    pub use crate::span::Span;
}
