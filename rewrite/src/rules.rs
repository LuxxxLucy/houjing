use std::{fmt::Debug, hash::Hash, mem::discriminant};

use indexmap::{IndexMap, IndexSet};
use itertools::Itertools;
use log::warn;

use egg::{rewrite as rw, *};

use crate::{
    base::list_op::{Partitioning, Permutation},
    base::num::{num, Num},
    cad::{Cad, EGraph, MetaAnalysis, Rewrite, Vec3},
    hyperparameters::{
        AFFINE_SIGNATURE_MAX_LEN, CAD_IDENTS, INV_TRANS, PARTITIONING, PARTITIONING_MAX,
        STRUCTURE_MATCH_LIMIT,
    },
};

fn is_not_zero(var: &'static str) -> impl Fn(&mut EGraph, Id, &Subst) -> bool {
    let var = var.parse().unwrap();
    let zero = Cad::Num(num(0.0));
    move |egraph, _, subst| !egraph[subst[var]].nodes.contains(&zero)
}

fn is_eq(v1: &'static str, v2: &'static str) -> ConditionEqual<Cad> {
    let p1: Pattern<Cad> = v1.parse().unwrap();
    let p2: Pattern<Cad> = v2.parse().unwrap();
    ConditionEqual::new(p1, p2)
}

fn is_pos(vars: &[&'static str]) -> impl Fn(&mut EGraph, Id, &Subst) -> bool {
    let vars: Vec<Var> = vars.iter().map(|v| v.parse().unwrap()).collect();
    move |egraph, _, subst| {
        vars.iter().all(|v| {
            egraph[subst[*v]].nodes.iter().all(|n| {
                if let Cad::Num(num) = n {
                    num.to_f64() > 0.0
                } else {
                    true
                }
            })
        })
    }
}

#[rustfmt::skip]
pub fn pre_rules() -> Vec<Rewrite> {
    vec![
        rw!("union_comm"; "(Binop Union ?a ?b)" => "(Binop Union ?b ?a)"),
        rw!("inter_comm"; "(Binop Inter ?a ?b)" => "(Binop Inter ?b ?a)"),
        rw!("binary_op_fold"; "(Binop ?bop ?a ?b)" => "(Fold ?bop (List ?a ?b))"),
        rw!("fold_cons"; "(Binop ?bop ?a (Fold ?bop ?list))" => "(Fold ?bop (Cons ?a ?list))"),

        rw!(
            "flatten_union";
            "(Fold Union ?list)" => {
                let list = "?list".parse().unwrap();
                let op = Cad::Union;
                Flatten { list, op }
            }
        ),
    ]
}

pub fn math_rules() -> Vec<Rewrite> {
    // math rules
    vec![
        // add
        rw!("add_comm"; "(+ ?a ?b)" => "(+ ?b ?a)"),
        rw!("add_zero"; "(+ 0 ?a)" => "?a"),

        // sub
        rw!("sub_zero"; "(- ?a 0)" => "?a"),

        // mul
        rw!("mul_zero"; "(* 0 ?a)" => "0"),
        rw!("mul_one"; "(* 1 ?a)" => "?a"),
        rw!("mul_comm"; "(* ?a ?b)" => "(* ?b ?a)"),
        rw!("mul_div"; "(* ?a (/ ?b ?a))" => "?b"
            if is_not_zero("?a")),

        // div
        rw!("div_one"; "(/ ?a 1)" => "?a"),
        rw!("div_mul"; "(/ (* ?a ?b) ?a)" => "?b"
            if is_not_zero("?a")),
    ]
}

pub fn list_rules() -> Vec<Rewrite> {
    // list rules
    vec![
        // Binop Fold
        rw!("binary_op_fold"; "(Binop ?bop ?a ?b)" => "(Fold ?bop (List ?a ?b))"),
    ]
}

pub fn cad_rules() -> Vec<Rewrite> {
    // rules related to the CAD domain
    vec![
        // Getting shared operation out
        rw!("fold_repeat"; "(Fold ?bop (Map2 ?aff (Repeat ?n ?param) ?cads))"=> "(Affine ?aff ?param (Fold ?bop ?cads))"),

        rw!("fold_op"; "(Fold ?bop (Affine ?aff ?param ?cad))"=> "(Affine ?aff ?param (Fold ?bop ?cad))"),

        rw!("union_trans"; "(Binop Union (Affine Trans (Vec3 ?x ?y ?z) ?a) (Affine Trans (Vec3 ?x ?y ?z) ?b))"=> "(Affine Trans (Vec3 ?x ?y ?z) (Binop Union ?a ?b))"),

        // Related to Boolean Operators
        rw!("union_same"; "(Binop Union ?a ?a)"=> "?a"),
        rw!("inter_same"; "(Binop Inter ?a ?a)"=> "?a"),
        rw!("inter_union"; "(Binop Inter ?a (Binop Union ?a ?b))"=> "?a"),
        rw!("inter_empty"; "(Binop Inter ?a Empty)"=> "Empty"),

        // MapI: aka Tabulate
        rw!("repeat_mapi"; "(Repeat ?n ?x)"=> "(MapI ?n ?x)"),
        rw!("map_repeat"; "(Map2 ?op (MapI ?n ?formula) (MapI ?n ?cad))"=> "(MapI ?n (Affine ?op ?formula ?cad))"),

        rw!("map_mapi2";
            "(Map2 ?op (MapI ?n1 ?n2 ?formula) (Repeat ?n ?cad))" =>
            "(MapI ?n1 ?n2 (Affine ?op ?formula ?cad))"
            if is_eq("?n", "(* ?n1 ?n2)")),
        rw!("mapi2_mapi2"; "(Map2 ?op (MapI ?n1 ?n2 ?param) (MapI ?n1 ?n2 ?cad))"=> "(MapI ?n1 ?n2 (Affine ?op ?param ?cad))"),

        // partitioning
        rw!("concat"; "(Unpart ?part ?lists)"=> "(Concat ?lists)"),
    ]
}

pub fn cad_identity_rules() -> Vec<Rewrite> {
    // identity rules related to CAD transformation that can be reduced.
    let mut rules = vec![
        rw!("scale_flip"; "(Affine Scale (Vec3 -1 -1 1) ?a)"=> "(Affine Rotate (Vec3 0 0 180) ?a)"),

        rw!("scale_trans";
           "(Affine Scale (Vec3 ?a ?b ?c) (Affine Trans (Vec3 ?x ?y ?z) ?m))" =>
           "(Affine Trans (Vec3 (* ?a ?x) (* ?b ?y) (* ?c ?z))
          (Affine Scale (Vec3 ?a ?b ?c) ?m))"),

        rw!("trans_scale"; "(Affine Trans (Vec3 ?x ?y ?z) (Affine Scale (Vec3 ?a ?b ?c) ?m))"=> "(Affine Scale (Vec3 ?a ?b ?c) (Affine Trans (Vec3 (/ ?x ?a) (/ ?y ?b) (/ ?z ?c)) ?m))"),

        rw!("cone_scale";
           "(Cylinder (Vec3 ?h ?r1 ?r2) ?params ?center)" =>
           "(Affine Scale (Vec3 1 1 ?h)
            (Cylinder (Vec3 1 ?r1 ?r2) ?params ?center))"),

        rw!("scale_cone";
            "(Affine Scale (Vec3 1 1 ?h)
              (Cylinder (Vec3 1 ?r1 ?r2) ?params ?center))" =>
            "(Cylinder (Vec3 ?h ?r1 ?r2) ?params ?center)"
            if is_pos(&["?h"])
        ),

        rw!("cylinder_scale";
           "(Cylinder (Vec3 ?h ?r ?r) ?params ?center)" =>
           "(Affine Scale (Vec3 ?r ?r ?h)
          (Cylinder (Vec3 1 1 1) ?params ?center))"),
        rw!("scale_cylinder";
            "(Affine Scale (Vec3 ?r ?r ?h)
          (Cylinder (Vec3 1 1 1) ?params ?center))" =>
            "(Cylinder (Vec3 ?h ?r ?r) ?params ?center)"
            if is_pos(&["?r", "?h"])
        ),

        rw!("cube_scale";
           "(Cube (Vec3 ?x ?y ?z) ?center)" =>
           "(Affine Scale (Vec3 ?x ?y ?z)
          (Cube (Vec3 1 1 1) ?center))"),
        rw!(
            "scale_cube";
            "(Affine Scale (Vec3 ?x ?y ?z)
          (Cube (Vec3 1 1 1) ?center))" =>
            "(Cube (Vec3 ?x ?y ?z) ?center)"
            if is_pos(&["?x", "?y", "?z"])
        ),

        rw!("sphere_scale";
           "(Sphere ?r ?params)" =>
           "(Affine Scale (Vec3 ?r ?r ?r)
          (Sphere 1 ?params))"),
        rw!(
            "scale_sphere";
            "(Affine Scale (Vec3 ?r ?r ?r)
          (Sphere 1 ?params))" =>
            "(Sphere ?r ?params)"
            if is_pos(&["?r"])
        ),

        rw!("id"; "(Affine Trans (Vec3 0 0 0) ?a)"=> "?a"),
        rw!("combine_scale"; "(Affine Scale (Vec3 ?a ?b ?c) (Affine Scale (Vec3 ?d ?e ?f) ?cad))"=> "(Affine Scale (Vec3 (* ?a ?d) (* ?b ?e) (* ?c ?f)) ?cad)"),
        rw!("combine_trans"; "(Affine Trans (Vec3 ?a ?b ?c) (Affine Trans (Vec3 ?d ?e ?f) ?cad))"=> "(Affine Trans (Vec3 (+ ?a ?d) (+ ?b ?e) (+ ?c ?f)) ?cad)"),
    ];
        
    // add the intro rules only for cads
    let id_affines = &[
        ("scale", "Affine Scale (Vec3 1 1 1)"),
        ("trans", "Affine Trans (Vec3 0 0 0)"),
        ("rotate", "Affine Rotate (Vec3 0 0 0)"),
    ];
    let possible_cads = &[
        ("affine", "(Affine ?op ?param ?cad)"),
        ("bop", "(Binop ?op ?cad1 ?cad2)"),
        ("fold", "(Fold ?op ?cads)"),
    ];
    for (aff_name, id_aff) in id_affines {
        for (cad_name, cad) in possible_cads {
            let intro = format!("id_{}_{}_intro", aff_name, cad_name);
            let outer: Pattern<_> = format!("({} {})", id_aff, cad).parse().unwrap();
            let cad: Pattern<_> = cad.parse().unwrap();
            rules.push(rw!(intro; cad => outer));
        }

        // elim rules work for everything
        let elim = format!("id_{}_elim", aff_name);
        let outer: Pattern<_> = format!("({} ?a)", id_aff).parse().unwrap();
        rules.push(rw!(elim; outer => "?a"));
    }
    rules
}

pub fn inv_trans_rules() -> Vec<Rewrite> {
    vec![
        rw!("map_unpart_r2";
           "  (Map2 ?op ?params (Unpart ?part ?cads))" =>
           "(Unpart ?part (Part ?part
          (Map2 ?op ?params (Unpart ?part ?cads))))"),
        rw!("map_unpart_l2";
           "  (Map2 ?op (Unpart ?part ?params) ?cads)" =>
           "(Unpart ?part (Part ?part
          (Map2 ?op (Unpart ?part ?params) ?cads)))"),

        // NOTE do we need part/unpart id?
        rw!("part_unpart"; "(Part ?part (Unpart ?part ?list))"=> "?list"),
        rw!("unpart_part"; "(Unpart ?part (Part ?part ?list))"=> "?list"),

        // unsort propagation
        // rw!("sort_repeat"; "(Sort ?perm (Repeat ?n ?elem))"=> "(Repeat ?n ?elem)"),
        rw!("sort_unsort"; "(Sort ?perm (Unsort ?perm ?list))"=> "?list"),
        rw!("unsort_sort"; "(Unsort ?perm (Sort ?perm ?list))"=> "?list"),

        rw!("map_unsort_l";
           "  (Map2 ?op (Unsort ?perm ?params) ?cads)" =>
           "(Unsort ?perm (Sort ?perm
          (Map2 ?op (Unsort ?perm ?params) ?cads)))"),

        rw!("map_unsort_r";
           "  (Map2 ?op ?params (Unsort ?perm ?cads))" =>
           "(Unsort ?perm (Sort ?perm
          (Map2 ?op ?params (Unsort ?perm ?cads))))"),

        rw!("unsort_repeat"; "(Unsort ?perm (Repeat ?n ?elem))"=> "(Repeat ?n ?elem)"),

        rw!("fold_union_unsort"; "(Fold Union (Unsort ?perm ?x))"=> "(Fold Union ?x)"),
        rw!("fold_inter_unsort"; "(Fold Inter (Unsort ?perm ?x))"=> "(Fold Inter ?x)"),

        // unpolar
        rw!("unpolar_trans"; "(Map2 Trans (Unpolar ?n ?center ?params) ?cads)"=> "(Map2 Trans (Repeat ?n ?center) (Map2 TransPolar ?params ?cads))"),
    ]
}

#[rustfmt::skip]
pub fn rules() -> Vec<Rewrite> {

    let mut rules = vec![];

    rules.extend(math_rules());
    rules.extend(list_rules());
    rules.extend(cad_rules());

    if INV_TRANS {
        rules.extend(inv_trans_rules());
    }

    if CAD_IDENTS {
        rules.extend(cad_identity_rules());
    }

    rules.push(rw!(
        "listapplier";
        "?list" => {
            let var = "?list".parse().unwrap();
            ListApplier { var }
        }
    ));

    rules.push(rw!(
        "sortapplier";
        "(Sort ?perm ?list)" => {
            let perm = "?perm".parse().unwrap();
            let list = "?list".parse().unwrap();
            SortApplier { perm, list }
        }
    ));

    rules.push(rw!(
        "partapplier";
        "(Part ?part ?list)" => {
            let part = "?part".parse().unwrap();
            let list = "?list".parse().unwrap();
            PartApplier { part, list }
        }
    ));

    // TODO should this perform concat when possible?
    rules.push(rw!(
        "unpart-unsort";
        "(Unpart ?part ?list)" => {
            let part = "?part".parse().unwrap();
            let list = "?list".parse().unwrap();
            UnpartApplier { part, list }
        }
    ));

    rules.push(rw!(
        "sort-unpart";
        "(Sort ?sort (Unpart ?part ?list))" => {
            let part = "?part".parse().unwrap();
            let sort = "?sort".parse().unwrap();
            let list = "?list".parse().unwrap();
            SortUnpartApplier { sort, part, list }
        }
    ));

    println!("Using {} rules", rules.len());

    rules
}

fn get_float(expr: &Cad) -> Option<Num> {
    match expr {
        Cad::Num(f) => Some(*f),
        _ => None,
    }
}

fn get_vec(egraph: &EGraph, expr: &Cad) -> Option<Vec3> {
    if let Cad::Vec3(args) = expr {
        assert_eq!(args.len(), 3);
        let f0 = get_float(&egraph[args[0]].data.best)?;
        let f1 = get_float(&egraph[args[1]].data.best)?;
        let f2 = get_float(&egraph[args[2]].data.best)?;
        Some((f0, f1, f2))
    } else {
        None
    }
}

#[derive(Debug)]
struct ListApplier {
    var: Var,
}

// this partition will partition all at once
fn partition_list<F, K>(egraph: &mut EGraph, ids: &[Id], mut key_fn: F) -> Option<Id>
where
    F: FnMut(usize, Id) -> K,
    K: Hash + Eq + Debug + Clone,
{
    // allow easy disabling
    if !PARTITIONING {
        return None;
    }

    // actually do the partitioning, keeping track of where we put things
    type Pair<T> = (Vec<usize>, Vec<T>);
    let mut parts: IndexMap<K, Pair<_>> = Default::default();
    for (i, &id) in ids.iter().enumerate() {
        let key = key_fn(i, id);
        let (is, ids) = parts.entry(key).or_default();
        is.push(i);
        ids.push(id);
    }

    if parts.len() <= 1 || parts.len() > PARTITIONING_MAX {
        return None;
    }

    let mut order = Vec::new();
    let mut list_ids = vec![];
    let mut lengths = Vec::new();
    for (_, (is, ids)) in &parts {
        order.extend(is);
        lengths.push(ids.len());
        list_ids.push(egraph.add(Cad::List(ids.clone())));
    }
    let part = Partitioning::from_vec(lengths);
    let part_id = egraph.add(Cad::Partitioning(part));
    let list_of_lists = egraph.add(Cad::List(list_ids));
    let concat = egraph.add(Cad::Unpart([part_id, list_of_lists]));

    let perm = Permutation::from_vec(&order);
    let res = if perm.is_ordered() {
        concat
    } else {
        let p = Cad::Permutation(perm);
        let e = Cad::Unsort([egraph.add(p), concat]);
        egraph.add(e)
    };

    Some(res)
}

fn get_single_cad(egraph: &EGraph, id: Id) -> Cad {
    let best = &egraph[id].data.best;
    assert!(best.is_leaf());
    best.clone()
}

fn get_affines(egraph: &EGraph, id: Id, affine_kind: &Cad) -> Vec<(Id, Id)> {
    egraph[id]
        .nodes
        .iter()
        .filter_map(|n| {
            if let Cad::Affine(args) = n {
                let kind = get_single_cad(egraph, args[0]);
                if affine_kind == &kind {
                    Some((args[1], args[2]))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect()
}

type AffineSig = [usize; 3];
fn affine_signature(egraph: &EGraph, id: Id) -> AffineSig {
    let mut scales = 0;
    let mut rotates = 0;
    let mut translates = 0;
    for n in &egraph[id].nodes {
        if let Cad::Affine(args) = n {
            let kind = get_single_cad(egraph, args[0]);
            #[rustfmt::skip]
            match kind {
                Cad::Trans => {translates += 1;}
                Cad::Scale => {scales += 1;}
                Cad::Rotate => {rotates += 1;}
                _ => (),
            };
        }
    }
    translates = AFFINE_SIGNATURE_MAX_LEN.min(translates);
    scales = AFFINE_SIGNATURE_MAX_LEN.min(scales);
    rotates = AFFINE_SIGNATURE_MAX_LEN.min(rotates);
    [translates, scales, rotates]
}

fn insert_map2s(egraph: &mut EGraph, list_ids: &[Id]) -> Vec<Id> {
    let mut results = vec![];

    let sigs: Vec<AffineSig> = list_ids
        .iter()
        .map(|&id| affine_signature(egraph, id))
        .collect();
    let unique_sigs: IndexSet<AffineSig> = sigs.iter().cloned().collect();

    for (cadi, cad) in [Cad::Trans, Cad::Scale, Cad::Rotate].iter().enumerate() {
        let affs_list: Vec<Vec<_>> = list_ids
            .iter()
            .map(|&id| get_affines(egraph, id, cad))
            .collect();

        assert!(affs_list
            .iter()
            .zip(&sigs)
            .all(|(affs, sig)| affs.len() >= sig[cadi]));

        let aff_id = egraph.add(cad.clone());

        let unique_sig_lengths = || unique_sigs.iter().map(|&sig| sig[cadi]);

        let total: usize = unique_sig_lengths().product();
        if total > STRUCTURE_MATCH_LIMIT {
            warn!(
                "Exceeding structure match limit: {} > {}",
                total, STRUCTURE_MATCH_LIMIT
            );
        }

        for choices in unique_sig_lengths()
            .map(|len| 0..len)
            .multi_cartesian_product()
            .take(STRUCTURE_MATCH_LIMIT)
        {
            let (param_ids, cad_ids): (Vec<Id>, Vec<Id>) = affs_list
                .iter()
                .zip(&sigs)
                .map(|(affs, sig)| {
                    let unique_sig_i = unique_sigs.get_full(sig).unwrap().0;
                    affs[choices[unique_sig_i]]
                })
                .unzip();

            assert_eq!(param_ids.len(), cad_ids.len());

            let param_list_id = egraph.add(Cad::List(param_ids));
            let cad_list_id = egraph.add(Cad::List(cad_ids));
            let map2 = Cad::Map2([aff_id, param_list_id, cad_list_id]);
            let id = egraph.add(map2);
            results.push(id)
        }
    }

    results
}

#[allow(dead_code)]
fn num_sign(n: Num) -> i32 {
    let f = n.to_f64();
    if f < 0.0 {
        -1
    } else if f > 0.0 {
        1
    } else {
        0
    }
}

macro_rules! get_meta_list {
    ($egraph:expr, $eclass:expr) => {
        match &$egraph[$eclass].data.list {
            Some(ids) => ids,
            None => return vec![],
        }
    };
}

impl Applier<Cad, MetaAnalysis> for ListApplier {
    fn apply_one(
        &self,
        egraph: &mut EGraph,
        eclass: Id,
        map: &Subst,
        _searcher_ast: Option<&PatternAst<Cad>>,
        _rule_name: Symbol,
    ) -> Vec<Id> {
        let ids: Vec<Id> = get_meta_list!(egraph, map[self.var]).clone();
        let bests: Vec<_> = ids.iter().map(|&id| egraph[id].data.best.clone()).collect();
        let ops: Option<Vec<_>> = ids
            .iter()
            .map(|&id| {
                egraph[id].nodes.iter().find_map(|n| match n {
                    Cad::Affine(args) => Some(get_single_cad(egraph, args[0])),
                    _ => None,
                })
            })
            .collect();
        let mut results = vec![];

        // insert repeats
        if ids.len() > 1 {
            let i0 = egraph.find(ids[0]);
            if ids.iter().all(|id| i0 == egraph.find(*id)) {
                let len = Cad::Num(ids.len().into());
                let e = Cad::Repeat([egraph.add(len), i0]);
                let id = egraph.add(e);
                results.push(id);

                for result in results.iter() {
                    egraph.union(eclass, *result);
                }
                return results;
            }
        }

        // don't partition a list of lists
        if ids
            .iter()
            .any(|&id| egraph[id].nodes.iter().any(|n| matches!(n, Cad::List(_))))
        {
            for result in results.iter() {
                egraph.union(eclass, *result);
            }
            return results;
        }

        results.extend(insert_map2s(egraph, &ids));

        // try to solve a list
        if let Some(vec_list) = bests
            .iter()
            .map(|n| get_vec(egraph, n))
            .collect::<Option<Vec<Vec3>>>()
        {
            let len = vec_list.len();
            if len > 2 {
                let solved = crate::solve::solve(egraph, &vec_list);
                results.extend(solved);
            }
            results.extend(partition_list(egraph, &ids, |i, _| vec_list[i].0));
            results.extend(partition_list(egraph, &ids, |i, _| vec_list[i].1));
            results.extend(partition_list(egraph, &ids, |i, _| vec_list[i].2));
            results.extend(partition_list(egraph, &ids, |i, _| {
                (vec_list[i].0, vec_list[i].1)
            }));
            results.extend(partition_list(egraph, &ids, |i, _| {
                (vec_list[i].0, vec_list[i].2)
            }));
            results.extend(partition_list(egraph, &ids, |i, _| {
                (vec_list[i].1, vec_list[i].2)
            }));
        }

        // try to partition things by eclass
        results.extend(partition_list(egraph, &ids, |i, _| ids[i]));

        // try to partition things by operator
        if let Some(ops) = ops {
            results.extend(partition_list(egraph, &ids, |i, _| discriminant(&ops[i])));
        }

        for result in results.iter() {
            egraph.union(eclass, *result);
        }
        results
    }
}

macro_rules! get_unit {
    ($egraph:expr, $eclass:expr, $cad:path) => {{
        let egraph = &$egraph;
        let eclass = $eclass;
        match &egraph[eclass].data.best {
            $cad(p) => p,
            _ => panic!("expected {}", stringify!($cad)),
        }
    }};
}

#[derive(Debug)]
struct SortApplier {
    perm: Var,
    list: Var,
}

impl Applier<Cad, MetaAnalysis> for SortApplier {
    fn apply_one(
        &self,
        egraph: &mut EGraph,
        eclass: Id,
        map: &Subst,
        _searcher_ast: Option<&PatternAst<Cad>>,
        _rule_name: Symbol,
    ) -> Vec<Id> {
        let items = get_meta_list!(egraph, map[self.list]);
        let perm: &Permutation = get_unit!(egraph, map[self.perm], Cad::Permutation);
        let sorted = perm.apply(items);
        let e = Cad::List(sorted);

        let id = egraph.add(e);
        egraph.union(eclass, id);
        vec![id]
    }
}

#[derive(Debug)]
struct PartApplier {
    part: Var,
    list: Var,
}

impl Applier<Cad, MetaAnalysis> for PartApplier {
    fn apply_one(
        &self,
        egraph: &mut EGraph,
        eclass: Id,
        map: &Subst,
        _searcher_ast: Option<&PatternAst<Cad>>,
        _rule_name: Symbol,
    ) -> Vec<Id> {
        let items = get_meta_list!(egraph, map[self.list]);
        let part: &Partitioning = get_unit!(egraph, map[self.part], Cad::Partitioning);
        let list_of_lists = part
            .apply(items)
            .into_iter()
            .map(|sublist| egraph.add(Cad::List(sublist)))
            .collect();

        let e = Cad::List(list_of_lists);

        let id = egraph.add(e);
        egraph.union(eclass, id);
        vec![id]
    }
}

#[derive(Debug)]
struct UnpartApplier {
    part: Var,
    list: Var,
}

impl Applier<Cad, MetaAnalysis> for UnpartApplier {
    fn apply_one(
        &self,
        egraph: &mut EGraph,
        eclass: Id,
        map: &Subst,
        _searcher_ast: Option<&PatternAst<Cad>>,
        _rule_name: Symbol,
    ) -> Vec<Id> {
        let items = get_meta_list!(egraph, map[self.list]).clone();
        let part: Partitioning = get_unit!(egraph, map[self.part], Cad::Partitioning).clone();
        assert_eq!(part.lengths.len(), items.len());

        if items.is_empty() {
            let nil = egraph.add(Cad::Nil);
            egraph.union(eclass, nil);
            return vec![nil];
        }

        let get_unsort = |id: Id| -> Option<(&Permutation, Id)> {
            egraph[id].nodes.iter().find_map(|n| match n {
                Cad::Unsort(args) => {
                    let perm = get_unit!(egraph, args[0], Cad::Permutation);
                    Some((perm, args[1]))
                }
                _ => None,
            })
        };

        let mut big_perm = vec![];
        let mut ids = vec![];
        let mut len_so_far = 0;
        for (&id, &len) in items.iter().zip(&part.lengths) {
            if let Some((perm, inner_id)) = get_unsort(id) {
                assert_eq!(perm.len(), len);
                big_perm.extend(perm.order.iter().map(|i| i + len_so_far));
                ids.push(inner_id);
            } else {
                big_perm.extend(len_so_far..len_so_far + len);
                ids.push(id);
            }
            len_so_far += len;
        }
        assert_eq!(len_so_far, part.total_len());
        assert_eq!(len_so_far, big_perm.len());

        let perm = Permutation::from_vec(&big_perm);
        let is_ordered = perm.is_ordered();
        let perm = egraph.add(Cad::Permutation(perm));
        let part = egraph.add(Cad::Partitioning(part));

        let list = egraph.add(Cad::List(ids));
        let unpart = egraph.add(Cad::Unpart([part, list]));

        let results = if is_ordered {
            vec![unpart]
        } else {
            let unsort = egraph.add(Cad::Unsort([perm, unpart]));
            vec![unsort]
        };
        for result in results.iter() {
            egraph.union(eclass, *result);
        }
        results
    }
}

#[derive(Debug)]
struct SortUnpartApplier {
    sort: Var,
    part: Var,
    list: Var,
}

impl Applier<Cad, MetaAnalysis> for SortUnpartApplier {
    fn apply_one(
        &self,
        egraph: &mut EGraph,
        eclass: Id,
        map: &Subst,
        _searcher_ast: Option<&PatternAst<Cad>>,
        _rule_name: Symbol,
    ) -> Vec<Id> {
        let sort: Permutation = get_unit!(egraph, map[self.sort], Cad::Permutation).clone();
        let part: Partitioning = get_unit!(egraph, map[self.part], Cad::Partitioning).clone();
        let items = get_meta_list!(egraph, map[self.list]).clone();

        let mut sorts = vec![];
        let mut len_so_far = 0;
        for len in &part.lengths {
            let slice = &sort.order[len_so_far..len_so_far + len];
            if !slice
                .iter()
                .all(|&i| len_so_far <= i && i < len_so_far + len)
            {
                return vec![];
            }
            sorts.push(slice.iter().map(|i| i - len_so_far).collect::<Vec<_>>());
            len_so_far += len;
        }

        let sorted_lists = sorts
            .into_iter()
            .zip(items)
            .map(|(p, list_id)| {
                let perm = Permutation::from_vec(&p);
                let sort_id = egraph.add(Cad::Permutation(perm));
                egraph.add(Cad::Sort([sort_id, list_id]))
            })
            .collect();
        let sorted = Cad::List(sorted_lists);
        let list = egraph.add(sorted);
        let part_id = egraph.add(Cad::Partitioning(part.clone()));

        let results = vec![egraph.add(Cad::Unpart([part_id, list]))];
        for result in results.iter() {
            egraph.union(eclass, *result);
        }
        results
    }
}

#[derive(Debug)]
struct Flatten {
    op: Cad,
    list: Var,
}

impl Applier<Cad, MetaAnalysis> for Flatten {
    fn apply_one(
        &self,
        egraph: &mut EGraph,
        eclass: Id,
        map: &Subst,
        _searcher_ast: Option<&PatternAst<Cad>>,
        _rule_name: Symbol,
    ) -> Vec<Id> {
        fn get_nested_fold<'a>(egraph: &'a EGraph, op: &'a Cad, id: Id) -> Option<&'a [Id]> {
            let is_op = |i| egraph[i].nodes.iter().any(|c| c == op);
            let get_list = |i| egraph[i].data.list.as_deref();
            egraph[id]
                .nodes
                .iter()
                .find(|n| matches!(n, Cad::Fold(_)) && is_op(n.children()[0]))
                .and_then(|n| get_list(n.children()[1]))
        }

        let ids = get_meta_list!(egraph, map[self.list]);
        let mut new_ids = Vec::new();
        for id in ids {
            match get_nested_fold(egraph, &self.op, *id) {
                Some(ids) => new_ids.extend(ids.iter().copied()),
                None => new_ids.push(*id),
            }
        }

        let new_list = egraph.add(Cad::List(new_ids));
        let op = egraph.add(self.op.clone());
        let new_fold = egraph.add(Cad::Fold([op, new_list]));

        let results = vec![new_fold];
        for result in results.iter() {
            egraph.union(eclass, *result);
        }
        results
    }
}
