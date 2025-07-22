// Example program that parses Perfetto binary trace files (.pftrace) and symbolizes
// backtraces found in the trace data. This is useful for analyzing runtime traces
// from tokio-monitor that include backtrace information.
//
// Usage: cargo run --example symbolize-perfetto <trace_file.pftrace>

use hopframe::symbolize::{read_aslr_offset, LookupAddress, SymbolMapBuilder};
use prost::Message;
use std::fs::File;
use std::io::Read;
use std::{env, path::Path};

// Import the Perfetto protobuf definitions
#[path = "../rt-trace/src/backend/perfetto_protos.rs"]
mod perfetto_protos;

use perfetto_protos::{trace_packet, DebugAnnotation, TracePacket, TrackEvent};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <perfetto_file.pftrace>", args[0]);
        std::process::exit(1);
    }

    let filename = &args[1];
    let aslr_offset = &args[2];
    let bin_file = &args[3];
    let aslr_offset: u64 = aslr_offset.parse().unwrap();

    // Read the Perfetto binary file
    let mut file = File::open(filename)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    // Build the symbol map for address symbolization
    let bin_file = Path::new(bin_file);
    println!(
        "Building symbol map..., bin_file = {:?}, offset: {}",
        bin_file, aslr_offset
    );

    let symbol_map = SymbolMapBuilder::new()
        .with_binary_path(Path::new(bin_file))
        .build()
        .await;
    // println!("ASLR offset: {:?}", aslr_offset);

    // Parse Perfetto trace packets
    let mut offset = 0;
    while offset < buffer.len() {
        // Try to decode a TracePacket
        let packet = match decode_trace_packet(&buffer[offset..]) {
            Ok((packet, bytes_consumed)) => {
                offset += bytes_consumed;
                packet
            }
            Err(_) => {
                // If we can't decode, skip a byte and try again
                offset += 1;
                continue;
            }
        };

        // Process the packet if it contains track events
        if let Some(trace_packet::Data::TrackEvent(track_event)) = packet.data {
            process_track_event(&track_event, &symbol_map, aslr_offset).await;
        }
    }

    Ok(())
}

// Decode a single TracePacket from the buffer
fn decode_trace_packet(buffer: &[u8]) -> Result<(TracePacket, usize), Box<dyn std::error::Error>> {
    // Perfetto uses length-delimited encoding for packets
    // First byte(s) contain the varint-encoded length
    let (length, length_bytes) = decode_varint(buffer)?;

    if buffer.len() < length_bytes + length {
        return Err("Insufficient buffer size".into());
    }

    let packet_data = &buffer[length_bytes..length_bytes + length];
    let packet = TracePacket::decode(packet_data)?;

    Ok((packet, length_bytes + length))
}

// Decode a varint (variable-length integer)
fn decode_varint(buffer: &[u8]) -> Result<(usize, usize), Box<dyn std::error::Error>> {
    let mut value = 0;
    let mut shift = 0;
    let mut bytes_read = 0;

    for &byte in buffer {
        bytes_read += 1;
        value |= ((byte & 0x7F) as usize) << shift;
        if byte & 0x80 == 0 {
            return Ok((value, bytes_read));
        }
        shift += 7;
        if shift > 35 {
            return Err("Varint too long".into());
        }
    }

    Err("Unexpected end of buffer".into())
}

// Process a TrackEvent and symbolize any backtraces found
async fn process_track_event(
    track_event: &TrackEvent,
    symbol_map: &hopframe::symbolize::SymbolMap,
    aslr_offset: u64,
) {
    // Check debug annotations for backtraces
    for annotation in &track_event.debug_annotations {
        if let Some(name) = get_annotation_name(annotation) {
            if name == "backtrace" {
                if let Some(backtrace_str) = get_annotation_string_value(annotation) {
                    if !backtrace_str.is_empty() {
                        symbolize_backtrace(&backtrace_str, symbol_map, aslr_offset).await;
                    }
                }
            }
        }
    }
}

// Get the name of a debug annotation
fn get_annotation_name(annotation: &DebugAnnotation) -> Option<String> {
    use perfetto_protos::debug_annotation::NameField;

    match &annotation.name_field {
        Some(NameField::Name(name)) => Some(name.clone()),
        Some(NameField::NameIid(_)) => {
            // Would need to resolve interned string here
            None
        }
        None => None,
    }
}

// Get the string value of a debug annotation
fn get_annotation_string_value(annotation: &DebugAnnotation) -> Option<String> {
    use perfetto_protos::debug_annotation::Value;

    match &annotation.value {
        Some(Value::StringValue(s)) => Some(s.clone()),
        _ => None,
    }
}

// Symbolize a backtrace string
async fn symbolize_backtrace(
    backtrace_str: &str,
    symbol_map: &hopframe::symbolize::SymbolMap,
    aslr_offset: u64,
) {
    // Parse addresses from the backtrace string
    // Backtraces are stored as comma-separated hex addresses
    let addresses: Vec<&str> = backtrace_str.split(',').filter(|s| !s.is_empty()).collect();

    for (i, addr_str) in addresses.iter().enumerate() {
        if let Ok(address) = parse_hex_address(addr_str) {
            // Convert to relative address by subtracting ASLR offset
            // let relative_addr = if address >= aslr_offset {
            //     (address - aslr_offset) as u32
            // } else {
            //     // If address is less than ASLR offset, it might already be relative
            //     address as u32
            // };

            // Look up the symbol
            let symbol = symbol_map
                .lookup(LookupAddress::Relative(address as u32))
                .await;

            match symbol {
                Some(ref sym) => {
                    let sym = symbol.map(|a| a.symbol.name).unwrap_or_default();
                    println!("  #{} 0x{:x} => {}", i, address, sym);
                }
                None => {
                    println!("  #{} 0x{:x} => <unknown>", i, address);
                }
            }
        }
    }
}

// Parse a hex address string (e.g., "0x1234abcd" or just "1234abcd")
fn parse_hex_address(addr_str: &str) -> Result<u64, Box<dyn std::error::Error>> {
    let addr_str = addr_str.trim();

    // Try parsing as decimal first (the format used by gen_backtrace)
    if let Ok(addr) = addr_str.parse::<u64>() {
        return Ok(addr);
    }

    // Otherwise try hex
    if addr_str.starts_with("0x") || addr_str.starts_with("0X") {
        Ok(u64::from_str_radix(&addr_str[2..], 16)?)
    } else {
        Ok(u64::from_str_radix(addr_str, 16)?)
    }
}
