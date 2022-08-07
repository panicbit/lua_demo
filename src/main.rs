use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result};
use rlua::{Context, Lua};
use structopt::StructOpt;

#[derive(StructOpt)]
struct Opt {
    // path to a lua file or directory with lua files
    #[structopt(parse(from_os_str))]
    path: PathBuf,
}
fn main() -> Result<()> {
    let opt = Opt::from_args();

    let lua = Lua::new();

    lua.context(|ctx| run(opt, ctx))?;

    Ok(())
}

fn run(opt: Opt, ctx: Context) -> Result<()> {
    let lua_files = load_lua_files(&opt.path)
        .with_context(|| format!("failed to load lua file(s) from {:?}", opt.path))?;

    for lua_file in &lua_files {
        run_lua_file(lua_file, ctx)
            .with_context(|| format!("failed to run lua file {:?}", lua_file.path))?;
    }

    Ok(())
}

fn run_lua_file(lua_file: &LuaFile, ctx: Context) -> Result<()> {
    eprintln!(">>> Running {:?}", lua_file.path);

    let chunk_name = lua_file.path.file_name()
        .unwrap_or_default()
        .to_string_lossy();

    ctx.load(&lua_file.content)
        .set_name(chunk_name.as_bytes())
        .context("failed to set chunk name")?
        .exec()
        .context("failed to exec script")?;

    Ok(())
}

fn load_lua_files(path: &Path) -> Result<Vec<LuaFile>> {
    let metadata = path.metadata().context("failed to get path metadata")?;

    if !metadata.is_dir() {
        let lua_file = load_lua_file(path)
            .with_context(|| format!("failed to load lua file {path:?}"))?;

        return Ok(vec![lua_file]);
    }

    let mut lua_files = Vec::new();
    let entries = fs::read_dir(path).context("failed to read directory")?;

    for entry in entries {
        let entry = entry.context("failed to get directory entry")?;
        let path = entry.path();
        let extension = path.extension().unwrap_or_default();

        if extension != "lua" {
            continue;
        }

        let lua_file =
            load_lua_file(&path).with_context(|| format!("failed to load lua file {path:?}"))?;

        lua_files.push(lua_file);
    }

    Ok(lua_files)
}

fn load_lua_file(path: &Path) -> Result<LuaFile> {
    let content = fs::read_to_string(&path).context("failed to read lua file content")?;

    Ok(LuaFile {
        path: path.to_owned(),
        content,
    })
}

struct LuaFile {
    path: PathBuf,
    content: String,
}
