use toybox::prelude::*;
use std::error::Error;

pub mod model;
pub mod view_model;
pub mod controller;


pub mod shaders {
	pub const COLOR_2D_VERT: &'static str = include_str!("shaders/color_2d.vert.glsl");
	pub const FLAT_COLOR_FRAG: &'static str = include_str!("shaders/flat_color.frag.glsl");
}


fn main() -> Result<(), Box<dyn Error>> {
	std::env::set_var("RUST_BACKTRACE", "1");

	let mut engine = toybox::Engine::new("nodes")?;

	let mut model = model::new_model();
	let mut view_model = view_model::ViewModel::new(&model);
	let mut controller = controller::Controller::new(&mut engine);


	let shader = engine.gfx.new_simple_shader(shaders::COLOR_2D_VERT, shaders::FLAT_COLOR_FRAG)?;
	let mut uniform_buffer = engine.gfx.new_buffer(gfx::BufferUsage::Stream);


	use gfx::vertex::ColorVertex2D;
	let mut node_mesh = gfx::mesh::Mesh::new(&mut engine.gfx);
	let mut line_mesh = gfx::mesh::BasicMesh::new(&mut engine.gfx);
	let mut node_mesh_data = gfx::mesh::MeshData::<ColorVertex2D>::new();
	let mut line_builder = view_model::LineBuilder2D::new();


	unsafe {
		gfx::raw::DepthFunc(gfx::raw::LEQUAL);
	}


	'main: loop {
		engine.process_events();

		controller.update(&mut engine, &mut view_model, &mut model);

		if engine.should_quit() || controller.quit_requested {
			break 'main
		}

		{
			view_model.update(&model);

			node_mesh_data.clear();
			view_model::build_nodes(&mut node_mesh_data, &view_model, &model);
			node_mesh.upload(&node_mesh_data);
			
			line_builder.clear();
			view_model::build_lines(&mut line_builder, view_model.projection(), &model);
			line_builder.upload(&mut line_mesh);
		}

		let view_matrix = view_model.view_matrix();
		let projection_view = Mat4::ortho_aspect(1.0, engine.gfx.aspect(), -1.0, 1.0)
			* view_matrix.to_mat4_xyw();
		uniform_buffer.upload_single(&projection_view);

		let mut gfx = engine.gfx.render_state();

		gfx.set_clear_color(Color::grey_a(0.1, 0.0));
		gfx.clear(gfx::ClearMode::ALL);

		gfx.bind_shader(shader);
		gfx.bind_uniform_buffer(0, uniform_buffer);

		node_mesh.draw(&mut gfx, gfx::DrawMode::Triangles);
		line_mesh.draw(&mut gfx, gfx::DrawMode::LineStrip);

		engine.end_frame();
	}


	Ok(())
}

