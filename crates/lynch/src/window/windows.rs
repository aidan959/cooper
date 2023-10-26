

// pub fn process_event_windows(event: Event) {
//     match event {
//         Event::WindowEvent { event, .. } => match event {
//             WindowEvent::CloseRequested => should_stop = true,
//             WindowEvent::Resized(LogicalSize { width, height }) => {
//                 resize_dimensions = Some([width as u32, height as u32]);
//             }
//             WindowEvent::MouseInput {
//                 button: MouseButton::Left,
//                 state,
//                 ..
//             } => {
//                 if state == ElementState::Pressed {
//                     is_left_clicked = Some(true);
//                 } else {
//                     is_left_clicked = Some(false);
//                 }
//             }
//             WindowEvent::CursorMoved { position, .. } => {
//                 let position: (i32, i32) = position.into();
//                 cursor_position = Some([position.0, position.1]);
//             }
//             WindowEvent::MouseWheel {
//                 delta: MouseScrollDelta::LineDelta(_, v_lines),
//                 ..
//             } => {
//                 wheel_delta = Some(v_lines);
//             }
//
//             _ => {}
//         },
//         _ => {}
//     };
//     self.resize_dimensions = resize_dimensions;
//     if let Some(is_left_clicked) = is_left_clicked {
//         self.is_left_clicked = is_left_clicked;
//     }
//     self.cursor_delta = if let Some(position) = cursor_position {
//         let last_position = self.cursor_position;
//         self.cursor_position = position;
//         Some([
//             position[0] - last_position[0],
//             position[1] - last_position[1],
//         ])
//     } else {
//         None
//     };
//     self.wheel_delta = wheel_delta;
//     should_stop;
// }
