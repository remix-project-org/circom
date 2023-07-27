use program_structure::ast::produce_report_with_message;
use program_structure::error_code::ReportCode;
use program_structure::error_definition::Report;
use std::collections::{HashMap, HashSet};

pub struct FileStack {
    locations: Vec<usize>,
    stack: Vec<String>,
    black_paths: HashSet<String>
}

impl FileStack {
    pub fn new(src: String) -> FileStack{
        let mut stack = Vec::new();

        stack.push(src);
        FileStack { locations: vec![0], stack, black_paths: HashSet::new() }
    }

    pub fn add_include(
        f_stack: &mut FileStack,
        name: String,
        libraries: &Vec<String>
    ) -> Result<String, Report> {
        if let Some(index) = libraries.iter().position(|x| x == &name) {
            if !f_stack.black_paths.contains(&name) {
                f_stack.stack.push(name.clone());
                f_stack.locations.push(index);
            }
            return Result::Ok(name);
        }
        Result::Err( produce_report_with_message(ReportCode::IncludeNotFound, name))
    }

    pub fn take_next(f_stack: &mut FileStack) -> Option<usize> {
        loop {
            match f_stack.stack.pop() {
                None => {
                    break None;
                }
                Some(file) if !f_stack.black_paths.contains(&file) => {
                    f_stack.black_paths.insert(file.clone());
                    if let Some(index) = f_stack.locations.pop() {
                        break Some(index);
                    } else {
                        break None; 
                    }
                }
                _ => {}
            }
        }
    }
}

pub struct IncludesNode {
    pub path: String,
    pub custom_gates_pragma: bool,
}
#[derive(Default)]
pub struct IncludesGraph {
    nodes: Vec<IncludesNode>,
    adjacency: HashMap<String, Vec<usize>>,
    custom_gates_nodes: Vec<usize>,
}

impl IncludesGraph {
    pub fn new() -> IncludesGraph {
        IncludesGraph::default()
    }

    pub fn add_node(&mut self, path: String, custom_gates_pragma: bool, custom_gates_usage: bool) {
        self.nodes.push(IncludesNode { path, custom_gates_pragma });
        if custom_gates_usage {
            self.custom_gates_nodes.push(self.nodes.len() - 1);
        }
    }

    pub fn add_edge(&mut self, old_path: String, libraries: &Vec<String>) -> Result<(), Report> {
        let mut crr = old_path.clone();

        if libraries.contains(&crr) {
            let edges = self.adjacency.entry(crr).or_insert(vec![]);
            edges.push(self.nodes.len() - 1);
            Ok(())
        } else {
            Err(produce_report_with_message(ReportCode::FileOs, old_path))
        }
    }

    pub fn get_problematic_paths(&self) -> Vec<Vec<String>> {
        let mut problematic_paths = Vec::new();
        for from in &self.custom_gates_nodes {
            problematic_paths.append(&mut self.traverse(*from, Vec::new(), HashSet::new()));
        }
        problematic_paths
    }

    fn traverse(
        &self,
        from: usize,
        path: Vec<String>,
        traversed_edges: HashSet<(usize, usize)>,
    ) -> Vec<Vec<String>> {
        let mut problematic_paths = Vec::new();
        let (from_path, using_pragma) = {
            let node = &self.nodes[from];
            (&node.path, node.custom_gates_pragma)
        };
        let new_path = {
            let mut new_path = path.clone();
            new_path.push(from_path.clone());
            new_path
        };
        if !using_pragma {
            problematic_paths.push(new_path.clone());
        }
        if let Some(edges) = self.adjacency.get(from_path) {
            for to in edges {
                let edge = (from, *to);
                if !traversed_edges.contains(&edge) {
                    let new_traversed_edges = {
                        let mut new_traversed_edges = traversed_edges.clone();
                        new_traversed_edges.insert(edge);
                        new_traversed_edges
                    };
                    problematic_paths.append(&mut self.traverse(
                        *to,
                        new_path.clone(),
                        new_traversed_edges,
                    ));
                }
            }
        }
        problematic_paths
    }

    pub fn display_path(path: &Vec<String>) -> String {
        let path = path.clone();
        let mut path_covered = format!("{}", path[0]);
        for file in &path[1..] {
            path_covered = format!("{} -> {}", path_covered, file);
        }
        path_covered
    }
}

