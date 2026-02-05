use std::ffi::CString;
use tracing::{info, error};
use gl::types::*;

pub struct SimpleRenderer {
    program: GLuint,
    vao: GLuint,
    vbo: GLuint,
}

impl SimpleRenderer {
    pub fn new() -> Self {
        // Shaders
        let vs_src = r#"
            #version 330 core
            layout (location = 0) in vec3 aPos;
            void main() {
                gl_Position = vec4(aPos.x, aPos.y, aPos.z, 1.0);
            }
        "#;
        let fs_src = r#"
            #version 330 core
            out vec4 FragColor;
            void main() {
                FragColor = vec4(1.0, 0.5, 0.2, 1.0);
            }
        "#;

        let program = unsafe { create_shader_program(vs_src, fs_src) };

        // Geometry (Square made of 2 triangles)
        let vertices: [f32; 18] = [
            // first triangle
             0.5,  0.5, 0.0,  // top right
             0.5, -0.5, 0.0,  // bottom right
            -0.5,  0.5, 0.0,  // top left
            // second triangle
             0.5, -0.5, 0.0,  // bottom right
            -0.5, -0.5, 0.0,  // bottom left
            -0.5,  0.5, 0.0   // top left
        ];

        let mut vao = 0;
        let mut vbo = 0;

        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);

            gl::BindVertexArray(vao);

            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * std::mem::size_of::<f32>()) as GLsizeiptr,
                vertices.as_ptr() as *const _,
                gl::STATIC_DRAW
            );

            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 3 * std::mem::size_of::<f32>() as GLsizei, std::ptr::null());
            gl::EnableVertexAttribArray(0);

            gl::BindBuffer(gl::ARRAY_BUFFER, 0); // Unbind VBO
            gl::BindVertexArray(0); // Unbind VAO
        }

        info!("SimpleRenderer: Initialized.");
        Self { program, vao, vbo }
    }

    pub fn render(&self) {
        unsafe {
            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::UseProgram(self.program);
            gl::BindVertexArray(self.vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
        }
    }

    pub fn resize(&self, width: i32, height: i32) {
        unsafe {
            gl::Viewport(0, 0, width, height);
        }
    }
}

impl Drop for SimpleRenderer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteProgram(self.program);
        }
    }
}

unsafe fn create_shader_program(vs_src: &str, fs_src: &str) -> GLuint {
    let vs = unsafe { compile_shader(vs_src, gl::VERTEX_SHADER) };
    let fs = unsafe { compile_shader(fs_src, gl::FRAGMENT_SHADER) };
    let program = unsafe { gl::CreateProgram() };
    unsafe {
        gl::AttachShader(program, vs);
        gl::AttachShader(program, fs);
        gl::LinkProgram(program);
    }
    
    let mut success = 0;
    unsafe { gl::GetProgramiv(program, gl::LINK_STATUS, &mut success); }
    if success == 0 {
         let mut len = 0;
        unsafe { gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len); }
        let mut buffer = Vec::with_capacity(len as usize);
        unsafe { 
            buffer.set_len((len as usize) - 1); 
            gl::GetProgramInfoLog(program, len, std::ptr::null_mut(), buffer.as_mut_ptr() as *mut GLchar);
        }
         error!("Shader Link Error: {}", String::from_utf8_lossy(&buffer));
    }
    
    unsafe {
        gl::DeleteShader(vs);
        gl::DeleteShader(fs);
    }
    program
}

unsafe fn compile_shader(src: &str, shader_type: GLenum) -> GLuint {
    let shader = unsafe { gl::CreateShader(shader_type) };
    let c_str = CString::new(src).unwrap();
    unsafe {
        gl::ShaderSource(shader, 1, &c_str.as_ptr(), std::ptr::null());
        gl::CompileShader(shader);
    }
    
     let mut success = 0;
    unsafe { gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success); }
    if success == 0 {
        let mut len = 0;
        unsafe { gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len); }
        let mut buffer = Vec::with_capacity(len as usize);
        unsafe {
            buffer.set_len((len as usize) - 1); 
            gl::GetShaderInfoLog(shader, len, std::ptr::null_mut(), buffer.as_mut_ptr() as *mut GLchar);
        }
        error!("Shader Compile Error: {}", String::from_utf8_lossy(&buffer));
    }
    shader
}