//! A tool that embeds a symbol information to the perfetto's output.

use bytes::Bytes;
use bytes::BytesMut;
use object::Object;
use std::borrow::Cow;
use std::path::Path;
// use clap::Parser;
// use hopframe::symbolize::{LookupAddress, SymbolMapBuilder};
use addr2line::LookupResult;
use object::ObjectSection;
use prost::Message;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::u64;

mod idl {
    include!(concat!(env!("OUT_DIR"), "/perfetto.protos.rs"));
}

#[cfg(not(target_os = "linux"))]
compile_error! {
    "This crate is only supported on linux."
}

// #[derive(Parser, Debug, Default)]
// #[command(version, about, long_about = None)]
// struct Cli {
//     /// The path to the executable binary that was traced by perfetto
//     #[arg(short, long)]
//     bin_path: PathBuf,

//     /// The path to the perfetto trace log
//     #[arg(short, long)]
//     perfetto_trace_log: PathBuf,

//     /// (optional) The output path
//     #[arg(short, long)]
//     output_path: Option<PathBuf>,
// }

#[tokio::main]
async fn main() {
    let bin_path = "./runtime-tracing";
    let perfetto_trace_log = "./test.pftrace";
    let output_path = "perfetto_symbolize.log";
    let mut file = File::open(perfetto_trace_log).unwrap();
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).unwrap();
    let bytes = Bytes::from(buf);
    let mut trace = idl::Trace::decode(bytes).unwrap();

    // todo: change the way to get an offset
    let mut offset = None;
    for packet in trace.packet.iter_mut() {
        let Some(data) = &packet.interned_data else {
            continue;
        };
        let Some(interned_string) = data.debug_annotation_string_values.get(0) else {
            continue;
        };
        offset = interned_string
            .str
            .as_ref()
            .map(|v| v.into_iter().fold(0_u64, |acc, x| acc * 10 + (*x as u64)));
    }

    println!("aslr_offset: {:?}", &offset);
    // let symbol_map = SymbolMapBuilder::new()
    //     .with_binary_path(&bin_path)
    //     .build()
    //     .await;

    for (i, packet) in trace.packet.iter_mut().enumerate() {
        println!("i: {i}");
        let Some(idl::trace_packet::Data::TrackEvent(ref mut e)) = &mut packet.data else {
            continue;
        };
        // track_event::Type::SliceBegin
        if e.r#type != Some(1) {
            continue;
        }
        let Some(f) = e.debug_annotations.iter_mut().find(|a| {
            let cond1 = match &a.value {
                Some(v) => match v {
                    idl::debug_annotation::Value::StringValue(v) => AsRef::<str>::as_ref(v) != "",
                    _ => false,
                },
                _ => false,
            };
            let cond2 = a.name_field.as_ref()
                == Some(&idl::debug_annotation::NameField::Name(
                    "stacktrace".to_string(),
                ));

            cond1 && cond2
        }) else {
            continue;
        };
        let Some(idl::debug_annotation::Value::StringValue(ref mut addr_string)) = &mut f.value
        else {
            panic!("unexpected result")
        };

        let addresses: Vec<_> = addr_string
            .split(",")
            .filter_map(|s| {
                let Ok(addr) = s.parse::<u64>() else {
                    return None;
                };

                Some(addr - offset.unwrap())
            })
            .collect();

        let mut res = String::new();

        let path = &Path::new(bin_path);
        let file = File::open(path).unwrap();
        let map = unsafe { memmap2::Mmap::map(&file).unwrap() };
        let obj = object::File::parse(&*map).unwrap();
        let dwarf = dwarf_load(&obj);
        let dwarf = dwarf_borrow(&dwarf);
        let ctx = addr2line::Context::from_dwarf(dwarf).unwrap();
        // Ensure nothing is lazily loaded.
        for addr in &addresses {
            let mut str = "".to_string();
            let Some(loc) = ctx.find_location(*addr).ok().unwrap() else {
                continue;
            };
            let mut frames_iter = match ctx.find_frames(*addr) {
                LookupResult::Output(result) => result,
                LookupResult::Load { .. } => {
                    unreachable!("addr2line promised we wouldn't get here")
                }
            };

            for frame in frames_iter.iter_mut() {
                while let Ok(Some(frame)) = frame.next() {
                    if let Ok(function) = frame.function.expect("REASON").demangle() {
                        str.push_str(function.as_ref())
                    }
                    if let Some(loc) = frame.location {
                        str.push_str("\n    ");
                        str.push_str(loc.file.unwrap_or(""));
                        str.push_str(&(loc.column.unwrap_or(0).to_string()));
                        str.push_str(":");
                        str.push_str(&(loc.column.unwrap_or(0).to_string()));
                        str.push_str("\n");
                    }
                }
            }
            res.push_str(&str);
        }

        // while let Some(frame) = iter.next() {
        //     // Get symbol for each frame.
        //     let mut str = "".to_string();
        //     let Some(mut symbol) = symbol_map
        //         .lookup(LookupAddress::Relative(frame as u32))
        //         .await
        //     else {
        //         continue;
        //     };
        //     symbol.symbol.name.push('\n');
        //     str.push_str(&symbol.symbol.name);

        //     let Some(frame) = symbol.frames else {
        //         res.push_str(&str);
        //         continue;
        //     };
        //     let Some(frame) = frame.get(0) else {
        //         res.push_str(&str);
        //         continue;
        //     };
        //     let (Some(filepath), Some(line_number)) = (&frame.file_path, &frame.line_number) else {
        //         res.push_str(&str);
        //         continue;
        //     };
        //     str.push_str(&format!("  {:?}:{:?}\n", filepath.raw_path(), line_number));
        //     res.push_str(&str);
        // }
        // println!("res: {:?}", res);
        *addr_string = res;
    }

    let mut buf = BytesMut::new();
    trace.encode(&mut buf).unwrap();
    let mut file = File::create(output_path).unwrap();

    file.write_all(&buf).unwrap();
}

fn dwarf_load<'a>(object: &object::File<'a>) -> gimli::DwarfSections<Cow<'a, [u8]>> {
    let load_section = |id: gimli::SectionId| -> Result<Cow<'a, [u8]>, gimli::Error> {
        Ok(object
            .section_by_name(id.name())
            .map(|section| section.uncompressed_data().unwrap())
            .unwrap_or(Cow::Borrowed(&[][..])))
    };
    gimli::DwarfSections::load(&load_section).unwrap()
}

fn dwarf_borrow<'a>(
    dwarf: &'a gimli::DwarfSections<Cow<'_, [u8]>>,
) -> gimli::Dwarf<gimli::EndianSlice<'a, gimli::LittleEndian>> {
    let borrow_section: &dyn for<'b> Fn(
        &'b Cow<'_, [u8]>,
    ) -> gimli::EndianSlice<'b, gimli::LittleEndian> =
        &|section| gimli::EndianSlice::new(section, gimli::LittleEndian);
    dwarf.borrow(&borrow_section)
}

// fn get_object_file(target: &std::path::Path) -> object::File {
//     let file = File::open(target).unwrap();
//     let map = unsafe { memmap2::Mmap::map(&file).unwrap() };
//     object::File::parse(&*map).unwrap()
// }
