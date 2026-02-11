//! Graph visualization command - export knowledge graph in DOT or JSON format.

use crate::cli::args::Args;
use crate::db::Database;
use crate::error::Result;
use owo_colors::OwoColorize;
use serde::Serialize;
use std::collections::{HashMap, HashSet};

use super::use_colors;

#[derive(Serialize)]
struct GraphOutput {
    nodes: Vec<GraphNode>,
    edges: Vec<GraphEdge>,
    stats: GraphStats,
}

#[derive(Serialize)]
struct GraphNode {
    id: String,
    label: String,
    repo: String,
}

#[derive(Serialize)]
struct GraphEdge {
    source: String,
    target: String,
}

#[derive(Serialize)]
struct GraphStats {
    total_nodes: usize,
    total_edges: usize,
    connected_nodes: usize,
    orphan_nodes: usize,
}

/// Generate knowledge graph visualization
pub fn run(format: &str, repo: Option<&str>, args: &Args) -> Result<()> {
    let db = Database::open()?;
    let colors = use_colors(args.no_color);

    // Get all links
    let links = db.get_all_links(repo)?;

    // Build node set and edges
    let mut nodes: HashSet<(String, String)> = HashSet::new(); // (path, repo)
    let mut edges: Vec<(String, String)> = Vec::new();
    let mut node_to_repo: HashMap<String, String> = HashMap::new();

    for link in &links {
        let source_id = format!("{}:{}", link.source_repo, link.source_path);
        nodes.insert((link.source_path.clone(), link.source_repo.clone()));
        node_to_repo.insert(source_id.clone(), link.source_repo.clone());

        // Target might be in any repo
        let target_id = link.target_name.clone();
        edges.push((source_id, target_id));
    }

    // Get all files to find nodes without outgoing links
    let all_files = db.get_all_file_paths()?;
    for (path, repo_name) in &all_files {
        if repo.is_none() || repo == Some(repo_name.as_str()) {
            nodes.insert((path.clone(), repo_name.clone()));
            let node_id = format!("{repo_name}:{path}");
            node_to_repo.insert(node_id, repo_name.clone());
        }
    }

    // Count connected vs orphan nodes
    let mut connected: HashSet<String> = HashSet::new();
    for (source, target) in &edges {
        connected.insert(source.clone());
        connected.insert(target.clone());
    }

    let total_nodes = nodes.len();
    let connected_count = nodes
        .iter()
        .filter(|(path, repo_name)| connected.contains(&format!("{repo_name}:{path}")))
        .count();
    let orphan_count = total_nodes - connected_count;

    match format {
        "json" => output_json(
            &nodes,
            &edges,
            &node_to_repo,
            total_nodes,
            connected_count,
            orphan_count,
        )?,
        _ => output_dot(
            &nodes,
            &edges,
            colors,
            total_nodes,
            connected_count,
            orphan_count,
        ),
    }

    Ok(())
}

fn output_json(
    nodes: &HashSet<(String, String)>,
    edges: &[(String, String)],
    _node_to_repo: &HashMap<String, String>,
    total_nodes: usize,
    connected_count: usize,
    orphan_count: usize,
) -> Result<()> {
    let graph_nodes: Vec<GraphNode> = nodes
        .iter()
        .map(|(path, repo)| {
            let id = format!("{repo}:{path}");
            GraphNode {
                id: id.clone(),
                label: path.clone(),
                repo: repo.clone(),
            }
        })
        .collect();

    let graph_edges: Vec<GraphEdge> = edges
        .iter()
        .map(|(source, target)| GraphEdge {
            source: source.clone(),
            target: target.clone(),
        })
        .collect();

    let output = GraphOutput {
        nodes: graph_nodes,
        edges: graph_edges,
        stats: GraphStats {
            total_nodes,
            total_edges: edges.len(),
            connected_nodes: connected_count,
            orphan_nodes: orphan_count,
        },
    };

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

fn output_dot(
    nodes: &HashSet<(String, String)>,
    edges: &[(String, String)],
    colors: bool,
    total_nodes: usize,
    connected_count: usize,
    orphan_count: usize,
) {
    // Output DOT format
    println!("digraph KnowledgeGraph {{");
    println!("  rankdir=LR;");
    println!("  node [shape=box, style=rounded];");
    println!();

    // Group nodes by repo
    let mut repos: HashMap<String, Vec<String>> = HashMap::new();
    for (path, repo) in nodes {
        repos.entry(repo.clone()).or_default().push(path.clone());
    }

    // Output subgraphs for each repo
    for (repo, paths) in &repos {
        println!("  subgraph \"cluster_{repo}\" {{");
        println!("    label=\"{repo}\";");
        println!("    style=filled;");
        println!("    color=lightgrey;");
        for path in paths {
            let node_id = format!("{repo}:{path}");
            let escaped_id = escape_dot_id(&node_id);
            let label = path.rsplit('/').next().unwrap_or(path);
            println!("    \"{escaped_id}\" [label=\"{label}\"];");
        }
        println!("  }}");
        println!();
    }

    // Output edges
    for (source, target) in edges {
        let escaped_source = escape_dot_id(source);
        let escaped_target = escape_dot_id(target);
        println!("  \"{escaped_source}\" -> \"{escaped_target}\";");
    }

    println!("}}");
    println!();

    // Print stats as comment
    if colors {
        eprintln!(
            "{} {} nodes, {} edges ({} connected, {} orphans)",
            "Graph:".bold(),
            total_nodes.to_string().cyan(),
            edges.len().to_string().cyan(),
            connected_count.to_string().green(),
            orphan_count.to_string().yellow()
        );
    } else {
        eprintln!(
            "Graph: {} nodes, {} edges ({} connected, {} orphans)",
            total_nodes,
            edges.len(),
            connected_count,
            orphan_count
        );
    }
}

/// Escape special characters for DOT node IDs
fn escape_dot_id(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}
