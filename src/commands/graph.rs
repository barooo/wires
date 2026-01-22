use anyhow::{anyhow, Result};
use serde::Serialize;
use wr::db;
use wr::models::WireId;

#[derive(Serialize)]
struct GraphNode {
    id: WireId,
    title: String,
    status: String,
    priority: i32,
}

#[derive(Serialize)]
struct GraphEdge {
    from: WireId,
    to: WireId,
}

#[derive(Serialize)]
struct Graph {
    nodes: Vec<GraphNode>,
    edges: Vec<GraphEdge>,
}

pub fn run(format: Option<&str>) -> Result<()> {
    let conn = db::open()?;

    // Get all wires as nodes
    let wires = db::list_wires(&conn, None)?;
    let nodes: Vec<GraphNode> = wires
        .iter()
        .map(|w| GraphNode {
            id: w.id.clone(),
            title: w.title.clone(),
            status: w.status.as_str().to_string(),
            priority: w.priority,
        })
        .collect();

    // Get all dependencies as edges
    let mut stmt = conn.prepare("SELECT wire_id, depends_on FROM dependencies")?;
    let edges: Vec<GraphEdge> = stmt
        .query_map([], |row| {
            Ok(GraphEdge {
                from: row.get(0)?,
                to: row.get(1)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let graph = Graph { nodes, edges };

    match format {
        Some("dot") => print_dot(&graph),
        Some("json") | None => println!("{}", serde_json::to_string(&graph)?),
        Some("table") => {
            return Err(anyhow!(
                "graph does not support table format. Use: json, dot"
            ))
        }
        Some(other) => return Err(anyhow!("Invalid format: {}. Valid: json, dot", other)),
    }

    Ok(())
}

fn print_dot(graph: &Graph) {
    println!("digraph wires {{");
    println!("    rankdir=LR;");
    println!("    node [shape=box];");

    for node in &graph.nodes {
        // Escape quotes in title for DOT format
        let escaped_title = node.title.replace('"', "\\\"");
        println!(
            "    \"{}\" [label=\"{}\\n{}\"];",
            node.id.as_str(),
            escaped_title,
            node.status
        );
    }

    for edge in &graph.edges {
        println!(
            "    \"{}\" -> \"{}\";",
            edge.from.as_str(),
            edge.to.as_str()
        );
    }

    println!("}}");
}
