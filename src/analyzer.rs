use std::collections::HashSet;

use swc_common::{GLOBALS, Span, Spanned};
use swc_ecma_ast::Ident;
use swc_ecma_ast::{ImportDecl, ImportSpecifier, Module};
use swc_ecma_visit::{Visit, VisitWith};

pub fn find_unused_imports(module: &Module) -> Vec<ImportBinding> {
    let mut inspector = ImportsIns {
        used_names: HashSet::new(),
        in_import: false,
        imports: Vec::new(),
    };

    GLOBALS.set(&Default::default(), || {
        module.visit_with(&mut inspector);
    });

    let used_names = inspector.used_names;

    inspector
        .imports
        .into_iter()
        .filter(|binding| !used_names.contains(&binding.local_name))
        .collect()
}

pub struct ImportBinding {
    pub local_name: String,
    pub specifier_span: Span,
    pub import_decl_span: Span,
}

struct ImportsIns {
    used_names: HashSet<String>,
    in_import: bool,
    imports: Vec<ImportBinding>,
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

            let n = ImportBinding {
                local_name: local.clone(),
                specifier_span: esp.span(),
                import_decl_span: node.span,
            };

            self.imports.push(n);
        }

        node.visit_children_with(self);

        self.in_import = false;
    }

    fn visit_ident(&mut self, n: &Ident) {
        let data = n.sym.to_string();
        if !self.in_import {
            self.used_names.insert(data);
        }
    }
}
