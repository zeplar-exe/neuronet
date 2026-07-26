fn main() {
    csbindgen::Builder::default()
        .input_extern_file("src/lib.rs")
        .input_extern_file("src/models/mod.rs")
        .input_extern_file("src/util.rs")
        .input_extern_file("src/simulation/events.rs")
        .input_extern_file("src/simulation/execution.rs")
        .input_extern_file("src/simulation/network.rs")
        .input_extern_file("src/simulation/state.rs")
        .csharp_dll_name("neuronet")
        .generate_csharp_file("../Sim.Frontend/Sim.Frontend/NativeMethods.g.cs")
        .unwrap();
}
