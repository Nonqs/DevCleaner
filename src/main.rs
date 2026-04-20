use rayon::prelude::*;
use std::env;
use std::path::Path;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use swc_common::{source_map::SourceMap, sync::Lrc};
use swc_ecma_parser::{Parser, StringInput, lexer::Lexer};

mod analyzer;
mod transform;
mod types;
mod walker;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let root = args
        .iter()
        .find(|a| !a.starts_with("--"))
        .cloned()
        .unwrap_or_else(|| ".".to_string());
    let root_path = PathBuf::from(&root);
    let write_to_dist = args.iter().any(|a| a == "--dist") || is_sample_root(&root_path);

    let files = walker::collect_files(&root_path);

    println!(
        "Found {} files in {} ({})",
        files.len(),
        root_path.display(),
        if write_to_dist {
            "dist mode"
        } else {
            "in-place mode"
        }
    );

    let read_errors = AtomicUsize::new(0);
    let parse_errors = AtomicUsize::new(0);
    let write_errors = AtomicUsize::new(0);

    files.par_iter().for_each(|file| {
        let cm = Lrc::new(SourceMap::new(swc_common::FilePathMapping::empty()));

        let syntax = walker::get_syntax(&file);
        let fm = match cm.load_file(&file.path) {
            Ok(fm) => fm,
            Err(_) => {
                read_errors.fetch_add(1, Ordering::Relaxed);
                return;
            }
        };

        let lexer = Lexer::new(syntax, Default::default(), StringInput::from(&*fm), None);

        let mut parser = Parser::new_from(lexer);
        let module = match parser.parse_module() {
            Ok(module) => module,
            Err(_) => {
                parse_errors.fetch_add(1, Ordering::Relaxed);
                return;
            }
        };

        let source = fm.src.to_string();
        let unused = analyzer::find_unused_imports(&module);
        let edits = transform::build_text_edits(&unused, &source, fm.start_pos);
        let code = transform::apply_text_edits(&source, &edits);

        if !write_to_dist && code == source {
            return;
        }

        let write_result = if write_to_dist {
            let mut out_path = PathBuf::from("dist");
            out_path.push(&file.relative_path);
            std::fs::create_dir_all(out_path.parent().unwrap()).ok();
            std::fs::write(out_path, code)
        } else {
            std::fs::write(&file.path, code)
        };

        if write_result.is_err() {
            write_errors.fetch_add(1, Ordering::Relaxed);
        }
    });

    let total_errors = read_errors.load(Ordering::Relaxed)
        + parse_errors.load(Ordering::Relaxed)
        + write_errors.load(Ordering::Relaxed);

    if total_errors > 0 {
        println!(
            "Finished with skips/errors: read={} parse={} write={}",
            read_errors.load(Ordering::Relaxed),
            parse_errors.load(Ordering::Relaxed),
            write_errors.load(Ordering::Relaxed)
        );
    }
}

fn is_sample_root(path: &Path) -> bool {
    path.file_name().and_then(|n| n.to_str()) == Some("sample")
}
