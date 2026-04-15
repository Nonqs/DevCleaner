use std::collections::HashSet;

use swc_common::GLOBALS;
use swc_ecma_ast::Ident;
use swc_ecma_ast::{ImportDecl, ImportSpecifier, Module};
use swc_ecma_visit::{Visit, VisitWith};

pub fn find_unused_imports(module: &Module) -> HashSet<String> {
    let mut inspector = ImportsIns {
        imp: HashSet::new(),
        cont: HashSet::new(),
        in_import: false,
    };

    GLOBALS.set(&Default::default(), || {
        module.visit_with(&mut inspector);
    });

    inspector.imp.difference(&inspector.cont).cloned().collect()
}

struct ImportsIns {
    imp: HashSet<String>,
    cont: HashSet<String>,
    in_import: bool,
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
