use std::collections::HashSet;
use std::env;
use std::path::Path;
use std::path::PathBuf;
use swc_common::{GLOBALS, source_map::SourceMap, sync::Lrc};
use swc_ecma_ast::Ident;
use swc_ecma_ast::{ImportDecl, ImportSpecifier};
use swc_ecma_codegen::text_writer::JsWriter;
use swc_ecma_codegen::{Config, Emitter};
use swc_ecma_parser::{EsSyntax, Parser, StringInput, Syntax, TsSyntax, lexer::Lexer};
use swc_ecma_visit::Visit;
use swc_ecma_visit::VisitMut;
use swc_ecma_visit::VisitMutWith;
use swc_ecma_visit::VisitWith;
use walkdir::WalkDir;

struct FileType {
    path: PathBuf,
    relative_path: PathBuf,
    is_ts: bool,
}

struct ImportsIns {
    imp: HashSet<String>,
    cont: HashSet<String>,
    in_import: bool,
}

struct ImportsToDel {
    imp: HashSet<String>,
}

impl Visit for ImportsIns {
    fn visit_import_decl(&mut self, node: &ImportDecl) {
        self.in_import = true;

        for esp in &node.specifiers {
            let local = match esp {
                ImportSpecifier::Named(s) => s.local.sym.to_string(),
                ImportSpecifier::Default(s) => s.local.sym.to_string(),
                ImportSpecifier::Namespace(s) => s.local.sym.to_string(),
            };

            self.imp.insert(local);
        }

        node.visit_children_with(self);

        self.in_import = false;
    }

    fn visit_ident(&mut self, n: &Ident) {
        let data = n.sym.to_string();
        if self.imp.contains(&data) && !self.in_import {
            self.cont.insert(data);
        }
    }
}

impl VisitMut for ImportsToDel {
    fn visit_mut_import_decl(&mut self, node: &mut ImportDecl) {
        node.specifiers.retain(|specifier| {
            let local = match specifier {
                ImportSpecifier::Named(s) => s.local.sym.to_string(),
                ImportSpecifier::Default(s) => s.local.sym.to_string(),
                ImportSpecifier::Namespace(s) => s.local.sym.to_string(),
            };

            !self.imp.contains(&local)
        });
    }
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let root = args
        .iter()
        .find(|a| !a.starts_with("--"))
        .cloned()
        .unwrap_or_else(|| ".".to_string());
    let root_path = PathBuf::from(&root);
    let write_to_dist = args.iter().any(|a| a == "--dist") || is_sample_root(&root_path);

    let mut files: Vec<FileType> = Vec::new();
    let cm = Lrc::new(SourceMap::new(swc_common::FilePathMapping::empty()));

    for entry in WalkDir::new(&root_path)
        .into_iter()
        .filter_entry(|e| !should_skip_dir(e.path()))
        .filter_map(|e| e.ok())
    {
        match entry
            .path()
            .extension()
            .and_then(|s: &std::ffi::OsStr| s.to_str())
        {
            Some("ts") | Some("tsx") => {
                let relative_path = entry
                    .path()
                    .strip_prefix(&root_path)
                    .unwrap_or(entry.path())
                    .to_path_buf();

                let f = FileType {
                    path: entry.path().to_path_buf(),
                    relative_path,
                    is_ts: true,
                };
                files.push(f);
            }
            Some("js") | Some("jsx") => {
                let relative_path = entry
                    .path()
                    .strip_prefix(&root_path)
                    .unwrap_or(entry.path())
                    .to_path_buf();

                let f = FileType {
                    path: entry.path().to_path_buf(),
                    relative_path,
                    is_ts: false,
                };
                files.push(f);
            }
            _ => {}
        }
    }

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

    for file in files {
        let syntax: Syntax = get_syntax(&file);
        let fm = cm.load_file(&file.path).expect("Error");
        let lexer = Lexer::new(syntax, Default::default(), StringInput::from(&*fm), None);

        let mut parser = Parser::new_from(lexer);
        let mut module = parser.parse_module().expect("Error 2");

        let mut inspector = ImportsIns {
            imp: HashSet::new(),
            cont: HashSet::new(),
            in_import: false,
        };

        GLOBALS.set(&Default::default(), || {
            module.visit_with(&mut inspector);
        });

        let unused: HashSet<String> = inspector.imp.difference(&inspector.cont).cloned().collect();
        let mut deleter = ImportsToDel { imp: unused };

        module.visit_mut_with(&mut deleter);

        module.body.retain(|item| {
            if let swc_ecma_ast::ModuleItem::ModuleDecl(swc_ecma_ast::ModuleDecl::Import(i)) = item
            {
                !i.specifiers.is_empty()
            } else {
                true
            }
        });

        let mut buf = Vec::new();
        {
            let mut emitter = Emitter {
                cfg: Config::default(),
                cm: cm.clone(),
                comments: None,
                wr: JsWriter::new(cm.clone(), "\n", &mut buf, None),
            };

            emitter.emit_module(&module).unwrap();
        }

        let code = String::from_utf8(buf).expect("error converting to UTF-8");

        if write_to_dist {
            let mut out_path = PathBuf::from("dist");
            out_path.push(&file.relative_path);
            std::fs::create_dir_all(out_path.parent().unwrap()).ok();
            std::fs::write(out_path, code).expect("Rrror writting file");
        } else {
            std::fs::write(&file.path, code).expect("Rrror writting file");
        }
    }
}

fn is_sample_root(path: &Path) -> bool {
    path.file_name().and_then(|n| n.to_str()) == Some("sample")
}

fn should_skip_dir(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
        return false;
    };

    matches!(name, "node_modules" | ".git" | "target" | "dist")
}

fn get_syntax(file: &FileType) -> Syntax {
    if file.is_ts {
        Syntax::Typescript(TsSyntax {
            tsx: true,
            decorators: true,
            ..Default::default()
        })
    } else {
        Syntax::Es(EsSyntax {
            jsx: true,
            ..Default::default()
        })
    }
}
