use anyhow::{anyhow, Result};
use serde::Serialize;
use wr::db;

#[derive(Serialize)]
struct GraphNode {
    id: String,
    title: String,
    status: String,
    priority: i32,
}

#[derive(Serialize)]
struct GraphEdge {
    from: String,
    to: String,
}

#[derive(Serialize)]
struct Graph {
    nodes: Vec<GraphNode>,
    edges: Vec<GraphEdge>,
}

pub fn run(format: Option<&str>) -> Result<()> {
    // Graph only supports json and dot formats
    match format {
        Some("json") | None => {}
        Some("dot") => return Err(anyhow!("dot format not yet implemented")),
        Some("table") => {
            return Err(anyhow!(
                "graph does not support table format. Use: json, dot"
            ))
        }
        Some(other) => return Err(anyhow!("Invalid format: {}. Valid: json, dot", other)),
    }

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

    println!("{}", serde_json::to_string(&graph)?);
    Ok(())
}
