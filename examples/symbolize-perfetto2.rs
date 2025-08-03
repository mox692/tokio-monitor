// Example program that parses Perfetto binary trace files (.pftrace), symbolizes
// backtraces found in the trace data, and rewrites the trace with symbolized
// stack traces embedded in debug annotations.
//
// # Usage
//
// ```bash
// $ cargo run --package examples --example symbolize-perfetto2 -- <input.pftrace> <binary_path> <output.pftrace>
// ```

use hopframe::symbolize::{LookupAddress, SymbolMapBuilder};
use prost::Message;
use std::fs::File;
use std::io::{Read, Write};
use std::{env, path::Path};

// Import the Perfetto protobuf definitions
#[path = "../rt-trace/src/backend/perfetto_protos.rs"]
mod perfetto_protos;

use perfetto_protos::{trace_packet, DebugAnnotation, Trace, TracePacket, TrackEvent};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        eprintln!(
            "Usage: {} <input.pftrace> <binary_path> <output.pftrace>",
            args[0]
        );
        std::process::exit(1);
    }

    let input_filename = &args[1];
    let bin_file = &args[2];
    let output_filename = &args[3];

    // Read the Perfetto binary file
    let mut file = File::open(input_filename)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    // Build the symbol map for address symbolization
    let bin_file = Path::new(bin_file);
    let symbol_map = SymbolMapBuilder::new()
        .with_binary_path(Path::new(bin_file))
        .build()
        .await;

    // Parse and process Perfetto trace packets
    let modified_trace = process_trace(&buffer, &symbol_map).await?;

    // Write the modified trace to output file
    let mut output_file = File::create(output_filename)?;
    output_file.write_all(&modified_trace)?;

    println!("Successfully wrote symbolized trace to {}", output_filename);

    Ok(())
}

// Process the entire trace and return modified trace data
async fn process_trace(
    buffer: &[u8],
    symbol_map: &hopframe::symbolize::SymbolMap,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Decode the entire trace
    let mut trace = Trace::decode(buffer)?;

    // Process each packet in the trace
    for packet in &mut trace.packet {
        if needs_symbolization(packet) {
            *packet = symbolize_packet(packet.clone(), symbol_map).await?;
        }
    }

    // Re-encode the modified trace
    Ok(trace.encode_to_vec())
}

// Check if a packet needs symbolization
fn needs_symbolization(packet: &TracePacket) -> bool {
    if let Some(trace_packet::Data::TrackEvent(track_event)) = &packet.data {
        for annotation in &track_event.debug_annotations {
            if let Some(name) = get_annotation_name(annotation) {
                if name == "backtrace" {
                    return true;
                }
            }
        }
    }
    false
}

// Symbolize a packet
async fn symbolize_packet(
    mut packet: TracePacket,
    symbol_map: &hopframe::symbolize::SymbolMap,
) -> Result<TracePacket, Box<dyn std::error::Error>> {
    if let Some(trace_packet::Data::TrackEvent(ref mut track_event)) = &mut packet.data {
        symbolize_track_event(track_event, symbol_map).await?;
    }
    Ok(packet)
}

// Symbolize a TrackEvent by updating its debug annotations
async fn symbolize_track_event(
    track_event: &mut TrackEvent,
    symbol_map: &hopframe::symbolize::SymbolMap,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut new_annotations = Vec::new();

    for annotation in &track_event.debug_annotations {
        if let Some(name) = get_annotation_name(annotation) {
            if name == "backtrace" {
                if let Some(backtrace_str) = get_annotation_string_value(annotation) {
                    if !backtrace_str.is_empty() {
                        // Symbolize the backtrace
                        let symbolized = symbolize_backtrace(&backtrace_str, symbol_map).await;

                        // Create a new annotation with symbolized stack trace
                        let mut symbolized_annotation = DebugAnnotation::default();
                        symbolized_annotation.name_field =
                            Some(perfetto_protos::debug_annotation::NameField::Name(
                                "symbolized_backtrace".to_string(),
                            ));
                        symbolized_annotation.value = Some(
                            perfetto_protos::debug_annotation::Value::StringValue(symbolized),
                        );
                        new_annotations.push(symbolized_annotation);
                    }
                }
            }
        }
    }

    // Add the new annotations to the track event
    track_event.debug_annotations.extend(new_annotations);

    Ok(())
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

// Symbolize a backtrace string and return formatted result
async fn symbolize_backtrace(
    backtrace_str: &str,
    symbol_map: &hopframe::symbolize::SymbolMap,
) -> String {
    let mut result = String::new();

    // Parse addresses from the backtrace string
    // Backtraces are stored as comma-separated hex addresses
    let addresses: Vec<&str> = backtrace_str.split(',').filter(|s| !s.is_empty()).collect();

    for (i, addr_str) in addresses.iter().enumerate() {
        if let Ok(address) = parse_hex_address(addr_str) {
            // Look up the symbol
            let symbol = symbol_map
                .lookup(LookupAddress::Relative(address as u32))
                .await;

            match symbol {
                Some(_) => {
                    let sym_name = symbol.map(|a| a.symbol.name).unwrap_or_default();
                    result.push_str(&format!("#{} 0x{:x} {}\n", i, address, sym_name));
                }
                None => {
                    result.push_str(&format!("#{} 0x{:x} <unknown>\n", i, address));
                }
            }
        }
    }

    result
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
