#![allow(dead_code)]

use rustdoc_types::{
    AssocItemConstraint, AssocItemConstraintKind, DynTrait, Enum, Function, FunctionPointer,
    FunctionSignature, GenericArg, GenericArgs, GenericBound, GenericParamDef, GenericParamDefKind,
    Generics, Impl, Item, ItemEnum, Path, PolyTrait, Static, Struct, StructKind, Term, Trait,
    TraitAlias, Type, TypeAlias, Union, Use, WherePredicate,
};

#[allow(unused_variables)]
pub trait Visitor {
    #[inline]
    fn visit_path(&mut self, path: &Path) {}

    #[inline]
    fn visit_use(&mut self, use_: &Use) {}
}

pub fn visit_item(item: &Item, v: &mut impl Visitor) {
    match &item.inner {
        ItemEnum::Function(fun) => visit_function(fun, v),
        ItemEnum::Struct(struct_) => visit_struct(struct_, v),
        ItemEnum::StructField(field_type) => visit_type(field_type, v),
        ItemEnum::AssocType {
            generics,
            bounds,
            type_,
        } => {
            visit_generics(generics, v);
            for bound in bounds {
                visit_generic_bound(bound, v);
            }
            if let Some(type_) = type_ {
                visit_type(type_, v);
            }
        }
        ItemEnum::AssocConst { type_, value: _ } => {
            visit_type(type_, v);
        }
        ItemEnum::Impl(impl_) => visit_impl(impl_, v),
        ItemEnum::TypeAlias(type_alias) => visit_type_alias(type_alias, v),
        ItemEnum::Union(union_) => visit_union(union_, v),
        ItemEnum::Enum(enum_) => visit_enum(enum_, v),

        ItemEnum::Trait(trait_) => visit_trait(trait_, v),
        ItemEnum::TraitAlias(trait_alias) => visit_trait_alias(trait_alias, v),
        ItemEnum::Constant { type_, const_: _ } => visit_type(type_, v),
        ItemEnum::Static(static_) => visit_static(static_, v),
        ItemEnum::Use(use_) => {
            v.visit_use(use_);
        }

        // ignore these because they don't contain anything of interest
        ItemEnum::Module(_) => {}
        ItemEnum::Variant(_) => {}
        ItemEnum::ExternCrate { .. } => {}
        ItemEnum::ExternType => todo!(),
        ItemEnum::Primitive(_) => {}
        ItemEnum::ProcMacro(_) => {}
        ItemEnum::Macro(_) => {}
    }
}

fn visit_static(static_: &Static, v: &mut impl Visitor) {
    let Static {
        is_mutable: _,
        is_unsafe: _,
        type_,
        expr: _,
    } = static_;
    visit_type(type_, v);
}

fn visit_trait_alias(trait_alias: &TraitAlias, v: &mut impl Visitor) {
    let TraitAlias { generics, params } = trait_alias;
    visit_generics(generics, v);
    for param in params {
        visit_generic_bound(param, v);
    }
}

fn visit_trait(trait_: &Trait, v: &mut impl Visitor) {
    let Trait {
        is_auto: _,
        is_unsafe: _,
        is_dyn_compatible: _,
        items: _,
        generics,
        bounds,
        implementations: _,
    } = trait_;
    visit_generics(generics, v);
    for bound in bounds {
        visit_generic_bound(bound, v);
    }
}

fn visit_enum(enum_: &Enum, v: &mut impl Visitor) {
    let Enum {
        generics,
        has_stripped_variants: _,
        variants: _,
        impls: _,
    } = enum_;
    visit_generics(generics, v);
}

fn visit_union(union: &Union, v: &mut impl Visitor) {
    let Union {
        generics,
        has_stripped_fields: _,
        fields: _,
        impls: _,
    } = union;
    visit_generics(generics, v);
}

fn visit_impl(impl_: &Impl, v: &mut impl Visitor) {
    let Impl {
        is_unsafe: _,
        is_negative: _,
        is_synthetic: _,
        generics,
        provided_trait_methods: _,
        trait_,
        for_,
        items: _,
        blanket_impl,
    } = impl_;
    // blanket impls in other crates that happen to match one of our types shouldn't count
    if blanket_impl.is_some() {
        return;
    }
    visit_generics(generics, v);
    if let Some(trait_) = trait_ {
        visit_path(trait_, v);
    }
    visit_type(for_, v);
}

fn visit_type_alias(type_alias: &TypeAlias, v: &mut impl Visitor) {
    let TypeAlias { type_, generics } = type_alias;
    visit_type(type_, v);
    visit_generics(generics, v);
}

fn visit_struct(struct_: &Struct, v: &mut impl Visitor) {
    let Struct {
        kind,
        generics,
        impls: _,
    } = struct_;
    visit_struct_kind(kind, v);
    visit_generics(generics, v);
}

fn visit_struct_kind(kind: &StructKind, _v: &mut impl Visitor) {
    match kind {
        StructKind::Unit => {}
        StructKind::Tuple(_) => {}
        StructKind::Plain {
            fields: _,
            has_stripped_fields: _,
        } => {}
    }
}

fn visit_function(fun: &Function, v: &mut impl Visitor) {
    let Function {
        sig,
        generics,
        header: _,
        has_body: _,
    } = fun;
    visit_fn_sig(sig, v);
    visit_generics(generics, v);
}

fn visit_fn_sig(decl: &FunctionSignature, v: &mut impl Visitor) {
    let FunctionSignature {
        is_c_variadic: _,
        inputs,
        output,
    } = decl;
    for (_, ty) in inputs {
        visit_type(ty, v);
    }
    if let Some(output) = output {
        visit_type(output, v);
    }
}

fn visit_generics(generics: &Generics, v: &mut impl Visitor) {
    let Generics {
        params,
        where_predicates,
    } = generics;
    for param in params {
        visit_generic_param_def(param, v);
    }
    for where_predicate in where_predicates {
        visit_where_predicate(where_predicate, v);
    }
}

fn visit_generic_param_def(param: &GenericParamDef, v: &mut impl Visitor) {
    let GenericParamDef { name: _, kind } = param;
    visit_generic_param_def_kind(kind, v);
}

fn visit_where_predicate(where_predicate: &WherePredicate, v: &mut impl Visitor) {
    match where_predicate {
        WherePredicate::BoundPredicate {
            type_,
            bounds,
            generic_params,
        } => {
            visit_type(type_, v);
            for bound in bounds {
                visit_generic_bound(bound, v);
            }
            for generic_param in generic_params {
                visit_generic_param_def(generic_param, v);
            }
        }
        WherePredicate::EqPredicate { lhs, rhs } => {
            visit_type(lhs, v);
            visit_term(rhs, v);
        }
        // lifetime predicates can only have outlives bounds, ignore
        WherePredicate::LifetimePredicate { .. } => {}
    }
}

fn visit_generic_param_def_kind(kind: &GenericParamDefKind, v: &mut impl Visitor) {
    match kind {
        GenericParamDefKind::Lifetime { outlives: _ } => {}
        GenericParamDefKind::Type {
            is_synthetic: _,
            bounds,
            default,
        } => {
            for bound in bounds {
                visit_generic_bound(bound, v);
            }
            if let Some(default) = default {
                visit_type(default, v);
            }
        }
        GenericParamDefKind::Const { type_, default: _ } => {
            visit_type(type_, v);
        }
    }
}

fn visit_generic_bound(bound: &GenericBound, v: &mut impl Visitor) {
    match bound {
        GenericBound::TraitBound {
            trait_,
            generic_params,
            modifier: _,
        } => {
            visit_path(trait_, v);
            for param in generic_params {
                visit_generic_param_def(param, v);
            }
        }
        GenericBound::Outlives(_) => {}
        GenericBound::Use(_) => {}
    }
}

fn visit_term(term: &Term, v: &mut impl Visitor) {
    match term {
        Term::Type(type_) => visit_type(type_, v),
        Term::Constant(_) => {}
    }
}

fn visit_path(path: &Path, v: &mut impl Visitor) {
    v.visit_path(path);
    let Path {
        path: _,
        id: _,
        args,
    } = path;
    if let Some(args) = args {
        visit_generic_args(args, v);
    }
}

fn visit_generic_args(args: &GenericArgs, v: &mut impl Visitor) {
    match args {
        GenericArgs::AngleBracketed { args, constraints } => {
            for arg in args {
                visit_generic_arg(arg, v);
            }
            for constraint in constraints {
                visit_assoc_item_constraint(constraint, v);
            }
        }
        GenericArgs::Parenthesized { inputs, output } => {
            for type_ in inputs {
                visit_type(type_, v);
            }
            if let Some(type_) = output {
                visit_type(type_, v);
            }
        }
        GenericArgs::ReturnTypeNotation => {}
    }
}

fn visit_assoc_item_constraint(binding: &AssocItemConstraint, v: &mut impl Visitor) {
    let AssocItemConstraint {
        name: _,
        args,
        binding,
    } = binding;
    if let Some(args) = args {
        visit_generic_args(args, v);
    }
    visit_assoc_item_constraint_kind(binding, v);
}

fn visit_assoc_item_constraint_kind(binding: &AssocItemConstraintKind, v: &mut impl Visitor) {
    match binding {
        AssocItemConstraintKind::Equality(term) => visit_term(term, v),
        AssocItemConstraintKind::Constraint(bounds) => {
            for bound in bounds {
                visit_generic_bound(bound, v)
            }
        }
    }
}

fn visit_generic_arg(arg: &GenericArg, v: &mut impl Visitor) {
    match arg {
        GenericArg::Lifetime(_) => {}
        GenericArg::Type(type_) => visit_type(type_, v),
        GenericArg::Const(_) => {}
        GenericArg::Infer => {}
    }
}

fn visit_type(type_: &Type, v: &mut impl Visitor) {
    match type_ {
        Type::ResolvedPath(path) => visit_path(path, v),
        Type::DynTrait(dyn_trait) => visit_dyn_trait(dyn_trait, v),
        Type::Generic(_) => {}
        Type::Primitive(_) => {}
        Type::FunctionPointer(fn_pointer) => visit_function_pointer(fn_pointer, v),
        Type::Tuple(types) => {
            for type_ in types {
                visit_type(type_, v);
            }
        }
        Type::Slice(type_) => visit_type(type_, v),
        Type::Array { type_, len: _ } => visit_type(type_, v),
        Type::ImplTrait(bounds) => {
            for bound in bounds {
                visit_generic_bound(bound, v);
            }
        }
        Type::Infer => {}
        Type::RawPointer {
            is_mutable: _,
            type_,
        } => visit_type(type_, v),
        Type::BorrowedRef {
            is_mutable: _,
            lifetime: _,
            type_,
        } => visit_type(type_, v),
        Type::QualifiedPath {
            name: _,
            args,
            self_type,
            trait_,
        } => {
            if let Some(args) = args {
                visit_generic_args(args, v);
            }
            visit_type(self_type, v);
            if let Some(trait_) = trait_ {
                visit_path(trait_, v);
            }
        }
        Type::Pat { type_, .. } => {
            visit_type(type_, v);
        }
    }
}

fn visit_function_pointer(fn_pointer: &FunctionPointer, v: &mut impl Visitor) {
    let FunctionPointer {
        sig,
        generic_params,
        header: _,
    } = fn_pointer;
    visit_fn_sig(sig, v);
    for generic_param in generic_params {
        visit_generic_param_def(generic_param, v);
    }
}

fn visit_dyn_trait(dyn_trait: &DynTrait, v: &mut impl Visitor) {
    let DynTrait {
        traits,
        lifetime: _,
    } = dyn_trait;
    for trait_ in traits {
        visit_poly_trait(trait_, v);
    }
}

fn visit_poly_trait(trait_: &PolyTrait, v: &mut impl Visitor) {
    let PolyTrait {
        trait_,
        generic_params,
    } = trait_;
    visit_path(trait_, v);
    for generic_param in generic_params {
        visit_generic_param_def(generic_param, v);
    }
}
