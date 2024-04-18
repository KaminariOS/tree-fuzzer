use crate::node_types::NodeTypes;
use core::{fmt::Debug, marker::PhantomData};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use libafl::corpus::{Testcase, Corpus};
use libafl::events::EventFirer;
use libafl::feedbacks::Feedback;
use libafl::inputs::{HasBytesVec, Input};
use libafl::observers::ObserversTuple;
use libafl_bolts::{HasLen, Named};
use tree_sitter::{Language, Tree, Node};
use libafl::state::{HasCorpus, HasMetadata, State};
use libafl::mutators::{Mutator, MutationResult};
use libafl::executors::ExitKind;
use libafl::Error;
use rand::{prelude::StdRng, Rng, SeedableRng};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use tree_sitter_edit::Editor;
use serde::{Deserialize, Serialize};

pub fn parse(language: Language, code: &str) -> Tree {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(language)
        .expect("Failed to set tree-sitter parser language");
    parser.parse(code, None).expect("Failed to parse code")
}


#[derive(Debug, Default)]
struct Edits(HashMap<usize, Vec<u8>>);

impl Editor for Edits {
    fn has_edit(&self, _tree: &Tree, node: &Node) -> bool {
        self.0.get(&node.id()).is_some()
    }

    fn edit(&self, _source: &[u8], tree: &Tree, node: &Node) -> Vec<u8> {
        debug_assert!(self.has_edit(tree, node));
        self.0.get(&node.id()).unwrap().clone()
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Branches(HashMap<String, (Vec<Vec<u8>>, HashSet<Vec<u8>>)>);

impl Branches {
    fn new(trees: Vec<(Vec<u8>, Tree)>) -> Self {
        let mut branches = HashMap::with_capacity(trees.len()); // min
        for (i, (text, tree)) in trees.into_iter().enumerate() {
            // println!("Branches {}", &String::from_utf8_lossy(&text));
            let mut nodes = vec![tree.root_node()];
            while !nodes.is_empty() {
                dbg!();
                let mut children = Vec::with_capacity(nodes.len()); // guesstimate
                for node in nodes {
                    branches
                        .entry(node.kind())
                        .or_insert_with(|| HashSet::with_capacity(1))
                        .insert(text[node.byte_range()].to_vec());
                    let mut i = 0;
                    while let Some(child) = node.child(i) {
                        children.push(child);
                        i += 1;
                    }
                }
                nodes = children;
            }
        }
        Branches(
            branches
                .into_iter()
                .map(|(k, s)| (k.to_owned(), (s.iter().map(|v| 
                        v.to_vec()).collect(),
                        s
                )))
                .collect(),
        )
    }
    
    fn add_tree(&mut self, (text, tree): (Vec<u8>, Tree)) {
        // dbg!("Adding tree");
            let mut nodes = vec![tree.root_node()];
            while !nodes.is_empty() {
                dbg!();
                let mut children = Vec::with_capacity(nodes.len()); // guesstimate
                for node in nodes {
                    let slot = self.0
                        .entry(node.kind().to_string())
                        .or_insert_with(|| (vec![], HashSet::with_capacity(1)));
                    let txt = text[node.byte_range()].to_vec();
                    let contains = slot.1.contains(&txt);
                    if !contains {
                        slot.0.push(txt.clone());
                        slot.1.insert(txt);
                    }
                    let mut i = 0;
                    while let Some(child) = node.child(i) {
                        children.push(child);
                        i += 1;
                    }
                }
                nodes = children;
            }

        // dbg!("Added tree");
    } 

    fn possible(&self) -> usize {
        let mut possible_mutations = 0;
        for s in self.0.values() {
            possible_mutations += s.0.len() - 1;
        }
        possible_mutations
    }
}

pub struct TreeContext {
    node_types: NodeTypes,
    language: Language,
    chaos: u8,
    deletions: u8,
    inter_splices: usize,
    max_size: usize,
    reparse: usize,
    rng: RefCell<StdRng>
}

impl TreeContext {
    pub fn new(language: Language, node_types_str: &'static str) -> Self {

        Self {
            node_types: NodeTypes::new(node_types_str).unwrap(),
            language,
            chaos: 5,
            deletions: 11,
            inter_splices: 11,
            max_size: 500,
            reparse: 11,
            rng: RefCell::new(rand::rngs::StdRng::seed_from_u64(11)),
        }
    }
}

pub struct TreeFeedback<'a, S> {
    ctx: &'a TreeContext,
    phantom: PhantomData<S>,
}


impl<S> Debug for TreeFeedback<'_, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TreeFeedback {{}}")
    }
}

impl<'a, S> TreeFeedback<'a, S> {
    /// Create a new [`NautilusFeedback`]
    #[must_use]
    pub fn new(context: &'a TreeContext) -> Self {
        Self {
            ctx: &context,
            phantom: PhantomData,
        }
    }
}

impl<'a, S> Named for TreeFeedback<'a, S> {
    fn name(&self) -> &str {
        "TreeFeedback"
    }
}

impl<'a, S> Feedback<S> for TreeFeedback<'a, S>
where
    S: HasMetadata + HasCorpus<Input = TestTree> + State<Input = TestTree>,
{
    #[allow(clippy::wrong_self_convention)]
    fn is_interesting<EM, OT>(
        &mut self,
        _state: &mut S,
        _manager: &mut EM,
        _input: &TestTree,
        _observers: &OT,
        _exit_kind: &ExitKind,
    ) -> Result<bool, Error>
    where
        EM: EventFirer<State = S>,
        OT: ObserversTuple<S>,
    {
        Ok(false)
    }

    fn append_metadata<OT>(
        &mut self,
        state: &mut S,
        _observers: &OT,
        testcase: &mut Testcase<S::Input>,
    ) -> Result<(), Error>
    where
        OT: ObserversTuple<S>,
    {
        state.corpus().load_input_into(testcase)?;
        let input = testcase.input().as_ref().unwrap().clone();
        let meta = state
            .metadata_map_mut()
            .get_mut::<TreeMetaData>()
            .expect("TreeMeta not in the state");
        meta.add_tree(input, self.ctx);
        Ok(())
    }

    fn discard_metadata(&mut self, _state: &mut S, _input: &TestTree) -> Result<(), Error> {
        Ok(())
    }
}

pub struct TreeSpliceMutator<'a> {
    ctx: &'a TreeContext,
}

impl Named for TreeSpliceMutator<'_> {
    fn name(&self) -> &str {
        "TreeSpliceMutator"
    }
}

impl<'a> TreeSpliceMutator<'a> {
    pub fn new(ctx: &'a TreeContext) -> Self {
        Self {
            ctx
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TestTree(pub Vec<u8>);
impl Input for TestTree {
    fn generate_name(&self, idx: usize) -> String {
        "Test tree".to_owned()
    }

    fn from_file<P>(path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        let mut file = File::open(path)?;
        let mut bytes: Vec<u8> = vec![];
        file.read_to_end(&mut bytes)?;
        Ok(Self(bytes))
    }
}

impl HasLen for TestTree {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl HasBytesVec for TestTree {
    fn bytes(&self) -> &[u8] {
        self.0.as_slice()
    }

    fn bytes_mut(&mut self) -> &mut Vec<u8> {
        &mut self.0
    }
}

impl<S> Mutator<TestTree, S> for TreeSpliceMutator<'_> 
where
    S: HasCorpus<Input = TestTree> + HasMetadata,
{
    fn mutate(
        &mut self,
        state: &mut S,
        input: &mut TestTree,
        stage_idx: i32,
    ) -> Result<libafl::prelude::MutationResult, libafl::prelude::Error> {
        let meta = state
            .metadata_map_mut()
            .get_mut::<TreeMetaData>()
            .expect("Tree meta data not in the state");
        let code =  String::from_utf8(input.0.clone());
        let mut tmp = vec![];
        if let Ok(code) = code {
            let tree = parse(self.ctx.language, &code);
            tmp = meta.splice_tree(&input.0, tree, self.ctx).unwrap_or_default();
        }
        if tmp.is_empty() {
            Ok(MutationResult::Skipped)
        } else {
            input.0 = tmp; 
            // input.tree = Tree::from_rule_vec(tmp, self.ctx);
            Ok(MutationResult::Mutated)
        }
    }
}


#[derive(Serialize, Deserialize, Debug)]
pub struct TreeMetaData {
    node_types: NodeTypes,
    chaos: u8,
    deletions: u8,
    inter_splices: usize,
    max_size: usize,
    reparse: usize,
    // rng: StdRng,
    branches: Branches,
    kinds: (Vec<String>, HashSet<String>),
}

libafl_bolts::impl_serdeany!(TreeMetaData);

impl TreeMetaData {
    fn add_tree(&mut self, txt: TestTree, ctx: &TreeContext) {
        let code = String::from_utf8(txt.0.clone());
        if let Ok(code) = code {
            let tree = parse(ctx.language, &code);
            self.branches.add_tree((txt.0, tree));
            self.branches.0.keys()
                .for_each(|s| {
                    let contains = self.kinds.1.contains(s);
                    if !contains {
                        self.kinds.0.push(s.to_owned());
                        self.kinds.1.insert(s.to_owned());
                    }
                }); 
        }
    }

   pub fn new( 
          node_types_str: &'static str,
          files: HashMap<String, (Vec<u8>, Tree)>
          ) -> Self {

        let branches = Branches::new(
            files
                .into_iter()
                .map(|(_, (txt, tree))| (txt, tree))
                .collect(),
        );

        let kinds = (
            branches.0.keys()
            .map(|s| 
                 s.clone()
                 ).collect(),
            branches.0.keys()
            .map(|s| 
                 s.clone()
                 ).collect()
                 );
        Self {
            node_types: NodeTypes::new(node_types_str).unwrap(),
            // language,
            chaos: 5,
            deletions: 5,
            inter_splices: 16,
            max_size: 4000,
            reparse: usize::MAX,
            // rng: rand::rngs::StdRng::seed_from_u64(11),
            branches,
            kinds
        }
   } 

    fn pick_usize(&mut self, n: usize, ctx: &TreeContext) -> usize {
        ctx.rng.borrow_mut().gen_range(0..n)
    }

    fn pick_idx<T>(&mut self, v: &Vec<T>, ctx: &TreeContext) -> usize {
        self.pick_usize(v.len(), ctx)
    }

    fn all_nodes<'b>(&self, tree: &'b Tree) -> Vec<Node<'b>> {
        let mut all = Vec::with_capacity(16); // min
        let root = tree.root_node();
        let mut cursor = tree.walk();
        let mut nodes: HashSet<_> = root.children(&mut cursor).collect();
        while !nodes.is_empty() {
            let mut next = HashSet::new();
            for node in nodes {
                debug_assert!(!next.contains(&node));
                all.push(node);
                let mut child_cursor = tree.walk();
                for child in node.children(&mut child_cursor) {
                    debug_assert!(child.id() != node.id());
                    debug_assert!(!next.contains(&child));
                    next.insert(child);
                }
            }

            nodes = next;
        }
        all
    }

    fn pick_node<'b>(&mut self, tree: &'b Tree, ctx: &TreeContext) -> Node<'b> {
        let nodes = self.all_nodes(tree);
        if nodes.is_empty() {
            return tree.root_node();
        }
        *nodes.get(self.pick_idx(&nodes, ctx)).unwrap()
    }

    fn delete_node(&mut self, _text: &[u8], tree: &Tree, ctx: &TreeContext) -> (usize, Vec<u8>, isize) {
        let chaotic = ctx.rng.borrow_mut().gen_range(0..100) < self.chaos;
        if chaotic {
            let node = self.pick_node(tree, ctx);
            return (node.id(), Vec::new(), Self::delta(node, &[]));
        }
        let nodes = self.all_nodes(tree);
        if nodes.iter().all(|n| !self.node_types.optional_node(n)) {
            let node = self.pick_node(tree, ctx);
            return (node.id(), Vec::new(), Self::delta(node, &[]));
        }
        let mut node = nodes.get(self.pick_idx(&nodes, ctx)).unwrap();
        while !self.node_types.optional_node(node) {
            dbg!("Delete");
            node = nodes.get(self.pick_idx(&nodes, ctx)).unwrap();
        }
        (node.id(), Vec::new(), Self::delta(*node, &[]))
    }

    fn splice_node(&mut self, text: &[u8], tree: &Tree, ctx: &TreeContext) -> (usize, Vec<u8>, isize) {
        let mut chaotic = ctx.rng.borrow_mut().gen_range(0..100) < self.chaos;

        let mut node = tree.root_node();
        let mut candidates = Vec::new();
        // When modified trees are re-parsed, their nodes may have novel kinds
        // not in Branches (candidates.len() == 0). Also, avoid not mutating
        // (candidates.len() == 1).
        let mut loop_times = 0;
        while candidates.len() <= 1 {
            loop_times += 1;
            if loop_times >= 100 {
                println!("text: {}; branches :{:?} ", node.kind(), String::from_utf8(text.to_vec()));
                chaotic = true;
            }
            dbg!("candidates");
            node = self.pick_node(tree, ctx);
            candidates = if chaotic {
                let kind_idx = ctx.rng.borrow_mut().gen_range(0..self.kinds.0.len());
                let kind = self.kinds.0.get(kind_idx).unwrap();
                self.branches.0.get(kind)
                    .map(|s| &s.0)
                    .unwrap().clone()
            } else {
                self.branches
                    .0
                    .get(node.kind())
                    .map(|s| &s.0)
                    .cloned()
                    .unwrap_or_default()
            };
        }

        let idx = ctx.rng.borrow_mut().gen_range(0..candidates.len());
        let mut candidate = candidates.get(idx).unwrap();
        // Try to avoid not mutating
        let node_text = &text[node.byte_range()];
        while candidates.len() > 1 && candidate == &node_text {
            dbg!("candidates");
            let idx = ctx.rng.borrow_mut().gen_range(0..candidates.len());
            candidate = candidates.get(idx).unwrap();
        }
        // eprintln!(
        //     "Replacing '{}' with '{}'",
        //     std::str::from_utf8(&text[node.byte_range()]).unwrap(),
        //     std::str::from_utf8(candidate).unwrap(),
        // );
        let replace = candidate.to_vec();
        let delta = Self::delta(node, replace.as_slice());
        (node.id(), replace, delta)
    }

    fn delta(node: Node<'_>, replace: &[u8]) -> isize {
        let range = node.byte_range();
        isize::try_from(replace.len()).unwrap_or_default()
            - isize::try_from(range.end - range.start).unwrap_or_default()
    }

    pub fn splice_tree(&mut self, text0: &[u8], mut tree: Tree, ctx: &TreeContext) -> Option<Vec<u8>> {
        // TODO: Assert that text0 and tree.root_node() are the same length?
        let mut edits = Edits::default();
        if self.inter_splices == 0 {
            return None;
        }
        let splices = ctx.rng.borrow_mut().gen_range(1..self.inter_splices);
        let mut text = Vec::from(text0);
        let mut sz = isize::try_from(text.len()).unwrap_or_default();
        for i in 0..splices {
            let (id, bytes, delta) = if ctx.rng.borrow_mut().gen_range(0..100) < self.deletions {
                self.delete_node(text.as_slice(), &tree, ctx)
            } else {
                self.splice_node(text.as_slice(), &tree, ctx)
            };
            sz += delta;
            let sized_out = usize::try_from(sz).unwrap_or_default() >= self.max_size;
            edits.0.insert(id, bytes);
            if i % self.reparse == 0 || i + 1 == splices || sized_out {
                let mut result = Vec::with_capacity(usize::try_from(sz).unwrap_or_default());
                tree_sitter_edit::render(&mut result, &tree, text.as_slice(), &edits).ok()?;
                text = result.clone();
                tree = parse(ctx.language, &String::from_utf8_lossy(text.as_slice()));
                edits = Edits::default();
            }
            if sized_out {
                break;
            }
        }
        Some(text)
    }
}




