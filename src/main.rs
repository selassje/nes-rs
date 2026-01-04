fn main() {
    println!("Nes size: {} bytes", size_of::<nes_rs::nes::Nes>());
    println!("EmulationFrame size: {} bytes", size_of::<nes_rs::nes::EmulationFrame>());

    let emulation = nes_rs::Emulation::new();
    #[cfg(target_os = "emscripten")]
    emscripten_main_loop::run(emulation);
    #[cfg(not(target_os = "emscripten"))]
    nes_rs::run(emulation);
}
