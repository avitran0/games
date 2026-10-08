use std::rc::Rc;

use glam::{vec2, Vec2};
use glow::HasContext;

pub(crate) struct Quad {
    gl: Rc<glow::Context>,
    vertex_array: glow::VertexArray,
    vertices: glow::Buffer,
}

impl Quad {
    pub(crate) fn new(gl: Rc<glow::Context>) -> Result<Self, String> {
        let vertex_array = unsafe { gl.create_vertex_array()? };
        let vertices = unsafe { gl.create_buffer()? };

        unsafe {
            gl.bind_vertex_array(Some(vertex_array));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vertices));

            let vertices_raw = [
                vec2(0.0, 0.0),
                vec2(0.0, 1.0),
                vec2(1.0, 1.0),
                vec2(0.0, 0.0),
                vec2(1.0, 1.0),
                vec2(1.0, 0.0),
            ];
            let bytes = std::slice::from_raw_parts(
                vertices_raw.as_ptr().cast(),
                size_of_val(&vertices_raw),
            );
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, bytes, glow::STATIC_DRAW);
            gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, size_of::<Vec2>() as i32, 0);
            gl.enable_vertex_attrib_array(0);

            gl.bind_buffer(glow::ARRAY_BUFFER, None);
            gl.bind_vertex_array(None);
        }

        Ok(Self {
            gl,
            vertex_array,
            vertices,
        })
    }

    pub(crate) fn draw(&self) {
        unsafe {
            self.gl.bind_vertex_array(Some(self.vertex_array));
            self.gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vertices));
            self.gl.draw_arrays(glow::TRIANGLES, 0, 6);
        }
    }
}

impl Drop for Quad {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_vertex_array(self.vertex_array);
            self.gl.delete_buffer(self.vertices);
        }
    }
}
