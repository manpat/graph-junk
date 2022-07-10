use std::collections::HashMap;
use toybox::prelude::*;
use petgraph::graph::NodeIndex;
use gfx::vertex::ColorVertex2D;

use crate::model::Model;


const COHESION_PRESSURE: f32 = 0.0;
const OUTWARD_EXTERNALS_PRESSURE: f32 = 10.0;

const REPULSION_FACTOR: f32 = 20.0;
const REPULSION_DISTANCE: f32 = 0.5;

const REORDER_PRESSURE: f32 = 1.0;

const NEIGHBOR_COHESION_FACTOR: f32 = 1.0;
const NEIGHBOR_COHESION_DISTANCE: f32 = 0.6;

const CENTER_OF_MASS_CORRECTIVE_FACTOR: f32 = 1.0;

const NODE_SIZE: f32 = 0.25;



pub struct ViewModel {
	projection: GraphProjection,

	camera_pos: Vec2,
	camera_zoom: f32,
}

impl ViewModel {
	pub fn new(model: &Model) -> ViewModel {
		let mut projection = GraphProjection::from_graph(&model.graph);

		for _ in 0..200 {
			simulate(&mut projection, &model.graph);
		}

		ViewModel {
			projection,
			camera_pos: Vec2::zero(),
			camera_zoom: 0.0,
		}
	}

	pub fn update(&mut self, model: &Model) {
		for _ in 0..20 {
			simulate(&mut self.projection, &model.graph);
		}
	}

	pub fn reset(&mut self, model: &Model) {
		self.projection = GraphProjection::from_graph(&model.graph);
	}

	pub fn set_projection(&mut self, mut projection: GraphProjection) {
		projection.copy_projections_from(&self.projection);
		self.projection = projection;
	}

	pub fn projection(&self) -> &GraphProjection {
		&self.projection
	}

	pub fn node_rects(&self) -> impl Iterator<Item=(NodeIndex, Aabb2)> + '_ {
		self.projection.iter()
			.map(|(index, node)| {
				let aabb = Aabb2::around_point(node.pos, Vec2::splat(NODE_SIZE / 2.0));
				(index, aabb)
			})
	}

	fn scale_factor(&self) -> f32 { 2.0f32.powf(self.camera_zoom) }

	pub fn pan_camera(&mut self, screen_delta: Vec2) {
		// TODO(pat.m): why is it 2.0
		let world_delta = -2.0 * screen_delta * self.scale_factor();
		self.camera_pos += world_delta;
	}

	pub fn zoom_camera(&mut self, ticks: i32) {
		self.camera_zoom -= ticks as f32 / 3.0;
	}

	pub fn view_matrix(&self) -> Mat2x3 {
		let scale = Vec2::splat(1.0 / self.scale_factor());
		Mat2x3::scale(scale)
			* Mat2x3::translate(-self.camera_pos)
	}

	pub fn inverse_view_matrix(&self) -> Mat2x3 {
		self.view_matrix().inverse()
	}
}



pub fn build_nodes(mesh_data: &mut gfx::mesh::MeshData<gfx::vertex::ColorVertex2D>, graph_projection: &GraphProjection, model: &Model) {
	use gfx::mesh::*;

	let mut mb = ColorMeshBuilder::new(mesh_data);

	for (index, projection) in graph_projection.iter() {
		let node = &model.graph[index];

		mb.set_color(node.color);
		mb.build(node_geom(projection.pos));
	}
}


pub fn build_lines(line_builder: &mut LineBuilder2D, graph_projection: &GraphProjection, model: &Model) {
	use petgraph::Direction;

	for (index, projection) in graph_projection.iter() {
		for neighbor_index in model.graph.neighbors_directed(index, Direction::Outgoing) {
			if let Some(neighbor_pos) = graph_projection.position(neighbor_index) {
				let dir = (neighbor_pos - projection.pos).normalize();
				let left = dir.perp();

				let end = neighbor_pos - dir * 0.15;
				let arrow_size = 0.02;
				let arrow_end = end - dir * arrow_size;

				line_builder.add(projection.pos, end, Color::white());
				line_builder.add(arrow_end - left * arrow_size, end, Color::white());
				line_builder.add(arrow_end + left * arrow_size, end, Color::white());
			} else {
				line_builder.add(projection.pos, projection.pos + Vec2::from_x(0.5), Color::grey(0.5));
			}
		}
	}
}

fn node_geom(pos: Vec2) -> gfx::mesh::geom::Quad {
	let txform = Mat2x3::scale_translate(Vec2::splat(NODE_SIZE), pos);
	gfx::mesh::geom::Quad::from_matrix(txform)
}


fn rand_vec() -> Vec2 {
	use rand::Rng;

	let mut rng = rand::thread_rng();
	Vec2::new(rng.gen(), rng.gen()) * 2.0 - Vec2::splat(1.0)
}






pub struct LineBuilder2D {
	vertices: Vec<ColorVertex2D>,
}

impl LineBuilder2D {
	pub fn new() -> LineBuilder2D {
		LineBuilder2D {
			vertices: Vec::new(),
		}
	}

	pub fn clear(&mut self) {
		self.vertices.clear();
	}

	pub fn upload(&self, mesh: &mut gfx::mesh::BasicMesh<ColorVertex2D>) {
		mesh.upload(&self.vertices);
	}

	pub fn add(&mut self, start: Vec2, end: Vec2, color: Color) {
		self.vertices.push(ColorVertex2D::new(start, color));
		self.vertices.push(ColorVertex2D::new(end, color));
		self.vertices.push(ColorVertex2D::new(Vec2::splat(f32::NAN), Color::black()));
	}
}





#[derive(Copy, Clone)]
pub struct NodeProjection {
	pub pos: Vec2,
}

pub struct GraphProjection {
	map: HashMap<NodeIndex, NodeProjection>,
}

impl GraphProjection {
	pub fn new() -> GraphProjection {
		GraphProjection {
			map: HashMap::new(),
		}
	}

	pub fn from_nodes(it: impl Iterator<Item=NodeIndex>) -> GraphProjection {
		GraphProjection {
			map: it.map(|index| (index, NodeProjection { pos: Vec2::zero() }))
				.collect()
		}
	}

	pub fn from_graph<G>(graph: G) -> GraphProjection
		where G: petgraph::visit::IntoNodeIdentifiers + petgraph::visit::GraphBase<NodeId=NodeIndex>
	{
		GraphProjection {
			map: graph.node_identifiers()
				.map(|index| (index, NodeProjection { pos: Vec2::zero() }))
				.collect()
		}
	}

	pub fn copy_projections_from(&mut self, other: &GraphProjection) {
		for (index, projection) in self.map.iter_mut() {
			if let Some(&other_projection) = other.map.get(&index) {
				*projection = other_projection;
			}
		}
	}

	pub fn iter(&self) -> impl Iterator<Item=(NodeIndex, &NodeProjection)> {
		self.map.iter()
			.map(|(idx, proj)| (*idx, proj))
	}

	pub fn iter_mut(&mut self) -> impl Iterator<Item=(NodeIndex, &mut NodeProjection)> {
		self.map.iter_mut()
			.map(|(idx, proj)| (*idx, proj))
	}

	pub fn position(&self, index: NodeIndex) -> Option<Vec2> {
		self.map.get(&index)
			.map(|proj| proj.pos)
	}

	pub fn update_position(&mut self, index: NodeIndex, f: impl FnOnce(Vec2) -> Vec2) {
		if let Some(projection) = self.map.get_mut(&index) {
			projection.pos = f(projection.pos);
		}
	}
}




fn simulate(graph_projection: &mut GraphProjection, model: &crate::model::Graph) {
	use petgraph::Direction;

	let mut velocities: HashMap<_, _> = graph_projection.iter()
		.map(|(index, _)| (index, Vec2::zero()))
		.collect();

	// Give small inwards pressure
	for (&index, velocity) in velocities.iter_mut() {
		let position = graph_projection.position(index).unwrap();
		*velocity -= position * COHESION_PRESSURE;
	}

	// TODO(pat.m): base target coordinate on width of graph
	// TODO(pat.m): these need to be the externals of the projection, not the whole graph
	// TODO(pat.m): exclude islands
	for node_index in model.externals(Direction::Incoming) {
		if let Some(pos) = graph_projection.position(node_index) {
			let diff = -1.0 - pos.x;
			*velocities.get_mut(&node_index).unwrap() += Vec2::from_x(diff * OUTWARD_EXTERNALS_PRESSURE);
		}
	}

	for node_index in model.externals(Direction::Outgoing) {
		if let Some(pos) = graph_projection.position(node_index) {
			let diff = 1.0 - pos.x;
			*velocities.get_mut(&node_index).unwrap() += Vec2::from_x(diff * OUTWARD_EXTERNALS_PRESSURE);
		}
	}

	for (node_index, node_projection) in graph_projection.iter() {
		let node_pos = node_projection.pos;
		let mut repulsion_force = Vec2::zero();
		let mut cohesion_force = Vec2::zero();
		let mut reorder_force = Vec2::zero();

		// Repulse nearby nodes
		for neighbor_index in model.node_indices() {
			if neighbor_index == node_index {
				continue
			}

			let neighbor_pos = match graph_projection.position(neighbor_index) {
				Some(p) => p,
				_ => continue
			};

			let diff = node_pos - neighbor_pos;
			let dist = diff.length();

			if dist < 0.01 {
				repulsion_force += rand_vec() * 0.2;

			} else if dist < REPULSION_DISTANCE {
				let dir = diff / dist;
				repulsion_force += dir * (1.0 - dist / REPULSION_DISTANCE).clamp(0.0, 1.0).powi(2);
			}

			match model.find_edge_undirected(node_index, neighbor_index) {
				Some((_, Direction::Outgoing)) => if diff.x < 0.0 {
					reorder_force -= Vec2::from_x(diff.x * REORDER_PRESSURE);
				}

				Some((_, Direction::Incoming)) => if diff.x > 0.0 {
					reorder_force -= Vec2::from_x(diff.x * REORDER_PRESSURE);
				}

				_ => {}
			};
		}

		// Attract neighbors
		for neighbor_index in model.neighbors_undirected(node_index) {
			let neighbor_pos = match graph_projection.position(neighbor_index) {
				Some(p) => p,
				_ => continue
			};

			let diff = neighbor_pos - node_pos;
			let dist = diff.length();

			if dist > NEIGHBOR_COHESION_DISTANCE {
				let dir = diff / dist;
				cohesion_force += dir * (dist - NEIGHBOR_COHESION_DISTANCE);
			}
		}

		let velocity = velocities.get_mut(&node_index).unwrap();
		*velocity += repulsion_force * REPULSION_FACTOR + cohesion_force * NEIGHBOR_COHESION_FACTOR;
	}


	// TODO(pat.m): add pressure to encourage connections to always be to the right


	let node_pos_sum: Vec2 = graph_projection.iter()
		.map(|(_, n)| n.pos)
		.sum();

	let node_center = node_pos_sum / velocities.len() as f32;

	// Push whole graph towards the center
	for (_, node_velocity) in velocities.iter_mut() {
		*node_velocity -= node_center * CENTER_OF_MASS_CORRECTIVE_FACTOR;
	}

	// Apply forces
	for (&index, &(mut velocity)) in velocities.iter() {
		let speed = velocity.length();
		if speed > 25.0 {
			velocity = velocity / speed * 25.0;
		}

		graph_projection.update_position(index, |old_pos| old_pos + velocity / 50.0);
	}
}