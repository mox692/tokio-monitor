// Reference
// * https://perfetto.dev/docs/reference/synthetic-track-event

use super::perfetto_protos;
use super::perfetto_protos::debug_annotation;
use super::perfetto_protos::debug_annotation::Value;
use super::perfetto_protos::DebugAnnotation;
use crate::consumer::SpanConsumer;
use crate::span::ProcessDiscriptor;
use crate::span::RawSpan;
use crate::span::ThreadDiscriptor;
use crate::span_queue::drain_descriptors;
use crate::span_queue::DEFAULT_BATCH_SIZE;
use crate::utils::object_pool::Pool;
use crate::utils::object_pool::Puller;
use crate::utils::object_pool::Reusable;
use crate::Type;
use bytes::BytesMut;
use core::cell::RefCell;
use fastant::Anchor;
use fastant::Instant;
use once_cell::sync::Lazy;
use perfetto_protos::{
    trace_packet::{Data, OptionalTrustedPacketSequenceId, OptionalTrustedUid},
    track_descriptor::StaticOrDynamicName,
    track_event::{self, NameField},
    ProcessDescriptor, ThreadDescriptor, TracePacket, TrackDescriptor, TrackEvent,
};
use prost::Message;
use std::io::Write;

#[inline]
fn buf_size() -> usize {
    // More precisely, it should be good enough to hold:
    // DEFAULT_BATCH_SIZE * 2 + SHARD_NUM.load(Ordering::Relaxed) + 1
    //
    // DEFAULT_BATCH_SIZE * 2: normal spans(both start and end events)
    // SHARD_NUM.load(Ordering::Relaxed): process and thread descriptors
    // + 1: process descriptor

    DEFAULT_BATCH_SIZE * 4
}

fn init() -> Vec<TracePacket> {
    vec![TracePacket::default(); buf_size()]
}

fn clear(_vec: &mut Vec<TracePacket>) {
    // do nothing
}

static TRACE_PACKETS_POOL: Lazy<Pool<Vec<TracePacket>>> = Lazy::new(|| Pool::new(init, clear));

thread_local! {
    static TRACE_PACKETS_PULLER: RefCell<Puller<'static, Vec<TracePacket>>> = RefCell::new(TRACE_PACKETS_POOL.puller(2));
}

struct TracePackets(pub(crate) Reusable<'static, Vec<TracePacket>>);

impl Default for TracePackets {
    fn default() -> Self {
        TRACE_PACKETS_PULLER
            .try_with(|puller| TracePackets(puller.borrow_mut().pull()))
            .unwrap_or_else(|_| TracePackets(Reusable::new(&*TRACE_PACKETS_POOL, init())))
    }
}

/// Reporter implementation for Perfetto tracing.
#[derive(Debug)]
pub struct PerfettoReporter {
    pid: i32,
}

impl PerfettoReporter {
    #[inline]
    pub fn new() -> Self {
        Self {
            pid: std::process::id() as i32,
        }
    }
}

/// Docs: https://perfetto.dev/docs/reference/trace-packet-proto#DebugAnnotation
#[inline]
fn create_debug_annotations(span: &RawSpan) -> Vec<DebugAnnotation> {
    let mut annotations = Vec::new();

    // Check if this is a RunTask span with a backtrace
    if let Type::RunTask(run_task) = &span.typ {
        if let Some(backtrace) = &run_task.backtrace {
            let mut debug_annotation = DebugAnnotation::default();
            // TODO: remove allocation
            let name_field = debug_annotation::NameField::Name("backtrace".to_string());
            // TODO: remove allocation
            let value = Value::StringValue(backtrace.clone());
            debug_annotation.name_field = Some(name_field);
            debug_annotation.value = Some(value);
            annotations.push(debug_annotation);
        }
    }

    annotations
}

/// Docs: https://perfetto.dev/docs/reference/trace-packet-proto#TrackEvent
#[inline]
fn create_track_event(
    name: Option<String>,
    track_uuid: u64,
    event_type: Option<track_event::Type>,
    debug_annotations: Vec<DebugAnnotation>,
) -> TrackEvent {
    TrackEvent {
        track_uuid: Some(track_uuid),
        name_field: name.map(NameField::Name),
        r#type: event_type.map(|typ| typ.into()),
        debug_annotations,
        ..Default::default()
    }
}

/// Docs: https://perfetto.dev/docs/reference/trace-packet-proto#ProcessDescriptor
#[inline]
fn create_process_descriptor(pid: i32) -> ProcessDescriptor {
    ProcessDescriptor {
        pid: Some(pid),
        ..Default::default()
    }
}

/// Docs https://perfetto.dev/docs/reference/trace-packet-proto#TrackDescriptor
#[inline]
fn create_track_descriptor(
    uuid: u64,
    name: Option<String>,
    process: Option<ProcessDescriptor>,
    thread: Option<ThreadDescriptor>,
) -> TrackDescriptor {
    TrackDescriptor {
        uuid: Some(uuid),
        static_or_dynamic_name: name.map(StaticOrDynamicName::Name),
        process,
        thread,
        ..Default::default()
    }
}

#[inline]
fn create_thread_descriptor(pid: i32, thread_id: usize, thread_name: String) -> ThreadDescriptor {
    ThreadDescriptor {
        pid: Some(pid),
        tid: Some(thread_id as i32),
        thread_name: Some(thread_name),
        ..Default::default()
    }
}
/// Appends a thread descriptor packet to the trace if not already sent.
fn append_thread_descriptor(
    trace: &mut TracePacket,
    thread_info: &crate::span::ThreadDiscriptor,
    pid: i32,
    track_uuid: u64,
) {
    // TODO: avoid string allocation
    // TODO: get_or_insert thread name to TLS.
    let thread_name = thread_info.thread_name.clone();
    let thread_descriptor = create_thread_descriptor(pid, track_uuid as usize, thread_name.clone());
    let track_descriptor = create_track_descriptor(
        track_uuid,
        // TODO: avoid allocation
        Some(thread_name),
        Some(create_process_descriptor(pid)),
        Some(thread_descriptor),
    );

    trace.data = Some(Data::TrackDescriptor(track_descriptor));
    trace.optional_trusted_uid = Some(OptionalTrustedUid::TrustedUid(42));
}

fn append_process_descriptor(trace: &mut TracePacket, pid: i32, track_uuid: u64) {
    let process_descriptor = create_process_descriptor(pid);
    let track_descriptor =
        create_track_descriptor(track_uuid, None, Some(process_descriptor), None);

    trace.data = Some(Data::TrackDescriptor(track_descriptor));
    trace.optional_trusted_uid = Some(OptionalTrustedUid::TrustedUid(42));
    // Insert the packet at the beginning
    // trace.insert(0, packet);
}
struct Trace {
    pub(self) inner: TracePackets,
}

impl Trace {
    #[inline]
    fn new() -> Self {
        Self {
            inner: TracePackets::default(),
        }
    }

    #[inline]
    fn write<T: Write>(&mut self, output: &mut T, num_packets: usize) {
        // The next pooled object will be temporarily assigned to `self.inner` to avoid borrowing issues.
        let next = TracePackets::default();
        let current = std::mem::replace(&mut self.inner, next);

        let mut packet = current.0.into_inner();

        // SAFETY: num_packets is less than DEFAULT_BATCH_SIZE * 2, and vec is initialized.
        unsafe { packet.set_len(num_packets) };

        let mut trace = perfetto_protos::Trace { packet };
        // TODO: use pool
        let mut buf = BytesMut::with_capacity(buf_size());
        trace.encode(&mut buf).unwrap();
        output.write_all(&buf).unwrap();
        output.flush().unwrap();

        // SAFETY: the cap of vec is `DEFAULT_BATCH_SIZE * 2` and vec is initialized.
        unsafe { trace.packet.set_len(buf_size()) };

        // The original `TracePackets` is now stored in `self.inner`, and the temporary pooled object
        // will be dropped (injected to the pool again).
        self.inner = TracePackets(Reusable::new(&*TRACE_PACKETS_POOL, trace.packet));
    }
}

impl SpanConsumer for PerfettoReporter {
    fn consume(&mut self, spans: &[RawSpan], writer: &mut Box<&mut dyn Write>) {
        let mut trace = Trace::new();

        let pid = self.pid;
        // TODO: move to elsewhere?
        let anchor = Anchor::new();

        let mut packets = trace.inner.0.into_inner();
        let mut num_packets = 0;
        let descriptors = drain_descriptors();
        for descriptor in descriptors {
            // SAFETY: it is garantee that index is less that 1024 and packets have 1024 len
            let packet = unsafe { packets.get_unchecked_mut(num_packets) };
            match &descriptor.typ {
                Type::ProcessDiscriptor(_) => {
                    append_process_descriptor(packet, pid, descriptor.thread_id);
                    packets.swap(0, num_packets);
                    num_packets += 1;
                }
                Type::ThreadDiscriptor(d) => {
                    append_thread_descriptor(packet, d, self.pid, descriptor.thread_id);
                    packets.swap(0, num_packets);
                    num_packets += 1;
                }
                _ => panic!(
                    "Unexpected descriptor type: {:?}",
                    descriptor.typ.type_name_string()
                ),
            };
        }

        for span in spans {
            // SAFETY: it is garantee that index is less that 1024 and packets have 1024 len
            let packet = unsafe { packets.get_unchecked_mut(num_packets) };
            match &span.typ {
                Type::ProcessDiscriptor(_) => {
                    panic!("Process descriptor should not be in spans")
                }
                Type::ThreadDiscriptor(_d) => {
                    panic!("Thread descriptor should not be in spans");
                }
                Type::RunTask(_)
                | Type::RuntimePark(_)
                | Type::RuntimeDriver(_)
                | Type::RuntimeTerminate(_) => {
                    // Start event packet
                    let debug_annotations = create_debug_annotations(span);
                    let start_event = create_track_event(
                        Some(span.typ.type_name_string()),
                        span.thread_id,
                        Some(track_event::Type::SliceBegin),
                        debug_annotations,
                    );
                    packet.data = Some(Data::TrackEvent(start_event));
                    packet.timestamp = Some(span.start.as_unix_nanos(&anchor));
                    packet.optional_trusted_packet_sequence_id =
                        Some(OptionalTrustedPacketSequenceId::TrustedPacketSequenceId(42));

                    num_packets += 1;
                    let packet = unsafe { packets.get_unchecked_mut(num_packets) };

                    // End event packet
                    let debug_annotations = create_debug_annotations(span);
                    let end_event = create_track_event(
                        None,
                        span.thread_id,
                        Some(track_event::Type::SliceEnd),
                        debug_annotations,
                    );

                    packet.data = Some(Data::TrackEvent(end_event));
                    packet.trusted_pid = Some(pid);
                    packet.timestamp = Some(span.end.as_unix_nanos(&anchor));
                    packet.optional_trusted_packet_sequence_id =
                        Some(OptionalTrustedPacketSequenceId::TrustedPacketSequenceId(42));

                    num_packets += 1;
                }
                Type::RuntimeStart(_) => {
                    unimplemented!()
                }
            };
        }

        trace.inner = TracePackets(Reusable::new(&*TRACE_PACKETS_POOL, packets));
        trace.write(writer, num_packets);
    }
}

pub(crate) fn thread_descriptor() -> RawSpan {
    let thread_id = crate::utils::thread_id::get() as u64;
    RawSpan {
        typ: Type::ThreadDiscriptor(ThreadDiscriptor {
            thread_name: std::thread::current()
                .name()
                .map(|str| str.into())
                .unwrap_or(format!("{thread_id}")),
        }),
        thread_id,
        start: Instant::ZERO,
        end: Instant::ZERO,
    }
}

pub(crate) fn process_descriptor() -> RawSpan {
    RawSpan {
        typ: Type::ProcessDiscriptor(ProcessDiscriptor {}),
        thread_id: crate::utils::thread_id::get() as u64,
        start: Instant::ZERO,
        end: Instant::ZERO,
    }
}
