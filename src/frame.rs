extern crate glfw;
use glfw::{Glfw, Window, Action, Context, Key, WindowEvent};
use std::io;
use std::mem::{size_of, size_of_val};
use std::sync::mpsc::Receiver;
use gl33::*;

/////////////////////////////////////////////////////
/////////////////////////////////////////////////////
/////////////////////////////////////////////////////

type Vertex = [f32; 2];

const VERTICES: [Vertex; 6] =
    [   [-1.0, -1.0], [1.0, -1.0], [-1.0, 1.0],
        [-1.0, 1.0], [1.0, -1.0], [1.0, 1.0]    ];

const VERT_SHADER: &str = r#"#version 330 core
    layout (location = 0) in vec2 pos;
    void main()
    {
        gl_Position = vec4(pos.x, pos.y, 0.0, 1.0);
    }
"#;

const FRAG_SHADER: &str = r#"#version 330 core
    out vec4 final_color;
    void main()
    {
        vec2 uv = gl_FragCoord.xy / 300.0;
        final_color = vec4(cos(uv.x), sin(uv.y), cos(uv.x) * sin(uv.y), 1.0) * 0.5 + 0.5;
    }
"#;

/////////////////////////////////////////////////////
/////////////////////////////////////////////////////
/////////////////////////////////////////////////////

pub struct MainFrame
{
    m_glfw:Glfw,
}

impl MainFrame
{
    pub fn new() -> Self
    {
        glfw::WindowHint::ContextVersion(3, 3); // opengl 3.2
        Self {
            m_glfw: glfw::init(glfw::FAIL_ON_ERRORS).unwrap()
        }
    }

    pub fn display(&mut self, width:u32, height:u32)
    {
        let (mut window, events) =
            self.m_glfw.create_window(width, height, "rsPipe Game", glfw::WindowMode::Windowed)
            .expect("Failed to create GLFW window.");

        window.make_current();
        window.set_key_polling(true);

        let gl = self.prepare();

        while !window.should_close()
        {
            // Poll for and process events
            self.m_glfw.poll_events();
            for (_, event) in glfw::flush_messages(&events)
            {
                println!("{:?}", event);
                match event {
                    glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                        window.set_should_close(true)
                    },
                    _ => {},
                }
            }

            unsafe {
                gl.Clear(GL_COLOR_BUFFER_BIT);
                gl.DrawArrays(GL_TRIANGLES, 0, 6);
            }

            window.swap_buffers();
        }
    }

    fn prepare(&mut self) -> GlFns
    {
        let gl = unsafe {
            GlFns::load_from(&|p| {
                let c_str = std::ffi::CStr::from_ptr(p as *const i8);
                let rust_str = c_str.to_str().unwrap();
                self.m_glfw.get_proc_address_raw(rust_str) as _
            }).unwrap()
        };

        unsafe {
            gl.ClearColor(0.2, 0.3, 0.3, 1.0);

            let mut vao = 0;
            gl.GenVertexArrays(1, &mut vao);
            assert_ne!(vao, 0);
            gl.BindVertexArray(vao);

            let mut vbo = 0;
            gl.GenBuffers(1, &mut vbo);
            assert_ne!(vbo, 0);
            gl.BindBuffer(GL_ARRAY_BUFFER, vbo);
            gl.BufferData(
                GL_ARRAY_BUFFER,
                size_of_val(&VERTICES) as isize,
                VERTICES.as_ptr().cast(),
                GL_STATIC_DRAW,
            );

            gl.VertexAttribPointer(
                0,
                2,
                GL_FLOAT,
                0,
                size_of::<Vertex>().try_into().unwrap(),
                0 as *const _,
            );
            gl.EnableVertexAttribArray(0);

            let vertex_shader = gl.CreateShader(GL_VERTEX_SHADER);
            assert_ne!(vertex_shader, 0);
            gl.ShaderSource(
                vertex_shader,
                1,
                &(VERT_SHADER.as_bytes().as_ptr().cast()),
                &(VERT_SHADER.len().try_into().unwrap()),
            );
            gl.CompileShader(vertex_shader);
            let mut success = 0;
            gl.GetShaderiv(vertex_shader, GL_COMPILE_STATUS, &mut success);
            if success == 0 {
                let mut v: Vec<u8> = Vec::with_capacity(1024);
                let mut log_len = 0_i32;
                gl.GetShaderInfoLog(
                    vertex_shader,
                    1024,
                    &mut log_len,
                    v.as_mut_ptr().cast(),
                );
                v.set_len(log_len.try_into().unwrap());
                panic!("Vertex Compile Error: {}", String::from_utf8_lossy(&v));
            }

            let fragment_shader = gl.CreateShader(GL_FRAGMENT_SHADER);
            assert_ne!(fragment_shader, 0);
            gl.ShaderSource(
                fragment_shader,
                1,
                &(FRAG_SHADER.as_bytes().as_ptr().cast()),
                &(FRAG_SHADER.len().try_into().unwrap()),
            );
            gl.CompileShader(fragment_shader);
            let mut success = 0;
            gl.GetShaderiv(fragment_shader, GL_COMPILE_STATUS, &mut success);
            if success == 0 {
                let mut v: Vec<u8> = Vec::with_capacity(1024);
                let mut log_len = 0_i32;
                gl.GetShaderInfoLog(
                    fragment_shader,
                    1024,
                    &mut log_len,
                    v.as_mut_ptr().cast(),
                );
                v.set_len(log_len.try_into().unwrap());
                panic!("Fragment Compile Error: {}", String::from_utf8_lossy(&v));
            }

            let shader_program = gl.CreateProgram();
            assert_ne!(shader_program, 0);
            gl.AttachShader(shader_program, vertex_shader);
            gl.AttachShader(shader_program, fragment_shader);
            gl.LinkProgram(shader_program);
            let mut success = 0;
            gl.GetProgramiv(shader_program, GL_LINK_STATUS, &mut success);
            if success == 0 {
                let mut v: Vec<u8> = Vec::with_capacity(1024);
                let mut log_len = 0_i32;
                gl.GetProgramInfoLog(
                    shader_program,
                    1024,
                    &mut log_len,
                    v.as_mut_ptr().cast(),
                );
                v.set_len(log_len.try_into().unwrap());
                panic!("Program Link Error: {}", String::from_utf8_lossy(&v));
            }
            gl.DeleteShader(vertex_shader);
            gl.DeleteShader(fragment_shader);

            gl.UseProgram(shader_program);
        }

        gl
    }
}