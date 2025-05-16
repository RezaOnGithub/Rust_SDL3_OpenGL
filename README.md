# Rust+SDL3+OpenGL DIY Template

This is an `extern "C"`-heavy and `unsafe` template for getting started with OpenGL in Rust, using SDL3. The hope is that you will find value, and base your next project on it- unless the very design of it discourges you away.

# What is the problem?

Rust gamedev is not quite mature, and some of the choices made here may turn out to be not the best ones as the ecosystem evolves:

- SDL3: I mainly chose raw SDL3 over GLFW or a Rust-native solutions because it has audio too. I reckon SDL2 will be maintained for quite some time but it will probably not get any new features.
    + [`sdl3-sys` crate](https://crates.io/crates/sdl3-sys) provides slightly more ergonomic bindings than a raw `bindgen` conversion.
    + [`sdl3` crate](https://crates.io/crates/sdl3) does not have feature-parity with `sdl3-sys` yet.
    
- C GLAD: There are a number of OpenGL wrappers for Rust, but unfortunately some have bitrotten and others have too little traction to be dependable. Taking the matters into your own hands may be a good idea.
    + Notably, not all bindings support 4.6 SPIR-V stuff.
    + Sticking to the standard is not a bad idea.

- OpenGL: Currently, I feel that **Rust Vulkan ecosystem seems more mature than its OpenGL counterpart**, so you may want to re-evaluate whether you *really* want OpenGL.

# How to use it, then?

- Probably don't fork this repository! Copy the code over and start from scratch.
- Go to [`glad.dav1d.de`](https://glad.dav1d.de/) and generate your own bindings with your favorite extensions, alongwith its loader.
- Replace my bindings in `glad-sys/cc/` directory with yours.
    + Apologies for the dirtiness! A much better way would be to generate bindings directly from the Khronos spec but [`gl_generator`](https://crates.io/crates/gl_generator) not getting updates is this template's *raison d'être*!
- [`sdl3-sys`](https://crates.io/crates/sdl3-sys) is still in churn; specify an exact version number in `sdl3-experiment/Cargo.toml` instead of a wildcard if doing a more serious projects.
    + Repository lacks a `Cargo.lock` for a similar reason.
- Compile `glad-sys`, and modify `build.rs` for your use-case and/or fix errors. DIY!
    + There are some guardrails to avoid surprises, but not nearly enough!

> [!NOTE]
> One interesting Rust quirk is that the equivalent C function pointer type is always wrapped in `Option`. This is unlike pointers to any other raw type. There is some helper code to unwrap everything before use. `struct GL` is a part of it and it contains unwrapped function pointer.
> 
> Unfortunately, this hits another quirk of Rust where `a.B()` only ever means calling method B `impl` ed on `a` and not invoking a function pointer. This leads to code that looks like:
>
> ```
> let mut gl = GL::unwrap();
> (gl.Clear)(GL_COLOR_BUFFER_BIT);
> ```
> which is ugly. A better wrapper may come in the future.

# License

This repository incorporates code from the following third-parties:

- [SDL3](https://wiki.libsdl.org/SDL3/FrontPage) under zlib license
- [`sdl3-sys`](https://crates.io/crates/sdl3-sys) under zlib license
- GLAD under ??? (which include `khrplatform.h` that is under ???)

To stay in line with SDL, this repository is also licensed under Zlib license.