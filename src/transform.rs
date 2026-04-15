use std::collections::HashSet;

use swc_ecma_ast::{ImportDecl, ImportSpecifier, Module};
use swc_ecma_visit::{VisitMut, VisitMutWith};

pub fn remove_unused_imports(module: &mut Module, unused: HashSet<String>) {
    let mut deleter = ImportsToDel { imp: unused };

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
