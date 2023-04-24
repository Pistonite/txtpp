use txtpp::{Config, execute};

fn main() {
    env_logger::init();
    let config = Config { 
        shell_cmd: "".to_string(), inputs: vec!["..".to_string()], recursive: true, num_threads: 4, verify: false };
    if let Err(e) =  execute(config) {
        eprintln!("{:?}", e);
    }

    println!("Hello, world!")
}
