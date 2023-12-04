mod analyzer;
mod declarations;
mod generation;
mod overrides;

use std::{
    ffi::OsString,
    fs::{self, File},
    io::{self, BufWriter, Write},
    os::unix::prelude::OsStringExt,
    path::PathBuf,
    process::Command,
};

use analyzer::Analyzer;
use clap::Parser;
//use overrides::apply_overrides;

use crate::{generation::PythonGenerator, overrides::apply_overrides};

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Args {
    /// Modules to generate typing stubs for. (e.g. Adw-1 GtkSource-5)
    modules: Vec<String>,

    // Directory to store the package typings. $site-packages/gi-stubs by default
    #[clap(short, long, parse(from_os_str))]
    out_dir: Option<PathBuf>,

    /// Exclude docstrings in the typings
    #[clap(short, long)]
    no_docs: bool,
}

fn create_stub_tree(dir: &PathBuf) -> io::Result<()> {
    let repo = dir.join("repository");

    if let Err(e) = fs::create_dir_all(&repo) {
        match e.kind() {
            io::ErrorKind::AlreadyExists => (),
            _ => return Err(e),
        }
    };

    let mut py_typed = File::create(dir.join("py.typed"))?;
    py_typed.write(b"partial\n")?;

    let mut init = File::create(dir.join("__init__.pyi"))?;
    init.write(include_bytes!("gi_init.pyi"))?;

    let mut repo_init = File::create(repo.join("__init__.pyi"))?;
    repo_init.write(b"")?;

    Ok(())
}

fn get_sitepackages() -> PathBuf {
    let out = Command::new("python")
        .arg("-c")
        .arg("import site, os; sp = site.getsitepackages()[0]; print(sp if os.access(sp, os.W_OK | os.X_OK) else site.getusersitepackages(), end='')")
        .output().unwrap();

    PathBuf::from(OsString::from_vec(out.stdout))
}

// "/usr/share/gir-1.0/Gtk-3.0.gir"
fn main() -> io::Result<()> {
    let cli = Args::parse();

    let out_dir = cli
        .out_dir
        .unwrap_or_else(|| get_sitepackages().join("gi-stubs"));

    println!("creating gi-stubs tree in {}", out_dir.display());

    create_stub_tree(&out_dir)?;

    let mut analyzer = Analyzer::new(cli.no_docs);
    if cli.modules.len() == 0 {
        analyzer.analyze_repository("", "", &true);
    } else {
        for gir in cli.modules {
            let split: Vec<_> = gir.split('-').take(2).collect();
            if split.len() == 2 {
                analyzer.analyze_repository(split[0], split[1], &false);
            }
        }
    }

    for mut ns in analyzer.namespaces {
        apply_overrides(&mut ns);

        let py = File::create(&out_dir.join("repository").join(ns.name.clone() + ".pyi"))?;
        let mut buf = BufWriter::new(py);
        let mut gen = PythonGenerator::new(&mut buf);
        gen.write_namespace(ns)?;
    }

    //apply_overrides(&out_dir)?;

    Ok(())
}
