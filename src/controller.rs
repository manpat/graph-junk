use toybox::prelude::*;
use crate::view_model::ViewModel;
use crate::model::Model;

toybox::declare_input_context! {
	struct Actions "" {
		trigger quit { "Quit" [input::raw::Scancode::Escape] }

		trigger reset_view { "Reset View" [input::raw::Scancode::Space] }
		trigger create_node { "Create Node" [input::raw::Scancode::C] }
		trigger delete_node { "Delete Node" [input::raw::MouseButton::Right] }

		trigger zoom_in { "Zoom In" [input::raw::Scancode::KpPlus] }
		trigger zoom_out { "Zoom Out" [input::raw::Scancode::KpMinus] }

		state pan { "Pan Camera" [input::raw::MouseButton::Middle] }

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

		if input_frame.active(self.actions.create_node) {
			if let Some(mouse_pos_view) = input_frame.mouse(self.actions.mouse) {
				let view_to_world = view_model.inverse_view_matrix();

				// TODO(pat.m): this sucks
				let mouse_pos_world = (view_to_world * mouse_pos_view.extend(0.0).extend(1.0)).to_vec3().to_xy();

				let new_node = model.graph.add_node(crate::model::Node{ color: Color::rgb(1.0, 0.0, 1.0) });

				let mut projection = crate::view_model::GraphProjection::from_graph(&model.graph);
				projection.update_position(new_node, |_| mouse_pos_world);
				view_model.set_projection(projection);
			}
		}

		if input_frame.active(self.actions.delete_node) {
			if let Some(mouse_pos_view) = input_frame.mouse(self.actions.mouse) {
				let view_to_world = view_model.inverse_view_matrix();

				// TODO(pat.m): this sucks
				let mouse_pos_world = (view_to_world * mouse_pos_view.extend(0.0).extend(1.0)).to_vec3().to_xy();

				let maybe_rect = view_model.node_rects()
					.find(|(_, aabb)| aabb.contains_point(mouse_pos_world));

				if let Some((index, _)) = maybe_rect {
					dbg!(index);
					model.graph.remove_node(index);
					view_model.set_projection(crate::view_model::GraphProjection::from_graph(&model.graph));
				}
			}
		}

		if input_frame.active(self.actions.zoom_in) {
			view_model.zoom_camera(1);
		}

		if input_frame.active(self.actions.zoom_out) {
			view_model.zoom_camera(-1);
		}

		// TODO(pat.m): this sucks
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
			view_model.pan_camera(-mouse_delta);
		}
	}
}