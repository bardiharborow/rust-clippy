use super::TRANSMUTE_INT_TO_NON_ZERO;
use clippy_utils::diagnostics::span_lint_and_then;
use clippy_utils::msrvs::{self, Msrv};
use clippy_utils::std_or_core;
use clippy_utils::sugg::Sugg;
use rustc_errors::Applicability;
use rustc_hir::def::{DefKind, Res};
use rustc_hir::{Expr, Item, ItemKind, Node, Path, StmtKind, UseKind, UseTree};
use rustc_lint::LateContext;
use rustc_middle::ty::{self, Ty};
use rustc_span::symbol::sym;

/// Checks for `transmute_int_to_non_zero` lint.
/// Returns `true` if it's triggered, otherwise returns `false`.
pub(super) fn check<'tcx>(
    cx: &LateContext<'tcx>,
    e: &'tcx Expr<'_>,
    from_ty: Ty<'tcx>,
    to_ty: Ty<'tcx>,
    arg: &'tcx Expr<'_>,
    path: &Path<'_>,
    msrv: Msrv,
) -> bool {
    let int_name = match from_ty.kind() {
        ty::Int(int_ty) => int_ty.name_str(),
        ty::Uint(uint_ty) => uint_ty.name_str(),
        _ => return false,
    };
    if let ty::Adt(adt, substs) = to_ty.kind()
        && cx.tcx.is_diagnostic_item(sym::NonZero, adt.did())
        && let int_ty = substs.type_at(0)
        && from_ty == int_ty
    {
        // Before the generic `NonZero<T>` was stabilized, only the `NonZeroU32` etc. types existed.
        let generic = msrv.meets(cx, msrvs::GENERIC_NON_ZERO);
        let non_zero_name = if generic {
            sym::NonZero.to_string()
        } else {
            let (first, rest) = int_name.split_at(1);
            format!("{}{}{rest}", sym::NonZero, first.to_uppercase())
        };
        span_lint_and_then(
            cx,
            TRANSMUTE_INT_TO_NON_ZERO,
            e.span,
            format!("transmute from a `{from_ty}` to a `{}<{int_ty}>`", sym::NonZero),
            |diag| {
                // A suggestion inside a macro body would also change every other expansion of the macro
                if !e.span.from_expansion()
                    && let Some(non_zero_path) = type_path(cx, e, to_ty, &non_zero_name)
                {
                    let mut app = Applicability::Unspecified;
                    let arg = Sugg::hir_with_context(cx, arg, e.span.ctxt(), "..", &mut app);
                    // The types given to `transmute` explicitly may be all that determines the integer type, so
                    // keep them for the generic `NonZero::new_unchecked`.
                    let turbofish = if generic && path.segments.last().is_some_and(|seg| seg.args.is_some()) {
                        format!("::<{int_ty}>")
                    } else {
                        String::new()
                    };
                    diag.span_suggestion(
                        e.span,
                        "consider using",
                        format!("{non_zero_path}{turbofish}::{}({arg})", sym::new_unchecked),
                        app,
                    );
                } else {
                    diag.help(format!("consider using `{non_zero_name}::{}`", sym::new_unchecked));
                }
            },
        );
        true
    } else {
        false
    }
}

/// Returns how to name the type `name`, which refers to `to_ty`: just `name` if it is in scope in
/// the enclosing module, or its absolute path through `std` or `core` otherwise.
fn type_path<'tcx>(cx: &LateContext<'tcx>, e: &Expr<'_>, to_ty: Ty<'tcx>, name: &str) -> Option<String> {
    let module = cx.tcx.parent_module(e.hir_id).to_local_def_id();
    let in_scope = cx.tcx.module_children_local(module).iter().any(|child| {
        child.ident.name.as_str() == name
            // A name introduced hygienically by a macro is not visible to `e`
            && child.ident.span.normalize_to_macros_2_0().ctxt().is_root()
            && match child.res {
                Res::Def(DefKind::Struct, def_id) => to_ty.ty_adt_def().is_some_and(|adt| adt.did() == def_id),
                Res::Def(DefKind::TyAlias, def_id) => {
                    cx.tcx.type_of(def_id).instantiate_identity().skip_norm_wip() == to_ty
                },
                _ => false,
            }
    });
    if in_scope && !is_shadowed_locally(cx, e, name) {
        Some(name.to_owned())
    } else {
        std_or_core(cx).map(|std_or_core| format!("::{std_or_core}::num::{name}"))
    }
}

/// Checks whether an item or import in a block around `e`, or a generic parameter of an item around
/// `e`, might bind `name`, which would shadow what the enclosing module binds it to.
fn is_shadowed_locally(cx: &LateContext<'_>, e: &Expr<'_>, name: &str) -> bool {
    cx.tcx
        .hir_parent_iter(e.hir_id)
        .take_while(|(_, node)| {
            !matches!(
                node,
                Node::Item(Item {
                    kind: ItemKind::Mod(..),
                    ..
                })
            )
        })
        .any(|(_, node)| {
            node.generics().is_some_and(|generics| {
                generics
                    .params
                    .iter()
                    .any(|param| param.name.ident().name.as_str() == name)
            }) || matches!(node, Node::Block(block) if block.stmts.iter().any(|stmt| {
                matches!(stmt.kind, StmtKind::Item(item_id) if item_binds(cx, cx.tcx.hir_item(item_id), name))
            }))
        })
}

fn item_binds(cx: &LateContext<'_>, item: &Item<'_>, name: &str) -> bool {
    match item.kind {
        ItemKind::Use(ref tree) => use_binds(cx, tree, name),
        _ => item.kind.ident().is_some_and(|ident| ident.name.as_str() == name),
    }
}

fn use_binds(cx: &LateContext<'_>, tree: &UseTree<'_>, name: &str) -> bool {
    match tree.kind {
        UseKind::Single(ident) => ident.name.as_str() == name,
        // Assume the worst if the glob's target can't be inspected
        UseKind::Glob => tree
            .prefix
            .res
            .type_ns
            .and_then(|res| res.opt_def_id())
            .is_none_or(|def_id| {
                let children = match def_id.as_local() {
                    Some(def_id) => cx.tcx.module_children_local(def_id),
                    None => cx.tcx.module_children(def_id),
                };
                children.iter().any(|child| child.ident.name.as_str() == name)
            }),
        UseKind::Nested { items } => items.iter().any(|(tree, ..)| use_binds(cx, tree, name)),
    }
}
