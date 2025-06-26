cfg_rt! {
    use std::marker::PhantomData;

    #[derive(Copy, Clone)]
    pub(crate) struct SpawnMeta<'a> {
        /// The name of the task
        #[cfg(all(tokio_unstable, feature = "tracing"))]
        pub(crate) name: Option<&'a str>,
        /// The original size of the future or function being spawned
        #[cfg(all(tokio_unstable, feature = "tracing"))]
        pub(crate) original_size: usize,
        _pd: PhantomData<&'a ()>,
    }

    impl<'a> SpawnMeta<'a> {
        /// Create new spawn meta with a name and original size (before possible auto-boxing)
        #[cfg(all(tokio_unstable, feature = "tracing"))]
        pub(crate) fn new(name: Option<&'a str>, original_size: usize) -> Self {
            Self {
                name,
                original_size,
                _pd: PhantomData,
            }
        }

        /// Create a new unnamed spawn meta with the original size (before possible auto-boxing)
        pub(crate) fn new_unnamed(original_size: usize) -> Self {
            #[cfg(not(all(tokio_unstable, feature = "tracing")))]
            let _original_size = original_size;

            Self {
                #[cfg(all(tokio_unstable, feature = "tracing"))]
                name: None,
                #[cfg(all(tokio_unstable, feature = "tracing"))]
                original_size,
                _pd: PhantomData,
            }
        }
    }

    cfg_trace! {
        use std::mem;
        use tracing::instrument::Instrument;
        pub(crate) use tracing::instrument::Instrumented;

        #[inline]
        #[track_caller]
        pub(crate) fn task<F>(task: F, kind: &'static str, meta: SpawnMeta<'_>, id: u64) -> Instrumented<F> {
            #[track_caller]
            fn get_span(kind: &'static str, spawn_meta: SpawnMeta<'_>, id: u64, task_size: usize) -> tracing::Span {
                let location = std::panic::Location::caller();
                let original_size = if spawn_meta.original_size != task_size {
                    Some(spawn_meta.original_size)
                } else {
                    None
                };
                tracing::trace_span!(
                    target: "tokio::task",
                    parent: None,
                    "runtime.spawn",
                    %kind,
                    task.name = %spawn_meta.name.unwrap_or_default(),
                    task.id = id,
                    original_size.bytes = original_size,
                    size.bytes = task_size,
                    loc.file = location.file(),
                    loc.line = location.line(),
                    loc.col = location.column(),
                )
            }
            use tracing::instrument::Instrument;
            let span = get_span(kind, meta, id, mem::size_of::<F>());
            task.instrument(span)
        }

        #[inline]
        #[track_caller]
        pub(crate) fn blocking_task<Fn, Fut>(task: Fut, spawn_meta: SpawnMeta<'_>, id: u64) -> Instrumented<Fut> {
            let location = std::panic::Location::caller();

            let fn_size = mem::size_of::<Fn>();
            let original_size = if spawn_meta.original_size != fn_size {
                Some(spawn_meta.original_size)
            } else {
                None
            };

            let span = tracing::trace_span!(
                target: "tokio::task::blocking",
                "runtime.spawn",
                kind = %"blocking",
                task.name = %spawn_meta.name.unwrap_or_default(),
                task.id = id,
                "fn" = %std::any::type_name::<Fn>(),
                original_size.bytes = original_size,
                size.bytes = fn_size,
                loc.file = location.file(),
                loc.line = location.line(),
                loc.col = location.column(),
            );
            task.instrument(span)

        }
    }

    cfg_not_trace! {
        #[inline]
        pub(crate) fn task<F>(task: F, _kind: &'static str, _meta: SpawnMeta<'_>, _id: u64) -> F {
            // nop
            task
        }

        #[inline]
        pub(crate) fn blocking_task<Fn, Fut>(task: Fut, _spawn_meta: SpawnMeta<'_>, _id: u64) -> Fut {
            let _ = PhantomData::<&Fn>;
            // nop
            task
        }
    }
}

cfg_time! {
    #[track_caller]
    pub(crate) fn caller_location() -> Option<&'static std::panic::Location<'static>> {
        #[cfg(all(tokio_unstable, feature = "tracing"))]
        return Some(std::panic::Location::caller());
        #[cfg(not(all(tokio_unstable, feature = "tracing")))]
        None
    }
}

cfg_trace! {
    use pin_project_lite::pin_project;

    #[allow(unused)]
    #[derive(Debug, Clone)]
    pub(crate) struct AsyncOpTracingCtx {
        pub(crate) async_op_span: tracing::Span,
        pub(crate) async_op_poll_span: tracing::Span,
        pub(crate) resource_span: tracing::Span,
    }

    #[allow(unused)]
    pub(crate) fn async_op<P,F>(inner: P, resource_span: tracing::Span, source: &str, poll_op_name: &'static str, inherits_child_attrs: bool) -> InstrumentedAsyncOp<F>
    where P: FnOnce() -> F {
        resource_span.in_scope(|| {
            let async_op_span = tracing::trace_span!("runtime.resource.async_op", source = source, inherits_child_attrs = inherits_child_attrs);
            let enter = async_op_span.enter();
            let async_op_poll_span = tracing::trace_span!("runtime.resource.async_op.poll");
            let inner = inner();
            drop(enter);
            let tracing_ctx = AsyncOpTracingCtx {
                async_op_span,
                async_op_poll_span,
                resource_span: resource_span.clone(),
            };
            InstrumentedAsyncOp {
                inner,
                tracing_ctx,
                poll_op_name,
            }
        })
    }

    pin_project! {
        #[derive(Debug, Clone)]
        pub(crate) struct InstrumentedAsyncOp<F> {
            #[pin]
            pub(crate) inner: F,
            pub(crate) tracing_ctx: AsyncOpTracingCtx,
            pub(crate) poll_op_name: &'static str
        }
    }

    impl<F: std::future::Future> std::future::Future for InstrumentedAsyncOp<F> {
        type Output = F::Output;

        fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
            let this = self.project();
            let poll_op_name = &*this.poll_op_name;
            let _res_enter = this.tracing_ctx.resource_span.enter();
            let _async_op_enter = this.tracing_ctx.async_op_span.enter();
            let _async_op_poll_enter = this.tracing_ctx.async_op_poll_span.enter();
            trace_poll_op!(poll_op_name, this.inner.poll(cx))
        }
    }
}

cfg_runtime_tracing_backtrace! {
    #[allow(unused)]
    pub(crate) fn gen_backtrace() -> String {
        use hopframe::unwinder::UnwindBuilderX86_64;
        use std::fmt::Write;

        let mut unwinder = UnwindBuilderX86_64::new().build();
        unwinder
            .unwind()
            .fold(String::new(), |mut acc, frame| {
                write!(&mut acc, "{:?},", frame.address())
                    .expect("writing to String cannot fail");
                acc
            })
    }
}
