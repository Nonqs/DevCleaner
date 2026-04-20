use std::collections::HashSet;

use swc_common::BytePos;
use swc_ecma_ast::{ImportDecl, ImportSpecifier, Module};
use swc_ecma_visit::{VisitMut, VisitMutWith};

use crate::analyzer::ImportBinding;
use crate::types::TextEdit;

fn byte_pos_to_offset(pos: BytePos, file_start: BytePos) -> Option<usize> {
    let pos = pos.0 as usize;
    let file_start = file_start.0 as usize;

    pos.checked_sub(file_start)
}

fn span_to_offsets(lo: BytePos, hi: BytePos, file_start: BytePos) -> Option<(usize, usize)> {
    let start = byte_pos_to_offset(lo, file_start)?;
    let end = byte_pos_to_offset(hi, file_start)?;

    if start > end {
        return None;
    }

    Some((start, end))
}

pub fn build_text_edits(
    unused: &[ImportBinding],
    source: &str,
    file_start: BytePos,
) -> Vec<TextEdit> {
    let mut edits: Vec<TextEdit> = unused
        .iter()
        .filter_map(|binding| {
            let (decl_start, decl_end) = span_to_offsets(
                binding.import_decl_span.lo,
                binding.import_decl_span.hi,
                file_start,
            )?;
            let (spec_start, spec_end) = span_to_offsets(
                binding.specifier_span.lo,
                binding.specifier_span.hi,
                file_start,
            )?;

            if decl_end > source.len() || spec_end > source.len() {
                return None;
            }

            let (start, end) = if let Some(decl_src) = source.get(decl_start..decl_end) {
                if let Some(local_idx) = decl_src.find(&binding.local_name) {
                    let start = decl_start + local_idx;
                    let end = start + binding.local_name.len();
                    (start, end)
                } else {
                    (spec_start, spec_end)
                }
            } else {
                (spec_start, spec_end)
            };

            Some(TextEdit {
                start,
                end,
                replacement: String::new(),
            })
        })
        .collect();

    edits.sort_by(|a, b| b.start.cmp(&a.start));
    edits
}

pub fn apply_text_edits(source: &str, edits: &[TextEdit]) -> String {
    let mut output = source.to_string();

    for edit in edits {
        if edit.start > edit.end || edit.end > output.len() {
            continue;
        }

        output.replace_range(edit.start..edit.end, &edit.replacement);
    }

    output
}

pub fn remove_unused_imports(module: &mut Module, unused: Vec<ImportBinding>) {
    let imp = unused
        .into_iter()
        .map(|binding| binding.local_name)
        .collect();

    let mut deleter = ImportsToDel { imp };

    module.visit_mut_with(&mut deleter);

    module.body.retain(|item| {
        if let swc_ecma_ast::ModuleItem::ModuleDecl(swc_ecma_ast::ModuleDecl::Import(i)) = item {
            !i.specifiers.is_empty()
        } else {
            true
        }
    });
}

struct ImportsToDel {
    imp: HashSet<String>,
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
