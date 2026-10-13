use std::{collections::HashMap, rc::Rc};

use glam::Vec2;
use glow::HasContext;

pub(crate) struct Shader {
    gl: Rc<glow::Context>,
    program: glow::Program,
    uniforms: HashMap<String, glow::UniformLocation>,
}

impl Shader {
    pub(crate) fn load(
        gl: Rc<glow::Context>,
        vertex: &str,
        fragment: &str,
    ) -> Result<Self, String> {
        let vertex = Self::compile_shader(&gl, ShaderType::Vertex, vertex)?;
        let fragment = match Self::compile_shader(&gl, ShaderType::Fragment, fragment) {
            Ok(fragment) => fragment,
            Err(error) => {
                unsafe { gl.delete_shader(vertex) };
                return Err(error);
            }
        };
        let program = Self::link_program(&gl, vertex, fragment)?;
        let uniforms = Self::get_uniforms(&gl, program);

        Ok(Self {
            gl,
            program,
            uniforms,
        })
    }

    fn compile_shader(
        gl: &glow::Context,
        kind: ShaderType,
        source: &str,
    ) -> Result<glow::Shader, String> {
        let shader = unsafe { gl.create_shader(kind.gl())? };
        unsafe {
            gl.shader_source(shader, source);
            gl.compile_shader(shader);
            if !gl.get_shader_compile_status(shader) {
                let log = gl.get_shader_info_log(shader);
                gl.delete_shader(shader);
                return Err(log);
            }
        }
        Ok(shader)
    }

    fn link_program(
        gl: &glow::Context,
        vertex: glow::Shader,
        fragment: glow::Shader,
    ) -> Result<glow::Program, String> {
        let program = match unsafe { gl.create_program() } {
            Ok(program) => program,
            Err(error) => {
                unsafe {
                    gl.delete_shader(vertex);
                    gl.delete_shader(fragment);
                }
                return Err(error);
            }
        };
        let link_result = unsafe {
            gl.attach_shader(program, vertex);
            gl.attach_shader(program, fragment);
            gl.link_program(program);
            if gl.get_program_link_status(program) {
                Ok(())
            } else {
                Err(gl.get_program_info_log(program))
            }
        };
        unsafe {
            gl.detach_shader(program, vertex);
            gl.detach_shader(program, fragment);
            gl.delete_shader(vertex);
            gl.delete_shader(fragment);
        }
        if let Err(error) = link_result {
            unsafe { gl.delete_program(program) };
            return Err(error);
        }
        Ok(program)
    }

    fn get_uniforms(
        gl: &glow::Context,
        program: glow::Program,
    ) -> HashMap<String, glow::UniformLocation> {
        let count = unsafe { gl.get_program_parameter_i32(program, glow::ACTIVE_UNIFORMS) };
        let mut uniforms = HashMap::with_capacity(count as usize);
        for i in 0..count {
            let Some(uniform) = (unsafe { gl.get_active_uniform(program, i as u32) }) else {
                continue;
            };
            let Some(location) = (unsafe { gl.get_uniform_location(program, &uniform.name) })
            else {
                continue;
            };
            uniforms.insert(uniform.name, location);
        }
        uniforms
    }

    pub(crate) fn bind(&self) {
        unsafe {
            self.gl.use_program(Some(self.program));
        }
    }

    pub(crate) fn set_i32(&self, name: &str, value: i32) {
        let loc = self.uniforms.get(name);
        unsafe {
            self.gl.uniform_1_i32(loc, value);
        }
    }

    pub(crate) fn set_f32(&self, name: &str, value: f32) {
        let loc = self.uniforms.get(name);
        unsafe {
            self.gl.uniform_1_f32(loc, value);
        }
    }

    pub(crate) fn set_vec2(&self, name: &str, value: Vec2) {
        let loc = self.uniforms.get(name);
        unsafe {
            self.gl.uniform_2_f32(loc, value.x, value.y);
        }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe { self.gl.delete_program(self.program) };
    }
}

enum ShaderType {
    Vertex,
    Fragment,
}

impl ShaderType {
    fn gl(&self) -> u32 {
        match self {
            Self::Vertex => glow::VERTEX_SHADER,
            Self::Fragment => glow::FRAGMENT_SHADER,
        }
    }
}
