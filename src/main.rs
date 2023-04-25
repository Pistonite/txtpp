use txtpp::{txtpp, Verbosity, Mode, Config};

fn main() {
    env_logger::init();
    let config = Config {
        base_dir: std::path::PathBuf::from("examples"),
        shell_cmd: "".to_string(),
        inputs: vec![".".to_string()],
        recursive: true,
        num_threads: 1,
        mode: Mode::Clean,
        verbosity: Verbosity::Quiet,
    };


    let _ = txtpp(config);

    println!("Hello, world!")
}
