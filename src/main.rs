use std::{fs::{self, File}, hint::black_box, io::BufReader, path::{Path, PathBuf}, sync::Arc};

use futures::future;
use mesh_parser::build_mesh;
use tokio::{sync::Semaphore, time::Instant};

fn get_files_to_load(dir: &Path) -> Vec<PathBuf> {
    let mut mylist: Vec<PathBuf> = Vec::new();
    if dir.is_dir() {
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            mylist.push(entry.path());
        }
    }
    mylist
}

async fn async_load_all() -> Vec<Result<mesh_parser::Mesh, String>> {
    let thelist = get_files_to_load(&PathBuf::from("C:\\Users\\Generic\\Downloads\\stormworks meshes"));
    let semaphore = Arc::new(Semaphore::new(15));
    let mut tasks = Vec::new();

    for path in thelist {
        let semaphore = Arc::clone(&semaphore);
        let task = tokio::spawn(async move {
            let _permit = semaphore.acquire_owned().await.unwrap();
            let file = File::open(&path).map_err(|e| e.to_string())?;
            build_mesh(BufReader::new(file))
        });
        tasks.push(task);
    }

    future::join_all(tasks).await.into_iter().map(|res| res.unwrap()).collect()
}

fn sync_load_all() {
    let thelist = get_files_to_load(&PathBuf::from("C:\\Users\\Generic\\Downloads\\stormworks meshes"));
    for path in thelist {
        let file = File::open(&path).unwrap();
        build_mesh(BufReader::new(file)).unwrap();
    }
}

#[tokio::main]
async fn main() {
    println!("Async START!!");
    let start = Instant::now();
    black_box(async_load_all().await);
    let duration = start.elapsed();
    println!("Async DONE after {:?} !!",duration);
    
    println!("--");

    println!("Sync START!!");
    let start = Instant::now();
    black_box(sync_load_all());
    let duration = start.elapsed();
    println!("Sync DONE after {:?} !!",duration);
}