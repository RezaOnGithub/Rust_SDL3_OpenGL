use core::{mem::transmute, ptr::null_mut};
use glad_sys::*;
use sdl3_sys::everything::*;
use std::{
    ffi::CStr,
    os::raw::c_void,
    ptr::{addr_of, addr_of_mut},
};

const VS1: &CStr = cr##"

#version 330 core

layout (location = 0) in vec3 IN_pos;

void main() {
    gl_Position = vec4(IN_pos, 1.0f);
}

"##;

const FS1: &CStr = cr##"

#version 330 core

out vec4 OUT_color;
void main() {
    OUT_color = vec4(0.1f, 0.5f, 0.0f, 1.0f);
}


"##;

extern "C" fn cb(
    _src: GLenum,
    _ty: GLenum,
    id: GLuint,
    _severity: GLenum,
    _length: GLsizei,
    message: *const i8,
    _user: *const c_void,
) {
    unsafe {
        println!(
            "DEBUG {}\t {}",
            id,
            CStr::from_ptr(message).to_str().unwrap()
        )
    }
}

fn main() {
    // println!("{}", BINDINGS);

    unsafe {
        if !SDL_Init(SDL_INIT_VIDEO | SDL_INIT_EVENTS) {
            SDL_Log(cr"Call to SDL_init failed: %s\n".as_ptr(), SDL_GetError());
            panic!("SDL initialization failed!")
        }

        // GOTCHA: Set attributes AFTER initializing SDL
        SDL_GL_SetAttribute(SDL_GL_CONTEXT_MAJOR_VERSION, 4);
        SDL_GL_SetAttribute(SDL_GL_CONTEXT_MINOR_VERSION, 6);
        SDL_GL_SetAttribute(SDL_GL_CONTEXT_FLAGS, SDL_GL_CONTEXT_DEBUG_FLAG);
        SDL_GL_SetAttribute(SDL_GL_CONTEXT_PROFILE_MASK, SDL_GL_CONTEXT_PROFILE_CORE);

        let window = SDL_CreateWindow(cr"My Amazing Game".as_ptr(), 800, 600, SDL_WINDOW_OPENGL);
        if window.is_null() {
            SDL_Log(
                cr"Call to SDL_CreateWindow failed: %s\n".as_ptr(),
                SDL_GetError(),
            );
            panic!("SDL Window initialization failed!");
        }

        let context = SDL_GL_CreateContext(window);
        if context.is_null() {
            panic!("");
        }

        if !SDL_GL_MakeCurrent(window, context) {
            panic!("");
        }
        SDL_GL_SetSwapInterval(1);

        // TODO before using an extension, verify that it is avaliable!
        gladLoadGLLoader(transmute(SDL_GL_GetProcAddress as *mut c_void));

        // WE HAVE OPENGL NOW
        let mut gl = GL::unwrap();

        (gl.Enable)(GL_DEBUG_OUTPUT);
        (gl.Enable)(GL_DEBUG_OUTPUT_SYNCHRONOUS);
        (gl.DebugMessageCallback)(Some(cb), null_mut());
        (gl.DebugMessageControl)(
            GL_DONT_CARE,
            GL_DONT_CARE,
            GL_DONT_CARE,
            0,
            null_mut(),
            GL_TRUE.try_into().unwrap(),
        );

        println!(
            "RENDERER\t{}",
            CStr::from_ptr(((gl.GetString)(GL_RENDERER)) as *const _)
                .to_str()
                .unwrap()
        );
        println!(
            "VERSION\t{}",
            CStr::from_ptr(((gl.GetString)(GL_VERSION)) as *const _)
                .to_str()
                .unwrap()
        );

        // Screw VAOs
        {
            let mut lol: u32 = 0;
            (gl.GenVertexArrays)(1, addr_of_mut!(lol));
            (gl.BindVertexArray)(lol);
        }

        let vs = (gl.CreateShader)(GL_VERTEX_SHADER);
        let vs1_len: i32 = SDL_strlen(VS1.as_ptr()).try_into().unwrap();
        let vs1_double_ptr = VS1.as_ptr();
        (gl.ShaderSource)(vs, 1, addr_of!(vs1_double_ptr), addr_of!(vs1_len));
        (gl.CompileShader)(vs);

        let fs = (gl.CreateShader)(GL_FRAGMENT_SHADER);
        let fs1_len: i32 = SDL_strlen(FS1.as_ptr()).try_into().unwrap();
        let fs1_double_ptr = FS1.as_ptr();
        (gl.ShaderSource)(fs, 1, addr_of!(fs1_double_ptr), addr_of!(fs1_len));
        (gl.CompileShader)(fs);

        let prog = (gl.CreateProgram)();
        (gl.AttachShader)(prog, vs);
        (gl.AttachShader)(prog, fs);
        (gl.LinkProgram)(prog);

        (gl.UseProgram)(prog);

        let mut tri: [[f32; 3]; 3] = [[-0.8, -0.8, 0.0], [0.8, -0.8, 0.0], [0.0, 0.8, 0.0]];

        let mut tri_vbo: u32 = 0;
        (gl.GenBuffers)(1, addr_of_mut!(tri_vbo));
        (gl.BindBuffer)(GL_ARRAY_BUFFER, tri_vbo);
        (gl.BufferData)(
            GL_ARRAY_BUFFER,
            size_of::<[[f32; 3]; 3]>().try_into().unwrap(),
            addr_of_mut!(tri) as *const c_void,
            GL_STATIC_DRAW,
        );

        (gl.EnableVertexAttribArray)(0);
        (gl.VertexAttribPointer)(
            0,
            3,
            GL_FLOAT,
            GL_FALSE.try_into().unwrap(),
            size_of::<[f32; 3]>().try_into().unwrap(),
            null_mut(),
        );

        let render = |context: &mut GL| {
            (context.ClearColor)(0.6, 0.0, 0.6, 1.0);
            (context.Clear)(GL_COLOR_BUFFER_BIT);
            (context.DrawArrays)(GL_TRIANGLES, 0, 3);
        };

        let mut should_quit: bool = false;
        while !should_quit {
            render(&mut gl);
            SDL_GL_SwapWindow(window);
            {
                let mut e: SDL_Event = Default::default();
                while SDL_PollEvent(&mut e) {
                    if e.r#type == SDL_EVENT_QUIT.0 {
                        should_quit = true;
                    }
                }
            }
        }

        SDL_GL_DestroyContext(context);
        SDL_DestroyWindow(window);
        SDL_Quit();
    }
}
