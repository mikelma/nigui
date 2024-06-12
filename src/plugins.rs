use std::process::Command;
use std::io::{self, Write};

pub fn run_script(py_path: String, script: String, py_flags: &[String], py_args: &[String]) -> Result<(), String> {
    let mut cmd = vec![script];
    for i in 0..py_flags.len() {
        cmd.push(py_flags[i].clone());
        cmd.push(py_args[i].clone());
    }

    let path = std::env::current_dir().unwrap();

    let result = Command::new(&py_path)
        .current_dir(&path)
        .args(&cmd)
        .output();

    match result {
        Ok(res) => {
            println!("[*] Plugin output state: {}", res.status);
            io::stdout().write_all(&res.stdout).unwrap();
            io::stderr().write_all(&res.stderr).unwrap();

            Ok(())
        },
        Err(err) => Err(err.to_string()),
    }
}
