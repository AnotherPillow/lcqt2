use asar::{AsarReader, Header, Result as AsarResult, AsarWriter};

use std::{env, io, thread};
use std::error::Error;
use std::io::{BufRead, BufReader, ErrorKind};
use std::net::{Ipv4Addr, TcpListener};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::string::String;
use std::time::Duration;
use std::fs;
use std::fs::File;

use serde_json::json;

fn find_lunar_executable() -> Result<String, String> {
    let paths: Vec<String> = match env::consts::OS {
        "windows" => {
            let localappdata = env::var("localappdata").or(Err("%localappdata% not defined"))?;

            vec![
                // format!(r"{localappdata}\Programs\launcher\Lunar Client.exe"),
                format!(r"{localappdata}\Programs\lunarclient\Lunar Client.exe")
            ]
        }
        "macos" => vec![
            "/Applications/Lunar Client.app/Contents/MacOS/Lunar Client".into(),
            format!(
                "{}/Applications/Lunar Client.app/Contents/MacOS/Lunar Client",
                env::var("HOME").or(Err("$HOME not defined"))?
            )
        ],
        "linux" => vec!["/usr/bin/lunarclient".into()],
        _ => Err("unsupported os")?
    };

    paths.iter()
        .map(|p| Path::new(&p).parent().unwrap().to_str().unwrap().to_string())
        .find(|p| Path::new(
            &format!("{}{}resources", p, std::path::MAIN_SEPARATOR)
        ).exists())
        .map(|p| format!("{}{}resources", p, std::path::MAIN_SEPARATOR))
        .ok_or(format!("searched in the following locations: [{}]", paths.join(", ")))
        
}

fn main() {
    let lunar_path_str = find_lunar_executable().unwrap();
    let lunar_path = Path::new(&lunar_path_str);

    let asar_file = fs::read(lunar_path.join("app.asar")).unwrap();
    fs::copy(lunar_path.join("app.asar"), lunar_path.join("backup.asar")).unwrap();
    println!("app.asar backed up to backup.asar");

    let mut asar = AsarReader::new(&asar_file, lunar_path.join("app.asar")).unwrap();

    let asar_files = asar.files().keys().map(|s| s.display().to_string()).collect::<Vec<_>>();
    
    //check if dist/html/index.html exists
    let html_path = "dist/html/index.html";
    // if !asar_files.contains(&html_path.to_string()) {
    //     println!("Unable to inject, {} does not exist", html_path);
    //     return;
    // }

    let main_path = "dist-electron/electron/main.js";
    // if !asar_files.contains(&main_path.to_string()) {
    //     println!("Unable to inject, {} does not exist", main_path);
    //     return;
    // }

    let gui_asar_path = r"C:\Coding\Mixed\lcqt2\gui\out\gui.asar";
    let main_inject_string = format!("require(`{}\\main-inject.js`)()", gui_asar_path);

    //prefix the main.js with the require
    let main_js = asar.files().get(&PathBuf::from(
        main_path
    )).unwrap().data().first().unwrap().clone();
    let mut main_js_contents = std::str::from_utf8(&[main_js]).unwrap().to_string();
    println!("main.js contents: {}", main_js_contents);
    
    main_js_contents = format!("{}\n{}", main_inject_string, main_js_contents);
    println!("main.js injected");
    println!("main.js contents: {}", main_js_contents);

    let mut out_asar = AsarWriter::new();

    for file in asar.files().keys() {
        if file.to_str().unwrap() == main_path {
            out_asar.write_file(file, main_js_contents.as_bytes(), false).unwrap();
        } else {
            out_asar.write_file(file, asar.files().get(file).unwrap().data(), false).unwrap();
        }
    }

    out_asar.finalize(File::create(lunar_path.join("app.asar")).unwrap()).unwrap();



    // asar.finalize(File::create(asar_file)).unwrap();

}
