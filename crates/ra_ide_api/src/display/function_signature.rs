use std::fmt::{self, Display};

use join_to_string::join;
use ra_syntax::ast::{self, AstNode, NameOwner, VisibilityOwner};
use std::convert::From;
use hir::{Docs, Documentation};

use crate::{db, display::{where_predicates, generic_parameters}};

/// Contains information about a function signature
#[derive(Debug)]
pub struct FunctionSignature {
    /// Optional visibility
    pub visibility: Option<String>,
    /// Name of the function
    pub name: Option<String>,
    /// Documentation for the function
    pub doc: Option<Documentation>,
    /// Parameters of the function
    pub parameters: Vec<String>,
    /// Optional return type
    pub ret_type: Option<String>,
    /// Generic parameters
    pub generic_parameters: Vec<String>,
    /// Where predicates
    pub where_predicates: Vec<String>,
}

impl FunctionSignature {
    pub(crate) fn from_hir(db: &db::RootDatabase, function: hir::Function) -> Self {
        // NOTE: we don't touch AST here to avoiding parsing
        let data = function.data(db);
        let doc = function.docs(db);
        let ret_type = match data.ret_type() {
            hir::TypeRef::Tuple(xs) if xs.len() == 0 => None,
            t => Some(t.to_string()),
        };
        let parameters = if data.has_self_param() {
            let self_param = match data.params()[0] {
                hir::TypeRef::Reference(_, m) => format!("&{}self", m.as_keyword_for_ref()),
                _ => "self".to_string(),
            };
            let mut params = vec![self_param];
            params.extend(data.params().iter().skip(1).map(|it| it.to_string()));
            params
        } else {
            data.params().iter().map(|it| it.to_string()).collect()
        };
        FunctionSignature {
            visibility: data.visibility().map(|it| it.to_string()),
            name: Some(data.name().to_string()),
            doc,
            parameters,
            ret_type,
            // FIXME: fill this as well
            generic_parameters: vec![],
            where_predicates: vec![],
        }
    }
}

impl From<&'_ ast::FnDef> for FunctionSignature {
    fn from(node: &ast::FnDef) -> FunctionSignature {
        fn param_list(node: &ast::FnDef) -> Vec<String> {
            let mut res = vec![];
            if let Some(param_list) = node.param_list() {
                if let Some(self_param) = param_list.self_param() {
                    res.push(self_param.syntax().text().to_string())
                }

                res.extend(param_list.params().map(|param| param.syntax().text().to_string()));
            }
            res
        }

        FunctionSignature {
            visibility: node.visibility().map(|n| n.syntax().text().to_string()),
            name: node.name().map(|n| n.text().to_string()),
            ret_type: node
                .ret_type()
                .and_then(|r| r.type_ref())
                .map(|n| n.syntax().text().to_string()),
            parameters: param_list(node),
            // docs are processed separately
            doc: None,
            generic_parameters: generic_parameters(node),
            where_predicates: where_predicates(node),
        }
    }
}

impl Display for FunctionSignature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(t) = &self.visibility {
            write!(f, "{} ", t)?;
        }

        if let Some(name) = &self.name {
            write!(f, "fn {}", name)?;
        }

        if !self.generic_parameters.is_empty() {
            join(self.generic_parameters.iter())
                .separator(", ")
                .surround_with("<", ">")
                .to_fmt(f)?;
        }

        join(self.parameters.iter()).separator(", ").surround_with("(", ")").to_fmt(f)?;

        if let Some(t) = &self.ret_type {
            write!(f, " -> {}", t)?;
        }

        if !self.where_predicates.is_empty() {
            write!(f, "\nwhere ")?;
            join(self.where_predicates.iter()).separator(",\n      ").to_fmt(f)?;
        }

        Ok(())
    }
}
