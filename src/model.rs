use toybox::prelude::*;

pub type Graph = petgraph::stable_graph::StableGraph<Node, ()>;

pub struct Node {
	pub color: Color,
}


pub struct Model {
	pub graph: Graph,
}


pub fn new_model() -> Model {
	let mut graph = Graph::new();

	let node_in = graph.add_node(Node{ color: Color::white() });
	let node_out_1 = graph.add_node(Node{ color: Color::rgb(0.5, 0.5, 0.5) });
	let node_out_2 = graph.add_node(Node{ color: Color::rgb(0.2, 0.2, 0.2) });

	let node_0 = graph.add_node(Node{ color: Color::rgb(1.0, 0.5, 0.5) });
	let node_1 = graph.add_node(Node{ color: Color::rgb(0.5, 1.0, 0.5) });
	let node_2 = graph.add_node(Node{ color: Color::rgb(0.5, 0.5, 1.0) });
	let node_3 = graph.add_node(Node{ color: Color::rgb(0.5, 1.0, 1.0) });
	let node_4 = graph.add_node(Node{ color: Color::rgb(0.5, 1.0, 1.0) });

	graph.add_edge(node_in, node_0, ());
	graph.add_edge(node_in, node_1, ());
	graph.add_edge(node_in, node_4, ());
	graph.add_edge(node_0, node_out_1, ());
	graph.add_edge(node_1, node_2, ());
	graph.add_edge(node_2, node_3, ());
	graph.add_edge(node_3, node_out_1, ());
	graph.add_edge(node_4, node_out_1, ());
	graph.add_edge(node_0, node_out_2, ());

	Model {
		graph
	}
}