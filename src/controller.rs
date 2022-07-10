use toybox::prelude::*;
use crate::view_model::ViewModel;
use crate::model::Model;
use input::raw::{Scancode, MouseButton};

toybox::declare_input_context! {
	struct Actions "" {
		trigger quit { "Quit" [Scancode::Escape] }

		trigger reset_view { "Reset View" [Scancode::Space] }
		trigger create_node { "Create Node" [Scancode::C] }
		trigger delete_node { "Delete Node" [MouseButton::Right] }

		trigger focus_node { "Focus Node" [Scancode::F] }

		trigger zoom_in { "Zoom In" [Scancode::KpPlus] }
		trigger zoom_out { "Zoom Out" [Scancode::KpMinus] }

		state pan { "Pan Camera" [MouseButton::Left] }

		pointer mouse { "Mouse" }
	}
}

toybox::declare_input_context! {
	struct MouseActions "" {
		mouse mouse { "Mouse" [1.0] }
	}
}


pub struct Controller {
	actions: Actions,
	mouse_actions: MouseActions,
	pub quit_requested: bool,
}

impl Controller {
	pub fn new(engine: &mut toybox::Engine) -> Controller {
		Controller {
			actions: Actions::new_active(&mut engine.input),
			mouse_actions: MouseActions::new(&mut engine.input),
			quit_requested: false,
		}
	}

	pub fn update(&mut self, engine: &mut toybox::Engine, view_model: &mut ViewModel, model: &mut Model) {
		let input_frame = engine.input.frame_state();

		if input_frame.active(self.actions.quit) {
			self.quit_requested = true;
		}

		if input_frame.active(self.actions.reset_view) {
			view_model.reset(model);
		}

		let view_to_world = view_model.inverse_view_matrix();
		let world_space_mouse = input_frame.mouse(self.actions.mouse)
			.map(|view_space| view_to_world * view_space);

		let hovered_node = world_space_mouse
			.and_then(|mouse_pos| {
				view_model.node_rects()
					.find(|(_, aabb)| aabb.contains_point(mouse_pos))
					.map(|(index, _)| index)
				});

		view_model.set_hovered_node(hovered_node);

		if input_frame.active(self.actions.create_node) {
			if let Some(mouse_pos) = world_space_mouse {
				let new_node = model.graph.add_node(crate::model::Node{ color: Color::rgb(1.0, 0.0, 1.0) });

				let mut projection = crate::view_model::GraphProjection::from_graph(&model.graph);
				projection.update_position(new_node, |_| mouse_pos);
				view_model.set_projection(projection);
			}
		}

		if input_frame.active(self.actions.delete_node) {
			if let Some(node_index) = hovered_node {
				model.graph.remove_node(node_index);
				view_model.set_projection(crate::view_model::GraphProjection::from_graph(&model.graph));
			}
		}

		if input_frame.active(self.actions.focus_node) {
			if let Some(node_index) = hovered_node {
				let projection = crate::view_model::GraphProjection::from_subgraph(&model.graph, node_index, 1);
				view_model.set_projection(projection);
			}
		}

		if input_frame.active(self.actions.zoom_in) {
			view_model.zoom_camera(1);
		}

		if input_frame.active(self.actions.zoom_out) {
			view_model.zoom_camera(-1);
		}

		// TODO(pat.m): this sucks - having to juggle contexts to implement dragging is annoying.
		// there needs to be another simpler way
		let pan_started = input_frame.entered(self.actions.pan);
		let pan_ended = input_frame.left(self.actions.pan);

		if pan_started {
			engine.input.enter_context(self.mouse_actions.context_id());
		}

		if pan_ended {
			engine.input.leave_context(self.mouse_actions.context_id());
		}

		let input_frame = engine.input.frame_state();

		if let Some(mouse_delta) = input_frame.mouse(self.mouse_actions.mouse) {
			let drawable_size = engine.gfx.backbuffer_size().to_vec2();
			// TODO(pat.m): this sucks - it should be possible to reconstruct the absolute pixel delta
			// from the input system without access to magic constants
			let mouse_delta_screen = (mouse_delta * 100.0) / drawable_size.y;

			view_model.pan_camera(mouse_delta_screen);
		}
	}
}