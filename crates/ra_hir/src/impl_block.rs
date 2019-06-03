use ra_syntax::{
    TreeArc, SmolStr,
    ast
};

use crate::{
    Const, TypeAlias, Function, HirFileId, AstDatabase,
    HirDatabase, DefDatabase, TraitRef,
    type_ref::TypeRef,
    ids::{ImplId, AstItemDef},
    resolve::Resolver,
    ty::Ty,
    generics::HasGenericParams,
    code_model::Module
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImplBlock {
    pub(crate) impl_id: ImplId,
}

impl ImplBlock {
    /// Returns the syntax of the impl block
    pub fn source(
        &self,
        db: &(impl DefDatabase + AstDatabase),
    ) -> (HirFileId, TreeArc<ast::ImplBlock>) {
        self.impl_id.source(db)
    }

    pub fn id(&self) -> ImplId {
        self.impl_id
    }

    pub fn module(&self, db: &impl DefDatabase) -> Module {
        db.lookup_intern_impl(self.impl_id).module
    }

    pub fn lang_item(&self, db: &impl DefDatabase) -> Option<SmolStr> {
        self.with_data(db, |data| data.lang_item.clone())
    }

    pub fn target_trait(&self, db: &impl DefDatabase) -> Option<TypeRef> {
        self.with_data(db, |data| data.target_trait.clone())
    }

    pub fn target_type(&self, db: &impl DefDatabase) -> TypeRef {
        self.with_data(db, |data| data.target_type.clone())
    }

    pub fn target_ty(&self, db: &impl HirDatabase) -> Ty {
        Ty::from_hir(db, &self.resolver(db), &self.target_type(db))
    }

    pub fn target_trait_ref(&self, db: &impl HirDatabase) -> Option<TraitRef> {
        let target_ty = self.target_ty(db);
        TraitRef::from_hir(db, &self.resolver(db), &self.target_trait(db)?, Some(target_ty))
    }

    pub fn items(&self, db: &impl DefDatabase) -> Vec<ImplItem> {
        self.with_data(db, |data| data.items.clone())
    }

    pub fn is_negative(&self, db: &impl DefDatabase) -> bool {
        self.with_data(db, |data| data.negative)
    }

    pub(crate) fn resolver(&self, db: &impl DefDatabase) -> Resolver {
        let r = self.module(db).resolver(db);
        // add generic params, if present
        let p = self.generic_params(db);
        let r = if !p.params.is_empty() { r.push_generic_params_scope(p) } else { r };
        let r = r.push_impl_block_scope(self.clone());
        r
    }

    fn with_data<F: FnOnce(&ImplData) -> T, T>(&self, db: &impl DefDatabase, f: F) -> T {
        let module = self.module(db);
        let def_map = db.crate_def_map(module.krate);
        f(&def_map[module.module_id].impls[&self])
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImplData {
    pub(super) lang_item: Option<SmolStr>,
    pub(crate) target_trait: Option<TypeRef>,
    pub(crate) target_type: TypeRef,
    pub(crate) items: Vec<ImplItem>,
    pub(crate) negative: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
//FIXME: rename to ImplDef?
pub enum ImplItem {
    Method(Function),
    Const(Const),
    TypeAlias(TypeAlias),
    // Existential
}
impl_froms!(ImplItem: Const, TypeAlias);

impl From<Function> for ImplItem {
    fn from(func: Function) -> ImplItem {
        ImplItem::Method(func)
    }
}
